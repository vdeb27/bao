//! Negamax with alpha-beta and a capture-only quiescence extension.
//!
//! Negamax convention: each call returns the score from the perspective of
//! the player to move; the caller negates when traversing into a child node.
//! This works for substates too, because applying a Kichwa/Safari resolution
//! doesn't necessarily flip the active player — the inner loop just keeps
//! sowing from the same player's view, and the score sign tracks whoever is
//! to move in the *resulting* state.

use crate::board::{Phase, Substate};
use crate::eval::Evaluator;
use crate::moves::Move;
use crate::rules::legal_moves;

use super::{order_moves, try_apply, MATE_SCORE};

const QSEARCH_MAX_DEPTH: u8 = 8;

pub(crate) struct SearchCtx<'a, E: Evaluator> {
    pub(crate) eval: &'a E,
    pub(crate) nodes: u64,
    pub(crate) node_budget: u64,
    pub(crate) aborted: bool,
}

impl<'a, E: Evaluator> SearchCtx<'a, E> {
    pub(crate) fn new(eval: &'a E, node_budget: u64) -> Self {
        Self {
            eval,
            nodes: 0,
            node_budget,
            aborted: false,
        }
    }

    #[inline]
    fn tick(&mut self) -> bool {
        self.nodes += 1;
        if self.nodes >= self.node_budget {
            self.aborted = true;
        }
        !self.aborted
    }
}

pub(crate) struct RootResult {
    pub(crate) best_move: Option<Move>,
    pub(crate) score: i32,
}

pub(crate) fn root_negamax<E: Evaluator>(
    ctx: &mut SearchCtx<'_, E>,
    state: &crate::board::BoardState,
    depth: u8,
) -> RootResult {
    if let Some(w) = state.winner {
        // Terminal at the root: encode loss/win from active player's view.
        let score = if w == state.active { MATE_SCORE } else { -MATE_SCORE };
        return RootResult {
            best_move: None,
            score,
        };
    }

    let mut moves = legal_moves(state);
    if moves.is_empty() {
        return RootResult {
            best_move: None,
            score: -MATE_SCORE,
        };
    }
    order_moves(state, &mut moves);

    let mut alpha = -MATE_SCORE;
    let beta = MATE_SCORE;
    let mut best_score = -MATE_SCORE;
    let mut best_move: Option<Move> = None;

    for mv in moves {
        let Some(child) = try_apply(state, mv) else { continue };
        if !ctx.tick() {
            break;
        }
        // If the child's active player is the same as ours (substate
        // transition without turn flip), don't negate.
        let score = if child.active == state.active {
            negamax(ctx, &child, depth.saturating_sub(1), alpha, beta)
        } else {
            -negamax(ctx, &child, depth.saturating_sub(1), -beta, -alpha)
        };
        if ctx.aborted {
            break;
        }
        if score > best_score {
            best_score = score;
            best_move = Some(mv);
        }
        if best_score > alpha {
            alpha = best_score;
        }
    }

    // If aborted before any score, return whatever we have so the caller
    // can fall back to the previous depth.
    if best_move.is_none() && !ctx.aborted {
        best_move = Some(legal_moves(state)[0]);
    }
    RootResult {
        best_move,
        score: best_score,
    }
}

fn negamax<E: Evaluator>(
    ctx: &mut SearchCtx<'_, E>,
    state: &crate::board::BoardState,
    depth: u8,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    if let Some(w) = state.winner {
        // Loss for the side currently to move is -MATE; win for them is +MATE.
        return if w == state.active { MATE_SCORE } else { -MATE_SCORE };
    }
    if depth == 0 {
        return quiescence(ctx, state, 0, alpha, beta);
    }
    if !ctx.tick() {
        return 0;
    }

    let mut moves = legal_moves(state);
    if moves.is_empty() {
        // Stalemate: side to move loses (engine maps this to a winner already
        // in most cases, but if not, treat as a loss).
        return -MATE_SCORE;
    }
    order_moves(state, &mut moves);

    let mut best = -MATE_SCORE;
    for mv in moves {
        let Some(child) = try_apply(state, mv) else { continue };
        let score = if child.active == state.active {
            negamax(ctx, &child, depth - 1, alpha, beta)
        } else {
            -negamax(ctx, &child, depth - 1, -beta, -alpha)
        };
        if ctx.aborted {
            return 0;
        }
        if score > best {
            best = score;
        }
        if best > alpha {
            alpha = best;
        }
        if alpha >= beta {
            break; // beta cutoff
        }
    }
    best
}

/// Capture-only extension. Standard recipe with a "stand-pat" baseline:
/// take the static eval, and only consider capture moves to see if we can
/// do better. The qdepth cap stops endelea-chains from exploding.
fn quiescence<E: Evaluator>(
    ctx: &mut SearchCtx<'_, E>,
    state: &crate::board::BoardState,
    qdepth: u8,
    mut alpha: i32,
    beta: i32,
) -> i32 {
    if let Some(w) = state.winner {
        return if w == state.active { MATE_SCORE } else { -MATE_SCORE };
    }
    if !ctx.tick() {
        return 0;
    }

    let stand_pat = ctx.eval.eval(state);

    // In substate transitions (AwaitKichwa / AwaitSafari) the player is
    // mid-capture; we must keep exploring even past qdepth to resolve the
    // chain. The terminating substate is AwaitMove.
    let in_substate = !matches!(
        state.phase,
        Phase::Namu(Substate::AwaitMove) | Phase::Mtaji(Substate::AwaitMove)
    );

    if !in_substate {
        if stand_pat >= beta {
            return beta;
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }
        if qdepth >= QSEARCH_MAX_DEPTH {
            return alpha;
        }
    }

    let mut moves = legal_moves(state);
    order_moves(state, &mut moves);

    // Filter to capture-style continuations (and substate resolutions).
    let mut explored = 0;
    for mv in moves {
        let is_capture_like = match mv {
            Move::Kichwa(_) | Move::Safari { .. } => true, // resolve in-flight capture
            Move::Mtaji { pit, dir } => {
                // Re-check the engine's capture eligibility cheaply.
                let count = state.sides[state.active as usize].vichwa[pit as usize] as i32;
                if !(2..=15).contains(&count) {
                    false
                } else {
                    let land = crate::rules::landing(pit, dir, count as u8) as usize;
                    let opp = state.opponent(state.active) as usize;
                    land < crate::board::MBELE_LEN
                        && state.sides[state.active as usize].vichwa[land] >= 1
                        && state.sides[opp].vichwa[7 - land] >= 1
                }
            }
            Move::Namu { col, .. } => {
                let opp = state.opponent(state.active) as usize;
                state.sides[opp].vichwa[(7 - col) as usize] >= 1
            }
        };
        if !is_capture_like && !in_substate {
            continue;
        }
        let Some(child) = try_apply(state, mv) else { continue };
        let score = if child.active == state.active {
            quiescence(ctx, &child, qdepth + 1, alpha, beta)
        } else {
            -quiescence(ctx, &child, qdepth + 1, -beta, -alpha)
        };
        if ctx.aborted {
            return 0;
        }
        explored += 1;
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    if in_substate && explored == 0 {
        // No legal continuation from a substate — fall back to stand-pat.
        return stand_pat;
    }
    alpha
}
