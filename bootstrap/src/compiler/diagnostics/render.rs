//! Error rendering
//!
//! Formats compiler errors for console output.
//! Part of modular compiler architecture (Ring-018).

use super::{error::CompilerError, span::Location};

/// Render a compiler error to user-friendly message
pub fn render_error(err: &CompilerError) -> String {
    match &err.span {
        Some(span) => {
            format!(
                "error: {} at {}:{}:{}",
                err.kind,
                span.start.line,
                span.start.column,
                err.message
            )
        }
        None => format!("error: {} - {}", err.kind, err.message),
    }
}
