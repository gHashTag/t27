//! t27 Language Server Protocol (LSP) Implementation
//!
//! Provides IDE support for t27 spec files including:
//! - Syntax highlighting
//! - Code completion
//! - Diagnostics
//! - Go-to-definition
//! - References
//! - Symbol search
//! - Semantic tokens
//! - Code actions

pub mod parser;
pub mod symbols;
pub mod completion;
pub mod diagnostics;
pub mod navigation;
pub mod hover;

use tower_lsp::LanguageServer;

pub use parser::{parse_t27_spec, T27SyntaxKind};
pub use symbols::{Symbol, SymbolKind};
pub use completion::{CompletionItem, CompletionContext};
pub use diagnostics::{Diagnostic, DiagnosticSeverity};
pub use navigation::{Location, Reference};
pub use hover::{HoverContent, HoverDocumentation};