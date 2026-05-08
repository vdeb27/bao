use serde::{Deserialize, Serialize};

/// Events emitted during move execution. The UI consumes the stream to drive
/// animations; the training pipeline ignores it in the hot path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveEvent {
    NamuPlace { player: u8, pit: u8 },
    Sow { player: u8, pit: u8 },
    Pickup { player: u8, pit: u8, count: u8 },
    Capture { from_player: u8, from_pit: u8, count: u8 },
    Tax { player: u8, pit: u8, taken: u8 },
    SafariTriggered { player: u8 },
    KichwaSelectionRequired { player: u8, capture_field: u8 },
    PhaseShift,
    NyumbaDestroyed { player: u8 },
    KutakatiaActivated { blocked_player: u8, blocked_field: u8 },
    KutakatiaCleared,
    GameOver { winner: u8 },
}
