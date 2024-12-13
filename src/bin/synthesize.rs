use hound;
use std::env;
use std::fs;
use std::path::Path;
use std::process;
use std::str::FromStr;
use symphonia::core::audio::{AudioBuffer, AudioBufferRef, Signal};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use tja::Course;
use tja::NoteType;
use tja::TJAParser;

struct AudioData {
    samples: Vec<f32>,
    sample_rate: u32,
}

#[derive(Debug, Clone)]
enum FilteredNoteType {
    Don,
    Ka,
    DrumRoll { duration: f64 },
    Balloon { duration: f64 },
}

#[derive(Debug, Clone)]
struct FilteredNote {
    note_type: FilteredNoteType,
    timestamp: f64,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 5 {
        eprintln!(
            "Usage: {} <tja_file> <music_file> <don_file> <ka_file> [--course <course>] [--branch <branch>]",
            args[0]
        );
        eprintln!("Courses: Oni, Hard, Normal, Easy");
        eprintln!("Branches: N (Normal), E (Expert), M (Master)");
        process::exit(1);
    }

    let tja_path = &args[1];
    let music_path = &args[2];
    let don_path = &args[3];
    let ka_path = &args[4];

    // Parse optional arguments
    let mut course = Course::Oni; // Default course
    let mut branch = None;

    let mut i = 5;
    while i < args.len() {
        match args[i].as_str() {
            "--course" => {
                if i + 1 < args.len() {
                    course = Course::from_str(&args[i + 1]).unwrap_or(Course::Oni);
                    i += 2;
                } else {
                    eprintln!("Missing course value");
                    process::exit(1);
                }
            }
            "--branch" => {
                if i + 1 < args.len() {
                    branch = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Missing branch value");
                    process::exit(1);
                }
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                process::exit(1);
            }
        }
    }

    // Parse TJA file
    let tja_content = match fs::read_to_string(tja_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading TJA file {}: {}", tja_path, e);
            process::exit(1);
        }
    };

    let mut parser = TJAParser::new();
    if let Err(e) = parser.parse_str(&tja_content) {
        eprintln!("Error parsing TJA file: {}", e);
        process::exit(1);
    }

    let parsed = parser.get_parsed_tja();

    // Find the specified course
    let course_data = parsed
        .charts
        .iter()
        .find(|c| c.course.as_ref() == Some(&course));
    let course_data = match course_data {
        Some(data) => data,
        None => {
            eprintln!("Course {:?} not found in TJA file", course);
            process::exit(1);
        }
    };

    // Generate output filename
    let output_path = format!(
        "{}_{:?}{}{}",
        Path::new(tja_path).file_stem().unwrap().to_string_lossy(),
        course,
        branch
            .as_ref()
            .map(|b| format!("_{}", b))
            .unwrap_or_default(),
        "_merged.wav"
    );

    // Merge audio files based on notes
    if let Err(e) = merge_audio_files(
        music_path,
        don_path,
        ka_path,
        &output_path,
        course_data,
        branch.as_deref(),
    ) {
        eprintln!("Error merging audio files: {}", e);
        process::exit(1);
    }

    println!("Successfully created merged audio file: {}", output_path);
}

// Modify load_audio_file to return sample rate
fn load_audio_file(path: &str) -> Result<AudioData, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let stream = MediaSourceStream::new(Box::new(file), Default::default());

    let mut reader = symphonia::default::get_probe()
        .format(
            &Default::default(),
            stream,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )?
        .format;

    let track = reader.default_track().unwrap();
    let sample_rate = track.codec_params.sample_rate.unwrap();
    let mut decoder =
        symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default())?;

    let mut samples = Vec::new();

    while let Ok(packet) = reader.next_packet() {
        let decoded = decoder.decode(&packet)?;
        match decoded {
            AudioBufferRef::F32(buf) => {
                // Handle mono files
                if buf.spec().channels.count() == 1 {
                    for &sample in buf.chan(0) {
                        samples.push(sample); // Left
                        samples.push(sample); // Right (duplicate mono)
                    }
                } else {
                    // Handle stereo files
                    for frame in 0..buf.frames() {
                        samples.push(buf.chan(0)[frame]); // Left
                        samples.push(buf.chan(1)[frame]); // Right
                    }
                }
            }
            _ => {
                let mut f32_buf =
                    AudioBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
                decoded.convert(&mut f32_buf);

                // Same handling as above for the converted buffer
                if f32_buf.spec().channels.count() == 1 {
                    for &sample in f32_buf.chan(0) {
                        samples.push(sample);
                        samples.push(sample);
                    }
                } else {
                    for frame in 0..f32_buf.frames() {
                        samples.push(f32_buf.chan(0)[frame]);
                        samples.push(f32_buf.chan(1)[frame]);
                    }
                }
            }
        }
    }

    Ok(AudioData {
        samples,
        sample_rate,
    })
}

// Add this function to resample audio
fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return samples.to_vec();
    }

    let ratio = to_rate as f64 / from_rate as f64;
    let new_len = (samples.len() as f64 * ratio) as usize;
    let mut resampled = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let pos = i as f64 / ratio;
        let pos_floor = pos.floor() as usize;
        let pos_ceil = (pos_floor + 1).min(samples.len() - 1);
        let fract = pos - pos_floor as f64;

        // Linear interpolation between samples
        let sample = samples[pos_floor] * (1.0 - fract as f32) + samples[pos_ceil] * fract as f32;
        resampled.push(sample);
    }

    resampled
}

// Modify merge_audio_files to handle resampling
fn merge_audio_files(
    music_path: &str,
    don_path: &str,
    ka_path: &str,
    output_path: &str,
    course_data: &tja::Chart,
    branch: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load audio files
    let music_data = load_audio_file(music_path)?;
    let mut don_data = load_audio_file(don_path)?;
    let mut ka_data = load_audio_file(ka_path)?;

    // Use music sample rate as the base
    let sample_rate = music_data.sample_rate;

    // Resample sound effects if needed
    if don_data.sample_rate != sample_rate {
        println!(
            "Resampling don sound from {}Hz to {}Hz",
            don_data.sample_rate, sample_rate
        );
        don_data.samples = resample(&don_data.samples, don_data.sample_rate, sample_rate);
        don_data.sample_rate = sample_rate;
    }

    if ka_data.sample_rate != sample_rate {
        println!(
            "Resampling ka sound from {}Hz to {}Hz",
            ka_data.sample_rate, sample_rate
        );
        ka_data.samples = resample(&ka_data.samples, ka_data.sample_rate, sample_rate);
        ka_data.sample_rate = sample_rate;
    }

    let output_samples = merge_samples(
        &music_data.samples,
        &don_data.samples,
        &ka_data.samples,
        sample_rate,
        course_data,
        branch,
    );

    // Write output file using the detected sample rate
    write_audio_file(output_path, &output_samples, sample_rate)?;

    Ok(())
}

fn filter_notes(course_data: &tja::Chart, branch: Option<&str>) -> Vec<FilteredNote> {
    let mut filtered_notes = Vec::new();

    for (seg_idx, segment) in course_data.segments.iter().enumerate() {
        // Skip if branch doesn't match
        if let Some(branch_name) = branch {
            if let Some(segment_branch) = &segment.branch {
                if segment_branch != branch_name {
                    continue;
                }
            }
        }

        let mut i = 0;
        while i < segment.notes.len() {
            let note = &segment.notes[i];

            match note.note_type {
                NoteType::Roll | NoteType::RollBig | NoteType::Balloon | NoteType::BalloonAlt => {
                    // Find the corresponding EndOf note
                    let mut end_time: Option<f64> = None;

                    // Search in current segment first
                    for future_note in segment.notes[i + 1..].iter() {
                        if matches!(future_note.note_type, NoteType::EndOf) {
                            end_time = Some(future_note.timestamp);
                            break;
                        }
                    }

                    // If not found in current segment, search in subsequent segments
                    if end_time.is_none() {
                        for future_segment in course_data.segments[seg_idx + 1..].iter() {
                            if let Some(branch_name) = branch {
                                if let Some(segment_branch) = &future_segment.branch {
                                    if segment_branch != branch_name {
                                        continue;
                                    }
                                }
                            }

                            for future_note in future_segment.notes.iter() {
                                if matches!(future_note.note_type, NoteType::EndOf) {
                                    end_time = Some(future_note.timestamp);
                                    break;
                                }
                            }
                            if end_time.is_some() {
                                break;
                            }
                        }
                    }

                    if let Some(end_time) = end_time {
                        let duration = end_time - note.timestamp;
                        let filtered_type = match note.note_type {
                            NoteType::Roll | NoteType::RollBig => {
                                FilteredNoteType::DrumRoll { duration }
                            }
                            NoteType::Balloon | NoteType::BalloonAlt => {
                                FilteredNoteType::Balloon { duration }
                            }
                            _ => unreachable!(),
                        };
                        filtered_notes.push(FilteredNote {
                            note_type: filtered_type,
                            timestamp: note.timestamp,
                        });
                    } else {
                        eprintln!(
                            "Warning: No end marker found for roll/balloon starting at {}s",
                            note.timestamp
                        );
                    }
                }
                NoteType::Don | NoteType::DonBig => {
                    filtered_notes.push(FilteredNote {
                        note_type: FilteredNoteType::Don,
                        timestamp: note.timestamp,
                    });
                }
                NoteType::Ka | NoteType::KaBig => {
                    filtered_notes.push(FilteredNote {
                        note_type: FilteredNoteType::Ka,
                        timestamp: note.timestamp,
                    });
                }
                _ => {}
            }
            i += 1;
        }
    }

    filtered_notes
}

fn merge_samples(
    music_samples: &[f32],
    don_samples: &[f32],
    ka_samples: &[f32],
    sample_rate: u32,
    course_data: &tja::Chart,
    branch: Option<&str>,
) -> Vec<f32> {
    let mut output_samples = music_samples.to_vec();
    let filtered_notes = filter_notes(course_data, branch);

    for note in filtered_notes {
        let sample_pos = (note.timestamp * sample_rate as f64) as usize * 2;

        match note.note_type {
            FilteredNoteType::DrumRoll { duration } | FilteredNoteType::Balloon { duration } => {
                let hits = (duration * 15.0) as usize;
                let interval = duration / hits as f64;

                for hit in 0..hits {
                    let hit_time = note.timestamp + (interval * hit as f64);
                    let hit_pos = (hit_time * sample_rate as f64) as usize * 2;

                    let volume = 1.0;
                    for (j, &sample) in don_samples.iter().enumerate() {
                        if hit_pos + j >= output_samples.len() {
                            break;
                        }
                        output_samples[hit_pos + j] =
                            clamp(output_samples[hit_pos + j] + (sample * volume), -1.0, 1.0);
                    }
                }
            }
            FilteredNoteType::Don => {
                let volume = 1.0;
                for (j, &sample) in don_samples.iter().enumerate() {
                    if sample_pos + j >= output_samples.len() {
                        break;
                    }
                    output_samples[sample_pos + j] = clamp(
                        output_samples[sample_pos + j] + (sample * volume),
                        -1.0,
                        1.0,
                    );
                }
            }
            FilteredNoteType::Ka => {
                let volume = 1.0;
                for (j, &sample) in ka_samples.iter().enumerate() {
                    if sample_pos + j >= output_samples.len() {
                        break;
                    }
                    output_samples[sample_pos + j] = clamp(
                        output_samples[sample_pos + j] + (sample * volume),
                        -1.0,
                        1.0,
                    );
                }
            }
        }
    }

    output_samples
}

fn write_audio_file(
    path: &str,
    samples: &[f32],
    sample_rate: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create(path, spec)?;

    // Write all samples
    for &sample in samples {
        writer.write_sample(sample)?;
    }

    writer.finalize()?;
    Ok(())
}

#[inline]
fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}
