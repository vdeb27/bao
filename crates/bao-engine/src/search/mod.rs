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

use crate::board::BoardState;
use crate::eval::Evaluator;
use crate::moves::Move;
use crate::rules::{apply, legal_moves};

pub mod alphabeta;
pub mod tt;

pub use tt::{Bound, TranspositionTable};

/// Scores ≥ MATE_THRESHOLD represent forced wins (and ≤ -MATE_THRESHOLD
/// forced losses), with the distance-to-mate encoded as `MATE_SCORE - ply`.
/// Keeps mate scores well outside any plausible evaluator output.
pub const MATE_SCORE: i32 = 1_000_000;
pub const MATE_THRESHOLD: i32 = 900_000;

/// Search budget. Soft bounds via `max_depth` and `max_nodes`; whichever runs
/// out first ends the search. We use a node budget rather than wall-clock
/// because `std::time::Instant` panics under `wasm32-unknown-unknown`. The
/// caller picks a node count appropriate for their target's throughput
/// (≈100 k nodes ≈ a few hundred ms on a desktop CPU). `tt_slots` is the
/// transposition-table size in entries (rounded up to a power of two,
/// ≈16 bytes each); 0 disables the TT entirely.
#[derive(Debug, Clone, Copy)]
pub struct SearchOptions {
    pub max_depth: u8,
    pub max_nodes: u64,
    pub tt_slots: usize,
}

impl SearchOptions {
    pub fn depth(d: u8) -> Self {
        Self {
            max_depth: d,
            max_nodes: u64::MAX,
            tt_slots: 1 << 16,
        }
    }
    pub fn nodes(n: u64) -> Self {
        Self {
            max_depth: 64,
            max_nodes: n,
            tt_slots: 1 << 16,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
    pub depth_reached: u8,
    pub nodes: u64,
}

/// Top-level entry. Runs iterative deepening from depth 1 upward and returns
/// the deepest fully-completed iteration. If time runs out mid-iteration the
/// previous iteration's best move is returned.
/// Aspiration window half-width (in centi-kete) that the root search
/// brackets around the previous iteration's score from depth 4 onwards.
const ASPIRATION_INITIAL: i32 = 25;

pub fn search<E: Evaluator>(
    state: &BoardState,
    eval: &E,
    opts: SearchOptions,
) -> SearchResult {
    let mut tt = if opts.tt_slots > 0 {
        Some(TranspositionTable::new(opts.tt_slots))
    } else {
        None
    };
    let mut best = SearchResult {
        best_move: None,
        score: 0,
        depth_reached: 0,
        nodes: 0,
    };
    let mut total_nodes: u64 = 0;
    let mut prev_score: i32 = 0;

    for depth in 1..=opts.max_depth {
        let remaining = opts.max_nodes.saturating_sub(total_nodes);
        if remaining == 0 {
            break;
        }

        // Aspiration loop: start with a narrow window around prev_score; on
        // a fail-low/high, widen exponentially and re-run.
        let mut window = ASPIRATION_INITIAL;
        let mut alpha = if depth >= 4 {
            prev_score - window
        } else {
            -MATE_SCORE
        };
        let mut beta = if depth >= 4 {
            prev_score + window
        } else {
            MATE_SCORE
        };

        let mut r = alphabeta::RootResult {
            best_move: None,
            score: 0,
        };
        let mut consumed_total: u64 = 0;
        let mut attempts = 0;
        loop {
            let budget = remaining.saturating_sub(consumed_total);
            if budget == 0 {
                break;
            }
            let mut ctx = alphabeta::SearchCtx::new(eval, budget, tt.as_mut());
            r = alphabeta::root_negamax_window(&mut ctx, state, depth, alpha, beta);
            consumed_total = consumed_total.saturating_add(ctx.nodes);
            if ctx.aborted {
                break;
            }
            attempts += 1;
            // Full-window already — no further widening possible.
            if alpha <= -MATE_SCORE && beta >= MATE_SCORE {
                break;
            }
            // Cap retries so a pathological eval can't spin us forever.
            if attempts >= 4 {
                alpha = -MATE_SCORE;
                beta = MATE_SCORE;
                continue;
            }
            if r.score <= alpha {
                alpha = alpha.saturating_sub(window).max(-MATE_SCORE);
                window = window.saturating_mul(2);
            } else if r.score >= beta {
                beta = beta.saturating_add(window).min(MATE_SCORE);
                window = window.saturating_mul(2);
            } else {
                break;
            }
        }
        total_nodes = total_nodes.saturating_add(consumed_total);
        if r.best_move.is_none() {
            break;
        }

        best = SearchResult {
            best_move: r.best_move,
            score: r.score,
            depth_reached: depth,
            nodes: total_nodes,
        };
        prev_score = r.score;

        if r.score.abs() >= MATE_THRESHOLD {
            break;
        }
    }

    best
}

/// Convenience: orders captures (Namu/Mtaji) before resolution moves
/// (Kichwa/Safari). When a `tt_move` is supplied it takes priority over
/// everything else so the search explores the previously-best line first.
/// For now the captures themselves aren't graded; a future slice introduces
/// MVV-LVA on `kete_taken`.
pub(crate) fn order_moves(state: &BoardState, moves: &mut [Move], tt_move: Option<Move>) {
    moves.sort_by_key(|m| {
        if tt_move.is_some_and(|tm| tm == *m) {
            return -1;
        }
        match m {
            Move::Kichwa(_) | Move::Safari { .. } => 2,
            Move::Namu { col, .. } => {
                let opp = state.opponent(state.active) as usize;
                let opp_idx = 7usize.saturating_sub(*col as usize);
                if state.sides[opp].vichwa[opp_idx] >= 1 {
                    0
                } else {
                    1
                }
            }
            Move::Mtaji { .. } => 1,
        }
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
    fn search_respects_node_budget() {
        let state = BoardState::new(Variant::Kiswahili);
        let e = HeuristicEval::new();
        let r = search(&state, &e, SearchOptions::nodes(200));
        assert!(r.nodes <= 200 + 1024, "nodes: {}", r.nodes);
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
        for s in &mut state.sides {
            *s = Side {
                vichwa: [0u8; PITS_PER_SIDE],
                ghala: s.ghala,
                nyumba_owned: false,
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

    #[test]
    fn search_finds_mate_in_one_via_mtaji_kula() {
        // Mtaji-phase mate. South pit 5 has 3 kete; sowing Ccw lands at
        // own.mbele[2]. Geometric opposite is opp.vichwa[5], which is
        // North's only mbele kete → hamna after capture.
        use crate::board::{Phase, Side, Substate, PITS_PER_SIDE};
        let mut state = BoardState::new(Variant::Kiswahili);
        for s in &mut state.sides {
            *s = Side {
                vichwa: [0u8; PITS_PER_SIDE],
                ghala: 0,
                nyumba_owned: false,
                nyumba_col: 4,
            };
        }
        state.sides[0].vichwa[5] = 3;
        state.sides[0].vichwa[2] = 1;
        state.sides[1].vichwa[5] = 1;
        state.phase = Phase::Mtaji(Substate::AwaitMove);

        let e = HeuristicEval::new();
        let r = search(&state, &e, SearchOptions::depth(4));
        assert!(r.best_move.is_some());
        assert!(
            r.score >= MATE_THRESHOLD,
            "expected forced-win score, got {}",
            r.score
        );
    }

    #[test]
    fn search_finds_mate_in_one_kichwa_kimbi() {
        // Force a kimbi-side kichwa (only one legal choice). North has
        // one mbele kete opposite South's preplace at col 1 (mirror of 6).
        // Namu placement at col 1 triggers a capture; kichwa is forced
        // Left (RULES.md §6.3, kimbi cols 0,1 → Left only).
        use crate::board::{Phase, Side, Substate, PITS_PER_SIDE};
        let mut state = BoardState::new(Variant::Kiswahili);
        for s in &mut state.sides {
            *s = Side {
                vichwa: [0u8; PITS_PER_SIDE],
                ghala: s.ghala,
                nyumba_owned: false,
                nyumba_col: 4,
            };
        }
        state.sides[0].vichwa[1] = 1;
        state.sides[1].vichwa[6] = 1;
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

    #[test]
    fn search_avoids_obvious_loss() {
        // If only one mbele kete remains on South's side and it's about to
        // be captured by North, the score from South's perspective should
        // be deeply negative when North is to move.
        use crate::board::{Phase, Side, Substate, PITS_PER_SIDE};
        let mut state = BoardState::new(Variant::Kiswahili);
        for s in &mut state.sides {
            *s = Side {
                vichwa: [0u8; PITS_PER_SIDE],
                ghala: s.ghala,
                nyumba_owned: false,
                nyumba_col: 4,
            };
        }
        state.sides[0].vichwa[4] = 1; // South's only mbele kete
        state.sides[1].vichwa[3] = 1; // North preplace (mirror of 4 = 3)
        state.active = 1; // North to move
        state.phase = Phase::Namu(Substate::AwaitMove);

        let e = HeuristicEval::new();
        let r = search(&state, &e, SearchOptions::depth(4));
        // From North's perspective this is a forced win.
        assert!(
            r.score >= MATE_THRESHOLD,
            "expected forced-win for North, got {}",
            r.score
        );
    }
}
