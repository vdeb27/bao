//! Bao Algebraic Notation (BAN) encoder.
//!
//! Decoding is deferred — the current consumers (UI move history, training
//! shard sidecar) only need readable output. Format details:
//!
//! - Columns `a..h` run left-to-right from South's seat.
//! - Rows: 1 = South mbele, 2 = South nyuma, 3 = North nyuma, 4 = North mbele.
//!   The two mbele rows (1 and 4) face each other across the capture line.
//! - Direction: `>` for clockwise, `<` for counter-clockwise (active-player
//!   own-ring orientation).
//!
//! Encodings:
//!
//! - `N:e1>`        — Namu, mbele col `e` (own-side idx 4), sow direction Cw.
//! - `N:e1>*`       — same, but the move triggered a capture.
//! - `c2<`          — Mtaji from col `c` row 2 (South nyuma), dir Ccw.
//! - `c2<*`         — capturing mtaji.
//! - `K:L` / `K:R`  — kichwa-side selection.
//! - `Sy` / `Sn`    — safari decision.

use crate::board::{BoardState, Direction};
use crate::events::MoveEvent;
use crate::moves::{KichwaSide, Move};

/// Encode `mv` against `state` (pre-move). `events` is the event slice
/// returned by `apply(state, mv)`; passing `&[]` skips the `*` capture
/// annotation but otherwise yields a valid string.
pub fn encode(state: &BoardState, mv: Move, events: &[MoveEvent]) -> String {
    let captured = events
        .iter()
        .any(|e| matches!(e, MoveEvent::Capture { .. }));
    let suffix = if captured { "*" } else { "" };
    match mv {
        Move::Namu { col, dir } => {
            let (letter, row) = mbele_to_ban(state.active, col);
            format!("N:{}{}{}{}", letter, row, dir_char(dir), suffix)
        }
        Move::Mtaji { pit, dir } => {
            let (letter, row) = pit_to_ban(state.active, pit);
            format!("{}{}{}{}", letter, row, dir_char(dir), suffix)
        }
        Move::Kichwa(side) => format!(
            "K:{}",
            match side {
                KichwaSide::Left => 'L',
                KichwaSide::Right => 'R',
            }
        ),
        Move::Safari { go } => format!("S{}", if go { 'y' } else { 'n' }),
    }
}

fn dir_char(d: Direction) -> char {
    match d {
        Direction::Cw => '>',
        Direction::Ccw => '<',
    }
}

fn mbele_to_ban(player: u8, col: u8) -> (char, u8) {
    if player == 0 {
        // South: screen col = col, row 1.
        ((b'a' + col) as char, 1)
    } else {
        // North: own-perspective col c sits at screen col (7-c), row 4.
        ((b'a' + (7 - col)) as char, 4)
    }
}

fn pit_to_ban(player: u8, pit: u8) -> (char, u8) {
    // vichwa[0..8] = own mbele L→R. vichwa[8..16] = own nyuma, indexed so that
    // idx 8 sits physically above mbele[7]; see board.rs comment on Side.
    if player == 0 {
        if pit < 8 {
            ((b'a' + pit) as char, 1)
        } else {
            ((b'a' + (15 - pit)) as char, 2)
        }
    } else if pit < 8 {
        ((b'a' + (7 - pit)) as char, 4)
    } else {
        ((b'a' + (pit - 8)) as char, 3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variant::Variant;

    fn south_to_move() -> BoardState {
        BoardState::new(Variant::Kiswahili)
    }

    fn north_to_move() -> BoardState {
        let mut s = BoardState::new(Variant::Kiswahili);
        s.active = 1;
        s
    }

    #[test]
    fn namu_south_col0_cw() {
        let s = south_to_move();
        let mv = Move::Namu {
            col: 0,
            dir: Direction::Cw,
        };
        assert_eq!(encode(&s, mv, &[]), "N:a1>");
    }

    #[test]
    fn namu_south_col4_capture() {
        let s = south_to_move();
        let mv = Move::Namu {
            col: 4,
            dir: Direction::Cw,
        };
        let evs = [MoveEvent::Capture {
            from_player: 1,
            from_pit: 4,
            count: 2,
        }];
        assert_eq!(encode(&s, mv, &evs), "N:e1>*");
    }

    #[test]
    fn namu_north_mirrors_screen_columns() {
        // North's idx-0 mbele is the rightmost pit on screen → letter h, row 4.
        let s = north_to_move();
        let mv = Move::Namu {
            col: 0,
            dir: Direction::Ccw,
        };
        assert_eq!(encode(&s, mv, &[]), "N:h4<");
    }

    #[test]
    fn mtaji_south_mbele_idx7_is_h1() {
        let s = south_to_move();
        let mv = Move::Mtaji {
            pit: 7,
            dir: Direction::Cw,
        };
        assert_eq!(encode(&s, mv, &[]), "h1>");
    }

    #[test]
    fn mtaji_south_nyuma_idx8_is_h2() {
        // nyuma idx 8 sits above mbele[7] → screen col h, row 2.
        let s = south_to_move();
        let mv = Move::Mtaji {
            pit: 8,
            dir: Direction::Ccw,
        };
        assert_eq!(encode(&s, mv, &[]), "h2<");
    }

    #[test]
    fn mtaji_south_nyuma_idx15_is_a2() {
        let s = south_to_move();
        let mv = Move::Mtaji {
            pit: 15,
            dir: Direction::Cw,
        };
        assert_eq!(encode(&s, mv, &[]), "a2>");
    }

    #[test]
    fn mtaji_north_mbele_idx7_is_a4() {
        let s = north_to_move();
        let mv = Move::Mtaji {
            pit: 7,
            dir: Direction::Cw,
        };
        assert_eq!(encode(&s, mv, &[]), "a4>");
    }

    #[test]
    fn mtaji_north_nyuma_idx8_is_a3() {
        let s = north_to_move();
        let mv = Move::Mtaji {
            pit: 8,
            dir: Direction::Cw,
        };
        assert_eq!(encode(&s, mv, &[]), "a3>");
    }

    #[test]
    fn kichwa_and_safari() {
        let s = south_to_move();
        assert_eq!(encode(&s, Move::Kichwa(KichwaSide::Left), &[]), "K:L");
        assert_eq!(encode(&s, Move::Kichwa(KichwaSide::Right), &[]), "K:R");
        assert_eq!(encode(&s, Move::Safari { go: true }, &[]), "Sy");
        assert_eq!(encode(&s, Move::Safari { go: false }, &[]), "Sn");
    }

    #[test]
    fn capture_suffix_on_mtaji() {
        let s = south_to_move();
        let mv = Move::Mtaji {
            pit: 2,
            dir: Direction::Cw,
        };
        let evs = [MoveEvent::Capture {
            from_player: 1,
            from_pit: 2,
            count: 3,
        }];
        assert_eq!(encode(&s, mv, &evs), "c1>*");
    }
}
