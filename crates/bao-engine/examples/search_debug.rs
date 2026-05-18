use bao_engine::*;
fn main() {
    let state = BoardState::new(Variant::Kiswahili);
    let eval = HeuristicEval::new();
    for d in 1..=8 {
        let opts = SearchOptions::depth(d);
        let r = search(&state, &eval, opts);
        println!("depth: {}, nodes: {}, score: {}, best: {:?}", r.depth_reached, r.nodes, r.score, r.best_move);
    }
}
