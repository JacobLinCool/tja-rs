use crate::{ParsingMode, TJAParser};
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
pub fn parse_tja(content: &str, mode: Option<WasmParsingMode>) -> Result<JsValue, JsValue> {
    let mut parser = TJAParser::with_mode(mode.unwrap_or(WasmParsingMode::Full).into());
    parser
        .parse_str(content)
        .map_err(|e| JsValue::from_str(&e))?;

    let parsed = parser.get_parsed_tja();
    serde_wasm_bindgen::to_value(&parsed).map_err(|e| JsValue::from_str(&e.to_string()))
}
