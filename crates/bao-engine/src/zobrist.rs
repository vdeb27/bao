//! Zobrist hashing for BoardState — used as transposition-table keys by the
//! search. Pure-function `zobrist_key(&BoardState)` for now; incremental update
//! via `apply` is a deferred optimisation (see plan §3 and §10).
//!
//! Determinism: the random tables are generated lazily on first access from a
//! fixed seed (xorshift64), so two runs of the binary produce identical keys.
//! `variant`, `ply`, `winner` and per-side `nyumba_col` are intentionally not
//! mixed in: variant and nyumba_col are constants for a given game, ply is
//! useful as ordering information but not as identity, and the TT does not
//! store terminal positions.
use crate::board::{BoardState, Direction, Phase, Substate, PITS_PER_SIDE};
use std::sync::OnceLock;

const MAX_PIT_KETE: usize = 64;
const MAX_GHALA_KETE: usize = 64;
const SEED: u64 = 0x9E3779B97F4A7C15;

struct Keys {
    vichwa: Box<[[[u64; MAX_PIT_KETE + 1]; PITS_PER_SIDE]; 2]>,
    ghala: [[u64; MAX_GHALA_KETE + 1]; 2],
    nyumba_owned: [u64; 2],
    active: u64,
    phase_base: [u64; 2],
    substate_tag: [u64; 3],
    kichwa_field: [u64; 16],
    kichwa_prior_dir: [u64; 3],
    safari_sow_dir: [u64; 2],
    kutakatia_present: u64,
    kutakatia_field: [u64; 16],
    kutakatia_player: [u64; 2],
    kutakatia_turns: [u64; 4],
}

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

fn build_keys() -> Keys {
    let mut rng = Rng::new(SEED);

    // The full vichwa table is ~16 KiB; Box it so we don't blow the stack
    // when constructing in-place inside OnceLock.
    let mut vichwa: Box<[[[u64; MAX_PIT_KETE + 1]; PITS_PER_SIDE]; 2]> =
        Box::new([[[0u64; MAX_PIT_KETE + 1]; PITS_PER_SIDE]; 2]);
    for p in 0..2 {
        for pit in 0..PITS_PER_SIDE {
            for c in 0..=MAX_PIT_KETE {
                vichwa[p][pit][c] = rng.next_u64();
            }
        }
    }

    let mut ghala = [[0u64; MAX_GHALA_KETE + 1]; 2];
    for p in 0..2 {
        for c in 0..=MAX_GHALA_KETE {
            ghala[p][c] = rng.next_u64();
        }
    }

    let nyumba_owned = [rng.next_u64(), rng.next_u64()];
    let active = rng.next_u64();
    let phase_base = [rng.next_u64(), rng.next_u64()];
    let substate_tag = [rng.next_u64(), rng.next_u64(), rng.next_u64()];

    let mut kichwa_field = [0u64; 16];
    for v in kichwa_field.iter_mut() {
        *v = rng.next_u64();
    }
    let kichwa_prior_dir = [rng.next_u64(), rng.next_u64(), rng.next_u64()];
    let safari_sow_dir = [rng.next_u64(), rng.next_u64()];

    let kutakatia_present = rng.next_u64();
    let mut kutakatia_field = [0u64; 16];
    for v in kutakatia_field.iter_mut() {
        *v = rng.next_u64();
    }
    let kutakatia_player = [rng.next_u64(), rng.next_u64()];
    let kutakatia_turns = [
        rng.next_u64(),
        rng.next_u64(),
        rng.next_u64(),
        rng.next_u64(),
    ];

    Keys {
        vichwa,
        ghala,
        nyumba_owned,
        active,
        phase_base,
        substate_tag,
        kichwa_field,
        kichwa_prior_dir,
        safari_sow_dir,
        kutakatia_present,
        kutakatia_field,
        kutakatia_player,
        kutakatia_turns,
    }
}

fn keys() -> &'static Keys {
    static KEYS: OnceLock<Keys> = OnceLock::new();
    KEYS.get_or_init(build_keys)
}

fn dir_idx(dir: Direction) -> usize {
    match dir {
        Direction::Cw => 0,
        Direction::Ccw => 1,
    }
}

fn prior_dir_idx(dir: Option<Direction>) -> usize {
    match dir {
        None => 0,
        Some(Direction::Cw) => 1,
        Some(Direction::Ccw) => 2,
    }
}

/// Zobrist key for the given board state. Deterministic across runs and
/// independent of `ply` / `winner` / `variant`.
pub fn zobrist_key(state: &BoardState) -> u64 {
    let k = keys();
    let mut h = 0u64;
    for p in 0..2 {
        let side = &state.sides[p];
        for pit in 0..PITS_PER_SIDE {
            let c = (side.vichwa[pit] as usize).min(MAX_PIT_KETE);
            h ^= k.vichwa[p][pit][c];
        }
        let g = (side.ghala as usize).min(MAX_GHALA_KETE);
        h ^= k.ghala[p][g];
        if side.nyumba_owned {
            h ^= k.nyumba_owned[p];
        }
    }
    if state.active == 1 {
        h ^= k.active;
    }
    let (phase_idx, sub) = match state.phase {
        Phase::Namu(s) => (0, s),
        Phase::Mtaji(s) => (1, s),
    };
    h ^= k.phase_base[phase_idx];
    match sub {
        Substate::AwaitMove => {
            h ^= k.substate_tag[0];
        }
        Substate::AwaitKichwa {
            capture_field,
            prior_dir,
        } => {
            h ^= k.substate_tag[1];
            h ^= k.kichwa_field[(capture_field as usize) & 0xF];
            h ^= k.kichwa_prior_dir[prior_dir_idx(prior_dir)];
        }
        Substate::AwaitSafari { sow_dir } => {
            h ^= k.substate_tag[2];
            h ^= k.safari_sow_dir[dir_idx(sow_dir)];
        }
    }
    if let Some(kb) = state.kutakatia {
        h ^= k.kutakatia_present;
        h ^= k.kutakatia_field[(kb.blocked_field as usize) & 0xF];
        h ^= k.kutakatia_player[(kb.blocked_player as usize) & 1];
        h ^= k.kutakatia_turns[(kb.turns_remaining as usize).min(3)];
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::BoardState;
    use crate::rules::{apply, legal_moves};
    use crate::variant::Variant;

    #[test]
    fn deterministic_for_initial_position() {
        let s1 = BoardState::new(Variant::Kiswahili);
        let s2 = BoardState::new(Variant::Kiswahili);
        assert_eq!(zobrist_key(&s1), zobrist_key(&s2));
    }

    #[test]
    fn initial_positions_differ_between_variants() {
        let kis = BoardState::new(Variant::Kiswahili);
        let kuj = BoardState::new(Variant::Kujifunza);
        // Different vichwa, ghala, phase, nyumba_owned — keys must differ.
        assert_ne!(zobrist_key(&kis), zobrist_key(&kuj));
    }

    #[test]
    fn key_changes_after_apply() {
        let state = BoardState::new(Variant::Kiswahili);
        let initial = zobrist_key(&state);
        let mv = *legal_moves(&state).first().expect("legal move");
        let (next, _) = apply(&state, mv).expect("apply");
        assert_ne!(zobrist_key(&next), initial);
    }

    #[test]
    fn state_equality_implies_key_equality_over_random_rollout() {
        // Drive two parallel games with identical move choices; keys must
        // match step-for-step. This is the "pure-function" property test.
        let mut a = BoardState::new(Variant::Kiswahili);
        let mut b = BoardState::new(Variant::Kiswahili);
        let mut seed = 0x1234_5678_9ABC_DEF0u64;
        for _ in 0..200 {
            if a.winner.is_some() {
                break;
            }
            let moves = legal_moves(&a);
            if moves.is_empty() {
                break;
            }
            // xorshift64 to pick a move deterministically.
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let mv = moves[(seed as usize) % moves.len()];
            a = apply(&a, mv).expect("apply a").0;
            b = apply(&b, mv).expect("apply b").0;
            assert_eq!(a, b, "states diverged");
            assert_eq!(zobrist_key(&a), zobrist_key(&b), "keys diverged");
        }
    }

    #[test]
    fn keys_are_well_distributed_across_states() {
        // Smoke test: 256 random states should yield 256 unique keys with
        // overwhelming probability. Collision in u64 over 256 samples ≈
        // 256² / 2^65 ≈ 1.8e-15. A hit would indicate a real bug.
        use std::collections::HashSet;
        let mut keyset: HashSet<u64> = HashSet::new();
        let mut state = BoardState::new(Variant::Kiswahili);
        let mut seed = 0xDEAD_BEEF_CAFE_BABEu64;
        let mut samples = 0;
        while samples < 256 {
            if state.winner.is_some() {
                state = BoardState::new(Variant::Kiswahili);
            }
            let moves = legal_moves(&state);
            if moves.is_empty() {
                state = BoardState::new(Variant::Kiswahili);
                continue;
            }
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            let mv = moves[(seed as usize) % moves.len()];
            state = apply(&state, mv).expect("apply").0;
            keyset.insert(zobrist_key(&state));
            samples += 1;
        }
        // Allow a tiny number of natural collisions from re-visited states.
        // We expect at least ~95% uniqueness.
        assert!(
            keyset.len() >= 240,
            "keys collide too often: {} unique out of 256",
            keyset.len()
        );
    }

    #[test]
    fn key_is_stable_across_invocations() {
        // Calling the function repeatedly on the same state must yield the
        // same value (sanity: no hidden state, no time-dependence).
        let state = BoardState::new(Variant::Kiswahili);
        let k1 = zobrist_key(&state);
        for _ in 0..10 {
            assert_eq!(zobrist_key(&state), k1);
        }
    }

    #[test]
    fn ply_does_not_affect_key() {
        let mut a = BoardState::new(Variant::Kiswahili);
        let mut b = a;
        b.ply = 999;
        assert_eq!(zobrist_key(&a), zobrist_key(&b));
        // Use the variable to keep linters quiet.
        a.ply = 0;
        assert_eq!(zobrist_key(&a), zobrist_key(&b));
    }

    #[test]
    fn winner_does_not_affect_key() {
        let mut a = BoardState::new(Variant::Kiswahili);
        let mut b = a;
        b.winner = Some(1);
        assert_eq!(zobrist_key(&a), zobrist_key(&b));
        a.winner = Some(0);
        assert_eq!(zobrist_key(&a), zobrist_key(&b));
    }

}
