use crate::{types::*, ParsingMode, TJAParser};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use serde::Serialize;
use std::collections::HashMap;

fn json_to_py(py: Python, value: &serde_json::Value) -> PyObject {
    match value {
        serde_json::Value::Null => py.None(),
        serde_json::Value::Bool(b) => b.into_py(py),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into_py(py)
            } else if let Some(u) = n.as_u64() {
                u.into_py(py)
            } else if let Some(f) = n.as_f64() {
                f.into_py(py)
            } else {
                py.None()
            }
        }
        serde_json::Value::String(s) => s.into_py(py),
        serde_json::Value::Array(arr) => {
            let list = PyList::empty(py);
            for item in arr {
                list.append(json_to_py(py, item)).unwrap();
            }
            list.into_py(py)
        }
        serde_json::Value::Object(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map {
                dict.set_item(k, json_to_py(py, v)).unwrap();
            }
            dict.into_py(py)
        }
    }
}

#[pyclass(get_all)]
#[derive(Clone, Debug, Serialize)]
struct PyNote {
    note_type: String,
    timestamp: f64,
    scroll: f64,
    delay: f64,
    bpm: f64,
    gogo: bool,
}

#[pymethods]
impl PyNote {
    fn __str__(&self) -> PyResult<String> {
        serde_json::to_string(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }
    fn __repr__(&self) -> PyResult<String> {
        serde_json::to_string(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }
    fn export(&self, py: Python) -> PyResult<PyObject> {
        let json_value = serde_json::to_value(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(json_to_py(py, &json_value))
    }
}

#[pyclass(get_all)]
#[derive(Clone, Debug, Serialize)]
struct PySegment {
    timestamp: f64,
    measure_num: i32,
    measure_den: i32,
    barline: bool,
    branch: Option<String>,
    branch_condition: Option<String>,
    notes: Vec<PyNote>,
}

#[pymethods]
impl PySegment {
    fn __str__(&self) -> PyResult<String> {
        serde_json::to_string(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }
    fn __repr__(&self) -> PyResult<String> {
        serde_json::to_string(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }
    fn export(&self, py: Python) -> PyResult<PyObject> {
        let json_value = serde_json::to_value(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(json_to_py(py, &json_value))
    }
}

#[pyclass(get_all)]
#[derive(Clone, Debug, Serialize)]
struct PyChart {
    player: i32,
    course: Option<String>,
    level: Option<i32>,
    balloons: Vec<i32>,
    headers: HashMap<String, String>,
    segments: Vec<PySegment>,
}

#[pymethods]
impl PyChart {
    fn __str__(&self) -> PyResult<String> {
        serde_json::to_string(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }
    fn __repr__(&self) -> PyResult<String> {
        serde_json::to_string(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }
    fn export(&self, py: Python) -> PyResult<PyObject> {
        let json_value = serde_json::to_value(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(json_to_py(py, &json_value))
    }
}

#[pyclass(get_all)]
#[derive(Serialize)]
pub struct PyParsedTJA {
    metadata: HashMap<String, String>,
    charts: Vec<PyChart>,
}

#[pymethods]
impl PyParsedTJA {
    fn __str__(&self) -> PyResult<String> {
        serde_json::to_string(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }
    fn __repr__(&self) -> PyResult<String> {
        serde_json::to_string(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }
    fn export(&self, py: Python) -> PyResult<PyObject> {
        let json_value = serde_json::to_value(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(json_to_py(py, &json_value))
    }
}

impl From<Note> for PyNote {
    fn from(note: Note) -> Self {
        PyNote {
            note_type: format!("{:?}", note.note_type),
            timestamp: note.timestamp,
            scroll: note.scroll,
            delay: note.delay,
            bpm: note.bpm,
            gogo: note.gogo,
        }
    }
}

impl From<Segment> for PySegment {
    fn from(segment: Segment) -> Self {
        PySegment {
            timestamp: segment.timestamp,
            measure_num: segment.measure_num,
            measure_den: segment.measure_den,
            barline: segment.barline,
            branch: segment.branch,
            branch_condition: segment.branch_condition,
            notes: segment.notes.into_iter().map(PyNote::from).collect(),
        }
    }
}

impl From<Chart> for PyChart {
    fn from(chart: Chart) -> Self {
        PyChart {
            player: chart.player,
            course: chart.course.clone().map(|c| format!("{:?}", c)),
            level: chart.level.map(|l| l.value()),
            balloons: chart.balloons,
            headers: chart.headers,
            segments: chart.segments.into_iter().map(PySegment::from).collect(),
        }
    }
}

impl From<ParsedTJA> for PyParsedTJA {
    fn from(parsed: ParsedTJA) -> Self {
        PyParsedTJA {
            metadata: parsed.metadata.raw,
            charts: parsed.charts.into_iter().map(PyChart::from).collect(),
        }
    }
}

#[pyclass(eq)]
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum PyParsingMode {
    MetadataOnly,
    MetadataAndHeader,
    Full,
}

#[pymethods]
impl PyParsingMode {
    fn __str__(&self) -> PyResult<String> {
        serde_json::to_string(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }
    fn __repr__(&self) -> PyResult<String> {
        serde_json::to_string(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }
}

impl From<PyParsingMode> for ParsingMode {
    fn from(mode: PyParsingMode) -> Self {
        match mode {
            PyParsingMode::MetadataOnly => ParsingMode::MetadataOnly,
            PyParsingMode::MetadataAndHeader => ParsingMode::MetadataAndHeader,
            PyParsingMode::Full => ParsingMode::Full,
        }
    }
}

#[pyfunction]
#[pyo3(signature = (content, mode = PyParsingMode::Full))]
pub fn parse_tja(content: &str, mode: PyParsingMode) -> PyResult<PyParsedTJA> {
    let mut parser = TJAParser::with_mode(mode.into());
    parser
        .parse_str(content)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

    let parsed = parser.get_parsed_tja();
    Ok(PyParsedTJA::from(parsed))
}

#[pymodule]
pub fn tja(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyNote>()?;
    m.add_class::<PySegment>()?;
    m.add_class::<PyChart>()?;
    m.add_class::<PyParsedTJA>()?;
    m.add_class::<PyParsingMode>()?;
    m.add_function(wrap_pyfunction!(parse_tja, m)?)?;
    Ok(())
}
