use crate::{Chart, Course, NoteType, ParsedTJA};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

impl AudioData {
    pub fn new(samples: Vec<f32>, sample_rate: u32) -> Self {
        Self {
            samples,
            sample_rate,
        }
    }
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

pub fn synthesize_tja_audio(
    tja: &ParsedTJA,
    music_data: &AudioData,
    don_data: &AudioData,
    ka_data: &AudioData,
    course: Course,
    branch: Option<&str>,
) -> Result<AudioData, Box<dyn std::error::Error>> {
    let course_data = tja
        .charts
        .iter()
        .find(|c| c.course.as_ref() == Some(&course));
    let course_data = match course_data {
        Some(data) => data,
        None => {
            return Err(Box::from(format!(
                "Course {:?} not found in TJA file",
                course
            )));
        }
    };
    let sample_rate = music_data.sample_rate;

    let resampled_don = if don_data.sample_rate != sample_rate {
        resample(&don_data.samples, don_data.sample_rate, sample_rate)
    } else {
        don_data.samples.clone()
    };

    let resampled_ka = if ka_data.sample_rate != sample_rate {
        resample(&ka_data.samples, ka_data.sample_rate, sample_rate)
    } else {
        ka_data.samples.clone()
    };

    let output_samples = merge_samples(
        &music_data.samples,
        &resampled_don,
        &resampled_ka,
        sample_rate,
        course_data,
        branch,
    );

    Ok(AudioData {
        samples: output_samples,
        sample_rate,
    })
}

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

fn filter_notes(course_data: &Chart, branch: Option<&str>) -> Vec<FilteredNote> {
    let mut filtered_notes = Vec::new();

    // Pre-collect all EndOf timestamps in order for efficient lookup
    // This avoids O(n*m) nested loop searching for EndOf notes
    let mut end_of_timestamps: Vec<f64> = Vec::new();
    for segment in &course_data.segments {
        // Skip if branch doesn't match
        if let Some(branch_name) = branch {
            if let Some(segment_branch) = &segment.branch {
                if segment_branch != branch_name {
                    continue;
                }
            }
        }
        for note in &segment.notes {
            if matches!(note.note_type, NoteType::EndOf) {
                end_of_timestamps.push(note.timestamp);
            }
        }
    }
    let mut end_of_iter = end_of_timestamps.iter().peekable();

    for segment in &course_data.segments {
        // Skip if branch doesn't match
        if let Some(branch_name) = branch {
            if let Some(segment_branch) = &segment.branch {
                if segment_branch != branch_name {
                    continue;
                }
            }
        }

        for note in &segment.notes {
            match note.note_type {
                NoteType::Roll | NoteType::RollBig | NoteType::Balloon | NoteType::BalloonAlt => {
                    // Find the next EndOf note timestamp that comes after this note
                    // Advance the iterator past any EndOf timestamps that are before or at the current note
                    while let Some(&&end_time) = end_of_iter.peek() {
                        if end_time > note.timestamp {
                            break;
                        }
                        end_of_iter.next();
                    }

                    if let Some(&&end_time) = end_of_iter.peek() {
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
                        // Consume this EndOf timestamp
                        end_of_iter.next();
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
        }
    }

    filtered_notes
}

fn merge_samples(
    music_samples: &[f32],
    don_samples: &[f32],
    ka_samples: &[f32],
    sample_rate: u32,
    course_data: &Chart,
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
                            (output_samples[hit_pos + j] + (sample * volume)).clamp(-1.0, 1.0);
                    }
                }
            }
            FilteredNoteType::Don => {
                let volume = 1.0;
                for (j, &sample) in don_samples.iter().enumerate() {
                    if sample_pos + j >= output_samples.len() {
                        break;
                    }
                    output_samples[sample_pos + j] =
                        (output_samples[sample_pos + j] + (sample * volume)).clamp(-1.0, 1.0);
                }
            }
            FilteredNoteType::Ka => {
                let volume = 1.0;
                for (j, &sample) in ka_samples.iter().enumerate() {
                    if sample_pos + j >= output_samples.len() {
                        break;
                    }
                    output_samples[sample_pos + j] =
                        (output_samples[sample_pos + j] + (sample * volume)).clamp(-1.0, 1.0);
                }
            }
        }
    }

    output_samples
}

impl ParsedTJA {
    pub fn synthesize_audio(
        &self,
        music_data: &AudioData,
        don_data: &AudioData,
        ka_data: &AudioData,
        course: Course,
        branch: Option<&str>,
    ) -> Result<AudioData, Box<dyn std::error::Error>> {
        synthesize_tja_audio(self, music_data, don_data, ka_data, course, branch)
    }
}
