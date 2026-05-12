// Formatting Service for t27 Language Server

use crate::types::Document;
use tower_lsp::lsp_types::TextEdit;

/// Formatting Service
pub struct FormattingService;

impl FormattingService {
    /// Format a document
    pub fn format_document(_doc: &Document) -> Vec<TextEdit> {
        // For now, return empty (will implement with rustfmt later)
        // This is a placeholder - real formatting will be added when
        // rustfmt is integrated
        Vec::new()
    }
}
