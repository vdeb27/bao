"""Compare PyTorch fp32 forward vs Rust quantised eval on random positions.

With i16 hidden weights (format v2), drift should be small (<~50 centi-kete
per position). Catastrophic drift (>500) indicates a quantisation bug.
"""

from __future__ import annotations

import json
import random
import sys
from pathlib import Path

import numpy as np
import torch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import bao_engine_py as bao
from bao_train.nnue.architecture import NNUE, OUTPUT_SCALE
from bao_train.nnue.transformer import indices as py_indices


def main() -> None:
    ckpt_path = Path("/home/johan/Documents/Claude-Code/bao/training/checkpoints/iter1.pt")
    nnue_path = Path("/home/johan/Documents/Claude-Code/bao/training/checkpoints/iter1.nnue")

    ckpt = torch.load(ckpt_path, map_location="cpu", weights_only=False)
    model = NNUE()
    model.load_state_dict(ckpt["state_dict"])
    model.eval()

    nnue_bytes = nnue_path.read_bytes()
    rng = random.Random(0)

    diffs = []
    fps = []
    qs = []
    for trial in range(30):
        s = bao.new_state("kiswahili")
        plies = rng.randint(0, 30)
        for _ in range(plies):
            moves = json.loads(bao.legal_moves(s))
            if not moves:
                break
            mv = rng.choice(moves)
            try:
                s, _, _ = bao.apply(s, json.dumps(mv))
            except ValueError:
                break
        # Rust quantised eval
        q_eval = bao.nnue_evaluate(nnue_bytes, s)
        # FP32 forward in PyTorch
        idx_list = list(bao.nnue_indices(s))
        idx_t = torch.full((1, 39), -1, dtype=torch.long)
        for i, v in enumerate(idx_list):
            idx_t[0, i] = v
        mask = idx_t >= 0
        with torch.no_grad():
            raw = model(idx_t, mask).item()
        fp_eval = raw / OUTPUT_SCALE
        diffs.append(abs(q_eval - fp_eval))
        fps.append(fp_eval)
        qs.append(q_eval)
        print(f"trial {trial:2d}: fp={fp_eval:+8.1f}  q={q_eval:+5d}  diff={q_eval-fp_eval:+6.1f}")

    print(f"\nmean |diff|={np.mean(diffs):.1f}  max |diff|={max(diffs):.1f}")
    print(f"fp range: [{min(fps):.0f}, {max(fps):.0f}]   q range: [{min(qs)}, {max(qs)}]")


if __name__ == "__main__":
    main()
