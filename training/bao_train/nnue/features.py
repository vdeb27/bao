"""Feature layout mirror for the training-side dataset loader.

Single source of truth: ``docs/feature_layout.md`` and Rust
``bao_engine::features``. Constants and field offsets here MUST match the
encoder byte-for-byte; the ``test_features_consistency.py`` integration test
asserts this against the Rust PyO3 binding.

The shards store **raw counts** per slot; this module is where we convert
those into the **sparse one-hot indices** the NNUE feature transformer
ingests. Keeping the bucketisation here (and not in the shard) means we can
re-tune bucket boundaries without regenerating data.
"""

from __future__ import annotations

import numpy as np

FEATURE_LEN = 80

# Field offsets (must match docs/feature_layout.md exactly)
OFFSET_OWN_VICHWA = 0       # 16 bytes
OFFSET_OPP_VICHWA = 16      # 16 bytes
OFFSET_OWN_GHALA = 32
OFFSET_OPP_GHALA = 33
OFFSET_OWN_NYUMBA_STATE = 34
OFFSET_OPP_NYUMBA_STATE = 35
OFFSET_OWN_NYUMBA_COL = 36
OFFSET_OPP_NYUMBA_COL = 37
OFFSET_PHASE_SUBSTATE = 38
OFFSET_KUTAKATIA_PRESENT = 39
OFFSET_KUTAKATIA_OWN = 40
OFFSET_KUTAKATIA_FIELD = 41
OFFSET_KUTAKATIA_TURNS = 42
OFFSET_VARIANT = 43
# bytes 44..80 reserved

# Bucket boundaries for pit-count one-hot encoding.
# 8 buckets: [0], [1], [2], [3], [4-5], [6-8], [9-15], [>=16].
PIT_BUCKETS = (0, 1, 2, 3, 4, 6, 9, 16)


def bucket_count(count: int) -> int:
    """Map a raw kete count to its bucket index (0..7)."""
    if count <= 0:
        return 0
    if count == 1:
        return 1
    if count == 2:
        return 2
    if count == 3:
        return 3
    if count <= 5:
        return 4
    if count <= 8:
        return 5
    if count <= 15:
        return 6
    return 7


def pit_bucket_indices(features: np.ndarray) -> np.ndarray:
    """Vectorised bucket-id lookup for the 32 pits.

    Input: ``(N, FEATURE_LEN)`` uint8 array (or ``(FEATURE_LEN,)``).
    Output: ``(N, 32)`` uint8 with values in 0..7.
    """
    if features.ndim == 1:
        features = features[np.newaxis, :]
    counts = features[:, OFFSET_OWN_VICHWA:OFFSET_OWN_VICHWA + 32].astype(np.int32)
    # boundaries: 1, 2, 3, 4, 6, 9, 16 → bucket 0..7
    bounds = np.array([1, 2, 3, 4, 6, 9, 16], dtype=np.int32)
    # np.digitize returns the index of the first bound > count, which lines
    # up with our 8-bucket scheme.
    return np.digitize(counts, bounds).astype(np.uint8)


def kete_sum(features: np.ndarray) -> np.ndarray:
    """Total kete count per row. Must equal 64 for non-mid-substate positions."""
    if features.ndim == 1:
        features = features[np.newaxis, :]
    pits = features[:, :32].sum(axis=1)
    ghalas = features[:, 32:34].sum(axis=1)
    return pits + ghalas


def is_perspective_flipped(features: np.ndarray) -> bool:
    """The encoder always emits from active-player perspective. This is a
    structural assertion: there's no 'flipped' bit because every row is
    already in the active POV. Provided for parity with `docs/feature_layout.md`.
    """
    _ = features
    return True
