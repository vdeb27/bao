# Feature Layout (Iter-1)

Encoder: `bao_engine::features::encode_features(state) -> [u8; FEATURE_LEN]`.
Layout is **perspective-flipped**: when `state.active == 1` (North), sides are
swapped so the encoder always sees "active player" in the first slots.
Consequence: an evaluator trained on this layout sees one geometric
orientation only — mirror-symmetric to the other side by construction.

`FEATURE_LEN = 80` bytes per position. We store **raw counts**, not one-hot
bucketing — the NNUE feature transformer in Python is free to re-bucket
without regenerating shards. Each byte ≤ 64 (kete-sum invariant), most ≤ 16.

| Offset | Len | Field | Notes |
|--------|-----|-------|-------|
| 0..16 | 16 | own.vichwa | own perspective, idx 0..7 = mbele L→R, 8..15 = nyuma L→R |
| 16..32 | 16 | opp.vichwa | opp perspective same layout |
| 32 | 1 | own.ghala | 0..64 |
| 33 | 1 | opp.ghala | 0..64 |
| 34 | 1 | own.nyumba_state | 0=Functional, 1=Disabled, 2=Destroyed |
| 35 | 1 | opp.nyumba_state | same |
| 36 | 1 | own.nyumba_col | 0..7, derives sow-direction effects |
| 37 | 1 | opp.nyumba_col | 0..7 |
| 38 | 1 | phase_substate | (Phase<<2 \| Substate); 0=Namu/AwaitMove, 1=Namu/AwaitKichwa, 2=Namu/AwaitSafari, 4=Mtaji/AwaitMove, 5=Mtaji/AwaitKichwa, 6=Mtaji/AwaitSafari |
| 39 | 1 | kutakatia_present | 0 or 1 |
| 40 | 1 | kutakatia_player_is_own | 1 if own side is blocked, 0 if opp, ignored if !present |
| 41 | 1 | kutakatia_field | 0..7 in blocked-side perspective; 255 if !present |
| 42 | 1 | kutakatia_turns_remaining | 0..3 |
| 43 | 1 | variant | 0=Kiswahili, 1=Kujifunza |
| 44..80 | 36 | reserved (zeros) | future: legal-move-count buckets, mobility, etc. |

## Perspective-flip rule

```
if state.active == 0:
    own = sides[0]; opp = sides[1]
else:
    own = sides[1]; opp = sides[0]   # encoder swaps before emitting
```

The active player is **never** stored as a feature because the encoding
already encodes their viewpoint.

## Invariants

- `sum(bytes 0..32) + bytes[32] + bytes[33] == 64` (kete-sum)
- All bytes ≤ 64
- `bytes[44..80] == 0` (reserved padding; future-compat marker)

## Stability

Bumping `FEATURE_LEN` or reshuffling fields requires a new shard format
version. Adding interpretation to reserved bytes does **not** — readers
default-zero those slots if they don't understand them.
