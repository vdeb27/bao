//! Transposition table for the alpha-beta search.
//!
//! Layout: a power-of-two-sized vector of fixed 16-byte entries, indexed by
//! the Zobrist key modulo the table size. Always-replace policy — entries
//! get overwritten on collision regardless of depth. Depth-prefer schemes
//! can come later once we have a perf benchmark to validate the trade.
//!
//! Move encoding (u16): bits 0..1 are the variant tag (Namu/Mtaji/Kichwa/
//! Safari), bits 2..5 hold the col/pit/side/go payload, bit 6 holds the
//! direction or the Kichwa side. Encoded moves survive a round trip through
//! `pack_move` / `unpack_move`; see the tests.

use crate::board::Direction;
use crate::moves::{KichwaSide, Move};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bound {
    None = 0,
    Exact = 1,
    Lower = 2,
    Upper = 3,
}

impl Bound {
    fn from_u8(v: u8) -> Self {
        match v {
            1 => Bound::Exact,
            2 => Bound::Lower,
            3 => Bound::Upper,
            _ => Bound::None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TtEntry {
    pub key: u64,
    pub best_move: u16,
    pub score: i32,
    pub depth: u8,
    pub flag: u8,
}

pub struct TranspositionTable {
    entries: Box<[TtEntry]>,
    mask: usize,
}

impl TranspositionTable {
    /// Construct a TT sized to the largest power of two ≤ `slots`.
    pub fn new(slots: usize) -> Self {
        let slots = slots.max(64).next_power_of_two();
        let entries = vec![TtEntry::default(); slots].into_boxed_slice();
        TranspositionTable {
            entries,
            mask: slots - 1,
        }
    }

    #[inline]
    fn idx(&self, key: u64) -> usize {
        (key as usize) & self.mask
    }

    pub fn probe(&self, key: u64) -> Option<TtEntry> {
        let e = self.entries[self.idx(key)];
        if e.key == key && Bound::from_u8(e.flag) != Bound::None {
            Some(e)
        } else {
            None
        }
    }

    pub fn store(&mut self, key: u64, best_move: u16, score: i32, depth: u8, bound: Bound) {
        let i = self.idx(key);
        self.entries[i] = TtEntry {
            key,
            best_move,
            score,
            depth,
            flag: bound as u8,
        };
    }

    pub fn clear(&mut self) {
        for e in self.entries.iter_mut() {
            *e = TtEntry::default();
        }
    }
}

const TAG_NAMU: u16 = 0;
const TAG_MTAJI: u16 = 1;
const TAG_KICHWA: u16 = 2;
const TAG_SAFARI: u16 = 3;

#[inline]
fn dir_bit(d: Direction) -> u16 {
    match d {
        Direction::Cw => 0,
        Direction::Ccw => 1,
    }
}

#[inline]
fn bit_to_dir(b: u16) -> Direction {
    if b & 1 == 0 {
        Direction::Cw
    } else {
        Direction::Ccw
    }
}

pub fn pack_move(m: Move) -> u16 {
    match m {
        Move::Namu { col, dir } => TAG_NAMU | ((col as u16) << 2) | (dir_bit(dir) << 6),
        Move::Mtaji { pit, dir } => TAG_MTAJI | ((pit as u16) << 2) | (dir_bit(dir) << 6),
        Move::Kichwa(side) => {
            let s = match side {
                KichwaSide::Left => 0,
                KichwaSide::Right => 1,
            };
            TAG_KICHWA | (s << 6)
        }
        Move::Safari { go } => TAG_SAFARI | ((go as u16) << 6),
    }
}

pub fn unpack_move(packed: u16) -> Option<Move> {
    if packed == 0 {
        // 0 also encodes a valid Namu{col:0,dir:Cw}; the search treats it as
        // "no TT move" by checking the bound flag, so this returns None only
        // when callers explicitly want to validate sentinel zero.
        // We still return Some here for correct round-tripping.
    }
    let tag = packed & 0b11;
    match tag {
        TAG_NAMU => Some(Move::Namu {
            col: ((packed >> 2) & 0b1111) as u8,
            dir: bit_to_dir(packed >> 6),
        }),
        TAG_MTAJI => Some(Move::Mtaji {
            pit: ((packed >> 2) & 0b1111) as u8,
            dir: bit_to_dir(packed >> 6),
        }),
        TAG_KICHWA => Some(Move::Kichwa(if (packed >> 6) & 1 == 0 {
            KichwaSide::Left
        } else {
            KichwaSide::Right
        })),
        TAG_SAFARI => Some(Move::Safari {
            go: (packed >> 6) & 1 == 1,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(m: Move) {
        let p = pack_move(m);
        assert_eq!(unpack_move(p), Some(m), "roundtrip failed for {:?}", m);
    }

    #[test]
    fn pack_unpack_all_namu_variants() {
        for col in 0..8 {
            for &dir in &[Direction::Cw, Direction::Ccw] {
                round_trip(Move::Namu { col, dir });
            }
        }
    }

    #[test]
    fn pack_unpack_all_mtaji_variants() {
        for pit in 0..16 {
            for &dir in &[Direction::Cw, Direction::Ccw] {
                round_trip(Move::Mtaji { pit, dir });
            }
        }
    }

    #[test]
    fn pack_unpack_kichwa_and_safari() {
        round_trip(Move::Kichwa(KichwaSide::Left));
        round_trip(Move::Kichwa(KichwaSide::Right));
        round_trip(Move::Safari { go: false });
        round_trip(Move::Safari { go: true });
    }

    #[test]
    fn tt_probe_returns_stored_entry() {
        let mut tt = TranspositionTable::new(1024);
        let key = 0xDEAD_BEEF_CAFE_BABE;
        let mv = pack_move(Move::Namu {
            col: 5,
            dir: Direction::Cw,
        });
        tt.store(key, mv, 42, 4, Bound::Exact);
        let got = tt.probe(key).expect("hit");
        assert_eq!(got.score, 42);
        assert_eq!(got.depth, 4);
        assert_eq!(got.best_move, mv);
        assert_eq!(Bound::from_u8(got.flag), Bound::Exact);
    }

    #[test]
    fn tt_miss_on_wrong_key() {
        let mut tt = TranspositionTable::new(1024);
        tt.store(123, 0, 0, 0, Bound::Exact);
        assert!(tt.probe(456).is_none());
    }

    #[test]
    fn tt_size_rounds_to_power_of_two() {
        let tt = TranspositionTable::new(1000);
        assert_eq!(tt.entries.len(), 1024);
    }
}
