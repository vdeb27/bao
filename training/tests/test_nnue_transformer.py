"""NNUE transformer roundtrip: Python mirror must agree with Rust SSoT.

Generates a small set of positions via random self-play and asserts that
``bao_engine_py.nnue_indices`` and ``transformer.indices`` return the same
ascending index list for every position.
"""

from __future__ import annotations

import json
import random
import sys
from pathlib import Path

import numpy as np
import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "training"))

from bao_train.nnue.transformer import (  # noqa: E402
    MAX_ACTIVE,
    N_FEATURES,
    indices,
    indices_batch,
)


@pytest.fixture(scope="module")
def bao():
    return pytest.importorskip("bao_engine_py")


def _walk(bao, plies: int, seed: int) -> list[bytes]:
    """Random self-play, return packed states along the way."""
    rng = random.Random(seed)
    states: list[bytes] = []
    s = bao.new_state("kiswahili")
    states.append(s)
    for _ in range(plies):
        moves = json.loads(bao.legal_moves(s))
        if not moves:
            break
        mv = rng.choice(moves)
        try:
            s, _, _ = bao.apply(s, json.dumps(mv))
        except ValueError:
            break
        states.append(s)
    return states


def test_n_features_matches(bao) -> None:
    assert bao.nnue_n_features() == N_FEATURES


def test_initial_position_matches(bao) -> None:
    s = bao.new_state("kiswahili")
    rust = list(bao.nnue_indices(s))
    py = indices(np.frombuffer(bao.encode_features(s), dtype=np.uint8))
    assert rust == py
    # Initial position: no kutakatia → 35 indices.
    assert len(rust) == 35
    # All within range.
    assert all(0 <= i < N_FEATURES for i in rust)
    # Strictly ascending (groups are emitted in ascending base order).
    assert all(a < b for a, b in zip(rust, rust[1:]))


def test_self_play_roundtrip(bao) -> None:
    """50 trajectories × ~30 plies → ~1500 positions, all must roundtrip."""
    mismatch = 0
    checked = 0
    for seed in range(50):
        for s in _walk(bao, plies=30, seed=seed):
            rust = list(bao.nnue_indices(s))
            py = indices(np.frombuffer(bao.encode_features(s), dtype=np.uint8))
            if rust != py:
                mismatch += 1
            checked += 1
    assert checked > 1000, f"only checked {checked} positions"
    assert mismatch == 0, f"{mismatch}/{checked} positions mismatched"


def test_indices_from_features_matches_from_state(bao) -> None:
    s = bao.new_state("kiswahili")
    feats = bytes(bao.encode_features(s))
    a = list(bao.nnue_indices(s))
    b = list(bao.nnue_indices_from_features(feats))
    assert a == b


def test_indices_batch_matches_single(bao) -> None:
    states = _walk(bao, plies=20, seed=7)
    feats = np.stack(
        [np.frombuffer(bao.encode_features(s), dtype=np.uint8) for s in states]
    )
    batch = indices_batch(feats)
    assert batch.shape == (len(states), MAX_ACTIVE)
    for i, s in enumerate(states):
        single = indices(feats[i])
        active = batch[i][batch[i] >= 0].tolist()
        assert active == single, f"row {i}: {active} != {single}"
