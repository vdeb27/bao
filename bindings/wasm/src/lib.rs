//! Minimal wasm-bindgen surface around `bao-engine`. Mirrors the PyO3 binding:
//! board state travels as packed bytes (`Uint8Array`, 50 B), moves and events
//! as JSON strings parsed/serialised browser-side.

use bao_engine::{
    apply as engine_apply, encode_ban, legal_moves as engine_legal_moves, search as engine_search,
    zobrist_key, BoardState, HeuristicEval, Move, SearchOptions, Variant,
};
use std::sync::Once;

fn install_panic_hook() {
    static HOOK: Once = Once::new();
    HOOK.call_once(|| {
        console_error_panic_hook::set_once();
    });
}
use wasm_bindgen::prelude::*;

fn parse_variant(name: &str) -> Result<Variant, JsValue> {
    match name.to_ascii_lowercase().as_str() {
        "kiswahili" => Ok(Variant::Kiswahili),
        "kujifunza" => Ok(Variant::Kujifunza),
        _ => Err(JsValue::from_str(&format!("unknown variant: {name}"))),
    }
}

fn unpack_state(bytes: &[u8]) -> Result<BoardState, JsValue> {
    BoardState::unpack(bytes).map_err(|e| JsValue::from_str(&format!("unpack failed: {e}")))
}

fn parse_move(json: &str) -> Result<Move, JsValue> {
    serde_json::from_str::<Move>(json)
        .map_err(|e| JsValue::from_str(&format!("invalid move json: {e}")))
}

#[wasm_bindgen]
pub fn engine_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen]
pub fn new_state(variant: &str) -> Result<Vec<u8>, JsValue> {
    let v = parse_variant(variant)?;
    Ok(BoardState::new(v).pack())
}

#[wasm_bindgen]
pub fn legal_moves(state_bytes: &[u8]) -> Result<String, JsValue> {
    let state = unpack_state(state_bytes)?;
    let moves = engine_legal_moves(&state);
    serde_json::to_string(&moves).map_err(|e| JsValue::from_str(&format!("serialize moves: {e}")))
}

/// Applies `move_json` to the packed state. On success the returned object
/// has `state` (Uint8Array) and `events` (JSON string).
#[wasm_bindgen]
pub fn apply(state_bytes: &[u8], move_json: &str) -> Result<JsValue, JsValue> {
    let state = unpack_state(state_bytes)?;
    let mv = parse_move(move_json)?;
    let (next, events) =
        engine_apply(&state, mv).map_err(|e| JsValue::from_str(&format!("apply: {e}")))?;
    let events_json = serde_json::to_string(&events)
        .map_err(|e| JsValue::from_str(&format!("serialize events: {e}")))?;
    let ban = encode_ban(&state, mv, &events);
    let out = ApplyResult {
        state: next.pack(),
        events: events_json,
        ban,
    };
    serde_wasm_bindgen::to_value(&out)
        .map_err(|e| JsValue::from_str(&format!("to_value: {e}")))
}

#[wasm_bindgen]
pub fn zobrist(state_bytes: &[u8]) -> Result<u64, JsValue> {
    let state = unpack_state(state_bytes)?;
    Ok(zobrist_key(&state))
}

/// Returns the unpacked `BoardState` as a JSON string. Used by the UI to
/// render pits, ghala, phase, and winner without re-implementing the
/// pack format on the JS side.
#[wasm_bindgen]
pub fn state_to_json(state_bytes: &[u8]) -> Result<String, JsValue> {
    let state = unpack_state(state_bytes)?;
    serde_json::to_string(&state).map_err(|e| JsValue::from_str(&format!("serialize state: {e}")))
}

/// Search for the best move with the handcrafted heuristic evaluator.
/// Returns a JSON object: `{ move: <move_json|null>, score: <i32>, depth,
/// nodes, elapsed_ms }`. Hard-bound by either `max_depth` or
/// `time_budget_ms`, whichever runs out first.
#[wasm_bindgen]
pub fn search_heuristic(
    state_bytes: &[u8],
    max_depth: u8,
    max_nodes: u32,
) -> Result<String, JsValue> {
    install_panic_hook();
    let state = unpack_state(state_bytes)?;
    let eval = HeuristicEval::new();
    let opts = SearchOptions {
        max_depth,
        max_nodes: max_nodes as u64,
    };
    let r = engine_search(&state, &eval, opts);
    let payload = SearchPayload {
        best_move: r.best_move.map(|m| serde_json::to_value(&m).unwrap()),
        score: r.score,
        depth: r.depth_reached,
        nodes: r.nodes,
    };
    serde_json::to_string(&payload)
        .map_err(|e| JsValue::from_str(&format!("serialize search result: {e}")))
}

#[derive(serde::Serialize)]
struct SearchPayload {
    best_move: Option<serde_json::Value>,
    score: i32,
    depth: u8,
    nodes: u64,
}

#[derive(serde::Serialize)]
struct ApplyResult {
    state: Vec<u8>,
    events: String,
    ban: String,
}
