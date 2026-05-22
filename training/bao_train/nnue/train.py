"""Supervised NNUE trainer.

Loads a shard, splits 95/5 train/val, trains the NNUE on (sparse-indices,
i16-label) pairs with MSE + L2-regularised AdamW. The last two epochs run
quantisation-aware training (QAT) so the post-export integer model matches
the fp32 model within ±1 centi-kete.

CLI::

    python -m bao_train.nnue.train \
        --shard training/data/iter1-5M.bin \
        --out training/checkpoints/iter1.pt \
        --epochs 10 --batch 16384 --lr 8.75e-4

Defaults mirror the plan (§5.3).
"""

from __future__ import annotations

import argparse
import math
import time
from dataclasses import dataclass
from pathlib import Path

import numpy as np
import torch
import torch.nn as nn
from torch.utils.data import DataLoader, Dataset

from .architecture import (
    NNUE,
    NNUEConfig,
    OUTPUT_SCALE,
    WEIGHT_SCALE_HIDDEN,
    WEIGHT_SCALE_L0,
)
from .dataset import Shard
from .transformer import MAX_ACTIVE, indices_batch


@dataclass
class TrainConfig:
    shard: Path
    out: Path
    epochs: int = 10
    qat_epochs: int = 2
    batch: int = 16384
    # Plan §5.3 suggested 8.75e-4 (carried over from AlphaZero defaults); in
    # practice MSE on centi-kete labels (std ~3100) stalls at that rate. lr=5e-2
    # gives monotone convergence on 180k positions; see scripts/smoke_train.py.
    lr: float = 5e-2
    weight_decay: float = 1e-7
    val_frac: float = 0.05
    num_workers: int = 0
    device: str = "cpu"
    seed: int = 42


class ShardDataset(Dataset):
    """Random-access view into a memmapped shard."""

    def __init__(self, shard: Shard, idx: np.ndarray) -> None:
        self.shard = shard
        self.idx = idx
        self._features = shard.features()
        self._labels = shard.labels()

    def __len__(self) -> int:
        return len(self.idx)

    def __getitem__(self, i: int) -> tuple[np.ndarray, int]:
        row = self.idx[i]
        return self._features[row], int(self._labels[row])


def collate(batch: list[tuple[np.ndarray, int]]) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
    feats = np.stack([b[0] for b in batch])
    labels = np.array([b[1] for b in batch], dtype=np.float32)
    packed = indices_batch(feats)
    indices = torch.from_numpy(packed).long()
    mask = indices >= 0
    return indices, mask, torch.from_numpy(labels)


def quantise_in_place(model: NNUE) -> None:
    """Fake-quantise weights in-place to simulate int8/int16 rounding.

    Mirrors the integer math in ``docs/nnue_format.md`` so QAT in the last
    epochs matches the runtime evaluator. Does **not** quantise activations
    (ClippedReLU is exact)."""
    with torch.no_grad():
        # L0 accumulator: int16 weights at scale 64
        q = (model.l0_weight * WEIGHT_SCALE_L0).round() / WEIGHT_SCALE_L0
        model.l0_weight.copy_(q)
        qb = (model.l0_bias * WEIGHT_SCALE_L0).round() / WEIGHT_SCALE_L0
        model.l0_bias.copy_(qb)
        # Hidden layers: int8 weights at scale 64
        for layer in (model.l1, model.l2, model.l3):
            qw = (layer.weight * WEIGHT_SCALE_HIDDEN).round().clamp(-127, 127) / WEIGHT_SCALE_HIDDEN
            layer.weight.copy_(qw)
            # Bias: int32 at scale 64*64 (combined input + weight scaling)
            qbias = (layer.bias * (WEIGHT_SCALE_HIDDEN * WEIGHT_SCALE_HIDDEN)).round() / (WEIGHT_SCALE_HIDDEN * WEIGHT_SCALE_HIDDEN)
            layer.bias.copy_(qbias)


def train(cfg: TrainConfig) -> dict:
    torch.manual_seed(cfg.seed)
    np.random.seed(cfg.seed)

    print(f"loading shard {cfg.shard}")
    shard = Shard(cfg.shard)
    n = len(shard)
    print(f"  {n:,} records")
    rng = np.random.default_rng(cfg.seed)
    perm = rng.permutation(n)
    val_n = int(n * cfg.val_frac)
    val_idx = perm[:val_n]
    train_idx = perm[val_n:]
    print(f"  train={len(train_idx):,} val={len(val_idx):,}")

    train_ds = ShardDataset(shard, train_idx)
    val_ds = ShardDataset(shard, val_idx)
    train_loader = DataLoader(
        train_ds, batch_size=cfg.batch, shuffle=True,
        collate_fn=collate, num_workers=cfg.num_workers, drop_last=True,
    )
    val_loader = DataLoader(
        val_ds, batch_size=cfg.batch, shuffle=False,
        collate_fn=collate, num_workers=cfg.num_workers,
    )

    model = NNUE(NNUEConfig()).to(cfg.device)
    opt = torch.optim.AdamW(model.parameters(), lr=cfg.lr, weight_decay=cfg.weight_decay)
    sched = torch.optim.lr_scheduler.CosineAnnealingLR(opt, T_max=cfg.epochs)

    history: list[dict] = []
    for epoch in range(cfg.epochs):
        is_qat = epoch >= cfg.epochs - cfg.qat_epochs
        model.train()
        t0 = time.time()
        train_loss = 0.0
        n_batches = 0
        for indices, mask, labels in train_loader:
            indices = indices.to(cfg.device, non_blocking=True)
            mask = mask.to(cfg.device, non_blocking=True)
            labels = labels.to(cfg.device, non_blocking=True)
            # Network output is at `OUTPUT_SCALE × centi-kete`; compare in
            # centi-kete space so loss magnitudes are interpretable.
            pred = model(indices, mask) / OUTPUT_SCALE
            loss = nn.functional.mse_loss(pred, labels)
            opt.zero_grad(set_to_none=True)
            loss.backward()
            opt.step()
            if is_qat:
                quantise_in_place(model)
            train_loss += loss.item()
            n_batches += 1
        train_loss /= max(n_batches, 1)

        model.eval()
        val_loss = 0.0
        nv = 0
        with torch.no_grad():
            for indices, mask, labels in val_loader:
                indices = indices.to(cfg.device); mask = mask.to(cfg.device); labels = labels.to(cfg.device)
                pred = model(indices, mask) / OUTPUT_SCALE
                val_loss += nn.functional.mse_loss(pred, labels, reduction="sum").item()
                nv += labels.numel()
        val_loss /= max(nv, 1)
        val_rmse = math.sqrt(val_loss)
        dt = time.time() - t0
        tag = "QAT" if is_qat else "FP "
        print(f"epoch {epoch+1:2d}/{cfg.epochs} [{tag}] train_mse={train_loss:9.1f} val_rmse={val_rmse:7.2f} ({dt:.1f}s)")
        history.append(dict(epoch=epoch+1, train_mse=train_loss, val_rmse=val_rmse, qat=is_qat))
        sched.step()

    cfg.out.parent.mkdir(parents=True, exist_ok=True)
    torch.save({"state_dict": model.state_dict(), "history": history, "config": cfg.__dict__}, cfg.out)
    print(f"saved checkpoint to {cfg.out}")
    return {"history": history, "model": model}


def _build_argparser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser()
    p.add_argument("--shard", type=Path, required=True)
    p.add_argument("--out", type=Path, required=True)
    p.add_argument("--epochs", type=int, default=10)
    p.add_argument("--qat-epochs", type=int, default=2)
    p.add_argument("--batch", type=int, default=16384)
    p.add_argument("--lr", type=float, default=8.75e-4)
    p.add_argument("--weight-decay", type=float, default=1e-7)
    p.add_argument("--val-frac", type=float, default=0.05)
    p.add_argument("--num-workers", type=int, default=0)
    p.add_argument("--device", default="cpu")
    p.add_argument("--seed", type=int, default=42)
    return p


def main() -> None:
    args = _build_argparser().parse_args()
    cfg = TrainConfig(**vars(args))
    train(cfg)


if __name__ == "__main__":
    main()
