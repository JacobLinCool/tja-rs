use std::fs;
use std::path::Path;
use tja::TJAParser;

fn main() {
    let data_dir = Path::new("data");

    // Collect all TJA files and their contents upfront
    let tja_files: Vec<_> = fs::read_dir(data_dir)
        .expect("Failed to read data directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("tja") {
                let content = fs::read_to_string(&path).expect("Failed to read file");
                Some((path, content))
            } else {
                None
            }
        })
        .collect();

    // Parse each file multiple times to get better profiling data
    for _ in 0..1000 {
        // Run 1000 iterations
        for (_path, content) in &tja_files {
            // Parse multiple times per file
            for _ in 0..10 {
                // Parse each file 10 times
                let mut parser = TJAParser::new();
                parser.parse_str(content).expect("Failed to parse");
            }
        }
    }
}
