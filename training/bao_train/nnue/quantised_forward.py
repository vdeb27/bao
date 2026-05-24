"""Pure-NumPy integer forward pass that mirrors ``eval::nnue::loader::NnueModel::forward_raw``.

Used to validate the Python↔Rust roundtrip: given an exported ``.nnue`` and
a position, both implementations must return the same centi-kete score
(plan §11). Operates on the **quantised** weights as written to disk, so
this is the ground-truth reference for what the Rust runtime computes.
"""

from __future__ import annotations

import struct
from dataclasses import dataclass

import numpy as np

from .export import HIDDEN_SIZES, MAGIC, VERSION
from .transformer import N_FEATURES

ACCUMULATOR_DIM = HIDDEN_SIZES[0]
HIDDEN_DIM = HIDDEN_SIZES[1]
WEIGHT_SCALE_L0 = 64
WEIGHT_SCALE_HIDDEN = 64
ACTIVATION_CLIP = 127
OUTPUT_SCALE = 1


@dataclass
class LoadedModel:
    l0_w: np.ndarray  # int16 (n_features, accumulator_dim)
    l0_b: np.ndarray  # int16 (accumulator_dim,)
    l1_w: np.ndarray  # int8 (accumulator_dim, hidden_dim)
    l1_b: np.ndarray  # int32 (hidden_dim,)
    l2_w: np.ndarray
    l2_b: np.ndarray
    l3_w: np.ndarray  # int8 (hidden_dim,)
    l3_b: np.ndarray  # int32 (1,)


def load(path: str | bytes) -> LoadedModel:
    if isinstance(path, (str,)):
        with open(path, "rb") as f:
            blob = f.read()
    else:
        blob = path
    return load_from_bytes(blob)


def load_from_bytes(blob: bytes) -> LoadedModel:
    if blob[:8] != MAGIC:
        raise ValueError(f"bad magic: {blob[:8]!r}")
    version = struct.unpack_from("<H", blob, 8)[0]
    if version != VERSION:
        raise ValueError(f"version mismatch: {version}")
    n_features = struct.unpack_from("<H", blob, 10)[0]
    if n_features != N_FEATURES:
        raise ValueError(f"n_features mismatch: {n_features}")
    pos = 8 + 2 + 2 + 8 + 4
    l0_w = np.frombuffer(blob, dtype=np.int16, count=N_FEATURES * ACCUMULATOR_DIM, offset=pos)
    l0_w = l0_w.reshape(N_FEATURES, ACCUMULATOR_DIM).copy()
    pos += N_FEATURES * ACCUMULATOR_DIM * 2
    l0_b = np.frombuffer(blob, dtype=np.int16, count=ACCUMULATOR_DIM, offset=pos).copy()
    pos += ACCUMULATOR_DIM * 2
    # v2: hidden weights are int16
    l1_w = np.frombuffer(blob, dtype=np.int16, count=ACCUMULATOR_DIM * HIDDEN_DIM, offset=pos)
    l1_w = l1_w.reshape(ACCUMULATOR_DIM, HIDDEN_DIM).copy()
    pos += ACCUMULATOR_DIM * HIDDEN_DIM * 2
    l1_b = np.frombuffer(blob, dtype=np.int32, count=HIDDEN_DIM, offset=pos).copy()
    pos += HIDDEN_DIM * 4
    l2_w = np.frombuffer(blob, dtype=np.int16, count=HIDDEN_DIM * HIDDEN_DIM, offset=pos)
    l2_w = l2_w.reshape(HIDDEN_DIM, HIDDEN_DIM).copy()
    pos += HIDDEN_DIM * HIDDEN_DIM * 2
    l2_b = np.frombuffer(blob, dtype=np.int32, count=HIDDEN_DIM, offset=pos).copy()
    pos += HIDDEN_DIM * 4
    l3_w = np.frombuffer(blob, dtype=np.int16, count=HIDDEN_DIM, offset=pos).copy()
    pos += HIDDEN_DIM * 2
    l3_b = np.frombuffer(blob, dtype=np.int32, count=1, offset=pos).copy()
    return LoadedModel(l0_w, l0_b, l1_w, l1_b, l2_w, l2_b, l3_w, l3_b)


def _clipped_relu(x: np.ndarray) -> np.ndarray:
    return np.clip(x, 0, ACTIVATION_CLIP)


def _trunc_div(x: np.ndarray | int, d: int) -> np.ndarray | int:
    """Truncated division (round toward zero), matching Rust's integer `/`.

    NumPy's `//` on int arrays *floors*, which diverges from Rust for
    negative numerators. Convert via signed magnitude / sign."""
    if isinstance(x, np.ndarray):
        sign = np.sign(x)
        return sign * (np.abs(x) // d)
    return int(x / d) if x >= 0 else -((-x) // d)


def forward_raw(model: LoadedModel, indices: list[int]) -> int:
    """Returns raw output (OUTPUT_SCALE × centi-kete)."""
    acc = model.l0_b.astype(np.int32).copy()
    if indices:
        rows = model.l0_w[np.asarray(indices, dtype=np.int64)]
        acc += rows.astype(np.int32).sum(axis=0)
    h1 = _clipped_relu(_trunc_div(acc, WEIGHT_SCALE_L0))
    h2_pre = model.l1_b + h1 @ model.l1_w.astype(np.int32)
    h2 = _clipped_relu(_trunc_div(h2_pre, WEIGHT_SCALE_HIDDEN))
    h3_pre = model.l2_b + h2 @ model.l2_w.astype(np.int32)
    h3 = _clipped_relu(_trunc_div(h3_pre, WEIGHT_SCALE_HIDDEN))
    out = int(model.l3_b[0] + (h3 @ model.l3_w.astype(np.int32)).item())
    return _trunc_div(out, WEIGHT_SCALE_HIDDEN)


def evaluate(model: LoadedModel, indices: list[int]) -> int:
    raw = forward_raw(model, indices)
    return _trunc_div(raw, OUTPUT_SCALE)
