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
use tja::ParsedTJA;
use tja::{synthesize_tja_audio, AudioData, Course, TJAParser};

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
        &parsed,
        course,
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

// Modify merge_audio_files to handle resampling
fn merge_audio_files(
    music_path: &str,
    don_path: &str,
    ka_path: &str,
    output_path: &str,
    parsed: &ParsedTJA,
    course: Course,
    branch: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load audio files
    let music_data = load_audio_file(music_path)?;
    let don_data = load_audio_file(don_path)?;
    let ka_data = load_audio_file(ka_path)?;

    let output_data =
        synthesize_tja_audio(&parsed, &music_data, &don_data, &ka_data, course, branch)?;

    write_audio_file(output_path, &output_data.samples, output_data.sample_rate)?;

    Ok(())
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
