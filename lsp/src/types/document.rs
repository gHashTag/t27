// Document representation for t27 Language Server

use crate::types::position::to_lsp_position;
use crate::types::Symbol;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticSeverity, Position, Range, TextDocumentContentChangeEvent,
    Url,
};

/// Document representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub uri: Url,
    pub version: i32,
    pub text: String,
    pub line_offsets: Vec<usize>,
    pub symbols: Vec<Symbol>,
    pub diagnostics: Vec<Diagnostic>,
    #[serde(skip)]
    pub parsed: bool,
}

impl Document {
    pub fn new(uri: Url, text: String) -> Self {
        let line_offsets = Self::calculate_line_offsets(&text);

        Self {
            uri,
            version: 0,
            text,
            line_offsets,
            symbols: Vec::new(),
            diagnostics: Vec::new(),
            parsed: false,
        }
    }

    pub fn from_path(path: PathBuf) -> Result<Self, std::io::Error> {
        let text = std::fs::read_to_string(&path)?;
        let uri = Url::from_file_path(path)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid path"))?;
        Ok(Self::new(uri, text))
    }

    fn calculate_line_offsets(text: &str) -> Vec<usize> {
        let mut offsets = Vec::new();
        let mut offset = 0;

        for line in text.lines() {
            offsets.push(offset);
            offset += line.len() + 1; // +1 for newline
        }

        offsets
    }

    pub fn update(&mut self, changes: &[TextDocumentContentChangeEvent]) {
        for change in changes {
            if let Some(range) = &change.range {
                // Apply incremental change
                self.apply_change_range(range, &change.text);
            } else {
                // Full document replacement
                self.text = change.text.clone();
            }
        }

        // Recalculate line offsets
        self.line_offsets = Self::calculate_line_offsets(&self.text);
        // Mark as needing reparse
        self.parsed = false;
        self.symbols.clear();
        self.diagnostics.clear();
    }

    fn apply_change_range(&mut self, range: &Range, new_text: &str) {
        let start = self.position_to_offset(&range.start);
        let end = self.position_to_offset(&range.end);

        if start <= end && end <= self.text.len() {
            let before = &self.text[..start];
            let after = &self.text[end..];
            self.text = format!("{}{}{}", before, new_text, after);
        }
    }

    pub fn position_to_offset(&self, position: &Position) -> usize {
        let line_idx = position.line as usize;
        if line_idx >= self.line_offsets.len() {
            return self.text.len();
        }

        let line_start = self.line_offsets[line_idx];
        let char_offset = position.character as usize;

        // Find the actual character offset (handle multi-byte UTF-8)
        self.text[line_start..]
            .char_indices()
            .nth(char_offset)
            .map(|(idx, _)| line_start + idx)
            .unwrap_or(self.text.len())
    }

    pub fn offset_to_position(&self, offset: usize) -> Position {
        let line = self
            .line_offsets
            .binary_search(&offset)
            .unwrap_or_else(|i| i.saturating_sub(1));

        if line >= self.line_offsets.len() {
            return to_lsp_position(self.line_offsets.len(), 0);
        }

        let line_start = self.line_offsets[line];
        let char_offset = self.text[line_start..offset].chars().count();

        to_lsp_position(line, char_offset)
    }

    pub fn line_text(&self, line: usize) -> Option<&str> {
        if line >= self.line_offsets.len() {
            return None;
        }

        let start = self.line_offsets[line];
        let end = self.line_offsets.get(line + 1).copied().unwrap_or(self.text.len());

        Some(&self.text[start..end])
    }

    pub fn get_text(&self, range: &Range) -> String {
        let start = self.position_to_offset(&range.start);
        let end = self.position_to_offset(&range.end);

        if start < end && end <= self.text.len() {
            self.text[start..end].to_string()
        } else {
            String::new()
        }
    }
}

/// Create a diagnostic
pub fn create_diagnostic(
    range: Range,
    message: String,
    severity: DiagnosticSeverity,
) -> Diagnostic {
    Diagnostic {
        range,
        severity: Some(severity),
        code: None,
        code_description: None,
        source: Some("t27".to_string()),
        message,
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Create an error diagnostic
pub fn error_diagnostic(range: Range, message: String) -> Diagnostic {
    create_diagnostic(range, message, DiagnosticSeverity::ERROR)
}

/// Create a warning diagnostic
pub fn warning_diagnostic(range: Range, message: String) -> Diagnostic {
    create_diagnostic(range, message, DiagnosticSeverity::WARNING)
}

/// Create an info diagnostic
pub fn info_diagnostic(range: Range, message: String) -> Diagnostic {
    create_diagnostic(range, message, DiagnosticSeverity::INFORMATION)
}

/// Create a hint diagnostic
pub fn hint_diagnostic(range: Range, message: String) -> Diagnostic {
    create_diagnostic(range, message, DiagnosticSeverity::HINT)
}
