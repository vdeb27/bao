"""Sanity-check the NNUE evaluation sign and magnitude on simple positions."""
from __future__ import annotations
import json, sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import bao_engine_py as bao

nnue_bytes = Path("/home/johan/Documents/Claude-Code/bao/training/checkpoints/iter1_30ep.nnue").read_bytes()

# 1) Initial position: should be ~0 (symmetric)
s = bao.new_state("kiswahili")
e0 = bao.nnue_evaluate(nnue_bytes, s)
print(f"initial position eval (active=south): {e0:+d}")

# 2) Play one ply, eval from north's POV
moves = json.loads(bao.legal_moves(s))
s1, _, _ = bao.apply(s, json.dumps(moves[0]))
e1 = bao.nnue_evaluate(nnue_bytes, s1)
print(f"after 1 south move (active=north): {e1:+d}")

# 3) Compare with heuristic on same position
hmv, _, _, _ = bao.search_best_heuristic(s, 4, 5000)
nmv, _, _, _ = bao.search_best_nnue(nnue_bytes, s, 4, 5000)
print(f"\ninitial best move (heuristic): {hmv}")
print(f"initial best move (NNUE):     {nmv}")

# 4) Play 20 random plies, compare evals
import random
rng = random.Random(0)
for trial in range(5):
    s = bao.new_state("kiswahili")
    for _ in range(rng.randint(5, 30)):
        moves = json.loads(bao.legal_moves(s))
        if not moves: break
        s, _, _ = bao.apply(s, json.dumps(rng.choice(moves)))
    moves = json.loads(bao.legal_moves(s))
    if not moves:
        continue
    e = bao.nnue_evaluate(nnue_bytes, s)
    hmv, hsc, _, _ = bao.search_best_heuristic(s, 4, 5000)
    nmv, nsc, _, _ = bao.search_best_nnue(nnue_bytes, s, 4, 5000)
    same = "SAME" if hmv == nmv else "DIFF"
    print(f"trial {trial}: nnue_eval={e:+5d} heur_search={hsc:+5d} nnue_search={nsc:+5d} moves: {same}")
