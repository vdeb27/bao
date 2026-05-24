"""Export a PyTorch ``NNUE`` checkpoint to ``.nnue`` bin format.

Usage::

    python scripts/export_checkpoint.py \
        --ckpt checkpoints/iter1.pt \
        --out  checkpoints/iter1.nnue
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from bao_train.nnue.architecture import NNUE
from bao_train.nnue.export import export


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--ckpt", type=Path, required=True)
    p.add_argument("--out", type=Path, required=True)
    args = p.parse_args()

    ckpt = torch.load(args.ckpt, map_location="cpu", weights_only=False)
    model = NNUE()
    model.load_state_dict(ckpt["state_dict"])
    model.eval()
    out = export(model, args.out)
    size = out.stat().st_size
    print(f"exported {args.ckpt} → {out} ({size:,} bytes)")
    if "history" in ckpt:
        h = ckpt["history"]
        if h:
            last = h[-1]
            print(f"final val_rmse={last['val_rmse']:.2f} train_mse={last['train_mse']:.1f}")


if __name__ == "__main__":
    main()
