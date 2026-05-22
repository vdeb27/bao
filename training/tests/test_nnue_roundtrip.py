"""End-to-end Python↔Rust NNUE roundtrip (plan §11 fase-5 acceptance).

A small randomly-initialised model is exported to .nnue, loaded by the Rust
runtime (via PyO3), and evaluated on the same positions as the pure-NumPy
quantised forward. Both must return the same integer centi-kete score.
"""

from __future__ import annotations

import json
import random
import sys
import tempfile
from pathlib import Path

import numpy as np
import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "training"))

import torch  # noqa: E402

from bao_train.nnue.architecture import NNUE  # noqa: E402
from bao_train.nnue.export import export  # noqa: E402
from bao_train.nnue.quantised_forward import evaluate as py_evaluate  # noqa: E402
from bao_train.nnue.quantised_forward import load as py_load  # noqa: E402


@pytest.fixture(scope="module")
def bao():
    return pytest.importorskip("bao_engine_py")


@pytest.fixture(scope="module")
def exported_model() -> tuple[bytes, "py_load.LoadedModel"]:
    torch.manual_seed(7)
    model = NNUE()
    # Spread weights a bit so the integer pipeline produces non-trivial scores.
    with torch.no_grad():
        for p in model.parameters():
            p.copy_(p + torch.randn_like(p) * 0.1)
    with tempfile.NamedTemporaryFile(suffix=".nnue", delete=False) as f:
        path = Path(f.name)
    try:
        export(model, path)
        blob = path.read_bytes()
        loaded = py_load(blob)
        yield blob, loaded
    finally:
        path.unlink(missing_ok=True)


def _walk(bao, plies: int, seed: int) -> list[bytes]:
    rng = random.Random(seed)
    states: list[bytes] = []
    s = bao.new_state("kiswahili")
    states.append(s)
    for _ in range(plies):
        moves = json.loads(bao.legal_moves(s))
        if not moves:
            break
        try:
            s, _, _ = bao.apply(s, json.dumps(rng.choice(moves)))
        except ValueError:
            break
        states.append(s)
    return states


def test_initial_position_matches(bao, exported_model) -> None:
    blob, py_model = exported_model
    s = bao.new_state("kiswahili")
    indices = list(bao.nnue_indices(s))
    py_score = py_evaluate(py_model, indices)
    rust_score = bao.nnue_evaluate(blob, s)
    assert py_score == rust_score, f"py={py_score} rust={rust_score}"


def test_self_play_matches(bao, exported_model) -> None:
    """100 positions; Python and Rust must agree exactly."""
    blob, py_model = exported_model
    mismatches: list[tuple[int, int]] = []
    checked = 0
    for seed in range(5):
        for s in _walk(bao, plies=20, seed=seed):
            indices = list(bao.nnue_indices(s))
            py_score = py_evaluate(py_model, indices)
            rust_score = bao.nnue_evaluate(blob, s)
            if py_score != rust_score:
                mismatches.append((py_score, rust_score))
            checked += 1
    assert checked >= 80, f"only checked {checked}"
    assert not mismatches, f"{len(mismatches)}/{checked} mismatched, e.g. {mismatches[:3]}"


def test_evaluate_returns_int(bao, exported_model) -> None:
    blob, _ = exported_model
    s = bao.new_state("kiswahili")
    score = bao.nnue_evaluate(blob, s)
    assert isinstance(score, int)
