//! Python extension module — compiled only when the `python` feature is enabled.
//!
//! Build with maturin:
//! ```bash
//! maturin develop --features python   # dev install into current venv
//! maturin build --release --features python  # release wheel
//! ```

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pythonize::pythonize;

use crate::core::extractor::Extractor;
use crate::core::model::Model;

/// Extract a CV/resume from `path` and return it as a Python dict.
///
/// The returned dict matches the `Resume` TypedDict defined in `cvxtract.pyi`.
///
/// # Errors
/// Raises `RuntimeError` if the file cannot be read, the model fails, or the
/// LLM response cannot be parsed.
#[pyfunction]
pub fn extract_resume<'py>(py: Python<'py>, path: &str) -> PyResult<Bound<'py, PyAny>> {
    let path = path.to_owned();
    let model = Model::from_copilot(None)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let mut extractor = Extractor::new(Some(model));

    // Release the GIL while waiting for the network / LLM response.
    let resume = py.allow_threads(|| {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        rt.block_on(extractor.extract_resume(std::path::PathBuf::from(path)))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    })?;

    pythonize(py, &resume).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// Extract a CV/resume from `path` and return it as a JSON string.
///
/// Equivalent to `json.dumps(extract_resume(path))` but avoids the Python
/// round-trip when you just need to persist or forward the raw JSON.
#[pyfunction]
pub fn extract_resume_json(py: Python<'_>, path: &str) -> PyResult<String> {
    let path = path.to_owned();
    let model = Model::from_copilot(None)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let mut extractor = Extractor::new(Some(model));

    let resume = py.allow_threads(|| {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        rt.block_on(extractor.extract_resume(std::path::PathBuf::from(path)))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    })?;

    serde_json::to_string_pretty(&resume).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

/// The `cvxtract` Python extension module.
#[pymodule]
pub fn cvxtract(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(extract_resume, m)?)?;
    m.add_function(wrap_pyfunction!(extract_resume_json, m)?)?;
    Ok(())
}
