// Minimal working parser for Ring-001

pub mod types {
    pub use crate::anyhow::{anyhow, Result};
}

pub fn parse(spec_path: &str) -> Result<ParseResult, ParseError> {
    let content = std::fs::read_to_string(spec_path)
        .map_err(|e| ParseError::IoError(e.to_string()))?;

    let mut modules = std::vec::Vec::new();
    let mut lines = content.lines();
    let mut i = 0;
    let mut brace_level = 0;
    let mut in_section = false;

    while i < 2000 {
        let line = match lines.next() {
            Some(l) => l,
            None => break,
        };

        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with("--") || trimmed.starts_with("//") {
            i += 1;
            continue;
        }

        // Module declaration
        if trimmed.starts_with("module ") && brace_level == 0 {
            if let Some(pos) = trimmed.find('{') {
                modules.push(ModuleDecl {
                    name: trimmed[7..pos].trim().to_string(),
                    line_number: i + 1,
                });
                brace_level += 1;
                i += 1;
            }
        }

        // Section start (constants, types, invariants, tests, benchmarks)
        if trimmed.starts_with("constants {") && brace_level == 0 {
            in_section = true;
            brace_level += 1;
            i += 1;
        } else if trimmed.starts_with("types {") && brace_level == 0 {
            in_section = true;
            brace_level += 1;
            i += 1;
        } else if trimmed.starts_with("invariants {") && brace_level == 0 {
            in_section = true;
            brace_level += 1;
            i += 1;
        } else if trimmed.starts_with("tests {") && brace_level == 0 {
            in_section = true;
            brace_level += 1;
            i += 1;
        } else if trimmed.starts_with("benchmarks {") && brace_level == 0 {
            in_section = true;
            brace_level += 1;
            i += 1;
        }

        // Section end
        if trimmed == "}" {
            brace_level -= 1;
            if brace_level == 0 {
                in_section = false;
            }
        }

        i += 1;
    }

    Ok(ParseResult {
        modules,
        total_lines: i,
    })
}

#[derive(Debug, Clone)]
pub struct ParseResult {
    pub modules: Vec<ModuleDecl>,
    pub total_lines: usize,
}

#[derive(Debug, Clone)]
pub struct ModuleDecl {
    pub name: String,
    pub line_number: usize,
}

#[derive(Debug, Clone)]
pub struct ModuleItem {
    pub kind: String,
    pub content: String,
    pub start_line: usize,
}

#[derive(Debug, Clone)]
pub enum ParseError {
    IoError(String),
    ModuleSyntax(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IoError(msg) => write!(f, "IO error: {}", msg),
            ParseError::ModuleSyntax(msg) => write!(f, "Module syntax error: {}", msg),
        }
    }
}

fn main() {
    let args: std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: parser_test <spec_path>");
        std::process::exit(1);
    }

    let spec_path = &args[1];
    println!("Parsing: {}", spec_path);

    match parse(spec_path) {
        Ok(parsed) => {
            println!("SUCCESS");
            println!("Modules: {}", parsed.modules.len());
            for module in &parsed.modules {
                println!("  Module: {} (line {})", module.name, module.line_number);
            }
            println!("Total lines processed: {}", parsed.total_lines);
        }
        Err(e) => {
            eprintln!("ERROR: {}", e);
            std::process::exit(1);
        }
    }
}
