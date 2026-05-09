// Diagnostics service for t27 Language Server

use crate::backend::parser::validate_document;
use crate::types::Document;
use crate::types::position::to_lsp_range;
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Url};
use tokio::sync::mpsc::Sender;

/// Diagnostics service
pub struct DiagnosticsService {
    diagnostics_tx: Option<Sender<Url>>,
}

impl DiagnosticsService {
    pub fn new(diagnostics_tx: Option<Sender<Url>>) -> Self {
        Self { diagnostics_tx }
    }

    /// Generate diagnostics for a document
    pub fn generate_from_document(doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = validate_document(doc);

        // Add additional linting
        diagnostics.extend(Self::lint_document(doc));

        diagnostics
    }

    /// Publish diagnostics for a document
    pub fn publish(&self, uri: Url) {
        if let Some(tx) = &self.diagnostics_tx {
            let _ = tx.try_send(uri);
        }
    }

    /// Clear diagnostics for a document
    pub fn clear(&self, uri: Url) {
        // Handled by publishing empty diagnostics
        self.publish(uri);
    }

    /// Additional linting rules
    fn lint_document(doc: &Document) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (line_idx, line) in doc.text.lines().enumerate() {
            let trimmed = line.trim();

            // Check for TODO comments
            if trimmed.contains("TODO") || trimmed.contains("FIXME") {
                let start = line.find(|c: char| c == 'T' || c == 'F').unwrap_or(0);
                diagnostics.push(Diagnostic {
                    range: to_lsp_range(line_idx, start, line_idx, line.len()),
                    severity: Some(DiagnosticSeverity::INFORMATION),
                    message: "Unresolved TODO comment".to_string(),
                    ..Default::default()
                });
            }

            // Check for magic numbers (heuristic)
            if trimmed.contains('=') && !trimmed.contains("const") {
                if let Some(eq_pos) = trimmed.find('=') {
                    let after_eq = &trimmed[eq_pos + 1..];
                    if after_eq.trim().chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-')
                    {
                        diagnostics.push(Diagnostic {
                            range: to_lsp_range(line_idx, eq_pos, line_idx, line.len()),
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: "Consider using a named constant instead of a magic number".to_string(),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        diagnostics
    }
}

impl Default for DiagnosticsService {
    fn default() -> Self {
        Self::new(None)
    }
}
