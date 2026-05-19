//! NPS (nodes per second) benchmark. Plan §3.4 target: ≥500k NPS
//! single-threaded with the heuristic eval on a desktop CPU.

use bao_engine::{search, BoardState, HeuristicEval, SearchOptions, Variant};

fn main() {
    let eval = HeuristicEval::new();
    let state = BoardState::new(Variant::Kiswahili);

    // Run a budgeted search and divide nodes by wall-clock time.
    let opts = SearchOptions {
        max_depth: 64,
        max_nodes: 5_000_000,
        tt_slots: 1 << 18, // 256k slots ≈ 4 MB
    };
    let t0 = std::time::Instant::now();
    let r = search(&state, &eval, opts);
    let elapsed = t0.elapsed();
    let secs = elapsed.as_secs_f64();
    let nps = (r.nodes as f64) / secs;
    println!("depth: {}, nodes: {}, time: {:.3}s", r.depth_reached, r.nodes, secs);
    println!("NPS:   {:.0}", nps);
    println!("score: {}", r.score);
    println!("move:  {:?}", r.best_move);
}
