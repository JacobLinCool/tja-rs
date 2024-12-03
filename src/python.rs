use crate::{types::*, ParsingMode, TJAParser};
use pyo3::prelude::*;
use std::collections::HashMap;

#[pyclass]
#[derive(Clone, Debug)]
struct PyNote {
    #[pyo3(get)]
    note_type: String,
    #[pyo3(get)]
    timestamp: f64,
    #[pyo3(get)]
    scroll: f64,
    #[pyo3(get)]
    delay: f64,
    #[pyo3(get)]
    bpm: f64,
    #[pyo3(get)]
    gogo: bool,
}

#[pyclass]
#[derive(Clone, Debug)]
struct PySegment {
    #[pyo3(get)]
    measure_num: i32,
    #[pyo3(get)]
    measure_den: i32,
    #[pyo3(get)]
    barline: bool,
    #[pyo3(get)]
    branch: Option<String>,
    #[pyo3(get)]
    branch_condition: Option<String>,
    #[pyo3(get)]
    notes: Vec<PyNote>,
}

#[pyclass]
#[derive(Clone, Debug)]
struct PyChart {
    #[pyo3(get)]
    player: i32,
    #[pyo3(get)]
    course: Option<String>,
    #[pyo3(get)]
    level: Option<i32>,
    #[pyo3(get)]
    balloons: Vec<i32>,
    #[pyo3(get)]
    headers: HashMap<String, String>,
    #[pyo3(get)]
    segments: Vec<PySegment>,
}

#[pyclass]
pub struct PyParsedTJA {
    #[pyo3(get)]
    metadata: HashMap<String, String>,
    #[pyo3(get)]
    charts: Vec<PyChart>,
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
            level: chart.level(),
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
#[derive(Clone, Debug, PartialEq)]
pub enum PyParsingMode {
    MetadataOnly,
    MetadataAndHeader,
    Full,
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
