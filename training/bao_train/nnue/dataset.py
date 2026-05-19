"""Shard reader for the binary training-data format.

Format spec: ``docs/shard_format.md``. The reader uses ``np.memmap`` so we
can stream multi-GB datasets without copying. Each record is
``FEATURE_LEN + 2 = 82`` bytes; the loader exposes both the raw feature
view and the int16 labels as zero-copy numpy arrays.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import numpy as np

from .features import FEATURE_LEN

MAGIC = b"BAOSHRD\0"
SHARD_VERSION = 1
HEADER_LEN = 32
LABEL_BYTES = 2
RECORD_STRIDE = FEATURE_LEN + LABEL_BYTES  # = 82
LABEL_DTYPE_I16 = 1
LABEL_UNIT_CENTIKETE = 1


@dataclass(frozen=True)
class ShardHeader:
    version: int
    feature_len: int
    label_bytes: int
    record_stride: int
    n_records: int
    label_dtype: int
    label_unit: int
    engine_major: int
    engine_minor: int

    @classmethod
    def parse(cls, header_bytes: bytes) -> "ShardHeader":
        if len(header_bytes) < HEADER_LEN:
            raise ValueError(f"shard header truncated: {len(header_bytes)} bytes")
        if header_bytes[0:8] != MAGIC:
            raise ValueError(f"bad shard magic: {header_bytes[0:8]!r}")
        version = int.from_bytes(header_bytes[8:10], "little")
        if version > SHARD_VERSION:
            raise ValueError(f"unsupported shard version {version} (max {SHARD_VERSION})")
        feature_len = int.from_bytes(header_bytes[10:12], "little")
        if feature_len != FEATURE_LEN:
            raise ValueError(f"feature_len mismatch: shard={feature_len}, expected={FEATURE_LEN}")
        record_stride = int.from_bytes(header_bytes[14:16], "little")
        if record_stride != RECORD_STRIDE:
            raise ValueError(f"record_stride mismatch: shard={record_stride}, expected={RECORD_STRIDE}")
        return cls(
            version=version,
            feature_len=feature_len,
            label_bytes=int.from_bytes(header_bytes[12:14], "little"),
            record_stride=record_stride,
            n_records=int.from_bytes(header_bytes[16:20], "little"),
            label_dtype=header_bytes[20],
            label_unit=header_bytes[21],
            engine_major=header_bytes[22],
            engine_minor=header_bytes[23],
        )


class Shard:
    """Memory-mapped read-only view of one shard file."""

    def __init__(self, path: str | Path) -> None:
        self.path = Path(path)
        with self.path.open("rb") as f:
            header_bytes = f.read(HEADER_LEN)
        self.header = ShardHeader.parse(header_bytes)
        size = self.path.stat().st_size
        records_bytes = size - HEADER_LEN
        # Tolerate generators that wrote fewer records than the header
        # claimed (e.g. interrupted runs). The on-disk byte-count is
        # authoritative.
        actual_n = records_bytes // RECORD_STRIDE
        if actual_n != self.header.n_records:
            # Don't fail — generators may legitimately write fewer than
            # the header announced. Use the actual count.
            self._n = actual_n
        else:
            self._n = self.header.n_records
        self._mm = np.memmap(
            self.path,
            dtype=np.uint8,
            mode="r",
            offset=HEADER_LEN,
            shape=(self._n, RECORD_STRIDE),
        )

    def __len__(self) -> int:
        return self._n

    def features(self) -> np.ndarray:
        """``(N, FEATURE_LEN)`` uint8 view, zero-copy."""
        return self._mm[:, :FEATURE_LEN]

    def labels(self) -> np.ndarray:
        """``(N,)`` int16 array, copied out of the memmap.

        Copying is fine: 5M labels × 2 bytes = 10 MB, negligible. A
        zero-copy view would need a structured-dtype memmap which we keep
        for a follow-up if profiler points here.
        """
        label_bytes = np.ascontiguousarray(self._mm[:, FEATURE_LEN:FEATURE_LEN + 2])
        return label_bytes.view(np.int16).reshape(self._n)


def open_shard(path: str | Path) -> Shard:
    """Convenience wrapper."""
    return Shard(path)
