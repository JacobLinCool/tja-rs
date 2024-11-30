use crate::directives::{Directive, DirectiveHandler, DirectiveType};
use crate::types::*;
use std::collections::HashMap;

pub const DEFAULT_METADATA_KEYS: &[&str] = &[
    "TITLE",
    "WAVE",
    "SUBTITLE",
    "BPM",
    "AUTHOR",
    "OFFSET",
    "SONGVOL",
    "SEVOL",
    "DEMOSTART",
    "SCOREMODE",
];
pub const DEFAULT_HEADER_KEYS: &[&str] = &[
    "LEVEL",
    "COURSE",
    "STYLE",
    "BALLOON",
    "SCOREINIT",
    "SCOREDIFF",
];
pub const DEFAULT_INHERITABLE_HEADER_KEYS: &[&str] =
    &["LEVEL", "COURSE", "STYLE", "SCOREINIT", "SCOREDIFF"];

#[derive(Debug, Clone)]
pub struct ParserState {
    pub bpm: f64,
    pub scroll: f64,
    pub gogo: bool,
    pub barline: bool,
    pub measure_num: i32,
    pub measure_den: i32,
    pub branch_active: bool,
    pub branch_condition: Option<String>,
    pub parsing_chart: bool,
    pub delay: f64,
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
            branch_active: false,
            branch_condition: None,
            parsing_chart: false,
            delay: 0.0,
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
}

impl Default for TJAParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TJAParser {
    pub fn new() -> Self {
        Self {
            metadata: None,
            charts: Vec::new(),
            state: None,
            inherited_headers: HashMap::new(),
            current_headers: HashMap::new(),
        }
    }

    pub fn parse_str(&mut self, content: &str) -> Result<(), String> {
        let mut metadata_dict = HashMap::new();
        let mut notes_buffer = Vec::new();

        // First pass: collect metadata
        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if line.contains(":") && !line.starts_with("#") {
                self.handle_metadata_or_header(line, &mut metadata_dict);
            }
        }

        // Initialize state with metadata
        self.metadata = Some(Metadata::new(metadata_dict));
        self.state = Some(ParserState::new(self.metadata.as_ref().unwrap().bpm()));

        // Second pass: process everything else
        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if line.contains(":") && !line.starts_with("#") {
                self.handle_metadata_or_header(line, &mut HashMap::new());
                continue;
            }

            if let Some(command) = line.strip_prefix("#") {
                let handler = DirectiveHandler::new();

                if let Some(directive_type) = handler.get_directive_type(command) {
                    match directive_type {
                        DirectiveType::Bar => {
                            // Process any accumulated notes before handling bar directive
                            if !notes_buffer.is_empty() {
                                self.process_notes_buffer(&notes_buffer)
                                    .map_err(|e| e.to_string())?;
                                notes_buffer.clear();
                            }
                            self.process_directive(command).map_err(|e| e.to_string())?;
                        }
                        DirectiveType::Note => {
                            notes_buffer.push(line.to_string());
                        }
                    }
                }
            } else if self.state.as_ref().map_or(false, |s| s.parsing_chart) {
                // Handle regular notes line
                if let Some(notes_part) = line.split("//").next() {
                    notes_buffer.push(notes_part.to_string());
                }
            }
        }

        // Process any remaining notes
        if !notes_buffer.is_empty() {
            self.process_notes_buffer(&notes_buffer)
                .map_err(|e| e.to_string())?;
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
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim().to_uppercase();
            let value = value.trim().to_string();

            if DEFAULT_METADATA_KEYS.contains(&key.as_str()) {
                metadata_dict.insert(key, value);
            } else if DEFAULT_HEADER_KEYS.contains(&key.as_str()) {
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
                if DEFAULT_INHERITABLE_HEADER_KEYS.contains(&key.as_str()) {
                    self.inherited_headers.insert(key, value);
                }
            }
        }
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
                }
                Directive::End => {
                    state.parsing_chart = false;
                    state.branch_active = false;
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
                    state.branch_active = true;
                    state.branch_condition = Some(condition);
                }
                Directive::BranchEnd => {
                    state.branch_active = false;
                    state.branch_condition = None;
                }
                Directive::Measure(num, den) => {
                    state.measure_num = num;
                    state.measure_den = den;
                }
                Directive::Delay(value) => {
                    state.delay += value;
                }
                Directive::Section => {
                    // Handle section if needed
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

        // Create initial segment if none exists
        if current_chart.segments.is_empty() {
            let new_segment = Segment::new(
                state.measure_num,
                state.measure_den,
                state.barline,
                state.branch_active,
                state.branch_condition.clone(),
            );
            current_chart.segments.push(new_segment);
        }

        for c in notes_str.chars() {
            match c {
                ',' => {
                    let new_segment = Segment::new(
                        state.measure_num,
                        state.measure_den,
                        state.barline,
                        state.branch_active,
                        state.branch_condition.clone(),
                    );
                    current_chart.segments.push(new_segment);
                }
                '0'..='9' => {
                    if let Some(note_type) = NoteType::from_char(c) {
                        let note = Note {
                            note_type,
                            scroll: state.scroll,
                            delay: state.delay,
                            bpm: state.bpm,
                            gogo: state.gogo,
                        };

                        if let Some(segment) = current_chart.segments.last_mut() {
                            segment.notes.push(note);
                        }
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
}
