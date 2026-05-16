//! Minimal PyO3 surface around `bao-engine`. Moves and events travel as JSON
//! strings; board states travel as packed bytes (50 B). This keeps the
//! binding tiny and language-agnostic — the Python training pipeline parses
//! the JSON, the engine remains the source of truth.

use bao_engine::{
    apply as engine_apply, encode_ban, legal_moves as engine_legal_moves, zobrist_key, BoardState,
    Move, Variant,
};
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

#[pymodule]
fn bao_engine_py(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(new_state, m)?)?;
    m.add_function(wrap_pyfunction!(legal_moves, m)?)?;
    m.add_function(wrap_pyfunction!(apply, m)?)?;
    m.add_function(wrap_pyfunction!(zobrist, m)?)?;
    m.add_function(wrap_pyfunction!(state_to_json, m)?)?;
    m.add_function(wrap_pyfunction!(engine_version, m)?)?;
    Ok(())
}
