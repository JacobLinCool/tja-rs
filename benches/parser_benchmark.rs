use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::path::Path;
use tja::TJAParser;

fn parse_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("TJA Parser");
    let data_dir = Path::new("data");

    // Collect all TJA files and their contents upfront
    let tja_files: Vec<_> = fs::read_dir(data_dir)
        .expect("Failed to read data directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("tja") {
                let filename = path.file_name().unwrap().to_string_lossy().into_owned();
                let content = fs::read_to_string(&path).expect("Failed to read file");
                Some((filename, content))
            } else {
                None
            }
        })
        .collect();

    // Benchmark each file
    for (filename, content) in tja_files {
        group.bench_function(format!("parse {}", filename), |b| {
            b.iter(|| {
                let mut parser = TJAParser::new();
                parser.parse_str(black_box(&content)).unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(benches, parse_benchmark);
criterion_main!(benches);
