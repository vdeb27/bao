//! Self-play arena: heuristic alpha-beta vs uniformly-random.
//! Plan §11 acceptance for fase 3 asks for ≥80% winrate over 200 games.
//! Run with `cargo run --release --example arena_vs_random`.

use bao_engine::{
    apply, legal_moves, search, BoardState, HeuristicEval, Move, SearchOptions, Variant,
};

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn pick<T: Copy>(&mut self, items: &[T]) -> T {
        items[(self.next_u64() as usize) % items.len()]
    }
}

/// Run one game; return Some(winner) or None for draw / inconclusive.
fn play_one(eval: &HeuristicEval, ai_side: u8, seed: u64, max_plies: u32) -> Option<u8> {
    let mut rng = Rng::new(seed);
    let mut state = BoardState::new(Variant::Kiswahili);
    let opts = SearchOptions {
        max_depth: 8,
        max_nodes: 50_000,
        tt_slots: 1 << 14,
    };
    for _ in 0..max_plies {
        if let Some(w) = state.winner {
            return Some(w);
        }
        let moves = legal_moves(&state);
        if moves.is_empty() {
            return Some(1 - state.active);
        }
        let mv: Move = if state.active == ai_side {
            let r = search(&state, eval, opts);
            r.best_move.unwrap_or_else(|| rng.pick(&moves))
        } else {
            rng.pick(&moves)
        };
        match apply(&state, mv) {
            Ok((next, _)) => state = next,
            Err(_) => return None,
        }
    }
    None
}

fn main() {
    let eval = HeuristicEval::new();
    let games = std::env::args()
        .nth(1)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(100);
    let mut ai_wins = 0u32;
    let mut random_wins = 0u32;
    let mut draws = 0u32;
    let mut total_elapsed = std::time::Duration::ZERO;
    for i in 0..games {
        // Alternate sides so the AI plays both South and North equally.
        let ai_side: u8 = (i % 2) as u8;
        let seed = 0xC0FFEE_BABE_DEAD ^ (i as u64).wrapping_mul(0x9E3779B97F4A7C15);
        let t0 = std::time::Instant::now();
        let result = play_one(&eval, ai_side, seed, 600);
        total_elapsed += t0.elapsed();
        match result {
            Some(w) if w == ai_side => ai_wins += 1,
            Some(_) => random_wins += 1,
            None => draws += 1,
        }
    }
    println!("games:        {}", games);
    println!("AI wins:      {} ({:.1}%)", ai_wins, 100.0 * ai_wins as f32 / games as f32);
    println!("Random wins:  {} ({:.1}%)", random_wins, 100.0 * random_wins as f32 / games as f32);
    println!("Inconclusive: {}", draws);
    println!("total time:   {:.2}s", total_elapsed.as_secs_f32());
    println!("per game:     {:.0}ms", total_elapsed.as_millis() as f32 / games as f32);
}
