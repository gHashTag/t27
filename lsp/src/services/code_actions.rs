// Code actions service for t27 Language Server

use crate::types::Document;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, Diagnostic, Position, Range, TextEdit, Url, WorkspaceEdit,
};

/// Code actions service
pub struct CodeActionsService;

impl CodeActionsService {
    /// Get code actions for a position/range
    pub fn get_code_actions(
        doc: &Document,
        range: Range,
    ) -> Vec<CodeAction> {
        let mut actions = Vec::new();

        // Add quick fixes for diagnostics in range
        for diagnostic in &doc.diagnostics {
            if Self::diagnostic_intersects_range(diagnostic, range) {
                actions.extend(Self::quick_fixes_for_diagnostic(diagnostic, doc));
            }
        }

        // Add general refactoring actions
        actions.extend(Self::refactoring_actions(doc, range));

        actions
    }

    /// Get quick fixes for a specific diagnostic
    fn quick_fixes_for_diagnostic(
        diagnostic: &Diagnostic,
        doc: &Document,
    ) -> Vec<CodeAction> {
        let mut fixes = Vec::new();

        // Quick fix for TODO/FIXME comments
        if let Some(message) = diagnostic.message.strip_prefix("TODO: ") {
            fixes.push(Self::create_add_task_action(message, diagnostic.range));
        }

        if let Some(message) = diagnostic.message.strip_prefix("FIXME: ") {
            fixes.push(Self::create_fix_issue_action(message, diagnostic.range));
        }

        // Quick fix for unused imports
        if diagnostic.message.contains("unused") && diagnostic.message.contains("import") {
            fixes.push(Self::create_remove_import_action(diagnostic.range, doc));
        }

        fixes
    }

    /// Get refactoring actions
    fn refactoring_actions(doc: &Document, range: Range) -> Vec<CodeAction> {
        let mut actions = Vec::new();

        // Extract to constant action
        if let Some(text) = Self::get_selected_text(doc, range) {
            if !text.is_empty() && text.len() < 50 {
                actions.push(Self::create_extract_constant_action(&text, range));
            }
        }

        actions
    }

    /// Check if a diagnostic intersects with a range
    fn diagnostic_intersects_range(diagnostic: &Diagnostic, range: Range) -> bool {
        let d_range = diagnostic.range;
        !(d_range.end < range.start || d_range.start > range.end)
    }

    /// Get text at a range
    fn get_selected_text(doc: &Document, range: Range) -> Option<String> {
        let lines: Vec<&str> = doc.text.lines().collect();
        if range.start.line as usize >= lines.len() {
            return None;
        }

        if range.start.line == range.end.line {
            let line = lines[range.start.line as usize];
            let start = range.start.character as usize;
            let end = range.end.character as usize;
            if end <= line.len() {
                return Some(line[start..end].to_string());
            }
        }

        None
    }

    /// Create action to add a task
    fn create_add_task_action(task: &str, range: Range) -> CodeAction {
        CodeAction {
            title: format!("Create task: {}", task),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: None,
            edit: None,
            command: None,
            is_preferred: Some(true),
            disabled: None,
            data: None,
        }
    }

    /// Create action to fix an issue
    fn create_fix_issue_action(issue: &str, range: Range) -> CodeAction {
        CodeAction {
            title: format!("Address issue: {}", issue),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: None,
            edit: None,
            command: None,
            is_preferred: Some(true),
            disabled: None,
            data: None,
        }
    }

    /// Create action to remove unused import
    fn create_remove_import_action(range: Range, doc: &Document) -> CodeAction {
        let uri = doc.uri.clone();
        let mut changes = std::collections::HashMap::new();
        changes.insert(uri, vec![TextEdit {
            range,
            new_text: String::new(),
        }]);

        CodeAction {
            title: "Remove unused import".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            diagnostics: None,
            edit: Some(WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            }),
            command: None,
            is_preferred: Some(true),
            disabled: None,
            data: None,
        }
    }

    /// Create action to extract constant
    fn create_extract_constant_action(text: &str, range: Range) -> CodeAction {
        CodeAction {
            title: format!("Extract to constant: {}", text),
            kind: Some(CodeActionKind::REFACTOR_EXTRACT),
            diagnostics: None,
            edit: None,
            command: None,
            is_preferred: Some(false),
            disabled: None,
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::lsp_types::{Position, Range, Url};

    #[test]
    fn test_diagnostic_intersects_range() {
        let diagnostic = Diagnostic {
            range: Range {
                start: Position::new(0, 5),
                end: Position::new(0, 10),
            },
            ..Default::default()
        };

        let intersecting = Range {
            start: Position::new(0, 0),
            end: Position::new(0, 15),
        };
        assert!(CodeActionsService::diagnostic_intersects_range(&diagnostic, intersecting));

        let non_intersecting = Range {
            start: Position::new(1, 0),
            end: Position::new(1, 10),
        };
        assert!(!CodeActionsService::diagnostic_intersects_range(&diagnostic, non_intersecting));
    }
}
