use crate::TJAParser;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse_tja(content: &str) -> Result<String, JsValue> {
    let mut parser = TJAParser::new();
    parser
        .parse_str(content)
        .map_err(|e| JsValue::from_str(&e))?;

    let parsed = parser.get_parsed_tja();
    serde_json::to_string(&parsed).map_err(|e| JsValue::from_str(&e.to_string()))
}
