//! NNUE sparse-feature transformer.
//!
//! Maps the 80-byte dense feature vector (`docs/feature_layout.md`) to a
//! sparse set of active indices in `0..N_FEATURES = 280`. See
//! `docs/nnue_format.md` for the index-group layout. This module is the
//! single source of truth shared with the Python trainer
//! (`training/bao_train/nnue/transformer.py`); a roundtrip test asserts
//! bytewise equality between the two.

use crate::features::{encode_features, FEATURE_LEN};
use crate::board::BoardState;

pub const N_FEATURES: usize = 280;

pub const PIT_BUCKETS_BASE: u16 = 0;
pub const NYUMBA_STATE_BASE: u16 = 256;
pub const PHASE_SUBSTATE_BASE: u16 = 262;
pub const KUTAKATIA_BASE: u16 = 268;

pub const KUTAKATIA_PRESENT: u16 = KUTAKATIA_BASE; // 268
pub const KUTAKATIA_OWN: u16 = KUTAKATIA_BASE + 1; // 269
pub const KUTAKATIA_FIELD_BASE: u16 = KUTAKATIA_BASE + 2; // 270..278
pub const KUTAKATIA_TURNS_BASE: u16 = KUTAKATIA_BASE + 10; // 278..280

pub const MAX_ACTIVE: usize = 32 + 2 + 1 + 4; // 39

/// Map a raw kete count to its bucket index (0..7).
#[inline]
pub fn bucket(count: u8) -> u8 {
    match count {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4..=5 => 4,
        6..=8 => 5,
        9..=15 => 6,
        _ => 7,
    }
}

/// Map a phase_substate byte (`features[38]`) to its dense index 0..6.
///
/// The dense feature byte uses `(phase << 2) | sub`, giving the value set
/// `{0,1,2,4,5,6}` (no value 3). We compact that to `0..6`.
#[inline]
fn phase_substate_dense(byte: u8) -> u16 {
    let phase = byte >> 2;
    let sub = byte & 0b11;
    (phase as u16) * 3 + (sub as u16)
}

/// Emit active sparse indices for the given dense feature byte-vector.
///
/// Indices are written in ascending group order; within a group they are
/// already monotonic. The returned vector length is between 35 and 39.
pub fn indices_from_features(features: &[u8; FEATURE_LEN]) -> Vec<u16> {
    let mut out: Vec<u16> = Vec::with_capacity(MAX_ACTIVE);

    for p in 0..32usize {
        let b = bucket(features[p]) as u16;
        out.push(PIT_BUCKETS_BASE + (p as u16) * 8 + b);
    }

    let own_ns = features[34] as u16;
    let opp_ns = features[35] as u16;
    out.push(NYUMBA_STATE_BASE + own_ns);
    out.push(NYUMBA_STATE_BASE + 3 + opp_ns);

    out.push(PHASE_SUBSTATE_BASE + phase_substate_dense(features[38]));

    if features[39] == 1 {
        out.push(KUTAKATIA_PRESENT);
        if features[40] == 1 {
            out.push(KUTAKATIA_OWN);
        }
        let field = features[41];
        if field < 8 {
            out.push(KUTAKATIA_FIELD_BASE + field as u16);
        }
        let turns = features[42];
        let turn_bucket = if turns <= 1 { 0 } else { 1 };
        out.push(KUTAKATIA_TURNS_BASE + turn_bucket);
    }

    out
}

/// Convenience wrapper: encode + index.
pub fn indices(state: &BoardState) -> Vec<u16> {
    let feats = encode_features(state);
    indices_from_features(&feats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::BoardState;
    use crate::rules::{apply, legal_moves};
    use crate::variant::Variant;

    #[test]
    fn bucket_boundaries() {
        assert_eq!(bucket(0), 0);
        assert_eq!(bucket(1), 1);
        assert_eq!(bucket(2), 2);
        assert_eq!(bucket(3), 3);
        assert_eq!(bucket(4), 4);
        assert_eq!(bucket(5), 4);
        assert_eq!(bucket(6), 5);
        assert_eq!(bucket(8), 5);
        assert_eq!(bucket(9), 6);
        assert_eq!(bucket(15), 6);
        assert_eq!(bucket(16), 7);
        assert_eq!(bucket(64), 7);
    }

    #[test]
    fn indices_in_range_and_monotone() {
        let s = BoardState::new(Variant::Kiswahili);
        let idx = indices(&s);
        assert!(idx.len() >= 35 && idx.len() <= 39, "len = {}", idx.len());
        for &i in idx.iter() {
            assert!((i as usize) < N_FEATURES, "idx {} out of range", i);
        }
        // Strictly increasing because groups are emitted in ascending base order.
        for w in idx.windows(2) {
            assert!(w[0] < w[1], "not monotone: {} >= {}", w[0], w[1]);
        }
    }

    #[test]
    fn initial_position_has_no_kutakatia() {
        let s = BoardState::new(Variant::Kiswahili);
        let idx = indices(&s);
        // No kutakatia in initial position → exactly 35 indices.
        assert_eq!(idx.len(), 35);
        // No index ≥ 268.
        assert!(idx.iter().all(|&i| (i as usize) < KUTAKATIA_BASE as usize));
    }

    #[test]
    fn phase_substate_dense_maps_correctly() {
        // Namu/AwaitMove → 0
        assert_eq!(phase_substate_dense(0b00_00), 0);
        // Namu/AwaitKichwa → 1
        assert_eq!(phase_substate_dense(0b00_01), 1);
        // Namu/AwaitSafari → 2
        assert_eq!(phase_substate_dense(0b00_10), 2);
        // Mtaji/AwaitMove (byte = 0b01_00 = 4) → 3
        assert_eq!(phase_substate_dense(0b01_00), 3);
        // Mtaji/AwaitKichwa → 4
        assert_eq!(phase_substate_dense(0b01_01), 4);
        // Mtaji/AwaitSafari → 5
        assert_eq!(phase_substate_dense(0b01_10), 5);
    }

    #[test]
    fn deterministic_across_self_play() {
        // After a handful of plies indices() must still be in range, unique,
        // and equal when re-encoded.
        let mut s = BoardState::new(Variant::Kiswahili);
        for _ in 0..20 {
            let moves = legal_moves(&s);
            if moves.is_empty() {
                break;
            }
            let mv = moves[0];
            let next = match apply(&s, mv) {
                Ok((n, _)) => n,
                Err(_) => break,
            };
            s = next;
            let a = indices(&s);
            let b = indices(&s);
            assert_eq!(a, b);
            for &i in a.iter() {
                assert!((i as usize) < N_FEATURES);
            }
        }
    }
}
