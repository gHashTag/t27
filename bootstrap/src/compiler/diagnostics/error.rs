//! Compiler errors
//!
//! Defines all compiler error types for structured error reporting.
//! Part of modular compiler architecture (Ring-018).

use std::fmt;

/// Compiler error with source location
#[derive(Debug, Clone)]
pub struct CompilerError {
    pub kind: ErrorKind,
    pub message: String,
    pub span: Option<Span>,
}

/// Error kind (categorizes errors)
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// Syntax error in parsing
    Syntax(String),

    /// Type checking error
    TypeError(String),

    /// Name resolution error
    NameError(String),

    /// Semantic error
    Semantic(String),

    /// Lowering error
    Lowering(String),

    /// Code generation error
    Codegen(String),
}

impl std::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CompilerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(self)
    }

    fn description(&self) -> String {
        self.message.clone()
    }
}

/// Diagnostic for IDE integration
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
    Note,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Diagnostic {
            level: DiagnosticLevel::Error,
            message: message.into(),
            span: None,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Diagnostic {
            level: DiagnosticLevel::Warning,
            message: message.into(),
            span: None,
        }
    }
}
