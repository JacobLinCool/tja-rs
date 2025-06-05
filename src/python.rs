use crate::synthesize::{synthesize_tja_audio, AudioData};
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

    #[pyo3(signature = (music_data, don_data, ka_data, course, branch=None))]
    fn synthesize_audio(
        &self,
        music_data: &PyAudioData,
        don_data: &PyAudioData,
        ka_data: &PyAudioData,
        course: PyCourse,
        branch: Option<&str>,
    ) -> PyResult<PyAudioData> {
        let parsed_tja = ParsedTJA {
            metadata: crate::types::Metadata::new(self.metadata.clone()),
            charts: self
                .charts
                .iter()
                .map(|c| Chart {
                    player: c.player,
                    course: c.course.as_ref().and_then(|s| s.parse().ok()),
                    level: c.level.map(|l| crate::types::Level(l)),
                    balloons: c.balloons.clone(),
                    headers: c.headers.clone(),
                    segments: c
                        .segments
                        .iter()
                        .map(|s| crate::types::Segment {
                            timestamp: s.timestamp,
                            measure_num: s.measure_num,
                            measure_den: s.measure_den,
                            barline: s.barline,
                            branch: s.branch.clone(),
                            branch_condition: s.branch_condition.clone(),
                            notes: s
                                .notes
                                .iter()
                                .map(|n| crate::types::Note {
                                    note_type: match n.note_type.as_str() {
                                        "Empty" => crate::types::NoteType::Empty,
                                        "Don" => crate::types::NoteType::Don,
                                        "Ka" => crate::types::NoteType::Ka,
                                        "DonBig" => crate::types::NoteType::DonBig,
                                        "KaBig" => crate::types::NoteType::KaBig,
                                        "Roll" => crate::types::NoteType::Roll,
                                        "RollBig" => crate::types::NoteType::RollBig,
                                        "Balloon" => crate::types::NoteType::Balloon,
                                        "EndOf" => crate::types::NoteType::EndOf,
                                        "BalloonAlt" => crate::types::NoteType::BalloonAlt,
                                        _ => crate::types::NoteType::Empty,
                                    },
                                    timestamp: n.timestamp,
                                    scroll: n.scroll,
                                    delay: n.delay,
                                    bpm: n.bpm,
                                    gogo: n.gogo,
                                })
                                .collect(),
                        })
                        .collect(),
                })
                .collect(),
        };

        let result = synthesize_tja_audio(
            &parsed_tja,
            &AudioData::from(music_data.clone()),
            &AudioData::from(don_data.clone()),
            &AudioData::from(ka_data.clone()),
            course.into(),
            branch,
        )
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        Ok(PyAudioData::from(result))
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

#[pyclass(get_all)]
#[derive(Clone, Debug, Serialize)]
pub struct PyAudioData {
    samples: Vec<f32>,
    sample_rate: u32,
}

#[pymethods]
impl PyAudioData {
    #[new]
    fn new(samples: Vec<f32>, sample_rate: u32) -> Self {
        Self {
            samples,
            sample_rate,
        }
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!(
            "AudioData(samples={}, sample_rate={})",
            self.samples.len(),
            self.sample_rate
        ))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }

    fn export(&self, py: Python) -> PyResult<PyObject> {
        let json_value = serde_json::to_value(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        Ok(json_to_py(py, &json_value))
    }

    fn get_samples(&self) -> Vec<f32> {
        self.samples.clone()
    }

    fn get_sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

impl From<AudioData> for PyAudioData {
    fn from(audio: AudioData) -> Self {
        Self {
            samples: audio.samples,
            sample_rate: audio.sample_rate,
        }
    }
}

impl From<PyAudioData> for AudioData {
    fn from(audio: PyAudioData) -> Self {
        AudioData::new(audio.samples, audio.sample_rate)
    }
}

#[pyclass(eq)]
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum PyCourse {
    Easy,
    Normal,
    Hard,
    Oni,
    Ura,
}

#[pymethods]
impl PyCourse {
    fn __str__(&self) -> PyResult<String> {
        serde_json::to_string(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }
    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}

impl From<PyCourse> for Course {
    fn from(course: PyCourse) -> Self {
        match course {
            PyCourse::Easy => Course::Easy,
            PyCourse::Normal => Course::Normal,
            PyCourse::Hard => Course::Hard,
            PyCourse::Oni => Course::Oni,
            PyCourse::Ura => Course::Ura,
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

#[pyfunction]
#[pyo3(signature = (tja, music_data, don_data, ka_data, course, branch = None))]
pub fn synthesize_tja_audio_py(
    tja: &PyParsedTJA,
    music_data: &PyAudioData,
    don_data: &PyAudioData,
    ka_data: &PyAudioData,
    course: PyCourse,
    branch: Option<&str>,
) -> PyResult<PyAudioData> {
    tja.synthesize_audio(music_data, don_data, ka_data, course, branch)
}

#[pymodule]
pub fn tja(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyNote>()?;
    m.add_class::<PySegment>()?;
    m.add_class::<PyChart>()?;
    m.add_class::<PyParsedTJA>()?;
    m.add_class::<PyAudioData>()?;
    m.add_class::<PyCourse>()?;
    m.add_class::<PyParsingMode>()?;
    m.add_function(wrap_pyfunction!(parse_tja, m)?)?;
    m.add_function(wrap_pyfunction!(synthesize_tja_audio_py, m)?)?;
    Ok(())
}
