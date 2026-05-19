use bao_engine::*;
fn main() {
    let state = BoardState::new(Variant::Kiswahili);
    let eval = HeuristicEval::new();
    println!("--- with TT ---");
    for d in 1..=10 {
        let opts = SearchOptions::depth(d);
        let t0 = std::time::Instant::now();
        let r = search(&state, &eval, opts);
        let dt = t0.elapsed().as_micros();
        println!(
            "depth: {}, nodes: {}, time: {}us, score: {}, best: {:?}",
            r.depth_reached, r.nodes, dt, r.score, r.best_move
        );
    }
    println!("--- without TT ---");
    for d in 1..=10 {
        let opts = SearchOptions {
            max_depth: d,
            max_nodes: u64::MAX,
            tt_slots: 0,
        };
        let t0 = std::time::Instant::now();
        let r = search(&state, &eval, opts);
        let dt = t0.elapsed().as_micros();
        println!(
            "depth: {}, nodes: {}, time: {}us, score: {}, best: {:?}",
            r.depth_reached, r.nodes, dt, r.score, r.best_move
        );
    }
}
