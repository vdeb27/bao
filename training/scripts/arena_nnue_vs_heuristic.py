"""Arena: NNUE vs handcrafted heuristic (plan §11 fase-5 acceptance).

Plays N games at the given depth budget, each with a randomised opening
(uniform random first 4-8 plies, identical for both colours via mirroring).
Reports NNUE win-rate; the plan demands ≥65% for iter-1 promotion.

Usage::

    python scripts/arena_nnue_vs_heuristic.py \\
        --nnue checkpoints/iter1.nnue \\
        --games 100 --depth 6 --max-nodes 50000 --opening-plies 6
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
    """Play `plies` uniformly-random legal moves; return the move list as JSON
    strings so both colours can replay them deterministically."""
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
    nnue_bytes: bytes,
    opening: list[str],
    nnue_is_south: bool,
    depth: int,
    max_nodes: int,
    max_plies: int,
) -> int:
    """Return +1 if NNUE wins, -1 if heuristic wins, 0 on stalemate/draw/cap.

    The engine reports active-player perspective scores; we don't need them —
    we play until terminal or move-cap, then read terminal state."""
    s = _replay(opening)
    plies = 0
    while plies < max_plies:
        moves = json.loads(bao.legal_moves(s))
        if not moves:
            break
        # active = south (0) or north (1) — first ply after random opening
        # depends on parity. We need the state's active player.
        state = json.loads(bao.state_to_json(s))
        active = int(state["active"])
        nnue_to_move = (active == 0 and nnue_is_south) or (active == 1 and not nnue_is_south)
        if nnue_to_move:
            mv_json, _, _, _ = bao.search_best_nnue(nnue_bytes, s, depth, max_nodes)
        else:
            mv_json, _, _, _ = bao.search_best_heuristic(s, depth, max_nodes)
        if mv_json is None:
            break
        try:
            s, _, _ = bao.apply(s, mv_json)
        except ValueError:
            break
        plies += 1

    # Terminal: side with no moves loses. If reached max plies, declare draw.
    moves = json.loads(bao.legal_moves(s))
    if moves:
        return 0  # cap reached → draw
    state = json.loads(bao.state_to_json(s))
    losing_side = int(state["active"])  # side to move with no legal moves
    nnue_side = 0 if nnue_is_south else 1
    return -1 if losing_side == nnue_side else +1


def main() -> None:
    p = argparse.ArgumentParser()
    p.add_argument("--nnue", type=Path, required=True)
    p.add_argument("--games", type=int, default=100)
    p.add_argument("--depth", type=int, default=6)
    p.add_argument("--max-nodes", type=int, default=50_000)
    p.add_argument("--opening-plies", type=int, default=6)
    p.add_argument("--max-plies", type=int, default=400)
    p.add_argument("--seed", type=int, default=42)
    args = p.parse_args()

    nnue_bytes = args.nnue.read_bytes()
    rng = random.Random(args.seed)

    wins = losses = draws = 0
    for g in range(args.games):
        opening = _opening(rng, args.opening_plies)
        # Each opening played twice with colours swapped → 2 game-pairs.
        for side_south in (True, False):
            r = _play_game(
                nnue_bytes, opening, side_south,
                args.depth, args.max_nodes, args.max_plies,
            )
            if r > 0:
                wins += 1
            elif r < 0:
                losses += 1
            else:
                draws += 1
        if (g + 1) % 10 == 0:
            total = wins + losses + draws
            wr = wins / total if total else 0.0
            print(f"after {g+1} openings ({total} games): W={wins} L={losses} D={draws} wr={wr:.1%}")

    total = wins + losses + draws
    wr = wins / total if total else 0.0
    # Wilson 95% lower bound
    z = 1.96
    denom = 1 + z*z/total
    centre = (wr + z*z/(2*total)) / denom
    margin = z * math.sqrt(wr*(1-wr)/total + z*z/(4*total*total)) / denom
    lower = centre - margin
    print(f"\nFinal: {wins}W {losses}L {draws}D over {total} games")
    print(f"NNUE win-rate: {wr:.1%}  Wilson95-lower: {lower:.1%}")
    print(f"Plan §5.5 promotion gate: lower ≥ 51% → {'PASS' if lower >= 0.51 else 'FAIL'}")
    print(f"Plan §11 iter-1 gate: win-rate ≥ 65% → {'PASS' if wr >= 0.65 else 'FAIL'}")


if __name__ == "__main__":
    main()
