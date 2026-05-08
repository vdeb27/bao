use pyo3::prelude::*;

#[pymodule]
fn bao_engine_py(_py: Python<'_>, _m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
