"""Quick sanity: heuristic-eval alpha-beta vs uniform-random over N games.

Plan §5.5 demands ≥80% over 200 games. If we score much lower the
"heuristic baseline" is too weak to be a meaningful arena oracle.
"""

from __future__ import annotations

import json
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import bao_engine_py as bao


def play(heuristic_is_south: bool, seed: int, depth: int, max_nodes: int, max_plies: int) -> int:
    rng = random.Random(seed)
    s = bao.new_state("kiswahili")
    plies = 0
    while plies < max_plies:
        moves = json.loads(bao.legal_moves(s))
        if not moves:
            break
        state = json.loads(bao.state_to_json(s))
        active = int(state["active"])
        heur_to_move = (active == 0 and heuristic_is_south) or (active == 1 and not heuristic_is_south)
        if heur_to_move:
            mv_json, _, _, _ = bao.search_best_heuristic(s, depth, max_nodes)
        else:
            mv = rng.choice(moves)
            mv_json = json.dumps(mv)
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
    heur_side = 0 if heuristic_is_south else 1
    return -1 if losing == heur_side else +1


def main() -> None:
    wins = losses = draws = 0
    for g in range(20):
        for side_south in (True, False):
            r = play(side_south, seed=g*2 + side_south, depth=4, max_nodes=20000, max_plies=400)
            if r > 0: wins += 1
            elif r < 0: losses += 1
            else: draws += 1
        if (g+1) % 5 == 0:
            t = wins+losses+draws
            print(f"after {g+1} pairs ({t} games): W={wins} L={losses} D={draws} wr={wins/t:.1%}")
    t = wins+losses+draws
    print(f"\nHeuristic vs random: {wins}W {losses}L {draws}D over {t} games — wr {wins/t:.1%}")


if __name__ == "__main__":
    main()
