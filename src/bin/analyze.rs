use std::collections::HashMap;
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
                    analyze_tja(&parsed);
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

fn analyze_tja(parsed: &tja::ParsedTJA) {
    println!("Analyzing TJA file: {}", parsed.metadata.raw["TITLE"]);

    let oni_course = parsed
        .charts
        .iter()
        .find(|c| c.course.as_ref() == Some(&tja::Course::Oni));
    if oni_course.is_none() {
        eprintln!("No Oni course found in the TJA file.");
        return;
    }

    let oni_course = oni_course.unwrap();
    let notes = tja::filter_notes(oni_course, Some(&"M"));

    let mut gaps = HashMap::<u64, u64>::new();

    let mut prev = 0.0;
    for note in &notes {
        let gap = note.timestamp - prev;
        if gap > 0.0 {
            let gap_us = (gap * 1_000_000.0) as u64;
            *gaps.entry(gap_us).or_insert(0) += 1;
        }
        prev = note.timestamp;
    }

    let smallest_gap = gaps.keys().min().cloned().unwrap_or(0);
    let base_unit = smallest_gap / 48;
    println!("Base unit (in microseconds): {}", base_unit);

    let tolerance_us = 1000;
    let mut unit = base_unit;
    while unit < smallest_gap {
        let mut should_stop = false;

        for (gap, _) in &gaps {
            println!("Checking gap: {} (unit: {})", gap, unit);
            let gap_unit = (gap + unit / 2) / unit;
            let reconstructed_gap = gap_unit * unit;
            println!("Reconstructed gap: {}", reconstructed_gap);
            if reconstructed_gap < base_unit {
                println!("Gap {} is smaller than base unit {}, skipping.", reconstructed_gap, base_unit);
                continue;
            }
            if reconstructed_gap < 1000 {
                println!("Gap {} is less than 1 ms, skipping.", reconstructed_gap);
                continue;
            }
            if reconstructed_gap > 1_000_000 {
                println!("Gap {} is greater than 1 s, skipping.", reconstructed_gap);
                continue;
            }
            if gap_unit * unit < *gap - tolerance_us || gap_unit * unit > *gap + tolerance_us {
                should_stop = true;
                break;
            }
        }

        if should_stop {
            break;
        }
        unit *= 2;
    }
    println!("Final unit (in microseconds): {}", unit);

    println!("Total notes: {}", notes.len());
    println!("Gaps (in milliseconds):");
    let mut sorted_gaps: Vec<_> = gaps.iter().collect();
    sorted_gaps.sort_by_key(|&(gap, _)| *gap);
    for (gap, count) in sorted_gaps {
        let gap_ms = (gap + 500) / 1000;
        let gap_base_unit = (gap + base_unit / 2) / base_unit;
        let gap_unit = (gap + unit / 2) / unit;
        println!("{} ms ({} base units, {} units): {}", gap_ms, gap_base_unit, gap_unit, count);
    }
}
