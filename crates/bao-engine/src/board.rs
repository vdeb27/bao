use crate::variant::Variant;
use serde::{Deserialize, Serialize};

pub const PITS_PER_SIDE: usize = 16;
pub const MBELE_LEN: usize = 8;
pub const NYUMA_LEN: usize = 8;
/// Mbele column index (0-based) of the nyumba for player South (= field 5 in
/// 1-based geziefer encoding). North's nyumba is mirrored to NYUMBA_COL_NORTH.
/// See RULES.md §1.2.
pub const NYUMBA_COL: usize = 4;
/// Mbele column index (0-based) of the nyumba for player North (= field 4 in
/// 1-based geziefer encoding). See RULES.md §1.2.
pub const NYUMBA_COL_NORTH: usize = 3;
/// Initial seed count in nyumba at game start (Kiswahili). See RULES.md §2.1.
pub const NYUMBA_INITIAL_KETE: u8 = 6;
/// Threshold below which nyumba becomes Disabled. See RULES.md §8.3.
pub const NYUMBA_FUNCTIONAL_THRESHOLD: u8 = 6;
/// Total kete on the board across both players. Invariant.
pub const TOTAL_KETE: u32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Cw,
    Ccw,
}

impl Direction {
    pub fn step(self) -> i32 {
        match self {
            Direction::Cw => 1,
            Direction::Ccw => -1,
        }
    }

    pub fn reverse(self) -> Direction {
        match self {
            Direction::Cw => Direction::Ccw,
            Direction::Ccw => Direction::Cw,
        }
    }
}

/// Three-state nyumba model — see RULES.md §8.3 (CONFIRMED across 5 sources).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NyumbaState {
    Functional,
    Disabled,
    Destroyed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Namu(Substate),
    Mtaji(Substate),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Substate {
    AwaitMove,
    /// Capture has triggered; player must select a kichwa to start the
    /// capture-sow from. `prior_dir` is the direction of the move that led
    /// to this capture (mtaji moves carry direction; namu-kula has none and
    /// uses `None`). See RULES.md §6.3.
    AwaitKichwa {
        capture_field: u8,
        prior_dir: Option<Direction>,
    },
    /// Capture-sow has landed in active player's own functional nyumba; the
    /// player must decide whether to plunder it ("safari") and continue, or
    /// stop. `sow_dir` is the direction the capture-sow was traveling when
    /// it hit the nyumba — needed to resume on go=true. See RULES.md §6.4.
    AwaitSafari {
        sow_dir: Direction,
    },
}

/// Active kutakatia block, see RULES.md §11.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kutakatia {
    pub blocked_field: u8,
    pub blocked_player: u8,
    pub turns_remaining: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Side {
    /// Indices 0..MBELE_LEN: mbele left-to-right from this player's perspective.
    /// Indices MBELE_LEN..PITS_PER_SIDE: nyuma, indexed so that vichwa[8] sits
    /// physically above mbele pit 7 and vichwa[15] above mbele pit 0. With this
    /// layout (i + 1) % 16 sows clockwise through the player's own two rows;
    /// (i + 15) % 16 sows counter-clockwise. Matches geziefer's getNextField.
    pub vichwa: [u8; PITS_PER_SIDE],
    pub ghala: u8,
    pub nyumba_owned: bool,
    /// Mbele index (0..8) of this side's nyumba. South: NYUMBA_COL (=4),
    /// North: NYUMBA_COL_NORTH (=3). Irrelevant in Kujifunza but still
    /// stored to keep Side variant-agnostic.
    pub nyumba_col: u8,
}

impl Side {
    pub fn nyumba_state(&self, variant: Variant) -> NyumbaState {
        if !variant.has_nyumba() || !self.nyumba_owned {
            NyumbaState::Destroyed
        } else if self.vichwa[self.nyumba_col as usize] >= NYUMBA_FUNCTIONAL_THRESHOLD {
            NyumbaState::Functional
        } else {
            NyumbaState::Disabled
        }
    }

    pub fn mbele(&self, col: usize) -> u8 {
        self.vichwa[col]
    }

    pub fn nyuma(&self, col: usize) -> u8 {
        self.vichwa[PITS_PER_SIDE - 1 - col]
    }

    pub fn mbele_total(&self) -> u32 {
        self.vichwa[..MBELE_LEN].iter().map(|&x| x as u32).sum()
    }

    pub fn nyuma_total(&self) -> u32 {
        self.vichwa[MBELE_LEN..].iter().map(|&x| x as u32).sum()
    }

    pub fn kete_total(&self) -> u32 {
        self.vichwa.iter().map(|&x| x as u32).sum::<u32>() + self.ghala as u32
    }

    pub fn mbele_is_empty(&self) -> bool {
        self.mbele_total() == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardState {
    pub sides: [Side; 2],
    pub phase: Phase,
    pub active: u8,
    pub ply: u32,
    pub variant: Variant,
    pub kutakatia: Option<Kutakatia>,
    /// Set when the game has ended; the value is the winning player index.
    /// `apply()` rejects further moves once this is `Some`. See RULES.md §9.
    pub winner: Option<u8>,
}

impl BoardState {
    pub fn new(variant: Variant) -> Self {
        match variant {
            Variant::Kiswahili => Self::initial_kiswahili(),
            Variant::Kujifunza => Self::initial_kujifunza(),
        }
    }

    fn initial_kiswahili() -> Self {
        // South's data layout: own perspective, field 1..8 mapped to idx 0..7
        // left-to-right. RULES.md §2.1: nyumba (field 5 = idx 4) holds 6;
        // fields 6,7 (idx 5,6) hold 2 each.
        let mut south_vichwa = [0u8; PITS_PER_SIDE];
        south_vichwa[NYUMBA_COL] = NYUMBA_INITIAL_KETE;
        south_vichwa[NYUMBA_COL + 1] = 2;
        south_vichwa[NYUMBA_COL + 2] = 2;

        // North's data: per-side perspective, but RULES.md §2.1 places
        // North's nyumba at field 4 (idx 3) and the two flanking 2-kete pits
        // at fields 2,3 (idx 1,2). This is asymmetric vs. South in stored
        // form; geziefer's capture rule reads opp[c] at the same index, so
        // the encoding is intentionally per-side.
        let mut north_vichwa = [0u8; PITS_PER_SIDE];
        north_vichwa[NYUMBA_COL_NORTH] = NYUMBA_INITIAL_KETE;
        north_vichwa[NYUMBA_COL_NORTH - 1] = 2;
        north_vichwa[NYUMBA_COL_NORTH - 2] = 2;

        let south = Side {
            vichwa: south_vichwa,
            ghala: 22,
            nyumba_owned: true,
            nyumba_col: NYUMBA_COL as u8,
        };
        let north = Side {
            vichwa: north_vichwa,
            ghala: 22,
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
            winner: None,
        }
    }

    fn initial_kujifunza() -> Self {
        let south = Side {
            vichwa: [2u8; PITS_PER_SIDE],
            ghala: 0,
            nyumba_owned: false,
            nyumba_col: NYUMBA_COL as u8,
        };
        let north = Side {
            vichwa: [2u8; PITS_PER_SIDE],
            ghala: 0,
            nyumba_owned: false,
            nyumba_col: NYUMBA_COL_NORTH as u8,
        };

        BoardState {
            sides: [south, north],
            phase: Phase::Mtaji(Substate::AwaitMove),
            active: 0,
            ply: 0,
            variant: Variant::Kujifunza,
            kutakatia: None,
            winner: None,
        }
    }

    pub fn opponent(&self, player: u8) -> u8 {
        1 - player
    }

    pub fn active_side(&self) -> &Side {
        &self.sides[self.active as usize]
    }

    pub fn opponent_side(&self) -> &Side {
        &self.sides[self.opponent(self.active) as usize]
    }

    pub fn total_kete(&self) -> u32 {
        self.sides.iter().map(Side::kete_total).sum()
    }

    pub fn check_invariants(&self) -> Result<(), &'static str> {
        if self.total_kete() != TOTAL_KETE {
            return Err("total kete invariant broken");
        }
        if self.active > 1 {
            return Err("active player out of range");
        }
        Ok(())
    }
}

/// Step from one pit to the next within a single player's two-row ring.
pub fn next_pit(pit: u8, dir: Direction) -> u8 {
    let p = pit as i32 + dir.step();
    p.rem_euclid(PITS_PER_SIDE as i32) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_pit_cw_walks_the_ring() {
        let mut p = 0u8;
        for _ in 0..PITS_PER_SIDE {
            p = next_pit(p, Direction::Cw);
        }
        assert_eq!(p, 0);
    }

    #[test]
    fn next_pit_cw_and_ccw_invert() {
        for p in 0..PITS_PER_SIDE as u8 {
            assert_eq!(next_pit(next_pit(p, Direction::Cw), Direction::Ccw), p);
        }
    }

    #[test]
    fn kiswahili_initial_position_has_64_kete() {
        let state = BoardState::new(Variant::Kiswahili);
        assert_eq!(state.total_kete(), TOTAL_KETE);
        state.check_invariants().expect("invariants");
    }

    #[test]
    fn kiswahili_initial_position_per_side() {
        let state = BoardState::new(Variant::Kiswahili);
        for side in &state.sides {
            assert_eq!(side.ghala, 22);
            let nc = side.nyumba_col as usize;
            assert_eq!(side.vichwa[nc], NYUMBA_INITIAL_KETE);
            assert_eq!(side.kete_total(), 32);
            assert_eq!(
                side.nyumba_state(Variant::Kiswahili),
                NyumbaState::Functional
            );
            assert_eq!(side.mbele_total(), 10);
            assert_eq!(side.nyuma_total(), 0);
        }
        // South: nyumba at idx 4, flanking 2-kete at idx 5,6.
        assert_eq!(state.sides[0].vichwa[NYUMBA_COL + 1], 2);
        assert_eq!(state.sides[0].vichwa[NYUMBA_COL + 2], 2);
        // North: nyumba at idx 3, flanking 2-kete at idx 1,2.
        assert_eq!(state.sides[1].vichwa[NYUMBA_COL_NORTH - 1], 2);
        assert_eq!(state.sides[1].vichwa[NYUMBA_COL_NORTH - 2], 2);
    }

    #[test]
    fn kiswahili_starts_in_namu_await_move() {
        let state = BoardState::new(Variant::Kiswahili);
        assert!(matches!(state.phase, Phase::Namu(Substate::AwaitMove)));
        assert!(state.kutakatia.is_none());
    }

    #[test]
    fn kujifunza_initial_position_has_64_kete() {
        let state = BoardState::new(Variant::Kujifunza);
        assert_eq!(state.total_kete(), TOTAL_KETE);
        state.check_invariants().expect("invariants");
    }

    #[test]
    fn kujifunza_initial_position_per_side() {
        let state = BoardState::new(Variant::Kujifunza);
        for side in &state.sides {
            assert_eq!(side.ghala, 0);
            assert!(side.vichwa.iter().all(|&c| c == 2));
            assert_eq!(side.kete_total(), 32);
            assert_eq!(
                side.nyumba_state(Variant::Kujifunza),
                NyumbaState::Destroyed
            );
        }
    }

    #[test]
    fn kujifunza_starts_in_mtaji_await_move() {
        let state = BoardState::new(Variant::Kujifunza);
        assert!(matches!(state.phase, Phase::Mtaji(Substate::AwaitMove)));
    }

    #[test]
    fn nyumba_state_transitions() {
        let mut side = Side {
            vichwa: [0u8; PITS_PER_SIDE],
            ghala: 0,
            nyumba_owned: true,
            nyumba_col: NYUMBA_COL as u8,
        };
        side.vichwa[NYUMBA_COL] = 6;
        assert_eq!(
            side.nyumba_state(Variant::Kiswahili),
            NyumbaState::Functional
        );

        side.vichwa[NYUMBA_COL] = 5;
        assert_eq!(
            side.nyumba_state(Variant::Kiswahili),
            NyumbaState::Disabled
        );

        side.vichwa[NYUMBA_COL] = 9;
        assert_eq!(
            side.nyumba_state(Variant::Kiswahili),
            NyumbaState::Functional
        );

        side.nyumba_owned = false;
        assert_eq!(
            side.nyumba_state(Variant::Kiswahili),
            NyumbaState::Destroyed
        );
    }

    #[test]
    fn kujifunza_never_has_nyumba() {
        let mut side = Side {
            vichwa: [6u8; PITS_PER_SIDE],
            ghala: 0,
            nyumba_owned: true,
            nyumba_col: NYUMBA_COL as u8,
        };
        assert_eq!(
            side.nyumba_state(Variant::Kujifunza),
            NyumbaState::Destroyed
        );
        side.nyumba_owned = false;
        assert_eq!(
            side.nyumba_state(Variant::Kujifunza),
            NyumbaState::Destroyed
        );
    }
}
