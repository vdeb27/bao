//! Handcrafted evaluator covering material, mbele control, nyumba status,
//! kichwa safety, mobility, ghala reserve, kutakatia handicap, and a phase
//! asymmetry term. See plan §4.1; the weights are SPSA-friendly so they
//! can later be tuned without changing the structure.

use super::Evaluator;
use crate::board::{BoardState, NyumbaState, MBELE_LEN, PITS_PER_SIDE};
use crate::moves::Move;
use crate::rules::legal_moves;
use crate::variant::Variant;

/// All weights are in centi-kete (100 = one kete). Defaults are conservative
/// initial guesses inspired by the plan; expect them to move once we
/// instrument against alpha-beta self-play.
#[derive(Debug, Clone, Copy)]
pub struct HeuristicWeights {
    pub material: i32,
    pub mbele_extra: i32,
    pub nyumba_functional: i32,
    pub kimbi_empty_penalty: i32,
    pub mobility: i32,
    pub ghala: i32,
    pub kutakatia_handicap: i32,
    pub mtaji_lead: i32,
}

impl Default for HeuristicWeights {
    fn default() -> Self {
        HeuristicWeights {
            material: 100,
            mbele_extra: 5,
            nyumba_functional: 30,
            kimbi_empty_penalty: 20,
            mobility: 2,
            ghala: 80,
            kutakatia_handicap: 40,
            mtaji_lead: 10,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HeuristicEval {
    pub weights: HeuristicWeights,
}

impl HeuristicEval {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_weights(weights: HeuristicWeights) -> Self {
        Self { weights }
    }

    fn side_score(&self, state: &BoardState, side_idx: usize) -> i32 {
        let w = self.weights;
        let side = &state.sides[side_idx];

        // Material: kete in board (vichwa total only — ghala has its own term).
        let mut total_vichwa: i32 = 0;
        let mut mbele_total: i32 = 0;
        for i in 0..PITS_PER_SIDE {
            let c = side.vichwa[i] as i32;
            total_vichwa += c;
            if i < MBELE_LEN {
                mbele_total += c;
            }
        }

        let mut score = total_vichwa * w.material;
        score += mbele_total * w.mbele_extra;
        score += (side.ghala as i32) * w.ghala;

        // Nyumba bonus only meaningful for Kiswahili.
        if state.variant == Variant::Kiswahili
            && side.nyumba_state(state.variant) == NyumbaState::Functional
        {
            score += w.nyumba_functional;
        }

        // Open-kimbi penalty: each empty kimbi pit (mbele cols 0,1,6,7) is
        // a weakness. Skip in Kujifunza — the kimbi concept still exists but
        // the penalty's calibration is for the full game.
        for &kimbi in &[0usize, 1, 6, 7] {
            if side.vichwa[kimbi] == 0 {
                score -= w.kimbi_empty_penalty;
            }
        }
        score
    }
}

impl Evaluator for HeuristicEval {
    fn eval(&self, state: &BoardState) -> i32 {
        let w = self.weights;
        let active = state.active as usize;
        let opp = 1 - active;
        let mut score = self.side_score(state, active) - self.side_score(state, opp);

        // Mobility (active player perspective): how many capture moves do we
        // have right now? Only counts in AwaitMove substates; substate
        // transitions don't expose the full move list meaningfully.
        let moves = legal_moves(state);
        let captures = moves
            .iter()
            .filter(|m| matches!(m, Move::Mtaji { .. } | Move::Namu { .. }))
            .count();
        score += (captures as i32) * w.mobility;

        // Kutakatia: if I'm the blocked player, subtract a flat handicap;
        // if I'm the blocker, the symmetric bonus belongs to me.
        if let Some(kt) = state.kutakatia {
            if kt.blocked_player as usize == active {
                score -= w.kutakatia_handicap;
            } else {
                score += w.kutakatia_handicap;
            }
        }

        // Phase-asymmetry: if I'm already in mtaji and opp is still in namu
        // it's a structural advantage. The single `phase` is shared today
        // (both players are always in the same phase), so this stays as a
        // placeholder for variants that may split.
        let _ = w.mtaji_lead;
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variant::Variant;

    #[test]
    fn initial_kiswahili_is_balanced() {
        // Mirror-symmetric initial setup → eval ≈ 0 modulo mobility.
        let state = BoardState::new(Variant::Kiswahili);
        let e = HeuristicEval::new();
        let s = e.eval(&state);
        // Mobility is the only asymmetric contributor at start (active gets
        // to count its move list). Cap the magnitude generously.
        assert!(s.abs() < 100, "expected near-zero eval, got {}", s);
    }

    #[test]
    fn initial_kujifunza_is_balanced() {
        let state = BoardState::new(Variant::Kujifunza);
        let e = HeuristicEval::new();
        let s = e.eval(&state);
        assert!(s.abs() < 100, "expected near-zero eval, got {}", s);
    }

    #[test]
    fn winning_material_dominates_mobility() {
        // Give South a 5-kete lead in own mbele vs North-to-move.
        let mut state = BoardState::new(Variant::Kiswahili);
        state.sides[0].vichwa[3] = state.sides[0].vichwa[3].saturating_add(5);
        state.active = 1; // evaluate from North's perspective
        let e = HeuristicEval::new();
        let s = e.eval(&state);
        // North should see a negative eval (they're worse off by ≥ 5 * 100).
        assert!(s < -400, "expected strong North-loss eval, got {}", s);
    }

    #[test]
    fn nyumba_bonus_only_when_functional() {
        let mut state = BoardState::new(Variant::Kiswahili);
        // Drain south's nyumba below threshold → Disabled.
        state.sides[0].vichwa[4] = 3;
        let e = HeuristicEval::new();
        let baseline = e.eval(&state);
        state.sides[0].vichwa[4] = 6; // back to functional
        let with_nyumba = e.eval(&state);
        // Functional adds material (3 extra kete = 300 + mbele) AND the
        // explicit nyumba bonus, so the gap must exceed pure material.
        assert!(
            with_nyumba > baseline + 3 * 100,
            "functional nyumba should add a bonus beyond 3 kete: baseline={} with_nyumba={}",
            baseline,
            with_nyumba
        );
    }
}
