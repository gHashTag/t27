// Code Actions Service for t27 Language Server

use crate::types::Document;
use tower_lsp::lsp_types::{CodeAction, CodeActionKind, Diagnostic, Range};

/// Code Actions Service
pub struct CodeActionsService;

impl CodeActionsService {
    /// Get code actions for a position/range
    pub fn get_code_actions(doc: &Document, range: Range) -> Vec<CodeAction> {
        let mut actions = Vec::new();

        // Add quick fixes for diagnostics in range
        for diagnostic in &doc.diagnostics {
            if Self::range_intersects(&diagnostic.range, range) {
                if let Some(action) = Self::quick_fix_for_diagnostic(diagnostic) {
                    actions.push(action);
                }
            }
        }

        // Add refactoring suggestions for symbols in range
        for symbol in &doc.symbols {
            if Self::range_intersects(&symbol.range, range) {
                if let Some(action) = Self::refactoring_action_for_symbol(symbol) {
                    actions.push(action);
                }
            }
        }

        actions
    }

    /// Check if two ranges intersect
    fn range_intersects(diagnostic_range: &Range, check_range: &Range) -> bool {
        !(diagnostic_range.end < check_range.start || diagnostic_range.start > check_range.end)
    }

    /// Get quick fix for a diagnostic
    fn quick_fix_for_diagnostic(diagnostic: &Diagnostic) -> Option<CodeAction> {
        let message = diagnostic.message.to_lowercase();

        if message.contains("todo:") || message.contains("fixme:") {
            Some(CodeAction {
                title: format!("Create task for {}", message.trim_start_matches(5..)),
                kind: Some(CodeActionKind::QuickFix),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: None,
                command: None,
                is_preferred: Some(true),
                data: None,
            })
        } else if message.contains("unused import") {
            Some(CodeAction {
                title: "Remove unused import".to_string(),
                kind: Some(CodeActionKind::QuickFix),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: Some(vec![{
                    range: diagnostic.range,
                    new_text: String::new(),
                }]),
                command: None,
                is_preferred: Some(true),
                data: None,
            })
        } else {
            None
        }
    }

    /// Get refactoring action for a symbol
    fn refactoring_action_for_symbol(symbol: &crate::types::Symbol) -> Option<CodeAction> {
        match &symbol.kind {
            crate::types::SymbolKind::Function => {
                Some(CodeAction {
                    title: "Add documentation comment".to_string(),
                    kind: Some(CodeActionKind::Refactor),
                    diagnostics: None,
                    edit: Some(vec![{
                        range: symbol.range,
                        new_text: format!("// {}\n", symbol.name),
                    }]),
                    command: None,
                    is_preferred: Some(false),
                    data: None,
                })
            },
            crate::types::SymbolKind::Variable | crate::types::SymbolKind::Constant => {
                if let Some(detail) = &symbol.detail {
                    if detail.contains(':') {
                        let type_part = detail.split(':').nth(1);
                        Some(CodeAction {
                            title: format!("Annotate variable as: {}", type_part.unwrap_or("unknown")),
                            kind: Some(CodeActionKind::RefactorInline),
                            diagnostics: None,
                            edit: Some(vec![{
                                range: symbol.range,
                                new_text: format!("{}: {}", symbol.name, type_part.unwrap_or("unknown")),
                            }]),
                            command: None,
                            is_preferred: Some(false),
                            data: None,
                        })
                    }
                } else {
                    None
                }
            } else {
                None
            }
            _ => None,
        }
    }
}
