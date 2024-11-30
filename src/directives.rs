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
}

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
        let mut parts = command.splitn(2, ' ');
        let base_directive = parts.next()?.to_uppercase();
        let args = parts.next().unwrap_or("").trim();

        match base_directive.as_str() {
            "START" => {
                let player = if !args.is_empty() {
                    Some(args.to_string())
                } else {
                    None
                };
                Some(Directive::Start(player))
            }
            "END" => Some(Directive::End),
            "BPMCHANGE" => args.parse().ok().map(Directive::BpmChange),
            "SCROLL" => args.parse().ok().map(Directive::Scroll),
            "GOGOSTART" => Some(Directive::GogoStart),
            "GOGOEND" => Some(Directive::GogoEnd),
            "BARLINEOFF" => Some(Directive::BarlineOff),
            "BARLINEON" => Some(Directive::BarlineOn),
            "BRANCHSTART" => Some(Directive::BranchStart(args.to_string())),
            "BRANCHEND" => Some(Directive::BranchEnd),
            "MEASURE" => {
                let parts: Vec<&str> = args.split('/').collect();
                if parts.len() == 2 {
                    if let (Ok(num), Ok(den)) = (parts[0].parse(), parts[1].parse()) {
                        return Some(Directive::Measure(num, den));
                    }
                }
                None
            }
            "DELAY" => args.parse().ok().map(Directive::Delay),
            "SECTION" => Some(Directive::Section),
            _ => None,
        }
    }

    pub fn get_directive_type(&self, command: &str) -> Option<DirectiveType> {
        let base_directive = command.split_whitespace().next()?.to_uppercase();
        match base_directive.as_str() {
            "START" | "END" | "MEASURE" | "BARLINEOFF" | "BARLINEON" | "BRANCHSTART"
            | "BRANCHEND" | "SECTION" => Some(DirectiveType::Bar),
            "SCROLL" | "DELAY" | "BPMCHANGE" | "GOGOSTART" | "GOGOEND" => Some(DirectiveType::Note),
            _ => None,
        }
    }
}
