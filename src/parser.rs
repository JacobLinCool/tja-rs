use crate::directives::{Directive, DirectiveHandler};
use crate::types::*;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum ParsingState {
    Metadata,
    Header,
    Notes,
}

#[derive(Debug, Clone)]
pub struct ParserState {
    pub bpm: f64,
    pub scroll: f64,
    pub gogo: bool,
    pub barline: bool,
    pub measure_num: i32,
    pub measure_den: i32,
    pub branch_condition: Option<String>,
    pub current_branch: Option<String>,
    pub parsing_chart: bool,
    pub delay: f64,
    pub timestamp: f64,
    pub timestamp_branch_start: f64,
    pub current_segment: Option<Segment>,
    pub parsing_state: ParsingState,
}

impl ParserState {
    pub fn new(bpm: f64) -> Self {
        Self {
            bpm,
            scroll: 1.0,
            gogo: false,
            barline: true,
            measure_num: 4,
            measure_den: 4,
            branch_condition: None,
            current_branch: None,
            parsing_chart: false,
            delay: 0.0,
            timestamp: 0.0,
            timestamp_branch_start: 0.0,
            current_segment: None,
            parsing_state: ParsingState::Metadata,
        }
    }

    pub fn measure(&self) -> f64 {
        self.measure_num as f64 / self.measure_den as f64
    }
}

#[derive(Debug, Clone)]
pub struct TJAParser {
    metadata: Option<Metadata>,
    charts: Vec<Chart>,
    state: Option<ParserState>,
    inherited_headers: HashMap<String, String>,
    current_headers: HashMap<String, String>,
    metadata_keys: HashSet<String>,
    header_keys: HashSet<String>,
    inheritable_header_keys: HashSet<String>,
}

impl Default for TJAParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TJAParser {
    pub fn new() -> Self {
        let metadata_keys: HashSet<String> = vec![
            "TITLE",
            "SUBTITLE",
            "WAVE",
            "BPM",
            "OFFSET",
            "DEMOSTART",
            "GENRE",
            "MAKER",
            "SONGVOL",
            "SEVOL",
            "SCOREMODE",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let header_keys: HashSet<String> = vec![
            "COURSE",
            "LEVEL",
            "BALLOON",
            "SCOREINIT",
            "SCOREDIFF",
            "STYLE",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let inheritable_header_keys: HashSet<String> =
            vec!["COURSE", "LEVEL", "SCOREINIT", "SCOREDIFF"]
                .into_iter()
                .map(String::from)
                .collect();

        Self {
            metadata: None,
            charts: Vec::new(),
            state: None,
            inherited_headers: HashMap::new(),
            current_headers: HashMap::new(),
            metadata_keys,
            header_keys,
            inheritable_header_keys,
        }
    }

    pub fn parse_str(&mut self, content: &str) -> Result<(), String> {
        let mut metadata_dict = HashMap::with_capacity(self.metadata_keys.len());
        let mut notes_buffer = Vec::new();

        // Initialize state
        self.state = Some(ParserState::new(120.0)); // Default BPM, will be updated when found

        for line in content.lines() {
            if let Some(line) = normalize_line(line) {
                match self.state.as_ref().unwrap().parsing_state {
                    ParsingState::Metadata => {
                        if let Some((key, value)) = self.parse_metadata_or_header(line) {
                            let state = self.state.as_mut().unwrap();
                            if self.metadata_keys.contains(&key) {
                                if key == "BPM" {
                                    if let Ok(bpm) = value.parse::<f64>() {
                                        state.bpm = bpm;
                                    }
                                }
                                metadata_dict.insert(key, value.clone());
                            } else {
                                // If we hit a non-metadata key, transition to Header state
                                self.metadata = Some(Metadata::new(metadata_dict.clone()));
                                state.parsing_state = ParsingState::Header;
                                // Process this line as header
                                self.handle_metadata_or_header(line, &mut HashMap::new());
                            }
                        }
                    }
                    ParsingState::Header => {
                        let state = self.state.as_mut().unwrap();
                        if line.starts_with("#START") {
                            state.parsing_state = ParsingState::Notes;
                            self.process_directive(&line[1..])?;
                        } else {
                            self.handle_metadata_or_header(line, &mut HashMap::new());
                        }
                    }
                    ParsingState::Notes => {
                        if line.starts_with("#END") {
                            // Process any buffered notes before transitioning
                            if !notes_buffer.is_empty() {
                                self.process_notes_buffer(&notes_buffer)?;
                                notes_buffer.clear();
                            }
                            self.process_directive(&line[1..])?;
                            let state = self.state.as_mut().unwrap();
                            state.parsing_state = ParsingState::Header;
                        } else if let Some(directive) = line.strip_prefix('#') {
                            // Process any buffered notes before handling directive
                            if !notes_buffer.is_empty() {
                                self.process_notes_buffer(&notes_buffer)?;
                                notes_buffer.clear();
                            }
                            self.process_directive(directive)?;
                        } else {
                            notes_buffer.push(line.to_string());
                        }
                    }
                }
            }
        }

        // Process any remaining notes
        if !notes_buffer.is_empty() {
            self.process_notes_buffer(&notes_buffer)?;
        }

        Ok(())
    }

    fn process_notes_buffer(&mut self, notes_buffer: &[String]) -> Result<(), String> {
        for line in notes_buffer {
            if let Some(command) = line.strip_prefix("#") {
                self.process_directive(command)?;
            } else {
                self.process_notes(line)?;
            }
        }
        Ok(())
    }

    fn handle_metadata_or_header(
        &mut self,
        line: &str,
        metadata_dict: &mut HashMap<String, String>,
    ) {
        if let Some((key, value)) = self.parse_metadata_or_header(line) {
            if self.metadata_keys.contains(&key) {
                metadata_dict.insert(key, value);
            } else if self.header_keys.contains(&key) {
                if key == "BALLOON" {
                    let cleaned_value = value
                        .split(',')
                        .filter_map(|num| num.trim().parse::<i32>().ok())
                        .map(|num| num.to_string())
                        .collect::<Vec<_>>()
                        .join(",");
                    self.current_headers.insert(key.clone(), cleaned_value);
                } else {
                    self.current_headers.insert(key.clone(), value.clone());
                }
                if self.inheritable_header_keys.contains(&key) {
                    self.inherited_headers.insert(key, value);
                }
            }
        }
    }

    fn parse_metadata_or_header(&self, line: &str) -> Option<(String, String)> {
        if line.starts_with('#') {
            return None;
        }

        line.split_once(':').and_then(|(key, val)| {
            let key = key.trim();
            let val = val.trim();

            if key.is_empty() {
                return None;
            }

            Some((key.to_uppercase(), val.to_string()))
        })
    }

    fn process_directive(&mut self, command: &str) -> Result<(), String> {
        let handler = DirectiveHandler::new();
        if let Some(directive) = handler.parse_directive(command) {
            let state = self
                .state
                .as_mut()
                .ok_or_else(|| "Parser state not initialized".to_string())?;

            match directive {
                Directive::Start(player) => {
                    let player_num = match player.as_deref() {
                        Some("P1") => 1,
                        Some("P2") => 2,
                        _ => 0,
                    };

                    let mut merged_headers = self.inherited_headers.clone();
                    merged_headers.extend(self.current_headers.clone());

                    let chart = Chart::new(merged_headers, player_num);
                    self.charts.push(chart);
                    state.parsing_chart = true;
                    state.timestamp = -self.metadata.as_ref().unwrap().offset;
                }
                Directive::End => {
                    // Finalize current segment if it exists
                    if let Some(mut segment) = state.current_segment.take() {
                        if let Some(current_chart) = self.charts.last_mut() {
                            calculate_note_timestamp(state, &mut segment);
                            current_chart.segments.push(segment);
                        }
                    }
                    state.parsing_chart = false;
                    state.branch_condition = None;
                }
                Directive::BpmChange(bpm) => {
                    state.bpm = bpm;
                }
                Directive::Scroll(value) => {
                    state.scroll = value;
                }
                Directive::GogoStart => {
                    state.gogo = true;
                }
                Directive::GogoEnd => {
                    state.gogo = false;
                }
                Directive::BarlineOff => {
                    state.barline = false;
                }
                Directive::BarlineOn => {
                    state.barline = true;
                }
                Directive::BranchStart(condition) => {
                    state.branch_condition = Some(condition);
                    state.timestamp_branch_start = state.timestamp;
                }
                Directive::BranchEnd => {
                    state.parsing_chart = false;
                    state.branch_condition = None;
                    state.current_branch = None;
                }
                Directive::Measure(num, den) => {
                    state.measure_num = num;
                    state.measure_den = den;
                }
                Directive::Delay(value) => {
                    state.delay += value;
                }
                Directive::Section => {
                    // Handle section if needed, i don't remember what's this
                }
                Directive::BranchNormal => {
                    state.current_branch = Some("N".to_string());
                    state.timestamp = state.timestamp_branch_start;
                }
                Directive::BranchMaster => {
                    state.current_branch = Some("M".to_string());
                    state.timestamp = state.timestamp_branch_start;
                }
                Directive::BranchExpert => {
                    state.current_branch = Some("E".to_string());
                    state.timestamp = state.timestamp_branch_start;
                }
            }
        }
        Ok(())
    }

    fn process_notes(&mut self, notes_str: &str) -> Result<(), String> {
        let state = self
            .state
            .as_mut()
            .ok_or_else(|| "Parser state not initialized".to_string())?;

        if !state.parsing_chart {
            return Ok(());
        }

        let current_chart = self
            .charts
            .last_mut()
            .ok_or_else(|| "No current chart".to_string())?;

        for c in notes_str.chars() {
            match c {
                '0'..='9' => {
                    if let Some(note_type) = NoteType::from_char(c) {
                        let note = Note {
                            note_type,
                            timestamp: -1.0,
                            bpm: state.bpm,
                            delay: state.delay,
                            scroll: state.scroll,
                            gogo: state.gogo,
                        };

                        if state.current_segment.is_none() {
                            state.current_segment = Some(Segment::new(
                                state.measure_num,
                                state.measure_den,
                                state.barline,
                                state.current_branch.clone(),
                                state.branch_condition.clone(),
                            ));
                            state.current_segment.as_mut().unwrap().notes.reserve(64);
                        }
                        if let Some(segment) = &mut state.current_segment {
                            segment.notes.push(note);
                        }
                    }
                }
                ',' => {
                    // Finalize current segment and create new one
                    if let Some(mut segment) = state.current_segment.take() {
                        calculate_note_timestamp(state, &mut segment);
                        current_chart.segments.push(segment);
                    }
                }
                _ => {} // Ignore other characters
            }
        }

        Ok(())
    }

    // Getter methods
    pub fn get_metadata(&self) -> Option<&Metadata> {
        self.metadata.as_ref()
    }

    pub fn get_charts(&self) -> &[Chart] {
        &self.charts
    }

    pub fn get_charts_for_player(&self, player: i32) -> Vec<&Chart> {
        self.charts
            .iter()
            .filter(|chart| chart.player == player)
            .collect()
    }

    pub fn get_double_charts(&self) -> Vec<(&Chart, &Chart)> {
        let mut double_charts = Vec::new();
        let p1_charts: Vec<_> = self.get_charts_for_player(1);
        let p2_charts: Vec<_> = self.get_charts_for_player(2);

        for p1_chart in p1_charts {
            for p2_chart in &p2_charts {
                if p1_chart
                    .headers
                    .get("STYLE")
                    .map_or(false, |s| s.to_uppercase() == "DOUBLE")
                    && p2_chart
                        .headers
                        .get("STYLE")
                        .map_or(false, |s| s.to_uppercase() == "DOUBLE")
                    && p1_chart.headers.get("COURSE") == p2_chart.headers.get("COURSE")
                {
                    double_charts.push((p1_chart, *p2_chart));
                    break;
                }
            }
        }

        double_charts
    }

    pub fn get_parsed_tja(&self) -> ParsedTJA {
        ParsedTJA {
            metadata: self.metadata.clone().unwrap(),
            charts: self.charts.clone(),
        }
    }

    pub fn add_metadata_key(&mut self, key: &str) {
        self.metadata_keys.insert(key.to_string());
    }

    pub fn add_header_key(&mut self, key: &str) {
        self.header_keys.insert(key.to_string());
    }

    pub fn add_inheritable_header_key(&mut self, key: &str) {
        self.inheritable_header_keys.insert(key.to_string());
    }
}

fn normalize_line(line: &str) -> Option<&str> {
    let line = line.split("//").next()?;
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    Some(line)
}

fn calculate_note_timestamp(state: &mut ParserState, segment: &mut Segment) {
    let count = segment.notes.len();
    if count > 0 {
        for note in segment.notes.iter_mut() {
            note.timestamp = state.timestamp + note.delay;
            state.timestamp +=
                60.0 / note.bpm * segment.measure_num as f64 / segment.measure_den as f64 * 4.0
                    / count as f64;
        }
    } else {
        state.timestamp +=
            60.0 / state.bpm * segment.measure_num as f64 / segment.measure_den as f64 * 4.0;
    }
    segment
        .notes
        .retain(|note| note.note_type != NoteType::Empty);
}
