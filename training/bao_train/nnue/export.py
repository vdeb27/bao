"""Export a trained PyTorch ``NNUE`` to the integer ``.nnue`` bin format.

Format spec: ``docs/nnue_format.md``. The exported file is consumed by
``bao_engine::eval::nnue::loader`` in Rust and (later) baked into the
browser WASM bundle via ``include_bytes!``.

Quantisation rules:

- L0 weight: ``int16 = round(fp32 * WEIGHT_SCALE_L0)``  (clamped to i16 range)
- L0 bias:   ``int16 = round(fp32 * WEIGHT_SCALE_L0)``
- L1..L3 weight: ``int8 = round(fp32 * WEIGHT_SCALE_HIDDEN)`` clamped to ±127
- L1..L3 bias:   ``int32 = round(fp32 * WEIGHT_SCALE_HIDDEN * WEIGHT_SCALE_HIDDEN)``

The combined input × weight scaling is `WEIGHT_SCALE_HIDDEN²`, so biases
must be pre-scaled the same way. Output is in `OUTPUT_SCALE × centi-kete`,
divide on the runtime side.
"""

from __future__ import annotations

import struct
from pathlib import Path

import numpy as np
import torch

from .architecture import (
    ACCUMULATOR_DIM,
    HIDDEN_DIM,
    NNUE,
    OUTPUT_SCALE,
    WEIGHT_SCALE_HIDDEN,
    WEIGHT_SCALE_L0,
)
from .transformer import N_FEATURES

MAGIC = b"BAONNUE\0"
VERSION = 1
HIDDEN_SIZES = (ACCUMULATOR_DIM, HIDDEN_DIM, HIDDEN_DIM, 1)


def _quantise_l0_weight(w: torch.Tensor) -> np.ndarray:
    """``(n_features, accumulator_dim)`` fp32 → int16 row-major."""
    q = (w * WEIGHT_SCALE_L0).round().clamp(-32768, 32767).to(torch.int16)
    return q.detach().cpu().numpy()


def _quantise_l0_bias(b: torch.Tensor) -> np.ndarray:
    q = (b * WEIGHT_SCALE_L0).round().clamp(-32768, 32767).to(torch.int16)
    return q.detach().cpu().numpy()


def _quantise_hidden_weight(w: torch.Tensor) -> np.ndarray:
    """PyTorch ``Linear`` stores weight as ``(out, in)``; export row-major
    ``(in, out)`` so the runtime can do ``output[j] = sum_i input[i] * W[i, j]``."""
    q = (w * WEIGHT_SCALE_HIDDEN).round().clamp(-127, 127).to(torch.int8)
    return q.detach().cpu().numpy().T.copy()  # transpose to (in, out)


def _quantise_hidden_bias(b: torch.Tensor) -> np.ndarray:
    combined = WEIGHT_SCALE_HIDDEN * WEIGHT_SCALE_HIDDEN
    q = (b * combined).round().clamp(-(2**31), 2**31 - 1).to(torch.int32)
    return q.detach().cpu().numpy()


def export(model: NNUE, path: str | Path) -> Path:
    """Write `model` to `path` in the BAONNUE format. Returns the path."""
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("wb") as f:
        f.write(MAGIC)
        f.write(struct.pack("<H", VERSION))
        f.write(struct.pack("<H", N_FEATURES))
        for s in HIDDEN_SIZES:
            f.write(struct.pack("<H", s))
        f.write(struct.pack("<f", float(OUTPUT_SCALE)))

        l0_w = _quantise_l0_weight(model.l0_weight)
        f.write(l0_w.tobytes())  # i16, (n_features, accumulator_dim) row-major
        l0_b = _quantise_l0_bias(model.l0_bias)
        f.write(l0_b.tobytes())  # i16

        for layer in (model.l1, model.l2, model.l3):
            f.write(_quantise_hidden_weight(layer.weight).tobytes())  # i8
            f.write(_quantise_hidden_bias(layer.bias).tobytes())  # i32

    return path


def expected_size() -> int:
    header = 8 + 2 + 2 + 8 + 4  # magic + version + n_features + hidden_sizes + quant_scale
    l0 = N_FEATURES * ACCUMULATOR_DIM * 2 + ACCUMULATOR_DIM * 2
    l1 = ACCUMULATOR_DIM * HIDDEN_DIM * 1 + HIDDEN_DIM * 4
    l2 = HIDDEN_DIM * HIDDEN_DIM * 1 + HIDDEN_DIM * 4
    l3 = HIDDEN_DIM * 1 * 1 + 1 * 4
    return header + l0 + l1 + l2 + l3
