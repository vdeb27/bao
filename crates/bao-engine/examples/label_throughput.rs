//! Fase-4 acceptance prototype: meet labeling-throughput op random posities
//! met dezelfde search-config die de generator straks gebruikt. Plan §11
//! eist ≥2k pos/sec single-core. Dit example draait BEFORE we de pipeline
//! bouwen zodat we kunnen kalibreren op depth/node-budget.
//!
//! Run: `cargo run --release --example label_throughput`

use bao_engine::{
    apply, legal_moves, search, BoardState, HeuristicEval, Move, SearchOptions, Variant,
};

struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self { Self(seed) }
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

/// Walk a random self-play game and return up to `n` mid-game snapshots.
fn collect_random_positions(n: usize, seed: u64) -> Vec<BoardState> {
    let mut out = Vec::with_capacity(n);
    let mut rng = Rng::new(seed);
    'outer: while out.len() < n {
        let mut state = BoardState::new(Variant::Kiswahili);
        let mut plies = 0;
        while state.winner.is_none() && plies < 200 {
            if plies >= 4 && plies % 3 == 0 {
                out.push(state.clone());
                if out.len() >= n {
                    break 'outer;
                }
            }
            let moves = legal_moves(&state);
            if moves.is_empty() { break; }
            let mv: Move = rng.pick(&moves);
            match apply(&state, mv) {
                Ok((next, _)) => { state = next; plies += 1; }
                Err(_) => break,
            }
        }
    }
    out
}

fn main() {
    let n_pos = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(200);
    let max_depth: u8 = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(8);
    let max_nodes: u64 = std::env::args().nth(3).and_then(|s| s.parse().ok()).unwrap_or(50_000);

    println!("collecting {} random positions...", n_pos);
    let positions = collect_random_positions(n_pos, 0xDEAD_BEEF_C0FFEE);
    println!("got {} positions", positions.len());

    let eval = HeuristicEval::new();
    let opts = SearchOptions {
        max_depth,
        max_nodes,
        tt_slots: 1 << 16,
    };

    let mut total_nodes: u64 = 0;
    let mut depth_hist = [0u32; 32];
    let t0 = std::time::Instant::now();
    for state in &positions {
        let r = search(state, &eval, opts);
        total_nodes += r.nodes;
        depth_hist[(r.depth_reached.min(31)) as usize] += 1;
    }
    let elapsed = t0.elapsed().as_secs_f64();
    let pps = positions.len() as f64 / elapsed;
    let nps = total_nodes as f64 / elapsed;

    println!("depth_cap: {}, node_cap: {}", max_depth, max_nodes);
    println!("labeled {} positions in {:.2}s", positions.len(), elapsed);
    println!("throughput: {:.1} pos/sec/core  (target ≥2000)", pps);
    println!("nodes:      {} total, {:.0} NPS", total_nodes, nps);
    print!("depth-hist: ");
    for (d, c) in depth_hist.iter().enumerate() {
        if *c > 0 { print!("d{}={} ", d, c); }
    }
    println!();
}
