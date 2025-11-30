use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum DirectiveType {
    Bar,
    Note,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Directive {
    Start(Option<String>), // Optional P1/P2
    End,
    BpmChange(f64),
    Scroll(f64),
    GogoStart,
    GogoEnd,
    BarlineOff,
    BarlineOn,
    BranchStart(String), // Branch condition
    BranchEnd,
    Measure(i32, i32), // num/den
    Delay(f64),
    Section,
    BranchNormal,
    BranchMaster,
    BranchExpert,
}

#[derive(Debug, Clone)]
pub struct DirectiveHandler;

impl Default for DirectiveHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl DirectiveHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_directive(&self, command: &str) -> Option<Directive> {
        // Find the split point without allocating
        let (base_directive, args) = match command.find(' ') {
            Some(idx) => (&command[..idx], command[idx + 1..].trim()),
            None => (command, ""),
        };

        // Use case-insensitive comparison without allocating a new String
        // Match against uppercase patterns using eq_ignore_ascii_case
        if base_directive.eq_ignore_ascii_case("START") {
            let player = if !args.is_empty() {
                Some(args.to_string())
            } else {
                None
            };
            Some(Directive::Start(player))
        } else if base_directive.eq_ignore_ascii_case("END") {
            Some(Directive::End)
        } else if base_directive.eq_ignore_ascii_case("BPMCHANGE") {
            args.parse().ok().map(Directive::BpmChange)
        } else if base_directive.eq_ignore_ascii_case("SCROLL") {
            args.parse().ok().map(Directive::Scroll)
        } else if base_directive.eq_ignore_ascii_case("GOGOSTART") {
            Some(Directive::GogoStart)
        } else if base_directive.eq_ignore_ascii_case("GOGOEND") {
            Some(Directive::GogoEnd)
        } else if base_directive.eq_ignore_ascii_case("BARLINEOFF") {
            Some(Directive::BarlineOff)
        } else if base_directive.eq_ignore_ascii_case("BARLINEON") {
            Some(Directive::BarlineOn)
        } else if base_directive.eq_ignore_ascii_case("BRANCHSTART") {
            Some(Directive::BranchStart(args.to_string()))
        } else if base_directive.eq_ignore_ascii_case("BRANCHEND") {
            Some(Directive::BranchEnd)
        } else if base_directive.eq_ignore_ascii_case("MEASURE") {
            // Avoid collecting into a Vec by using split_once
            if let Some((num_str, den_str)) = args.split_once('/') {
                if let (Ok(num), Ok(den)) = (num_str.parse(), den_str.parse()) {
                    return Some(Directive::Measure(num, den));
                }
            }
            None
        } else if base_directive.eq_ignore_ascii_case("DELAY") {
            args.parse().ok().map(Directive::Delay)
        } else if base_directive.eq_ignore_ascii_case("SECTION") {
            Some(Directive::Section)
        } else if base_directive.eq_ignore_ascii_case("N") {
            Some(Directive::BranchNormal)
        } else if base_directive.eq_ignore_ascii_case("M") {
            Some(Directive::BranchMaster)
        } else if base_directive.eq_ignore_ascii_case("E") {
            Some(Directive::BranchExpert)
        } else {
            None
        }
    }

    pub fn get_directive_type(&self, command: &str) -> Option<DirectiveType> {
        // Find the first word without allocating
        let base_directive = match command.find(char::is_whitespace) {
            Some(idx) => &command[..idx],
            None => command,
        };
        
        // Use case-insensitive comparison without allocating
        if base_directive.eq_ignore_ascii_case("START")
            || base_directive.eq_ignore_ascii_case("END")
            || base_directive.eq_ignore_ascii_case("MEASURE")
            || base_directive.eq_ignore_ascii_case("BARLINEOFF")
            || base_directive.eq_ignore_ascii_case("BARLINEON")
            || base_directive.eq_ignore_ascii_case("BRANCHSTART")
            || base_directive.eq_ignore_ascii_case("BRANCHEND")
            || base_directive.eq_ignore_ascii_case("SECTION")
            || base_directive.eq_ignore_ascii_case("N")
            || base_directive.eq_ignore_ascii_case("M")
            || base_directive.eq_ignore_ascii_case("E")
        {
            Some(DirectiveType::Bar)
        } else if base_directive.eq_ignore_ascii_case("SCROLL")
            || base_directive.eq_ignore_ascii_case("DELAY")
            || base_directive.eq_ignore_ascii_case("BPMCHANGE")
            || base_directive.eq_ignore_ascii_case("GOGOSTART")
            || base_directive.eq_ignore_ascii_case("GOGOEND")
        {
            Some(DirectiveType::Note)
        } else {
            None
        }
    }
}
