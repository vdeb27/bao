//! Search: iterative-deepening negamax + quiescence. Designed to be driven
//! by any `Evaluator` so the same loop can host the handcrafted bootstrap
//! today and an NNUE evaluator after fase 5.
//!
//! Out of scope for this slice (deferred to follow-ups):
//! - Transposition table (Zobrist is ready in `crate::zobrist`)
//! - Killer / history move-ordering heuristics
//! - Aspiration windows
//! - Late-move reductions
//! - Null-move pruning
//!
//! The current ordering is just "captures before takata", which is enough
//! for the alpha-beta cutoffs to fire on most Bao positions.

use std::time::{Duration, Instant};

use crate::board::BoardState;
use crate::eval::Evaluator;
use crate::moves::Move;
use crate::rules::{apply, legal_moves};

pub mod alphabeta;

/// Scores ≥ MATE_THRESHOLD represent forced wins (and ≤ -MATE_THRESHOLD
/// forced losses), with the distance-to-mate encoded as `MATE_SCORE - ply`.
/// Keeps mate scores well outside any plausible evaluator output.
pub const MATE_SCORE: i32 = 1_000_000;
pub const MATE_THRESHOLD: i32 = 900_000;

#[derive(Debug, Clone, Copy)]
pub struct SearchOptions {
    pub max_depth: u8,
    pub time_budget: Duration,
}

impl SearchOptions {
    pub fn depth(d: u8) -> Self {
        Self {
            max_depth: d,
            time_budget: Duration::from_secs(60),
        }
    }
    pub fn budget(ms: u64) -> Self {
        Self {
            max_depth: 64,
            time_budget: Duration::from_millis(ms),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
    pub depth_reached: u8,
    pub nodes: u64,
    pub elapsed: Duration,
}

/// Top-level entry. Runs iterative deepening from depth 1 upward and returns
/// the deepest fully-completed iteration. If time runs out mid-iteration the
/// previous iteration's best move is returned.
pub fn search<E: Evaluator>(
    state: &BoardState,
    eval: &E,
    opts: SearchOptions,
) -> SearchResult {
    let started = Instant::now();
    let deadline = started + opts.time_budget;
    let mut best = SearchResult {
        best_move: None,
        score: 0,
        depth_reached: 0,
        nodes: 0,
        elapsed: Duration::ZERO,
    };

    for depth in 1..=opts.max_depth {
        let mut ctx = alphabeta::SearchCtx::new(eval, deadline);
        let r = alphabeta::root_negamax(&mut ctx, state, depth);
        if ctx.aborted {
            // Abandon partial iteration — best from prior depth is more trustworthy.
            break;
        }
        best = SearchResult {
            best_move: r.best_move,
            score: r.score,
            depth_reached: depth,
            nodes: best.nodes + ctx.nodes,
            elapsed: started.elapsed(),
        };
        // If we already proved a forced mate, no need to go deeper.
        if r.score.abs() >= MATE_THRESHOLD {
            break;
        }
    }

    best.elapsed = started.elapsed();
    best
}

/// Convenience: orders captures (Namu/Mtaji) before resolution moves
/// (Kichwa/Safari). For now the captures themselves aren't graded; a future
/// slice introduces MVV-LVA on `kete_taken`.
pub(crate) fn order_moves(state: &BoardState, moves: &mut [Move]) {
    moves.sort_by_key(|m| match m {
        // Kichwa and Safari resolve substates; ranking them last is fine.
        Move::Kichwa(_) | Move::Safari { .. } => 2,
        Move::Namu { col, .. } => {
            let opp = state.opponent(state.active) as usize;
            // Mirror per the capture-mirror rule (see rules::opp_mbele).
            let opp_idx = 7usize.saturating_sub(*col as usize);
            if state.sides[opp].vichwa[opp_idx] >= 1 {
                0 // potential capture
            } else {
                1 // takata
            }
        }
        Move::Mtaji { .. } => 1,
    });
}

/// Apply move and detect simple terminal outcomes from the engine's
/// `winner` field. Returns `None` on illegal moves so the search can skip.
pub(crate) fn try_apply(
    state: &BoardState,
    mv: Move,
) -> Option<BoardState> {
    apply(state, mv).ok().map(|(next, _events)| next)
}

/// Move-count helper for the root: used by `bestmove` to disambiguate
/// "no legal moves" from "terminal".
pub fn root_legal_count(state: &BoardState) -> usize {
    legal_moves(state).len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::HeuristicEval;
    use crate::variant::Variant;

    #[test]
    fn search_initial_returns_a_move() {
        let state = BoardState::new(Variant::Kiswahili);
        let e = HeuristicEval::new();
        let r = search(&state, &e, SearchOptions::depth(2));
        assert!(r.best_move.is_some(), "initial position must have a legal move");
        assert!(r.depth_reached >= 1);
        assert!(r.nodes > 0);
    }

    #[test]
    fn search_respects_time_budget() {
        let state = BoardState::new(Variant::Kiswahili);
        let e = HeuristicEval::new();
        let opts = SearchOptions::budget(50);
        let r = search(&state, &e, opts);
        // 50 ms must hard-bound elapsed by some slack (CI noise tolerant).
        assert!(r.elapsed < Duration::from_millis(500));
        assert!(r.best_move.is_some());
    }

    #[test]
    fn search_finds_mate_in_one_via_hamna() {
        // North has just one mbele kete at vichwa[4]. The geometric opposite
        // of that on South's side is south.vichwa[7-4=3]. If South plays
        // a namu placing into mbele[3] this becomes a kula; the kichwa
        // empties North's only mbele kete → hamna → South wins.
        use crate::board::{Phase, Side, Substate, PITS_PER_SIDE};
        let mut state = BoardState::new(Variant::Kiswahili);
        // Wipe pieces; rebuild a clean 1-mbele-kete-on-each-side setup.
        for s in &mut state.sides {
            *s = Side {
                vichwa: [0u8; PITS_PER_SIDE],
                ghala: s.ghala,
                nyumba_owned: false, // remove nyumba to simplify
                nyumba_col: 4,
            };
        }
        state.sides[0].vichwa[3] = 1; // South pre-place position
        state.sides[1].vichwa[4] = 1; // North's only mbele kete (mirror of 3)
        state.phase = Phase::Namu(Substate::AwaitMove);

        let e = HeuristicEval::new();
        let r = search(&state, &e, SearchOptions::depth(4));
        assert!(r.best_move.is_some());
        assert!(
            r.score >= MATE_THRESHOLD,
            "expected forced-win score, got {}",
            r.score
        );
    }
}
