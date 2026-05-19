//! Minimal PyO3 surface around `bao-engine`. Moves and events travel as JSON
//! strings; board states travel as packed bytes (50 B). This keeps the
//! binding tiny and language-agnostic — the Python training pipeline parses
//! the JSON, the engine remains the source of truth.

use bao_engine::{
    apply as engine_apply, encode_ban, encode_features as engine_encode_features,
    legal_moves as engine_legal_moves, search as engine_search, zobrist_key, BoardState,
    HeuristicEval, Move, SearchOptions, Variant, FEATURE_LEN,
};
use bao_engine::shard::{ShardHeader, HEADER_LEN};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

fn parse_variant(name: &str) -> PyResult<Variant> {
    match name.to_ascii_lowercase().as_str() {
        "kiswahili" => Ok(Variant::Kiswahili),
        "kujifunza" => Ok(Variant::Kujifunza),
        _ => Err(PyValueError::new_err(format!("unknown variant: {name}"))),
    }
}

fn unpack_state(bytes: &[u8]) -> PyResult<BoardState> {
    BoardState::unpack(bytes).map_err(|e| PyValueError::new_err(format!("unpack failed: {e}")))
}

fn parse_move(json: &str) -> PyResult<Move> {
    serde_json::from_str::<Move>(json)
        .map_err(|e| PyValueError::new_err(format!("invalid move json: {e}")))
}

#[pyfunction]
fn new_state(py: Python<'_>, variant: &str) -> PyResult<PyObject> {
    let v = parse_variant(variant)?;
    let state = BoardState::new(v);
    Ok(PyBytes::new_bound(py, &state.pack()).into())
}

#[pyfunction]
fn legal_moves(state_bytes: &[u8]) -> PyResult<String> {
    let state = unpack_state(state_bytes)?;
    let moves = engine_legal_moves(&state);
    serde_json::to_string(&moves)
        .map_err(|e| PyValueError::new_err(format!("serialize moves: {e}")))
}

#[pyfunction]
fn apply(
    py: Python<'_>,
    state_bytes: &[u8],
    move_json: &str,
) -> PyResult<(PyObject, String, String)> {
    let state = unpack_state(state_bytes)?;
    let mv = parse_move(move_json)?;
    let (next, events) =
        engine_apply(&state, mv).map_err(|e| PyValueError::new_err(format!("apply: {e}")))?;
    let events_json = serde_json::to_string(&events)
        .map_err(|e| PyValueError::new_err(format!("serialize events: {e}")))?;
    let ban = encode_ban(&state, mv, &events);
    Ok((PyBytes::new_bound(py, &next.pack()).into(), events_json, ban))
}

#[pyfunction]
fn zobrist(state_bytes: &[u8]) -> PyResult<u64> {
    let state = unpack_state(state_bytes)?;
    Ok(zobrist_key(&state))
}

#[pyfunction]
fn state_to_json(state_bytes: &[u8]) -> PyResult<String> {
    let state = unpack_state(state_bytes)?;
    serde_json::to_string(&state)
        .map_err(|e| PyValueError::new_err(format!("serialize state: {e}")))
}

#[pyfunction]
fn engine_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Encode a packed BoardState into the fixed-size feature vector used by
/// the training pipeline. See `docs/feature_layout.md`. Returned as raw
/// bytes; callers should `np.frombuffer(..., dtype=np.uint8)` for vector ops.
#[pyfunction]
fn encode_features(py: Python<'_>, state_bytes: &[u8]) -> PyResult<PyObject> {
    let state = unpack_state(state_bytes)?;
    let f = engine_encode_features(&state);
    Ok(PyBytes::new_bound(py, &f).into())
}

#[pyfunction]
fn feature_len() -> usize {
    FEATURE_LEN
}

/// Search with the handcrafted heuristic and return `(score, depth, nodes)`.
/// Centi-kete from the active-player perspective at the input state. The
/// best move is intentionally **not** returned — the training pipeline only
/// needs the label.
#[pyfunction]
fn search_heuristic(
    state_bytes: &[u8],
    max_depth: u8,
    max_nodes: u64,
) -> PyResult<(i32, u8, u64)> {
    let state = unpack_state(state_bytes)?;
    let eval = HeuristicEval::new();
    let opts = SearchOptions {
        max_depth,
        max_nodes,
        tt_slots: 1 << 16,
    };
    let r = engine_search(&state, &eval, opts);
    Ok((r.score, r.depth_reached, r.nodes))
}

/// Parse a shard file's 32-byte header. Returns
/// `(version, feature_len, record_stride, n_records, label_dtype)`.
#[pyfunction]
fn read_shard_header(bytes: &[u8]) -> PyResult<(u16, u16, u16, u32, u8)> {
    if bytes.len() < HEADER_LEN {
        return Err(PyValueError::new_err("header truncated"));
    }
    let mut cursor = std::io::Cursor::new(bytes);
    let h = ShardHeader::read_from(&mut cursor)
        .map_err(|e| PyValueError::new_err(format!("shard header: {e}")))?;
    Ok((
        h.version,
        h.feature_len,
        h.record_stride,
        h.n_records,
        h.label_dtype,
    ))
}

#[pymodule]
fn bao_engine_py(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(new_state, m)?)?;
    m.add_function(wrap_pyfunction!(legal_moves, m)?)?;
    m.add_function(wrap_pyfunction!(apply, m)?)?;
    m.add_function(wrap_pyfunction!(zobrist, m)?)?;
    m.add_function(wrap_pyfunction!(state_to_json, m)?)?;
    m.add_function(wrap_pyfunction!(engine_version, m)?)?;
    m.add_function(wrap_pyfunction!(encode_features, m)?)?;
    m.add_function(wrap_pyfunction!(feature_len, m)?)?;
    m.add_function(wrap_pyfunction!(search_heuristic, m)?)?;
    m.add_function(wrap_pyfunction!(read_shard_header, m)?)?;
    Ok(())
}
