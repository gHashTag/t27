//! Source location tracking
//!
//! Tracks positions in source files for error reporting.
//! Part of modular compiler architecture (Ring-018).

use std::fmt;

/// Position in source file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub column: u32,
    pub offset: usize,
}

impl Position {
    pub fn new(line: u32, column: u32, offset: usize) -> Self {
        Position {
            line,
            column,
            offset,
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Location in source file (file:line:col)
#[derive(Debug, Clone)]
pub struct Location {
    pub file: String,
    pub start: Position,
    pub end: Position,
}

impl Location {
    pub fn new(file: String, start: Position, end: Position) -> Self {
        Location {
            file,
            start,
            end,
        }
    }
}

/// Span (location with optional context)
#[derive(Debug, Clone)]
pub struct Span {
    pub start: Position,
    pub end: Position,
    pub context: Option<String>,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Span {
            start,
            end,
            context: None,
        }
    }
}
