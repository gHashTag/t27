// Position conversion utilities

use tower_lsp::lsp_types::{Position, Range};

/// Convert line/column to LSP Position (0-based)
pub fn to_lsp_position(line: usize, column: usize) -> Position {
    Position {
        line: line as u32,
        character: column as u32,
    }
}

/// Create an LSP Range from start and end positions
pub fn to_lsp_range(start_line: usize, start_col: usize, end_line: usize, end_col: usize) -> Range {
    Range {
        start: to_lsp_position(start_line, start_col),
        end: to_lsp_position(end_line, end_col),
    }
}

/// Convert LSP Position to byte offset (for parser integration)
pub fn position_to_offset(text: &str, position: &Position) -> usize {
    let mut offset = 0;
    let mut line = 0;

    for (idx, c) in text.char_indices() {
        if line == position.line as usize {
            return offset + position.character as usize;
        }
        if c == '\n' {
            line += 1;
        }
        offset = idx;
    }

    text.len()
}
