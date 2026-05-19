//! Dense feature encoder for the NNUE training pipeline.
//!
//! See `docs/feature_layout.md` for the byte-level spec. The encoder is the
//! single source of truth shared between Rust (shard writer) and Python
//! (shard reader → NNUE feature transformer).
//!
//! **Perspective flip**: features are always emitted from `state.active`'s
//! viewpoint. When `active == 1` the encoder swaps `sides[0]` and `sides[1]`
//! before writing, so an evaluator trained on this layout only ever sees one
//! geometric orientation.
//!
//! Format version is paired with `shard_format.md` v1. Any field reshuffling
//! or `FEATURE_LEN` change requires bumping the shard version too.
//!
//! The encoder stores **raw counts**, not one-hot buckets — the Python-side
//! NNUE input transformer is free to re-bucket without regenerating shards.

use crate::board::{BoardState, NyumbaState, Phase, Side, Substate, PITS_PER_SIDE};
use crate::variant::Variant;

/// Bytes per encoded position. See `docs/feature_layout.md`.
pub const FEATURE_LEN: usize = 80;

/// Label clip range (centi-kete). Matches `docs/shard_format.md`.
pub const LABEL_CLIP: i32 = 8000;

fn nyumba_state_byte(state: NyumbaState) -> u8 {
    match state {
        NyumbaState::Functional => 0,
        NyumbaState::Disabled => 1,
        NyumbaState::Destroyed => 2,
    }
}

fn phase_substate_byte(phase: Phase) -> u8 {
    let (p, s) = match phase {
        Phase::Namu(s) => (0u8, s),
        Phase::Mtaji(s) => (1u8, s),
    };
    let sub = match s {
        Substate::AwaitMove => 0u8,
        Substate::AwaitKichwa { .. } => 1,
        Substate::AwaitSafari { .. } => 2,
    };
    (p << 2) | sub
}

fn write_side(out: &mut [u8; FEATURE_LEN], offset: usize, side: &Side) {
    out[offset..offset + PITS_PER_SIDE].copy_from_slice(&side.vichwa);
}

/// Encode `state` into a fixed-size feature vector from the active player's
/// perspective. The encoding is deterministic — same state always produces
/// the same bytes.
pub fn encode_features(state: &BoardState) -> [u8; FEATURE_LEN] {
    let mut out = [0u8; FEATURE_LEN];

    let active = state.active as usize;
    let opp = 1 - active;
    let own_side = &state.sides[active];
    let opp_side = &state.sides[opp];

    // 0..16  own.vichwa
    // 16..32 opp.vichwa
    write_side(&mut out, 0, own_side);
    write_side(&mut out, 16, opp_side);

    // 32, 33 ghala
    out[32] = own_side.ghala;
    out[33] = opp_side.ghala;

    // 34, 35 nyumba_state
    out[34] = nyumba_state_byte(own_side.nyumba_state(state.variant));
    out[35] = nyumba_state_byte(opp_side.nyumba_state(state.variant));

    // 36, 37 nyumba_col
    out[36] = own_side.nyumba_col;
    out[37] = opp_side.nyumba_col;

    // 38 phase_substate
    out[38] = phase_substate_byte(state.phase);

    // 39..43 kutakatia
    match state.kutakatia {
        Some(k) => {
            out[39] = 1;
            out[40] = if k.blocked_player as usize == active { 1 } else { 0 };
            out[41] = k.blocked_field;
            out[42] = k.turns_remaining;
        }
        None => {
            out[39] = 0;
            out[40] = 0;
            out[41] = 255;
            out[42] = 0;
        }
    }

    // 43 variant
    out[43] = match state.variant {
        Variant::Kiswahili => 0,
        Variant::Kujifunza => 1,
    };

    // 44..80 reserved zeros (already initialised)
    out
}

/// Clip a raw eval score (centi-kete) to the i16 label range. Mate-scores
/// collapse to ±LABEL_CLIP — the trainer can downweight saturated labels
/// on load if desired.
pub fn clip_label(score: i32) -> i16 {
    score.clamp(-LABEL_CLIP, LABEL_CLIP) as i16
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{apply, legal_moves};

    #[test]
    fn encode_length_is_fixed() {
        let s = BoardState::new(Variant::Kiswahili);
        let f = encode_features(&s);
        assert_eq!(f.len(), FEATURE_LEN);
    }

    #[test]
    fn reserved_bytes_are_zero() {
        let s = BoardState::new(Variant::Kiswahili);
        let f = encode_features(&s);
        for (i, b) in f[44..].iter().enumerate() {
            assert_eq!(*b, 0, "reserved byte {} = {}", 44 + i, b);
        }
    }

    #[test]
    fn kete_sum_invariant_in_features() {
        let s = BoardState::new(Variant::Kiswahili);
        let f = encode_features(&s);
        let sum: u32 = f[0..32].iter().map(|&b| b as u32).sum::<u32>()
            + f[32] as u32
            + f[33] as u32;
        assert_eq!(sum, 64);
    }

    #[test]
    fn perspective_flip_when_active_changes() {
        // After one ply the active side flips. The first-32-byte view should
        // then be the *new* active player's sides — which is the old opp.
        let s = BoardState::new(Variant::Kiswahili);
        let f0 = encode_features(&s);
        let moves = legal_moves(&s);
        let mv = *moves.first().expect("initial position has moves");
        let (s1, _) = apply(&s, mv).expect("first move legal");
        // Only test if the turn actually changed (not a substate continuation).
        if s1.active != s.active {
            let f1 = encode_features(&s1);
            // After flip, f1's "opp" (bytes 16..32) must be the side that
            // just played. The initial position is symmetric so the *content*
            // of f1[0..16] may coincide with f0[0..16] — we instead verify
            // structural identity: f1's opp-view equals s.sides[0] mutated by
            // the move, and f1's own-view equals s.sides[1] unchanged.
            assert_eq!(f1[0..16], s1.sides[1].vichwa);
            assert_eq!(f1[16..32], s1.sides[0].vichwa);
            let sum: u32 = f1[0..32].iter().map(|&b| b as u32).sum::<u32>()
                + f1[32] as u32
                + f1[33] as u32;
            assert_eq!(sum, 64);
        }
    }

    #[test]
    fn deterministic_encoding() {
        let s = BoardState::new(Variant::Kiswahili);
        let a = encode_features(&s);
        let b = encode_features(&s);
        assert_eq!(a, b);
    }

    #[test]
    fn clip_label_bounds() {
        assert_eq!(clip_label(0), 0);
        assert_eq!(clip_label(LABEL_CLIP), LABEL_CLIP as i16);
        assert_eq!(clip_label(LABEL_CLIP + 1), LABEL_CLIP as i16);
        assert_eq!(clip_label(-LABEL_CLIP - 100), -LABEL_CLIP as i16);
        assert_eq!(clip_label(crate::search::MATE_SCORE), LABEL_CLIP as i16);
        assert_eq!(clip_label(-crate::search::MATE_SCORE), -LABEL_CLIP as i16);
    }
}
