# Shard Format (Iter-1)

Binary container for `(feature_vector, label)` pairs. Single source of truth
between Rust generator and Python loader. All multi-byte fields are
**little-endian**.

```
+----------------------+
| Header (32 bytes)    |
+----------------------+
| Records (N × stride) |
+----------------------+
```

## Header

| Offset | Len | Field | Value |
|--------|-----|-------|-------|
| 0 | 8 | magic | ASCII `"BAOSHRD\0"` |
| 8 | 2 | version | u16, currently `1` |
| 10 | 2 | feature_len | u16, currently `80` (see feature_layout.md) |
| 12 | 2 | label_bytes | u16, currently `2` (int16) |
| 14 | 2 | record_stride | u16 = feature_len + label_bytes = `82` |
| 16 | 4 | n_records | u32 |
| 20 | 1 | label_dtype | `1`=i16 |
| 21 | 1 | label_unit | `1`=centi-kete |
| 22 | 1 | engine_major | `bao_engine` CARGO_PKG_VERSION major (currently 0) |
| 23 | 1 | engine_minor | minor (currently 0) |
| 24 | 8 | reserved | zeros |

## Record

```
[feature_len bytes : u8 features][label_bytes : i16 LE]
```

## Label semantics

- **Unit**: centi-kete from the **perspective of `state.active` at encode
  time** (matches the feature-encoder's flip).
- **Range**: clipped to ±`LABEL_CLIP = 8000` so all values fit in i16 with
  margin. Mate scores compress to ±`LABEL_CLIP`; the trainer can re-clip on
  load.
- **NaN/Inf**: not representable — generator must filter terminal states
  (no legal moves) before labeling.

## Roundtrip invariants

For any shard `S`:
- `read_header(write_header(h)) == h`
- `read_records(write_shard(records))` byte-identical
- Python `np.frombuffer(file_bytes[32:], dtype=record_dtype)` produces the
  same struct array Rust wrote

## Throughput

Plan §11 target is ≥2k pos/sec/core. On this hardware single-core hits
~918 pos/sec at depth 8 / 25k node cap (May 2026 bench). The generator
therefore runs **multi-threaded via rayon**; 8 cores comfortably exceed
the target. Single-core throughput is a soft regression check, not a
hard gate.

## Versioning

Bump `version` when the **layout** changes (feature_len, stride, dtype).
Adding reserved-byte interpretation does NOT bump version. Readers must
reject `version > supported_version` to avoid silently-wrong reads.
