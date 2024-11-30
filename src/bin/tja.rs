use std::env;
use std::fs;
use std::process;
use tja::TJAParser;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <tja_file>", args[0]);
        process::exit(1);
    }

    let file_path = &args[1];

    match fs::read_to_string(file_path) {
        Ok(content) => {
            let mut parser = TJAParser::new();
            match parser.parse_str(&content) {
                Ok(()) => {
                    let parsed = parser.get_parsed_tja();
                    let json = serde_json::to_string_pretty(&parsed).unwrap();
                    println!("{}", json);
                }
                Err(e) => {
                    eprintln!("Error parsing TJA file: {}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error reading file {}: {}", file_path, e);
            process::exit(1);
        }
    }
}
