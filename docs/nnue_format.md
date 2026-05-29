# NNUE Format (v2, Iter-1)

**Version 2** (current): hidden weights are int16 at scale 64 (fp range
±512). v1 had int8 hidden weights which capped fp range at ±1.98 — too
tight for trained L3 weights that need to span ±50 for full ±8000 cent
output range. Bumping i8→i16 adds ~17 KB to the model. The loader rejects
v1 blobs.


## Feature transformer (280 features)

The raw byte features (`docs/feature_layout.md`, 80 bytes) are mapped to a
sparse-index set drawn from `0..280`. The mapping is **deterministic and
total**: for any legal `BoardState`, the encoder emits a fixed number of
active indices per group.

| Group | Range | Count | Active per state | Notes |
|-------|-------|-------|------------------|-------|
| Pit-buckets | 0..256 | 256 | 32 | 32 pits × 8 buckets, one bucket active per pit |
| Nyumba-state | 256..262 | 6 | 2 | own (256+s) + opp (259+s) |
| Phase×Substate | 262..268 | 6 | 1 | maps `phase_substate` byte to dense 0..6 idx |
| Kutakatia | 268..280 | 12 | 1..4 | see breakdown below |

**Kutakatia sub-layout** (12 indices, 268..280):
- 268: present-bit (1 if kutakatia active)
- 269: blocked_player_is_own
- 270..278: blocked_field one-hot (8 indices, position 0..7 on blocked side)
- 278..280: turns_remaining buckets (0-1 → 278, 2-3 → 279)

When `kutakatia` is absent only index 268 is omitted (no active kutakatia
indices); when present indices 269+field+turn-bucket are active.

### Bucket boundaries

Pit-buckets (mirror of `training/bao_train/nnue/features.py::PIT_BUCKETS`):

| Bucket | Range |
|--------|-------|
| 0 | count = 0 |
| 1 | count = 1 |
| 2 | count = 2 |
| 3 | count = 3 |
| 4 | count = 4..5 |
| 5 | count = 6..8 |
| 6 | count = 9..15 |
| 7 | count ≥ 16 |

Pit-index ordering: 0..16 = own.vichwa (mbele L→R, then nyuma L→R), 16..32 =
opp.vichwa. Pit `p` (0..32) gets active index `p*8 + bucket(count[p])`.

### Active-count

Total active indices per state: **32 (pits) + 2 (nyumba) + 1 (phase) +
{0 or 3} (kutakatia)** = 35 or 38.

## Architecture

```
sparse-indices (35..38 active in 0..280)
  → Linear(280 → 512)          accumulator, int16
  → ClippedReLU (clip = 127)
  → Linear(512 → 32)           int8 weights, int32 bias
  → ClippedReLU
  → Linear(32 → 32)            int8 weights, int32 bias
  → ClippedReLU
  → Linear(32 → 1)             int8 weights, int32 bias
  → scaled to centi-kete score
```

### Quantisation constants

| Constant | Value | Where |
|----------|-------|-------|
| `WEIGHT_SCALE_L0` | 64 | int16 = round(fp32 * 64); accumulator |
| `WEIGHT_SCALE_HIDDEN` | 64 | int8 = round(fp32 * 64) for L1..L3 |
| `ACTIVATION_CLIP` | 127 | ClippedReLU upper bound |
| `OUTPUT_SCALE` | 16 | divides L3-output to yield centi-kete (model trains raw outputs ~16× labels) |

Final score: `score_centikete = clamp(rdiv(L3_output, OUTPUT_SCALE), ±LABEL_CLIP)`.

**Rounding**: all per-layer rescales use round-half-away-from-zero (`rdiv` in
`loader.rs`, `_round_div` in `quantised_forward.py`). Truncation toward zero
biases activations downward after ClippedReLU (everything ≥0 floors), giving
a systematic ~115 cp negative drift on iter-1. Round-to-nearest restores
symmetry (drift mean 22 cp on the same model).

## Bin layout

```
offset  size       field
0       8          magic "BAONNUE\0"
8       2          version: u16 LE (= 1)
10      2          n_features: u16 LE (= 280)
12      8          hidden_sizes: [u16;4] LE (= 512, 32, 32, 1)
20      4          quant_scale: f32 LE (OUTPUT_SCALE)
24      280*512*2  layer_0_weights: i16 LE, row-major [n_features × 512]
...     512*2      layer_0_bias: i16 LE
...     512*32     layer_1_weights: i8, row-major [512 × 32]
...     32*4       layer_1_bias: i32 LE
...     32*32      layer_2_weights: i8, row-major [32 × 32]
...     32*4       layer_2_bias: i32 LE
...     32*1       layer_3_weights: i8
...     1*4        layer_3_bias: i32 LE
```

Total size: 8+2+2+8+4 + 280*512*2 + 512*2 + 512*32 + 32*4 + 32*32 + 32*4 +
32 + 4 ≈ **305 KB**. The accumulator dominates; if we squeeze L0 weights to
int8 the model drops below 160 KB but accuracy may suffer — defer to a later
format version.

## Stability

Bumping `n_features`, `hidden_sizes`, or any layout offset requires a new
version. The Rust loader rejects unknown versions.

## NPS

Baseline (iter-1, scalar i32 forward, `target-cpu=native`): **~118k full
re-eval NPS** on a 16-core desktop. Plan §11 target was ≥200k; the gap is
the L0 accumulator (32 active indices × 512 i16 adds) plus the L1 matmul
(512 × 32 i8 mul-add). Path to target: pack weights for AVX2 i16 SIMD on
load, vectorise `forward_raw`. Deferred to a later iteration; the search
hit-rate from TT raises effective NPS in real games.

## Roundtrip invariants

- Given a `.nnue` written by `nnue/export.py` and a `BoardState`,
  `eval::nnue::evaluate(state)` and `nnue/architecture.py::forward_quantised(state)`
  must agree to within ±1 centi-kete.
- Sparse-index sets must be bytewise identical between
  `bao_engine::nnue::transformer::indices(state)` and
  `nnue/transformer.py::indices(features)`.
