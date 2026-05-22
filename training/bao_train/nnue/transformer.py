"""Sparse NNUE feature transformer (Python mirror of the Rust SSoT).

The 80-byte dense features (``docs/feature_layout.md``) become a sparse set
of active indices in ``0..N_FEATURES = 280``. See ``docs/nnue_format.md``
for the index-group layout. This module must agree bytewise with
``bao_engine::eval::nnue::transformer``; a pytest in
``training/tests/test_nnue_transformer.py`` checks 1k random positions.

For batched conversion we expose ``indices_batch`` that emits a packed
``(N, MAX_ACTIVE)`` int32 array with a ``-1`` sentinel for unused slots.
This is the layout PyTorch's ``EmbeddingBag`` / ``nn.functional.embedding``
prefer for sparse input transformers.
"""

from __future__ import annotations

import numpy as np

from .features import (
    FEATURE_LEN,
    OFFSET_KUTAKATIA_FIELD,
    OFFSET_KUTAKATIA_OWN,
    OFFSET_KUTAKATIA_PRESENT,
    OFFSET_KUTAKATIA_TURNS,
    OFFSET_OPP_NYUMBA_STATE,
    OFFSET_OWN_NYUMBA_STATE,
    OFFSET_PHASE_SUBSTATE,
)

N_FEATURES = 280
MAX_ACTIVE = 39  # 32 pits + 2 nyumba + 1 phase + 4 kutakatia

PIT_BUCKETS_BASE = 0
NYUMBA_STATE_BASE = 256
PHASE_SUBSTATE_BASE = 262
KUTAKATIA_BASE = 268

KUTAKATIA_PRESENT_IDX = KUTAKATIA_BASE          # 268
KUTAKATIA_OWN_IDX = KUTAKATIA_BASE + 1          # 269
KUTAKATIA_FIELD_BASE = KUTAKATIA_BASE + 2       # 270..278
KUTAKATIA_TURNS_BASE = KUTAKATIA_BASE + 10      # 278..280

# Mirrors Rust `bucket()`.
_BUCKET_BOUNDS = np.array([1, 2, 3, 4, 6, 9, 16], dtype=np.int32)


def bucket(count: int) -> int:
    """Map a raw kete count to its bucket index (0..7). Matches Rust."""
    return int(np.digitize(count, _BUCKET_BOUNDS))


def _phase_substate_dense(byte: int) -> int:
    phase = byte >> 2
    sub = byte & 0b11
    return phase * 3 + sub


def indices(features: np.ndarray) -> list[int]:
    """Return ascending sparse indices for one 80-byte feature vector."""
    if features.shape != (FEATURE_LEN,):
        raise ValueError(f"expected ({FEATURE_LEN},), got {features.shape}")
    feats = features.astype(np.int32)
    out: list[int] = []

    # 32 pit buckets
    pit_counts = feats[:32]
    pit_buckets = np.digitize(pit_counts, _BUCKET_BOUNDS)
    for p in range(32):
        out.append(PIT_BUCKETS_BASE + p * 8 + int(pit_buckets[p]))

    # nyumba state
    own_ns = int(feats[OFFSET_OWN_NYUMBA_STATE])
    opp_ns = int(feats[OFFSET_OPP_NYUMBA_STATE])
    out.append(NYUMBA_STATE_BASE + own_ns)
    out.append(NYUMBA_STATE_BASE + 3 + opp_ns)

    # phase / substate
    out.append(PHASE_SUBSTATE_BASE + _phase_substate_dense(int(feats[OFFSET_PHASE_SUBSTATE])))

    # kutakatia
    if int(feats[OFFSET_KUTAKATIA_PRESENT]) == 1:
        out.append(KUTAKATIA_PRESENT_IDX)
        if int(feats[OFFSET_KUTAKATIA_OWN]) == 1:
            out.append(KUTAKATIA_OWN_IDX)
        field = int(feats[OFFSET_KUTAKATIA_FIELD])
        if 0 <= field < 8:
            out.append(KUTAKATIA_FIELD_BASE + field)
        turns = int(feats[OFFSET_KUTAKATIA_TURNS])
        turn_bucket = 0 if turns <= 1 else 1
        out.append(KUTAKATIA_TURNS_BASE + turn_bucket)

    return out


def indices_batch(features: np.ndarray) -> np.ndarray:
    """Vectorised conversion: ``(N, FEATURE_LEN) → (N, MAX_ACTIVE)`` int32.

    Slot 0..k holds the active indices for row k; remaining slots are
    ``-1``. Use ``mask = result >= 0`` to ignore padding.
    """
    if features.ndim != 2 or features.shape[1] != FEATURE_LEN:
        raise ValueError(f"expected (N, {FEATURE_LEN}), got {features.shape}")
    n = features.shape[0]
    feats = features.astype(np.int32)
    out = np.full((n, MAX_ACTIVE), -1, dtype=np.int32)

    # Pit buckets: 32 indices per row, no padding.
    pit_counts = feats[:, :32]
    pit_buckets = np.digitize(pit_counts, _BUCKET_BOUNDS)
    pit_offsets = np.arange(32, dtype=np.int32) * 8
    out[:, :32] = PIT_BUCKETS_BASE + pit_offsets[None, :] + pit_buckets

    # Nyumba state
    out[:, 32] = NYUMBA_STATE_BASE + feats[:, OFFSET_OWN_NYUMBA_STATE]
    out[:, 33] = NYUMBA_STATE_BASE + 3 + feats[:, OFFSET_OPP_NYUMBA_STATE]

    # Phase/substate
    ps = feats[:, OFFSET_PHASE_SUBSTATE]
    out[:, 34] = PHASE_SUBSTATE_BASE + (ps >> 2) * 3 + (ps & 0b11)

    # Kutakatia: present-bit first, then optional follow-ups. We fill the
    # variable-length suffix per-row because slot-position depends on
    # which fields are active.
    present_mask = feats[:, OFFSET_KUTAKATIA_PRESENT] == 1
    rows = np.where(present_mask)[0]
    for i in rows:
        slot = 35
        out[i, slot] = KUTAKATIA_PRESENT_IDX
        slot += 1
        if feats[i, OFFSET_KUTAKATIA_OWN] == 1:
            out[i, slot] = KUTAKATIA_OWN_IDX
            slot += 1
        field = int(feats[i, OFFSET_KUTAKATIA_FIELD])
        if 0 <= field < 8:
            out[i, slot] = KUTAKATIA_FIELD_BASE + field
            slot += 1
        turns = int(feats[i, OFFSET_KUTAKATIA_TURNS])
        out[i, slot] = KUTAKATIA_TURNS_BASE + (0 if turns <= 1 else 1)

    return out
