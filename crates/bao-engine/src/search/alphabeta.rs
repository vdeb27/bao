//! Negamax with alpha-beta, transposition table and a capture-only
//! quiescence extension.
//!
//! Negamax convention: each call returns the score from the perspective of
//! the player to move; the caller negates when traversing into a child node.
//! Substate transitions (AwaitKichwa/AwaitSafari) don't necessarily flip the
//! active player — the inner loop checks the child's `active` and only
//! negates the recursive score when it differs from the parent's.

use crate::board::{Phase, Substate};
use crate::eval::Evaluator;
use crate::moves::Move;
use crate::rules::legal_moves;
use crate::zobrist::zobrist_key;

use super::tt::{pack_move, unpack_move, Bound, TranspositionTable};
use super::{order_moves, try_apply, MATE_SCORE, MATE_THRESHOLD};

const QSEARCH_MAX_DEPTH: u8 = 8;

pub(crate) struct SearchCtx<'a, E: Evaluator> {
    pub(crate) eval: &'a E,
    pub(crate) nodes: u64,
    pub(crate) node_budget: u64,
    pub(crate) aborted: bool,
    pub(crate) tt: Option<&'a mut TranspositionTable>,
}

impl<'a, E: Evaluator> SearchCtx<'a, E> {
    pub(crate) fn new(
        eval: &'a E,
        node_budget: u64,
        tt: Option<&'a mut TranspositionTable>,
    ) -> Self {
        Self {
            eval,
            nodes: 0,
            node_budget,
            aborted: false,
            tt,
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

    let key = zobrist_key(state);
    let tt_move = ctx
        .tt
        .as_deref()
        .and_then(|tt| tt.probe(key))
        .and_then(|e| unpack_move(e.best_move));
    order_moves(state, &mut moves, tt_move);

    let mut alpha = -MATE_SCORE;
    let beta = MATE_SCORE;
    let mut best_score = -MATE_SCORE;
    let mut best_move: Option<Move> = None;

    for mv in moves {
        let Some(child) = try_apply(state, mv) else { continue };
        if !ctx.tick() {
            break;
        }
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

    if let (Some(mv), Some(tt)) = (best_move, ctx.tt.as_deref_mut()) {
        tt.store(key, pack_move(mv), best_score, depth, Bound::Exact);
    }
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
    mut beta: i32,
) -> i32 {
    if let Some(w) = state.winner {
        return if w == state.active { MATE_SCORE } else { -MATE_SCORE };
    }
    if depth == 0 {
        return quiescence(ctx, state, 0, alpha, beta);
    }
    if !ctx.tick() {
        return 0;
    }

    // --- TT probe ---
    let key = zobrist_key(state);
    let alpha_orig = alpha;
    let mut tt_move: Option<Move> = None;
    if let Some(entry) = ctx.tt.as_deref().and_then(|tt| tt.probe(key)) {
        tt_move = unpack_move(entry.best_move);
        if entry.depth >= depth {
            let bound = Bound::from_u8_external(entry.flag);
            match bound {
                Bound::Exact => return entry.score,
                Bound::Lower => alpha = alpha.max(entry.score),
                Bound::Upper => beta = beta.min(entry.score),
                Bound::None => {}
            }
            if alpha >= beta {
                return entry.score;
            }
        }
    }

    let mut moves = legal_moves(state);
    if moves.is_empty() {
        return -MATE_SCORE;
    }
    order_moves(state, &mut moves, tt_move);

    let mut best = -MATE_SCORE;
    let mut best_mv: Option<Move> = None;
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
            best_mv = Some(mv);
        }
        if best > alpha {
            alpha = best;
        }
        if alpha >= beta {
            break;
        }
    }

    // --- TT store ---
    if let (Some(mv), Some(tt)) = (best_mv, ctx.tt.as_deref_mut()) {
        let bound = if best <= alpha_orig {
            Bound::Upper
        } else if best >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };
        tt.store(key, pack_move(mv), best, depth, bound);
    }
    best
}

/// Capture-only extension. Stand-pat baseline; only capture moves are
/// considered to refine it. The qdepth cap stops endelea-chains from
/// exploding, but substate continuations bypass it so a capture-sow in
/// flight is always resolved.
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
    order_moves(state, &mut moves, None);

    let mut explored = 0;
    for mv in moves {
        let is_capture_like = match mv {
            Move::Kichwa(_) | Move::Safari { .. } => true,
            Move::Mtaji { pit, dir } => {
                let count =
                    state.sides[state.active as usize].vichwa[pit as usize] as i32;
                if !(2..=15).contains(&count) {
                    false
                } else {
                    let land =
                        crate::rules::landing(pit, dir, count as u8) as usize;
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
        return stand_pat;
    }
    // Suppress unused-variable warnings in mate threshold tests that don't
    // touch MATE_THRESHOLD via this function.
    let _ = MATE_THRESHOLD;
    alpha
}

// Extension: Bound::from_u8 is private to the tt module; mirror it locally
// to keep the alphabeta translation unit self-contained.
trait BoundExt {
    fn from_u8_external(v: u8) -> Bound;
}
impl BoundExt for Bound {
    fn from_u8_external(v: u8) -> Bound {
        match v {
            1 => Bound::Exact,
            2 => Bound::Lower,
            3 => Bound::Upper,
            _ => Bound::None,
        }
    }
}
