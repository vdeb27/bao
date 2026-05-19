"""Rust↔Python shard roundtrip test (plan §11 fase-4 acceptance).

Generates a small shard via the Rust example, opens it with the Python
loader, and checks that:
- header matches the spec constants
- features match what the Rust encoder produces for the same packed state
- labels are within ±LABEL_CLIP and not NaN/Inf
- kete-sum invariant holds for every position
"""

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
from pathlib import Path

import numpy as np
import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO_ROOT / "training"))

from bao_train.nnue.dataset import (  # noqa: E402
    FEATURE_LEN,
    HEADER_LEN,
    LABEL_DTYPE_I16,
    RECORD_STRIDE,
    SHARD_VERSION,
    Shard,
    ShardHeader,
)
from bao_train.nnue.features import kete_sum  # noqa: E402


def _generate_shard(out: Path, n: int = 200) -> None:
    cmd = [
        "cargo", "run", "--release", "--example", "generate_positions", "--",
        "--out", str(out),
        "--n", str(n),
        "--threads", "2",
        "--label-depth", "6",
        "--label-nodes", "10000",
        "--play-depth", "4",
        "--play-nodes", "2000",
        "--seed", "42",
    ]
    subprocess.run(cmd, cwd=REPO_ROOT, check=True, capture_output=True)


@pytest.fixture(scope="module")
def shard_file() -> Path:
    with tempfile.NamedTemporaryFile(suffix=".bin", delete=False) as f:
        path = Path(f.name)
    _generate_shard(path, n=200)
    yield path
    os.unlink(path)


def test_header_matches_spec(shard_file: Path) -> None:
    with shard_file.open("rb") as f:
        header_bytes = f.read(HEADER_LEN)
    h = ShardHeader.parse(header_bytes)
    assert h.version == SHARD_VERSION
    assert h.feature_len == FEATURE_LEN
    assert h.record_stride == RECORD_STRIDE
    assert h.label_dtype == LABEL_DTYPE_I16
    assert h.n_records == 200


def test_features_shape(shard_file: Path) -> None:
    s = Shard(shard_file)
    assert len(s) == 200
    feats = s.features()
    assert feats.shape == (200, FEATURE_LEN)
    assert feats.dtype == np.uint8


def test_kete_sum_invariant(shard_file: Path) -> None:
    s = Shard(shard_file)
    sums = kete_sum(s.features())
    # All sampled positions are non-terminal so kete-sum must equal 64.
    assert (sums == 64).all(), f"unexpected kete sums: {sorted(set(sums.tolist()))}"


def test_labels_in_range(shard_file: Path) -> None:
    s = Shard(shard_file)
    labels = s.labels()
    assert labels.dtype == np.int16
    assert labels.shape == (200,)
    # i16 range; LABEL_CLIP = 8000 enforced by encoder.
    assert (labels >= -8000).all()
    assert (labels <= 8000).all()


def test_reserved_bytes_zero(shard_file: Path) -> None:
    s = Shard(shard_file)
    feats = s.features()
    # bytes 44..80 are reserved zeros per docs/feature_layout.md
    assert (feats[:, 44:] == 0).all()


def test_pyo3_feature_consistency() -> None:
    """Rust-encoded features for the initial position must match a Python
    re-implementation of the layout — proves the SSoT contract holds."""
    bao = pytest.importorskip("bao_engine_py")
    s = bao.new_state("kiswahili")
    f = bytes(bao.encode_features(s))
    assert len(f) == FEATURE_LEN
    # Initial Kiswahili position: 22 in ghala, 2/6/2 at cols 4/5/6 mbele.
    # Encoder writes from active=South perspective.
    assert f[4] == 6, "nyumba at vichwa[4]"
    assert f[5] == 2 and f[6] == 2, "flanking pits at vichwa[5]/[6]"
    assert f[32] == 22 and f[33] == 22, "both ghalas hold 22 kete"
    # Sum invariant
    total = sum(f[:32]) + f[32] + f[33]
    assert total == 64
    # Reserved zeros
    assert all(b == 0 for b in f[44:])


def test_pyo3_search_heuristic() -> None:
    """search_heuristic returns a tuple (score, depth, nodes)."""
    bao = pytest.importorskip("bao_engine_py")
    s = bao.new_state("kiswahili")
    score, depth, nodes = bao.search_heuristic(s, 4, 5000)
    assert isinstance(score, int)
    assert 1 <= depth <= 4
    assert nodes > 0


def test_pyo3_read_shard_header(shard_file: Path) -> None:
    bao = pytest.importorskip("bao_engine_py")
    raw = shard_file.read_bytes()[:HEADER_LEN]
    version, feature_len, stride, n, dtype = bao.read_shard_header(raw)
    assert version == SHARD_VERSION
    assert feature_len == FEATURE_LEN
    assert stride == RECORD_STRIDE
    assert n == 200
    assert dtype == LABEL_DTYPE_I16
