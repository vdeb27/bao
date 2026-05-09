//! Rules: legal-move generation, sowing, capture, endelea, nyumba mechanics,
//! phase transitions, win detection. Each branch references a RULES.md
//! section. This module currently implements move generation only; sowing
//! and apply() come in a follow-up step.

use crate::board::{
    next_pit, BoardState, Direction, NyumbaState, Phase, Side, Substate, MBELE_LEN, PITS_PER_SIDE,
};
use crate::moves::{KichwaSide, Move};
use crate::variant::Variant;

/// Field where a sow of `count` kete starting at `start` and going in `dir`
/// will drop its last kete. Wraps around the 16-pit ring. See RULES.md §1.3.
pub fn landing(start: u8, dir: Direction, count: u8) -> u8 {
    let mut p = start;
    for _ in 0..count {
        p = next_pit(p, dir);
    }
    p
}

/// All legal moves in the current substate. Empty result implies the active
/// player has lost (handled by win detection in apply()).
pub fn legal_moves(state: &BoardState) -> Vec<Move> {
    match state.phase {
        Phase::Namu(Substate::AwaitMove) => namu_legal_moves(state),
        Phase::Mtaji(Substate::AwaitMove) => mtaji_legal_moves(state),
        Phase::Namu(Substate::AwaitKichwa {
            capture_field,
            prior_dir,
        })
        | Phase::Mtaji(Substate::AwaitKichwa {
            capture_field,
            prior_dir,
        }) => kichwa_legal_moves(capture_field, prior_dir),
        Phase::Namu(Substate::AwaitSafari) | Phase::Mtaji(Substate::AwaitSafari) => {
            vec![Move::Safari { go: true }, Move::Safari { go: false }]
        }
    }
}

// ---------- Namu (Kiswahili only) -------------------------------------------------

fn namu_legal_moves(state: &BoardState) -> Vec<Move> {
    debug_assert!(matches!(state.variant, Variant::Kiswahili));
    let own = state.active_side();
    let opp = state.opponent_side();

    // Mandatory-kula: try captures first, only fall back to non-captures.
    let captures = namu_captures(own, opp);
    if !captures.is_empty() {
        return captures;
    }
    namu_non_captures(own, state.variant)
}

/// RULES.md §6.2: namu-kula valid at mbele col `c` when own.mbele[c] >= 1
/// (pre-drop) AND opp.mbele[c] >= 1.
fn namu_captures(own: &Side, opp: &Side) -> Vec<Move> {
    let mut out = Vec::new();
    for c in 0..MBELE_LEN as u8 {
        if own.vichwa[c as usize] >= 1 && opp.vichwa[c as usize] >= 1 {
            out.push(Move::Namu { col: c });
        }
    }
    out
}

/// RULES.md §8.5: three-branch nyumba-conditional non-capture selection.
fn namu_non_captures(own: &Side, variant: Variant) -> Vec<Move> {
    let mut out = Vec::new();
    let nyumba_state = own.nyumba_state(variant);
    let nyumba_col = own.nyumba_col;

    match nyumba_state {
        NyumbaState::Functional => {
            // Branch 2: any non-empty mbele EXCEPT nyumba; if none such,
            // the nyumba itself (then tax-rule applies).
            let mut found_other = false;
            for c in 0..MBELE_LEN as u8 {
                if c == nyumba_col {
                    continue;
                }
                if own.vichwa[c as usize] >= 1 {
                    out.push(Move::Namu { col: c });
                    found_other = true;
                }
            }
            if !found_other && own.vichwa[nyumba_col as usize] >= 1 {
                out.push(Move::Namu { col: nyumba_col });
            }
        }
        NyumbaState::Disabled => {
            // Branch 3: any non-empty mbele, no restriction.
            for c in 0..MBELE_LEN as u8 {
                if own.vichwa[c as usize] >= 1 {
                    out.push(Move::Namu { col: c });
                }
            }
        }
        NyumbaState::Destroyed => {
            // Branch 1: prefer mbele with >=2 kete; only fall back to >=1
            // if no >=2 exists.
            let mut twos = Vec::new();
            for c in 0..MBELE_LEN as u8 {
                if own.vichwa[c as usize] >= 2 {
                    twos.push(Move::Namu { col: c });
                }
            }
            if !twos.is_empty() {
                return twos;
            }
            for c in 0..MBELE_LEN as u8 {
                if own.vichwa[c as usize] >= 1 {
                    out.push(Move::Namu { col: c });
                }
            }
        }
    }
    out
}

// ---------- Mtaji ----------------------------------------------------------------

fn mtaji_legal_moves(state: &BoardState) -> Vec<Move> {
    let own = state.active_side();
    let opp = state.opponent_side();

    let captures = mtaji_captures(own, opp);
    if !captures.is_empty() {
        // TODO RULES.md §11: when active player is the kutakatia-blocker and
        // the blocked field is among capturable fields, restrict to that one.
        return captures;
    }
    mtaji_takata(own)
}

/// RULES.md §6.1 + §4 16-seeds-no-capture: source has 2..=15 kete; landing
/// must be in own mbele (idx 0..7); own.mbele[land] >= 1 (pre-drop) AND
/// opp.mbele[land] >= 1.
fn mtaji_captures(own: &Side, opp: &Side) -> Vec<Move> {
    let mut out = Vec::new();
    for pit in 0..PITS_PER_SIDE as u8 {
        let count = own.vichwa[pit as usize];
        if !(2..=15).contains(&count) {
            continue;
        }
        for &dir in &[Direction::Cw, Direction::Ccw] {
            let land = landing(pit, dir, count);
            if (land as usize) >= MBELE_LEN {
                continue;
            }
            if own.vichwa[land as usize] >= 1 && opp.vichwa[land as usize] >= 1 {
                out.push(Move::Mtaji { pit, dir });
            }
        }
    }
    out
}

/// RULES.md §12.1 (no-singleton): source >=2. Prefer mbele over nyuma per
/// geziefer's `getMtajiPossibleNonCaptures`. RULES.md §12.2.2 (no-suicide):
/// if the only filled mbele pit is a kichwa, sow must go toward center.
fn mtaji_takata(own: &Side) -> Vec<Move> {
    let mbele_with_two: Vec<u8> = (0..MBELE_LEN as u8)
        .filter(|&c| own.vichwa[c as usize] >= 2)
        .collect();

    let mut out = Vec::new();
    if !mbele_with_two.is_empty() {
        let only_one_filled = mbele_with_two.len() == 1;
        for &pit in &mbele_with_two {
            for &dir in &[Direction::Cw, Direction::Ccw] {
                if only_one_filled && is_suicidal_kichwa(pit, dir) {
                    continue;
                }
                out.push(Move::Mtaji { pit, dir });
            }
        }
        return out;
    }

    // No mbele >=2: nyuma sources only (still no-singleton: >=2).
    for pit in MBELE_LEN as u8..PITS_PER_SIDE as u8 {
        if own.vichwa[pit as usize] >= 2 {
            for &dir in &[Direction::Cw, Direction::Ccw] {
                out.push(Move::Mtaji { pit, dir });
            }
        }
    }
    out
}

/// RULES.md §12.2.2: from kichwa pit 0, sow must go toward center (Cw,
/// step +1 → pit 1). From pit 7, must go Ccw (step -1 → pit 6). The
/// suicidal direction is the one stepping into nyuma (off the front row).
fn is_suicidal_kichwa(pit: u8, dir: Direction) -> bool {
    (pit == 0 && matches!(dir, Direction::Ccw)) || (pit == 7 && matches!(dir, Direction::Cw))
}

// ---------- Kichwa selection ----------------------------------------------------

/// RULES.md §6.3 + geziefer `getPossibleKichwas`. capture_field is the mbele
/// index (0..7) where the capture occurred.
fn kichwa_legal_moves(capture_field: u8, prior_dir: Option<Direction>) -> Vec<Move> {
    let cf = capture_field;
    // Left kimbi (cols 0,1) → LEFT only.
    if cf <= 1 {
        return vec![Move::Kichwa(KichwaSide::Left)];
    }
    // Right kimbi (cols 6,7) → RIGHT only.
    if cf >= 6 {
        return vec![Move::Kichwa(KichwaSide::Right)];
    }
    // Middle (cols 2..=5): direction determines kichwa, OR both if no prior.
    match prior_dir {
        Some(Direction::Cw) => vec![Move::Kichwa(KichwaSide::Left)],
        Some(Direction::Ccw) => vec![Move::Kichwa(KichwaSide::Right)],
        None => vec![
            Move::Kichwa(KichwaSide::Left),
            Move::Kichwa(KichwaSide::Right),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{BoardState, Side, NYUMBA_COL, NYUMBA_COL_NORTH, PITS_PER_SIDE};

    fn empty_kiswahili_state() -> BoardState {
        let south = Side {
            vichwa: [0u8; PITS_PER_SIDE],
            ghala: 32,
            nyumba_owned: true,
            nyumba_col: NYUMBA_COL as u8,
        };
        let north = Side {
            vichwa: [0u8; PITS_PER_SIDE],
            ghala: 32,
            nyumba_owned: true,
            nyumba_col: NYUMBA_COL_NORTH as u8,
        };
        BoardState {
            sides: [south, north],
            phase: Phase::Namu(Substate::AwaitMove),
            active: 0,
            ply: 0,
            variant: Variant::Kiswahili,
            kutakatia: None,
        }
    }

    fn empty_mtaji_state() -> BoardState {
        let south = Side {
            vichwa: [0u8; PITS_PER_SIDE],
            ghala: 0,
            nyumba_owned: false,
            nyumba_col: NYUMBA_COL as u8,
        };
        let north = Side {
            vichwa: [0u8; PITS_PER_SIDE],
            ghala: 0,
            nyumba_owned: false,
            nyumba_col: NYUMBA_COL_NORTH as u8,
        };
        BoardState {
            sides: [south, north],
            phase: Phase::Mtaji(Substate::AwaitMove),
            active: 0,
            ply: 0,
            variant: Variant::Kiswahili,
            kutakatia: None,
        }
    }

    // ---------- landing helper ----------

    #[test]
    fn landing_one_step_cw() {
        assert_eq!(landing(0, Direction::Cw, 1), 1);
        assert_eq!(landing(7, Direction::Cw, 1), 8);
        assert_eq!(landing(15, Direction::Cw, 1), 0);
    }

    #[test]
    fn landing_wraps() {
        // 5 kete from pit 14 cw: 15, 0, 1, 2, 3
        assert_eq!(landing(14, Direction::Cw, 5), 3);
        // 5 kete from pit 2 ccw: 1, 0, 15, 14, 13
        assert_eq!(landing(2, Direction::Ccw, 5), 13);
    }

    // ---------- Namu legal moves ----------

    #[test]
    fn namu_initial_position_no_captures_falls_to_takata() {
        let state = BoardState::new(Variant::Kiswahili);
        let moves = legal_moves(&state);
        // Initial Kiswahili: own mbele has cols 4,5,6 filled but opp.mbele
        // mirror has cols 1,2,3 filled (from opp perspective those are their
        // own initial positions). So columns 4,5,6 vs opponent 4,5,6 — opp's
        // 4,5,6 are all 0. No captures.
        for m in &moves {
            assert!(matches!(m, Move::Namu { .. }), "got {:?}", m);
        }
        // Functional nyumba present → branch 2 of §8.5 → cols 5, 6 (non-empty
        // mbele excluding nyumba at col 4).
        assert!(moves.iter().any(|m| matches!(m, Move::Namu { col: 5 })));
        assert!(moves.iter().any(|m| matches!(m, Move::Namu { col: 6 })));
        // Nyumba (col 4) excluded because other non-empty mbele exist.
        assert!(!moves.iter().any(|m| matches!(m, Move::Namu { col: 4 })));
    }

    #[test]
    fn namu_capture_when_aligned() {
        let mut state = empty_kiswahili_state();
        state.sides[0].vichwa[3] = 1; // own mbele
        state.sides[1].vichwa[3] = 1; // opp mbele same col → capture available
        state.sides[0].vichwa[5] = 4; // some non-capture filler
        let moves = legal_moves(&state);
        // Mandatory-kula → only capture moves.
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0], Move::Namu { col: 3 });
    }

    #[test]
    fn namu_takata_disabled_nyumba_no_restriction() {
        let mut state = empty_kiswahili_state();
        // Owned but <6 → Disabled.
        state.sides[0].vichwa[NYUMBA_COL] = 3;
        state.sides[0].vichwa[1] = 1;
        // No opp kete → no captures.
        let moves = legal_moves(&state);
        assert!(moves.iter().any(|m| matches!(m, Move::Namu { col: 1 })));
        assert!(moves
            .iter()
            .any(|m| matches!(m, Move::Namu { col: c } if *c == NYUMBA_COL as u8)));
    }

    #[test]
    fn namu_takata_destroyed_nyumba_prefers_two() {
        let mut state = empty_kiswahili_state();
        state.sides[0].nyumba_owned = false;
        state.sides[0].vichwa[1] = 2;
        state.sides[0].vichwa[3] = 1;
        let moves = legal_moves(&state);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0], Move::Namu { col: 1 });
    }

    #[test]
    fn namu_takata_destroyed_nyumba_falls_back_to_one() {
        let mut state = empty_kiswahili_state();
        state.sides[0].nyumba_owned = false;
        state.sides[0].vichwa[2] = 1;
        state.sides[0].vichwa[5] = 1;
        let moves = legal_moves(&state);
        assert_eq!(moves.len(), 2);
    }

    #[test]
    fn namu_takata_functional_falls_back_to_nyumba_when_no_other() {
        let mut state = empty_kiswahili_state();
        state.sides[0].vichwa[NYUMBA_COL] = 6; // functional, only filled mbele
        let moves = legal_moves(&state);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0], Move::Namu { col: NYUMBA_COL as u8 });
    }

    // ---------- Mtaji legal moves ----------

    #[test]
    fn mtaji_capture_basic() {
        let mut state = empty_mtaji_state();
        // Own pit 5 has 3 kete; sowing Cw → pits 6, 7, 0 (wraps). Land=0.
        // Wait: 3 from pit 5 Cw lands at (5+3)%16 = 8 (nyuma). Not mbele,
        // so no capture there. Use Ccw: (5-3) mod 16 = 2 (mbele).
        state.sides[0].vichwa[5] = 3;
        state.sides[0].vichwa[2] = 1; // landing has >=1 pre-drop
        state.sides[1].vichwa[2] = 1; // opponent same col has stones
        let moves = legal_moves(&state);
        assert_eq!(
            moves,
            vec![Move::Mtaji {
                pit: 5,
                dir: Direction::Ccw,
            }]
        );
    }

    #[test]
    fn mtaji_no_capture_with_16_seeds() {
        let mut state = empty_mtaji_state();
        state.sides[0].vichwa[0] = 16; // exceeds 15 → no capture eligibility
        state.sides[0].vichwa[3] = 2; // alternative source
        state.sides[1].vichwa[3] = 1;
        let moves = legal_moves(&state);
        // From pit 3 Cw lands at 5 (own=0, no cap). Ccw lands at 1 (own=0, no cap).
        // From pit 0 with 16 kete → ineligible for capture.
        // So no captures → fall through to takata.
        assert!(!moves.is_empty());
        assert!(moves.iter().all(|m| matches!(m, Move::Mtaji { .. })));
    }

    #[test]
    fn mtaji_takata_prefers_mbele() {
        let mut state = empty_mtaji_state();
        state.sides[0].vichwa[2] = 3; // mbele >=2
        state.sides[0].vichwa[10] = 5; // nyuma >=2
        let moves = legal_moves(&state);
        // Must come from mbele (pit 2) only.
        for m in &moves {
            match m {
                Move::Mtaji { pit, .. } => assert!((*pit as usize) < MBELE_LEN),
                _ => panic!("unexpected move {:?}", m),
            }
        }
    }

    #[test]
    fn mtaji_takata_falls_back_to_nyuma() {
        let mut state = empty_mtaji_state();
        state.sides[0].vichwa[10] = 4;
        let moves = legal_moves(&state);
        assert_eq!(moves.len(), 2);
        for m in &moves {
            assert!(matches!(m, Move::Mtaji { pit: 10, .. }));
        }
    }

    #[test]
    fn mtaji_no_singleton_source() {
        let mut state = empty_mtaji_state();
        state.sides[0].vichwa[3] = 1; // singleton — must not appear as source
        let moves = legal_moves(&state);
        assert!(moves.is_empty());
    }

    #[test]
    fn mtaji_no_suicide_kichwa_left() {
        let mut state = empty_mtaji_state();
        state.sides[0].vichwa[0] = 3; // only filled mbele = left kichwa
        let moves = legal_moves(&state);
        // Only Cw (toward center) allowed; Ccw would step into nyuma.
        assert_eq!(
            moves,
            vec![Move::Mtaji {
                pit: 0,
                dir: Direction::Cw,
            }]
        );
    }

    #[test]
    fn mtaji_no_suicide_kichwa_right() {
        let mut state = empty_mtaji_state();
        state.sides[0].vichwa[7] = 3;
        let moves = legal_moves(&state);
        assert_eq!(
            moves,
            vec![Move::Mtaji {
                pit: 7,
                dir: Direction::Ccw,
            }]
        );
    }

    #[test]
    fn mtaji_kichwa_with_other_filled_allows_both_dirs() {
        let mut state = empty_mtaji_state();
        state.sides[0].vichwa[0] = 3;
        state.sides[0].vichwa[3] = 2; // additional filled mbele → no-suicide doesn't apply
        let moves = legal_moves(&state);
        // Pit 0 Cw, Pit 0 Ccw, Pit 3 Cw, Pit 3 Ccw → 4 moves.
        assert_eq!(moves.len(), 4);
    }

    // ---------- Kujifunza ----------

    #[test]
    fn kujifunza_initial_state_has_legal_moves() {
        let state = BoardState::new(Variant::Kujifunza);
        let moves = legal_moves(&state);
        // All 16 pits have 2 kete; each opponent col has 2 → all mtaji
        // captures from mbele (pits 0..7). From mbele, both directions: do
        // any land in own mbele with opp non-empty?
        // From pit i with 2 kete Cw lands at i+2; Ccw lands at i-2 mod 16.
        // For pits 0..5 Cw → 2..7 (mbele) ✓ capture eligible.
        // For pit 6 Cw → 8 (nyuma) — no.
        // For pit 7 Cw → 9 (nyuma) — no.
        // For pit 0 Ccw → 14 (nyuma) — no.
        // For pit 1 Ccw → 15 (nyuma) — no.
        // For pits 2..7 Ccw → 0..5 (mbele) ✓.
        // From nyuma pits 8..15 with 2 Cw/Ccw — many land in mbele too.
        // Just assert non-empty and all captures.
        assert!(!moves.is_empty());
        for m in &moves {
            assert!(matches!(m, Move::Mtaji { .. }));
        }
    }

    // ---------- Substates ----------

    #[test]
    fn await_kichwa_left_kimbi() {
        let moves = kichwa_legal_moves(0, None);
        assert_eq!(moves, vec![Move::Kichwa(KichwaSide::Left)]);
        let moves = kichwa_legal_moves(1, Some(Direction::Cw));
        assert_eq!(moves, vec![Move::Kichwa(KichwaSide::Left)]);
    }

    #[test]
    fn await_kichwa_right_kimbi() {
        let moves = kichwa_legal_moves(7, None);
        assert_eq!(moves, vec![Move::Kichwa(KichwaSide::Right)]);
        let moves = kichwa_legal_moves(6, Some(Direction::Ccw));
        assert_eq!(moves, vec![Move::Kichwa(KichwaSide::Right)]);
    }

    #[test]
    fn await_kichwa_middle_with_dir_is_deterministic() {
        assert_eq!(
            kichwa_legal_moves(4, Some(Direction::Cw)),
            vec![Move::Kichwa(KichwaSide::Left)]
        );
        assert_eq!(
            kichwa_legal_moves(4, Some(Direction::Ccw)),
            vec![Move::Kichwa(KichwaSide::Right)]
        );
    }

    #[test]
    fn await_kichwa_middle_no_dir_offers_both() {
        let moves = kichwa_legal_moves(4, None);
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Move::Kichwa(KichwaSide::Left)));
        assert!(moves.contains(&Move::Kichwa(KichwaSide::Right)));
    }

    #[test]
    fn await_safari_offers_both() {
        let mut state = empty_mtaji_state();
        state.phase = Phase::Mtaji(Substate::AwaitSafari);
        let moves = legal_moves(&state);
        assert_eq!(moves.len(), 2);
        assert!(moves.contains(&Move::Safari { go: true }));
        assert!(moves.contains(&Move::Safari { go: false }));
    }
}
