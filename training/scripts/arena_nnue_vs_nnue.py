"""Arena: two NNUE models head-to-head (plan §5.4 promotion gate).

Each opening is played twice with sides swapped to remove first-mover
bias. Reports challenger win-rate and Wilson95 lower bound; plan §5.4
requires ≥55% with Wilson lower ≥51%.

Usage::

    python scripts/arena_nnue_vs_nnue.py \\
        --challenger checkpoints/iter2.nnue \\
        --incumbent checkpoints/iter1_30ep.nnue \\
        --games 50 --depth 4 --max-nodes 20000 --opening-plies 6
"""
from __future__ import annotations

import argparse
import json
import math
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import bao_engine_py as bao


def _opening(rng: random.Random, plies: int) -> list[str]:
    s = bao.new_state("kiswahili")
    moves_played: list[str] = []
    for _ in range(plies):
        moves = json.loads(bao.legal_moves(s))
        if not moves:
            break
        mv = rng.choice(moves)
        mv_json = json.dumps(mv)
        try:
            s, _, _ = bao.apply(s, mv_json)
        except ValueError:
            break
        moves_played.append(mv_json)
    return moves_played


def _replay(opening: list[str]) -> bytes:
    s = bao.new_state("kiswahili")
    for mv_json in opening:
        s, _, _ = bao.apply(s, mv_json)
    return s


def _play_game(
    chal_bytes: bytes,
    inc_bytes: bytes,
    opening: list[str],
    chal_is_south: bool,
    depth: int,
    max_nodes: int,
    max_plies: int,
) -> int:
    s = _replay(opening)
    plies = 0
    while plies < max_plies:
        moves = json.loads(bao.legal_moves(s))
        if not moves:
            break
        state = json.loads(bao.state_to_json(s))
        active = int(state["active"])
        chal_to_move = (active == 0 and chal_is_south) or (active == 1 and not chal_is_south)
        if chal_to_move:
            mv_json, _, _, _ = bao.search_best_nnue(chal_bytes, s, depth, max_nodes)
        else:
            mv_json, _, _, _ = bao.search_best_nnue(inc_bytes, s, depth, max_nodes)
        if mv_json is None:
            break
        try:
            s, _, _ = bao.apply(s, mv_json)
        except ValueError:
            break
        plies += 1
    moves = json.loads(bao.legal_moves(s))
    if moves:
        return 0
    state = json.loads(bao.state_to_json(s))
    losing = int(state["active"])
    chal_side = 0 if chal_is_south else 1
    return -1 if losing == chal_side else +1


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--challenger", type=Path, required=True)
    p.add_argument("--incumbent", type=Path, required=True)
    p.add_argument("--games", type=int, default=50)
    p.add_argument("--depth", type=int, default=4)
    p.add_argument("--max-nodes", type=int, default=20_000)
    p.add_argument("--opening-plies", type=int, default=6)
    p.add_argument("--max-plies", type=int, default=400)
    p.add_argument("--seed", type=int, default=42)
    args = p.parse_args()

    chal_bytes = args.challenger.read_bytes()
    inc_bytes = args.incumbent.read_bytes()
    rng = random.Random(args.seed)

    wins = losses = draws = 0
    for g in range(args.games):
        opening = _opening(rng, args.opening_plies)
        for side_south in (True, False):
            r = _play_game(
                chal_bytes, inc_bytes, opening, side_south,
                args.depth, args.max_nodes, args.max_plies,
            )
            if r > 0: wins += 1
            elif r < 0: losses += 1
            else: draws += 1
        if (g + 1) % 10 == 0:
            total = wins + losses + draws
            wr = wins / total if total else 0.0
            print(f"after {g+1} openings ({total} games): W={wins} L={losses} D={draws} wr={wr:.1%}")

    total = wins + losses + draws
    wr = wins / total if total else 0.0
    z = 1.96
    denom = 1 + z*z/total
    centre = (wr + z*z/(2*total)) / denom
    margin = z * math.sqrt(wr*(1-wr)/total + z*z/(4*total*total)) / denom
    lower = centre - margin
    print(f"\nFinal (challenger view): {wins}W {losses}L {draws}D over {total} games")
    print(f"Challenger win-rate: {wr:.1%}  Wilson95-lower: {lower:.1%}")
    print(f"Plan §5.4 promotion gate: lower ≥ 51% → {'PASS' if lower >= 0.51 else 'FAIL'}")


if __name__ == "__main__":
    main()
