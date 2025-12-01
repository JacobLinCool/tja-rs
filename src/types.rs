use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NoteType {
    Empty,      // "0"
    Don,        // "1"
    Ka,         // "2"
    DonBig,     // "3"
    KaBig,      // "4"
    Roll,       // "5"
    RollBig,    // "6"
    Balloon,    // "7"
    EndOf,      // "8"
    BalloonAlt, // "9"
}

impl NoteType {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '0' => Some(NoteType::Empty),
            '1' => Some(NoteType::Don),
            '2' => Some(NoteType::Ka),
            '3' => Some(NoteType::DonBig),
            '4' => Some(NoteType::KaBig),
            '5' => Some(NoteType::Roll),
            '6' => Some(NoteType::RollBig),
            '7' => Some(NoteType::Balloon),
            '8' => Some(NoteType::EndOf),
            '9' => Some(NoteType::BalloonAlt),
            _ => None,
        }
    }

    /// Faster version that works directly with ASCII bytes (0-9)
    #[inline]
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            b'0' => Some(NoteType::Empty),
            b'1' => Some(NoteType::Don),
            b'2' => Some(NoteType::Ka),
            b'3' => Some(NoteType::DonBig),
            b'4' => Some(NoteType::KaBig),
            b'5' => Some(NoteType::Roll),
            b'6' => Some(NoteType::RollBig),
            b'7' => Some(NoteType::Balloon),
            b'8' => Some(NoteType::EndOf),
            b'9' => Some(NoteType::BalloonAlt),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Course {
    Easy,
    Normal,
    Hard,
    Oni,
    Ura,
}

impl std::str::FromStr for Course {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "EASY" | "0" => Ok(Course::Easy),
            "NORMAL" | "1" => Ok(Course::Normal),
            "HARD" | "2" => Ok(Course::Hard),
            "ONI" | "3" => Ok(Course::Oni),
            "URA" | "EDIT" | "4" => Ok(Course::Ura),
            _ => Err(format!("Invalid course: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Level(pub i32);

impl Level {
    pub fn value(&self) -> i32 {
        self.0
    }
}

impl std::str::FromStr for Level {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse()
            .map(Level)
            .map_err(|_| format!("Invalid level: {}", s))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub bpm: f64,
    pub offset: f64,
    pub demostart: f64,
    pub songvol: i32,
    pub sevol: i32,
    pub raw: HashMap<String, String>,
}

impl Metadata {
    pub fn new(data: HashMap<String, String>) -> Self {
        let bpm = data
            .get("BPM")
            .and_then(|s| s.parse().ok())
            .unwrap_or(120.0);
        let offset = data
            .get("OFFSET")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let demostart = data
            .get("DEMOSTART")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        let songvol = data
            .get("SONGVOL")
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);
        let sevol = data
            .get("SEVOL")
            .and_then(|s| s.parse().ok())
            .unwrap_or(100);

        Self {
            raw: data,
            bpm,
            offset,
            demostart,
            songvol,
            sevol,
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.raw.get(key)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub note_type: NoteType,
    pub timestamp: f64,
    pub bpm: f64,
    pub delay: f64,
    pub scroll: f64,
    #[serde(skip_serializing_if = "is_false")]
    pub gogo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub timestamp: f64,
    pub measure_num: i32,
    pub measure_den: i32,
    pub barline: bool,
    pub branch: Option<String>,
    pub branch_condition: Option<String>,
    pub notes: Vec<Note>,
}

impl Segment {
    pub fn new(
        timestamp: f64,
        measure_num: i32,
        measure_den: i32,
        barline: bool,
        branch: Option<String>,
        branch_condition: Option<String>,
    ) -> Self {
        Self {
            timestamp,
            measure_num,
            measure_den,
            barline,
            branch,
            branch_condition,
            notes: Vec::new(),
        }
    }

    pub fn measure(&self) -> f64 {
        self.measure_num as f64 / self.measure_den as f64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    pub player: i32,
    pub course: Option<Course>,
    pub level: Option<Level>,
    pub balloons: Vec<i32>,
    pub headers: HashMap<String, String>,
    pub segments: Vec<Segment>,
}

impl Chart {
    pub fn new(headers: HashMap<String, String>, player: i32) -> Self {
        let course = headers.get("COURSE").and_then(|s| s.parse().ok());
        let level = headers.get("LEVEL").and_then(|s| s.parse().ok());

        let balloons = headers
            .get("BALLOON")
            .map(|s| {
                s.split(',')
                    .filter_map(|num| num.trim().parse::<i32>().ok())
                    .collect()
            })
            .unwrap_or_default();

        Self {
            player,
            course,
            level,
            balloons,
            headers,
            segments: Vec::new(),
        }
    }

    pub fn course(&self) -> Option<&Course> {
        self.course.as_ref()
    }

    pub fn level(&self) -> Option<i32> {
        self.level.as_ref().map(|l| l.value())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTJA {
    pub metadata: Metadata,
    pub charts: Vec<Chart>,
}
