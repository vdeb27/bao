//! Bao la Kiswahili rules engine.
//!
//! Single source of truth for the rules of Bao la Kiswahili and Bao la
//! Kujifunza. The engine drives both the browser UI (via wasm-bindgen) and
//! the training pipeline (via PyO3). All rule claims trace to `RULES.md` at
//! the repo root.

pub mod board;
pub mod eval;
pub mod events;
pub mod features;
pub mod mcts;
pub mod moves;
pub mod notation;
pub mod rules;
pub mod search;
pub mod shard;
pub mod variant;
pub mod zobrist;

pub use rules::{apply, legal_moves};
pub use board::{
    BoardState, Direction, Kutakatia, NyumbaState, Phase, Side, Substate, MBELE_LEN, NYUMBA_COL,
    NYUMBA_FUNCTIONAL_THRESHOLD, NYUMBA_INITIAL_KETE, NYUMA_LEN, PACKED_LEN, PACK_MAGIC,
    PACK_VERSION, PITS_PER_SIDE, TOTAL_KETE,
};
pub use eval::{Evaluator, HeuristicEval, HeuristicWeights};
pub use features::{clip_label, encode_features, FEATURE_LEN, LABEL_CLIP};
pub use shard::{read_shard, ShardHeader, ShardWriter, HEADER_LEN, RECORD_STRIDE, SHARD_VERSION};
pub use events::MoveEvent;
pub use moves::{KichwaSide, Move};
pub use notation::encode as encode_ban;
pub use search::{search, SearchOptions, SearchResult, MATE_SCORE, MATE_THRESHOLD};
pub use variant::Variant;
pub use zobrist::zobrist_key;
