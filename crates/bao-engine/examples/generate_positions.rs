//! Fase-4 position-generation pipeline (plan §5.1).
//!
//! Generates training shards by running heuristic alpha-beta self-play with
//! random openings, sampling intermediate positions, and labeling them with a
//! deeper alpha-beta search. Output is the binary shard format described in
//! `docs/shard_format.md`.
//!
//! Usage:
//! ```text
//! cargo run --release --example generate_positions -- \
//!     --out training/data/shard-0001.bin \
//!     --n 100000 \
//!     --threads 8 \
//!     --label-depth 8 \
//!     --label-nodes 25000 \
//!     --play-depth 5 \
//!     --play-nodes 5000 \
//!     --seed 1
//! ```
//!
//! Mix (matches plan §5.1):
//! - ~70% self-play positions sampled every 3rd ply
//! - ~20% random self-play positions (uniform-random move selection)
//! - ~10% reserved for hand-curated puzzles (added separately)

use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::Mutex;

use bao_engine::{
    apply, encode_features, legal_moves, search, BoardState, HeuristicEval, Move, SearchOptions,
    ShardWriter, Variant, FEATURE_LEN,
};
use bao_engine::features::clip_label;

struct Args {
    out: PathBuf,
    n: u32,
    threads: usize,
    label_depth: u8,
    label_nodes: u64,
    play_depth: u8,
    play_nodes: u64,
    seed: u64,
}

fn parse_args() -> Args {
    let mut out = PathBuf::from("shard-0001.bin");
    let mut n: u32 = 10_000;
    let mut threads: usize = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let mut label_depth: u8 = 8;
    let mut label_nodes: u64 = 25_000;
    let mut play_depth: u8 = 5;
    let mut play_nodes: u64 = 5_000;
    let mut seed: u64 = 0xC0FFEE_BABE_DEAD;

    let mut args = std::env::args().skip(1);
    while let Some(k) = args.next() {
        match k.as_str() {
            "--out" => out = PathBuf::from(args.next().expect("--out needs value")),
            "--n" => n = args.next().unwrap().parse().unwrap(),
            "--threads" => threads = args.next().unwrap().parse().unwrap(),
            "--label-depth" => label_depth = args.next().unwrap().parse().unwrap(),
            "--label-nodes" => label_nodes = args.next().unwrap().parse().unwrap(),
            "--play-depth" => play_depth = args.next().unwrap().parse().unwrap(),
            "--play-nodes" => play_nodes = args.next().unwrap().parse().unwrap(),
            "--seed" => seed = args.next().unwrap().parse().unwrap(),
            other => panic!("unknown flag: {}", other),
        }
    }
    Args { out, n, threads, label_depth, label_nodes, play_depth, play_nodes, seed }
}

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
    fn range(&mut self, max: u32) -> u32 {
        (self.next_u64() as u32) % max
    }
}

/// Generate positions from one self-play game. Mix:
/// - games whose first 4-8 plies are uniform-random, then heuristic ply
/// - games whose every move is uniform-random (for distribution coverage)
fn play_one_game(
    eval: &HeuristicEval,
    rng: &mut Rng,
    play_opts: SearchOptions,
    random_only: bool,
    out: &mut Vec<BoardState>,
    sample_every: u32,
) {
    let mut state = BoardState::new(Variant::Kiswahili);
    let random_opening_plies = 4 + rng.range(5); // 4..=8
    let mut ply: u32 = 0;
    let max_plies = 220;
    while state.winner.is_none() && ply < max_plies {
        if ply >= 4 && ply % sample_every == 0 {
            out.push(state.clone());
        }
        let moves = legal_moves(&state);
        if moves.is_empty() { break; }
        let mv: Move = if random_only || ply < random_opening_plies {
            rng.pick(&moves)
        } else {
            match search(&state, eval, play_opts).best_move {
                Some(m) => m,
                None => rng.pick(&moves),
            }
        };
        match apply(&state, mv) {
            Ok((next, _)) => state = next,
            Err(_) => break,
        }
        ply += 1;
    }
}

fn worker(
    target_records: u32,
    written: &Mutex<u32>,
    out_writer: &Mutex<ShardWriter<BufWriter<std::fs::File>>>,
    seed: u64,
    args: &Args,
) {
    let eval = HeuristicEval::new();
    let mut rng = Rng::new(seed);
    let play_opts = SearchOptions {
        max_depth: args.play_depth,
        max_nodes: args.play_nodes,
        tt_slots: 1 << 14,
    };
    let label_opts = SearchOptions {
        max_depth: args.label_depth,
        max_nodes: args.label_nodes,
        tt_slots: 1 << 16,
    };
    let mut batch: Vec<BoardState> = Vec::with_capacity(64);

    loop {
        // Two flavours: 70% heuristic-self-play, 30% random-only.
        let random_only = rng.range(10) < 3;
        let sample_every = if random_only { 5 } else { 3 };
        batch.clear();
        play_one_game(&eval, &mut rng, play_opts, random_only, &mut batch, sample_every);

        for pos in batch.drain(..) {
            // Skip terminal / no-move positions (encode would still work but
            // labels are degenerate).
            if pos.winner.is_some() { continue; }
            if legal_moves(&pos).is_empty() { continue; }

            let r = search(&pos, &eval, label_opts);
            let label = clip_label(r.score);

            let mut feats = [0u8; FEATURE_LEN];
            feats.copy_from_slice(&encode_features(&pos));

            let mut w_guard = out_writer.lock().unwrap();
            let mut n_guard = written.lock().unwrap();
            if *n_guard >= target_records {
                return;
            }
            // Best-effort: ignore write errors (disk-full → caller sees short file).
            let _ = w_guard.write_record(&feats, label);
            *n_guard += 1;
            let done = *n_guard;
            drop(n_guard);
            drop(w_guard);
            if done % 5_000 == 0 {
                eprintln!("  progress: {} / {}", done, target_records);
            }
            if done >= target_records {
                return;
            }
        }
    }
}

fn main() {
    let args = parse_args();
    eprintln!("generate_positions:");
    eprintln!("  out         = {}", args.out.display());
    eprintln!("  n           = {}", args.n);
    eprintln!("  threads     = {}", args.threads);
    eprintln!("  label_depth = {}", args.label_depth);
    eprintln!("  label_nodes = {}", args.label_nodes);
    eprintln!("  play_depth  = {}", args.play_depth);
    eprintln!("  play_nodes  = {}", args.play_nodes);
    eprintln!("  seed        = {:#x}", args.seed);

    if let Some(parent) = args.out.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let file = std::fs::File::create(&args.out).expect("create output");
    let buf = BufWriter::new(file);
    let writer = ShardWriter::new(buf, args.n).expect("write header");
    let out_writer = Mutex::new(writer);
    let written: Mutex<u32> = Mutex::new(0);

    let t0 = std::time::Instant::now();
    std::thread::scope(|scope| {
        for t in 0..args.threads {
            let out_writer_ref = &out_writer;
            let written_ref = &written;
            let args_ref = &args;
            let seed = args.seed ^ ((t as u64).wrapping_mul(0x9E3779B97F4A7C15));
            scope.spawn(move || {
                worker(args_ref.n, written_ref, out_writer_ref, seed, args_ref);
            });
        }
    });
    let elapsed = t0.elapsed();

    let writer = out_writer.into_inner().unwrap();
    let total = writer.finish().expect("finish writer");
    eprintln!("done: {} records in {:.2}s ({:.1} pos/sec aggregate, {:.1} pos/sec/core)",
        total,
        elapsed.as_secs_f64(),
        total as f64 / elapsed.as_secs_f64(),
        total as f64 / elapsed.as_secs_f64() / args.threads as f64,
    );
}
