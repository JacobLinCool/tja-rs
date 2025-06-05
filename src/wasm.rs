use crate::synthesize::{synthesize_tja_audio, AudioData};
use crate::{Course, ParsingMode, TJAParser};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub enum WasmParsingMode {
    MetadataOnly,
    MetadataAndHeader,
    Full,
}

impl From<WasmParsingMode> for ParsingMode {
    fn from(mode: WasmParsingMode) -> Self {
        match mode {
            WasmParsingMode::MetadataOnly => ParsingMode::MetadataOnly,
            WasmParsingMode::MetadataAndHeader => ParsingMode::MetadataAndHeader,
            WasmParsingMode::Full => ParsingMode::Full,
        }
    }
}

#[wasm_bindgen]
pub enum WasmCourse {
    Easy,
    Normal,
    Hard,
    Oni,
    Ura,
}

impl From<WasmCourse> for Course {
    fn from(course: WasmCourse) -> Self {
        match course {
            WasmCourse::Easy => Course::Easy,
            WasmCourse::Normal => Course::Normal,
            WasmCourse::Hard => Course::Hard,
            WasmCourse::Oni => Course::Oni,
            WasmCourse::Ura => Course::Ura,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct WasmAudioData {
    samples: Vec<f32>,
    sample_rate: u32,
}

impl From<AudioData> for WasmAudioData {
    fn from(audio: AudioData) -> Self {
        Self {
            samples: audio.samples,
            sample_rate: audio.sample_rate,
        }
    }
}

impl From<WasmAudioData> for AudioData {
    fn from(audio: WasmAudioData) -> Self {
        AudioData::new(audio.samples, audio.sample_rate)
    }
}

#[wasm_bindgen]
pub fn parse_tja(content: &str, mode: Option<WasmParsingMode>) -> Result<JsValue, JsValue> {
    let mut parser = TJAParser::with_mode(mode.unwrap_or(WasmParsingMode::Full).into());
    parser
        .parse_str(content)
        .map_err(|e| JsValue::from_str(&e))?;

    let parsed = parser.get_parsed_tja();
    serde_wasm_bindgen::to_value(&parsed).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn synthesize_tja_audio_wasm(
    tja_js: &JsValue,
    music_data_js: &JsValue,
    don_data_js: &JsValue,
    ka_data_js: &JsValue,
    course: WasmCourse,
    branch: Option<String>,
) -> Result<JsValue, JsValue> {
    let tja: crate::ParsedTJA = serde_wasm_bindgen::from_value(tja_js.clone())
        .map_err(|e| JsValue::from_str(&format!("Failed to parse TJA: {}", e)))?;

    let music_data: WasmAudioData = serde_wasm_bindgen::from_value(music_data_js.clone())
        .map_err(|e| JsValue::from_str(&format!("Failed to parse music data: {}", e)))?;

    let don_data: WasmAudioData = serde_wasm_bindgen::from_value(don_data_js.clone())
        .map_err(|e| JsValue::from_str(&format!("Failed to parse don data: {}", e)))?;

    let ka_data: WasmAudioData = serde_wasm_bindgen::from_value(ka_data_js.clone())
        .map_err(|e| JsValue::from_str(&format!("Failed to parse ka data: {}", e)))?;

    let result = synthesize_tja_audio(
        &tja,
        &AudioData::from(music_data),
        &AudioData::from(don_data),
        &AudioData::from(ka_data),
        course.into(),
        branch.as_deref(),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let wasm_result = WasmAudioData::from(result);
    serde_wasm_bindgen::to_value(&wasm_result).map_err(|e| JsValue::from_str(&e.to_string()))
}
