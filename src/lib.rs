mod directives;
mod parser;
mod types;
mod synthesize;

pub use directives::*;
pub use parser::*;
pub use types::*;
pub use synthesize::*;

#[cfg(feature = "python")]
mod python;
#[cfg(feature = "python")]
pub use python::*;

#[cfg(feature = "wasm")]
mod wasm;
#[cfg(feature = "wasm")]
pub use wasm::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_supernova() {
        let content = fs::read_to_string("data/SUPERNOVA.tja").unwrap();
        let mut parser = TJAParser::new();
        parser.parse_str(&content).unwrap();
        let parsed_tja = parser.get_parsed_tja();

        insta::with_settings!({sort_maps => true}, {
            insta::assert_json_snapshot!("supernova", parsed_tja);
        });
    }

    #[test]
    fn test_parse_nijiiro_baton() {
        let content = fs::read_to_string("data/ニジイロバトン.tja").unwrap();
        let mut parser = TJAParser::new();
        parser.parse_str(&content).unwrap();
        let parsed_tja = parser.get_parsed_tja();
        insta::with_settings!({sort_maps => true}, {
            insta::assert_json_snapshot!("nijiiro_baton", parsed_tja);
        });
    }

    #[test]
    fn test_parse_mint_tears() {
        let content = fs::read_to_string("data/mint tears.tja").unwrap();
        let mut parser = TJAParser::new();
        parser.parse_str(&content).unwrap();
        let parsed_tja = parser.get_parsed_tja();
        insta::with_settings!({sort_maps => true}, {
            insta::assert_json_snapshot!("mint_tears", parsed_tja);
        });
    }

    #[test]
    fn test_parse_supernova_metadata_only() {
        let content = fs::read_to_string("data/SUPERNOVA.tja").unwrap();
        let mut parser = TJAParser::with_mode(ParsingMode::MetadataOnly);
        parser.parse_str(&content).unwrap();
        let parsed_tja = parser.get_parsed_tja();

        insta::with_settings!({sort_maps => true}, {
            insta::assert_json_snapshot!("supernova_metadata_only", parsed_tja);
        });
    }

    #[test]
    fn test_parse_supernova_metadata_and_header() {
        let content = fs::read_to_string("data/SUPERNOVA.tja").unwrap();
        let mut parser = TJAParser::with_mode(ParsingMode::MetadataAndHeader);
        parser.parse_str(&content).unwrap();
        let parsed_tja = parser.get_parsed_tja();

        insta::with_settings!({sort_maps => true}, {
            insta::assert_json_snapshot!("supernova_metadata_and_header", parsed_tja);
        });
    }
}
