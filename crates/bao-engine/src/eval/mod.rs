//! Position evaluation. The search treats evaluators as a trait so the same
//! search loop can drive the handcrafted bootstrap evaluator today and an
//! NNUE evaluator later without further changes.

use crate::board::BoardState;

pub mod heuristic;
pub mod nnue;

pub use heuristic::{HeuristicEval, HeuristicWeights};

/// Score units are "centi-kete": a single kete is worth 100. Mate scores
/// live outside the eval range so the search can distinguish a forced
/// win/loss from a merely good position; see `search::MATE_SCORE`.
pub trait Evaluator {
    /// Score the position from the *active* player's perspective. Positive
    /// means the player to move is winning; negative means losing.
    fn eval(&self, state: &BoardState) -> i32;
}
