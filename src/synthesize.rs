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
