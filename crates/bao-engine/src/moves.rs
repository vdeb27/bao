use crate::board::Direction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KichwaSide {
    Left,
    Right,
}

impl KichwaSide {
    pub fn pit(self) -> u8 {
        match self {
            KichwaSide::Left => 0,
            KichwaSide::Right => 7,
        }
    }

    /// Direction the capture-sow proceeds when starting from this kichwa.
    /// Per RULES.md §6.3: left kichwa → clockwise (towards center); right
    /// kichwa → counter-clockwise (towards center).
    pub fn sow_direction(self) -> Direction {
        match self {
            KichwaSide::Left => Direction::Cw,
            KichwaSide::Right => Direction::Ccw,
        }
    }
}

/// A move that a player can submit. Validity depends on phase and substate;
/// the engine validates at apply-time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Move {
    /// Namu / kunamua: place one kete from ghala into mbele[col]. Direction
    /// of any subsequent capture-sow is decided by the kichwa selection
    /// (which may be deterministic or a player choice — see RULES.md §6.3).
    Namu { col: u8 },

    /// Mtaji: take all kete from `pit` and sow in `dir`. The engine
    /// determines whether the move is a capture, a takata, or triggers
    /// a sub-state.
    Mtaji { pit: u8, dir: Direction },

    /// Resolves an `AwaitKichwa` sub-state.
    Kichwa(KichwaSide),

    /// Resolves an `AwaitSafari` sub-state.
    Safari { go: bool },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kichwa_pit_indices() {
        assert_eq!(KichwaSide::Left.pit(), 0);
        assert_eq!(KichwaSide::Right.pit(), 7);
    }

    #[test]
    fn kichwa_sow_direction() {
        assert_eq!(KichwaSide::Left.sow_direction(), Direction::Cw);
        assert_eq!(KichwaSide::Right.sow_direction(), Direction::Ccw);
    }
}
