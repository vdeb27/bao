"""NNUE architecture: 280→512→32→32→1 with ClippedReLU.

Stockfish-style **single accumulator** (we don't dual-perspective because
features are already encoded from active-player POV by the Rust encoder,
``docs/feature_layout.md``).

The model is trained in fp32 and then **post-quantised** to the integer
format in ``docs/nnue_format.md``. Quantisation-aware training (QAT) in the
last two epochs simulates int8 rounding so accuracy doesn't drift after
export. The Rust runtime evaluates in pure integer arithmetic.

Quantisation constants (see ``docs/nnue_format.md``):
- ``WEIGHT_SCALE_L0 = 64`` for the int16 accumulator
- ``WEIGHT_SCALE_HIDDEN = 64`` for int8 hidden layers
- ``ACTIVATION_CLIP = 127`` for ClippedReLU upper bound
- ``OUTPUT_SCALE = 16`` divides the final L3 output to centi-kete units
"""

from __future__ import annotations

from dataclasses import dataclass

import torch
import torch.nn as nn
import torch.nn.functional as F

from .transformer import MAX_ACTIVE, N_FEATURES

ACCUMULATOR_DIM = 512
HIDDEN_DIM = 32
WEIGHT_SCALE_L0 = 64
WEIGHT_SCALE_HIDDEN = 64
ACTIVATION_CLIP = 127
# Format v2: hidden weights are int16 at scale 64, giving fp range ±512 —
# plenty of headroom for unconstrained Kaiming-style training. Earlier i8
# at scale 64 capped weights at ±1.98 which the optimiser couldn't honour
# (trained L3 max was ±47, fully clipped to ±2 by export). Bumping i8→i16
# adds ~17 KB to the model (negligible vs 305 KB total).
OUTPUT_SCALE = 1


@dataclass(frozen=True)
class NNUEConfig:
    n_features: int = N_FEATURES
    accumulator_dim: int = ACCUMULATOR_DIM
    hidden_dim: int = HIDDEN_DIM


class ClippedReLU(nn.Module):
    """``clamp(x, 0, ACTIVATION_CLIP)``. Identical at fp32 and quantised."""

    def __init__(self, clip: int = ACTIVATION_CLIP) -> None:
        super().__init__()
        self.clip = clip

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        return torch.clamp(x, 0.0, float(self.clip))


class NNUE(nn.Module):
    """Sparse-input NNUE model.

    Forward signature: ``forward(indices: (B, MAX_ACTIVE) int64, mask: (B, MAX_ACTIVE) bool)``.
    Uses ``EmbeddingBag`` semantics manually via gather+sum so we keep
    control over the accumulator's bias addition (matches the Rust runtime).
    """

    def __init__(self, cfg: NNUEConfig = NNUEConfig()) -> None:
        super().__init__()
        self.cfg = cfg
        # Accumulator: feature embedding (n_features × accumulator_dim). The
        # forward pass sums the rows selected by `indices` and adds bias.
        self.l0_weight = nn.Parameter(
            torch.empty(cfg.n_features, cfg.accumulator_dim)
        )
        self.l0_bias = nn.Parameter(torch.zeros(cfg.accumulator_dim))
        self.l1 = nn.Linear(cfg.accumulator_dim, cfg.hidden_dim)
        self.l2 = nn.Linear(cfg.hidden_dim, cfg.hidden_dim)
        self.l3 = nn.Linear(cfg.hidden_dim, 1)
        self.act = ClippedReLU()
        self._init_weights()

    def _init_weights(self) -> None:
        # L0 is a sparse-sum embedding of ~36 active rows per forward. With
        # uniform(-10, 10) init the accumulator pre-activation has std ~35,
        # so post-CReLU activations span [0, ~127] without saturating most
        # units (which would kill gradients).
        nn.init.uniform_(self.l0_weight, -10.0, 10.0)
        nn.init.zeros_(self.l0_bias)
        for m in (self.l1, self.l2, self.l3):
            nn.init.kaiming_uniform_(m.weight, a=5**0.5)
            nn.init.zeros_(m.bias)

    def clamp_hidden_weights_(self) -> None:
        """No-op in format v2 (hidden weights are int16, no clamping needed).
        Kept for API stability with the trainer."""
        pass

    def forward(
        self,
        indices: torch.Tensor,  # (B, MAX_ACTIVE) int64; -1 = padding
        mask: torch.Tensor,     # (B, MAX_ACTIVE) bool
    ) -> torch.Tensor:
        # Sum-pool active feature rows. Padding rows are gathered safely by
        # clamping to 0 then masking, so the gather index is always valid.
        safe_idx = indices.clamp(min=0)
        # (B, MAX_ACTIVE, accumulator_dim)
        gathered = self.l0_weight[safe_idx]
        gathered = gathered * mask.unsqueeze(-1).to(gathered.dtype)
        acc = gathered.sum(dim=1) + self.l0_bias  # (B, accumulator_dim)
        x = self.act(acc)
        x = self.act(self.l1(x))
        x = self.act(self.l2(x))
        x = self.l3(x)  # (B, 1) raw
        return x.squeeze(-1)

    def output_scale(self) -> float:
        """Score the network's raw output is in. Divide by ``OUTPUT_SCALE``
        to get centi-kete; equivalent to ``self.forward(...) / OUTPUT_SCALE``
        but kept explicit so QAT and the Rust loader stay in sync."""
        return float(OUTPUT_SCALE)


def pack_indices(batch_indices: list[list[int]]) -> tuple[torch.Tensor, torch.Tensor]:
    """Pack variable-length index lists into ``(B, MAX_ACTIVE)`` int64 tensors
    with a boolean mask. Padding value is ``-1``."""
    b = len(batch_indices)
    out = torch.full((b, MAX_ACTIVE), -1, dtype=torch.long)
    for i, idx in enumerate(batch_indices):
        out[i, : len(idx)] = torch.tensor(idx, dtype=torch.long)
    mask = out >= 0
    return out, mask
