//! Diagnostics for t27 specs

use tower_lsp::lsp_types::*;
use crate::parser::{Diagnostic as T27Diagnostic, DiagnosticSeverity};

/// Convert t27 diagnostic to LSP diagnostic
pub fn convert_diagnostic(diag: T27Diagnostic) -> lsp_types::Diagnostic {
    Diagnostic {
        range: convert_range(&diag.range),
        severity: Some(convert_severity(diag.severity)),
        message: diag.message,
        code: diag.code.map(|c| NumberOrString::String(c)),
        ..Default::default()
    }
}

/// Convert t27 range to LSP range
fn convert_range(range: &crate::parser::Range) -> lsp_types::Range {
    lsp_types::Range {
        start: lsp_types::Position {
            line: range.start.line,
            character: range.start.character,
        },
        end: lsp_types::Position {
            line: range.end.line,
            character: range.end.character,
        },
    }
}

/// Convert t27 severity to LSP severity
fn convert_severity(sev: DiagnosticSeverity) -> lsp_types::DiagnosticSeverity {
    match sev {
        DiagnosticSeverity::Error => lsp_types::DiagnosticSeverity::ERROR,
        DiagnosticSeverity::Warning => lsp_types::DiagnosticSeverity::WARNING,
        DiagnosticSeverity::Info => lsp_types::DiagnosticSeverity::INFORMATION,
        DiagnosticSeverity::Hint => lsp_types::DiagnosticSeverity::HINT,
    }
}

/// Diagnostics for common t27 errors
pub const COMMON_ERRORS: &[(&str, &str)] = &[
    ("MISSING_MODULE", "Module must have a name"),
    ("MISSING_FN_NAME", "Function must have a name"),
    ("MISSING_FN_BODY", "Function body is empty"),
    ("MISSING_TEST_GIVEN", "Test must have 'given' section"),
    ("MISSING_TEST_THEN", "Test must have 'then' section"),
    ("MISSING_TEST_EXPECT", "Test must have 'expect' section"),
    ("INVALID_TYPE", "Type not recognized"),
    ("INVALID_PHI_LITERAL", "φ literal must be in range [0.5, 2.618]"),
];