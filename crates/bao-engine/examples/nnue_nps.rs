//! NPS bench for the NNUE evaluator (plan §11 fase-5 acceptance: ≥200k full
//! re-eval/sec on desktop). Uses a synthetic zero-model so we measure the
//! transformer + forward pass alone, not training quality.
//!
//! Usage: `cargo run --release --example nnue_nps -- <iterations>`
//! Default iterations: 100_000.

use std::time::Instant;

use bao_engine::board::BoardState;
use bao_engine::eval::nnue::loader::{
    load_from_bytes, ACCUMULATOR_DIM, HIDDEN_DIM, MAGIC, OUTPUT_SCALE, VERSION,
};
use bao_engine::eval::nnue::N_FEATURES;
use bao_engine::rules::{apply, legal_moves};
use bao_engine::variant::Variant;

fn synth_zero_blob() -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(MAGIC);
    buf.extend_from_slice(&VERSION.to_le_bytes());
    buf.extend_from_slice(&(N_FEATURES as u16).to_le_bytes());
    for &s in &[ACCUMULATOR_DIM as u16, HIDDEN_DIM as u16, HIDDEN_DIM as u16, 1u16] {
        buf.extend_from_slice(&s.to_le_bytes());
    }
    buf.extend_from_slice(&(OUTPUT_SCALE as f32).to_le_bytes());
    buf.resize(buf.len() + N_FEATURES * ACCUMULATOR_DIM * 2, 0);
    buf.resize(buf.len() + ACCUMULATOR_DIM * 2, 0);
    buf.resize(buf.len() + ACCUMULATOR_DIM * HIDDEN_DIM, 0);
    buf.resize(buf.len() + HIDDEN_DIM * 4, 0);
    buf.resize(buf.len() + HIDDEN_DIM * HIDDEN_DIM, 0);
    buf.resize(buf.len() + HIDDEN_DIM * 4, 0);
    buf.resize(buf.len() + HIDDEN_DIM, 0);
    buf.resize(buf.len() + 4, 0);
    buf
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let iters: usize = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(100_000);

    let blob = synth_zero_blob();
    let model = load_from_bytes(&blob).expect("load");

    // Generate a varied set of positions via random self-play.
    let mut positions: Vec<BoardState> = Vec::new();
    let mut s = BoardState::new(Variant::Kiswahili);
    let mut seed: u64 = 0x1234_5678_9abc_def0;
    for _ in 0..500 {
        positions.push(s);
        let moves = legal_moves(&s);
        if moves.is_empty() {
            s = BoardState::new(Variant::Kiswahili);
            continue;
        }
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let idx = (seed >> 33) as usize % moves.len();
        match apply(&s, moves[idx]) {
            Ok((n, _)) => s = n,
            Err(_) => s = BoardState::new(Variant::Kiswahili),
        }
    }

    // Warm-up
    let mut total: i64 = 0;
    for p in positions.iter().take(1000) {
        total = total.wrapping_add(model.evaluate(p) as i64);
    }

    let start = Instant::now();
    let mut total: i64 = 0;
    for i in 0..iters {
        let p = &positions[i % positions.len()];
        total = total.wrapping_add(model.evaluate(p) as i64);
    }
    let dt = start.elapsed();
    let nps = (iters as f64) / dt.as_secs_f64();
    println!("iters: {iters}  time: {:.3}s  nps: {:.0}  total: {total}", dt.as_secs_f64(), nps);
}
