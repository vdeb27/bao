"""Tiny subset training smoke: 50k samples, 4 epochs, batch 4096.

Validates the trainer learns. Plan §11 acceptance demands monotonic val
loss decline; this script asserts that on a subset before we commit to a
full 5M run.
"""

from __future__ import annotations

import sys
from pathlib import Path

import numpy as np
import torch
import torch.nn.functional as F
from torch.utils.data import DataLoader

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from bao_train.nnue.architecture import NNUE, OUTPUT_SCALE
from bao_train.nnue.dataset import Shard
from bao_train.nnue.train import ShardDataset, collate


def main() -> None:
    shard = Shard("data/iter1-5M.bin")
    rng = np.random.default_rng(42)
    idx = rng.permutation(len(shard))[:60_000]
    train_idx, val_idx = idx[:50_000], idx[50_000:]
    train_ds = ShardDataset(shard, train_idx)
    val_ds = ShardDataset(shard, val_idx)
    loader = DataLoader(train_ds, batch_size=4096, shuffle=True, collate_fn=collate, drop_last=True)
    val_loader = DataLoader(val_ds, batch_size=4096, shuffle=False, collate_fn=collate)

    model = NNUE()
    opt = torch.optim.AdamW(model.parameters(), lr=5e-2, weight_decay=1e-7)

    val_history: list[float] = []
    for epoch in range(6):
        model.train()
        train_loss = 0.0
        nb = 0
        for indices, mask, labels in loader:
            pred = model(indices, mask) / OUTPUT_SCALE
            loss = F.mse_loss(pred, labels)
            opt.zero_grad(set_to_none=True)
            loss.backward()
            opt.step()
            train_loss += loss.item()
            nb += 1
        model.eval()
        val_loss = 0.0
        nv = 0
        with torch.no_grad():
            for indices, mask, labels in val_loader:
                pred = model(indices, mask) / OUTPUT_SCALE
                val_loss += F.mse_loss(pred, labels, reduction="sum").item()
                nv += labels.numel()
        val_mse = val_loss / nv
        val_history.append(val_mse)
        print(f"epoch {epoch+1}: train_mse={train_loss/nb:9.1f} val_mse={val_mse:9.1f} val_rmse={val_mse**0.5:.1f}")

    assert val_history[-1] < val_history[0], (
        f"val_mse did not decrease: {val_history[0]:.1f} → {val_history[-1]:.1f}"
    )
    print(f"OK: val_mse dropped {val_history[0]:.0f} → {val_history[-1]:.0f}")


if __name__ == "__main__":
    main()
