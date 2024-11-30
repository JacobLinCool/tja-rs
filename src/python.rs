use crate::TJAParser;
use pyo3::prelude::*;

#[pyfunction]
pub fn parse_tja(content: &str) -> PyResult<String> {
    let mut parser = TJAParser::new();
    parser
        .parse_str(content)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

    let parsed = parser.get_parsed_tja();
    serde_json::to_string(&parsed)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
}

#[pymodule]
pub fn tja(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_tja, m)?)?;
    Ok(())
}
