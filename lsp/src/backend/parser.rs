// Parser interface for t27 Language Server

use crate::types::{Document, Symbol, SymbolKind};
use crate::types::position::{to_lsp_position, to_lsp_range};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, Url};
use std::path::Path;

/// Parser trait for t27 specification files
pub trait T27Parser {
    /// Parse text into AST and extract symbols
    fn parse(&self, text: &str, uri: &Url) -> ParseResult;

    /// Validate document and produce diagnostics
    fn validate(&self, doc: &Document) -> Vec<Diagnostic>;
}

/// Parse result containing symbols and diagnostics
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub symbols: Vec<Symbol>,
    pub diagnostics: Vec<Diagnostic>,
    pub is_valid: bool,
}

/// Default parser implementation
pub struct DefaultParser;

impl DefaultParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultParser {
    fn default() -> Self {
        Self::new()
    }
}

impl T27Parser for DefaultParser {
    fn parse(&self, text: &str, uri: &Url) -> ParseResult {
        let mut symbols = Vec::new();
        let mut diagnostics = Vec::new();

        // Simple regex-based extraction for now
        // TODO: Integrate with actual t27c parser

        let lines: Vec<&str> = text.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Extract module declarations
            if let Some(rest) = trimmed.strip_prefix("module ") {
                if let Some(name_end) = rest.find('{') {
                    let name = rest[..name_end].trim();
                    let range = to_lsp_range(line_idx, 0, line_idx, line.len());
                    symbols.push(
                        Symbol::new(name.to_string(), SymbolKind::Module, uri.clone(), range)
                            .with_detail("module declaration".to_string()),
                    );
                }
            }

            // Extract const declarations
            if let Some(rest) = trimmed.strip_prefix("const ") {
                if let Some(colon_pos) = rest.find(':') {
                    let name = rest[..colon_pos].trim();
                    let range = to_lsp_range(line_idx, 0, line_idx, line.len());
                    symbols.push(
                        Symbol::new(name.to_string(), SymbolKind::Constant, uri.clone(), range)
                            .with_detail("constant".to_string()),
                    );
                }
            }

            // Extract type declarations
            if let Some(rest) = trimmed.strip_prefix("type ") {
                if let Some(assign_pos) = rest.find('=') {
                    let name = rest[..assign_pos].trim();
                    let range = to_lsp_range(line_idx, 0, line_idx, line.len());
                    symbols.push(
                        Symbol::new(name.to_string(), SymbolKind::Type, uri.clone(), range)
                            .with_detail("type declaration".to_string()),
                    );
                }
            }

            // Extract function declarations
            if let Some(rest) = trimmed.strip_prefix("fn ") {
                if let Some(paren_pos) = rest.find('(') {
                    let name = rest[..paren_pos].trim();
                    let range = to_lsp_range(line_idx, 0, line_idx, line.len());
                    symbols.push(
                        Symbol::new(name.to_string(), SymbolKind::Function, uri.clone(), range)
                            .with_detail("function".to_string()),
                    );
                }
            }

            // Extract test blocks
            if let Some(rest) = trimmed.strip_prefix("test ") {
                if let Some(name_end) = rest.find('{') {
                    let name = rest[..name_end].trim();
                    let range = to_lsp_range(line_idx, 0, line_idx, line.len());
                    symbols.push(
                        Symbol::new(name.to_string(), SymbolKind::Test, uri.clone(), range)
                            .with_detail("test block".to_string()),
                    );
                }
            }

            // Extract invariant blocks
            if let Some(rest) = trimmed.strip_prefix("invariant ") {
                if let Some(name_end) = rest.find('{') {
                    let name = rest[..name_end].trim();
                    let range = to_lsp_range(line_idx, 0, line_idx, line.len());
                    symbols.push(
                        Symbol::new(name.to_string(), SymbolKind::Invariant, uri.clone(), range)
                            .with_detail("invariant".to_string()),
                    );
                }
            }

            // Extract bench blocks
            if let Some(rest) = trimmed.strip_prefix("bench ") {
                if let Some(name_end) = rest.find('{') {
                    let name = rest[..name_end].trim();
                    let range = to_lsp_range(line_idx, 0, line_idx, line.len());
                    symbols.push(
                        Symbol::new(name.to_string(), SymbolKind::Bench, uri.clone(), range)
                            .with_detail("benchmark".to_string()),
                    );
                }
            }

            // Extract import statements
            if let Some(rest) = trimmed.strip_prefix("import ") {
                let import_path = rest.trim().trim_matches(';');
                let range = to_lsp_range(line_idx, 0, line_idx, line.len());
                symbols.push(
                    Symbol::new(
                        import_path.to_string(),
                        SymbolKind::Import,
                        uri.clone(),
                        range,
                    )
                    .with_detail("import".to_string()),
                );
            }

            // Extract let bindings (variables)
            if let Some(rest) = trimmed.strip_prefix("let ") {
                if let Some(colon_pos) = rest.find(':') {
                    let name = rest[..colon_pos].trim();
                    let range = to_lsp_range(line_idx, 0, line_idx, line.len());
                    symbols.push(
                        Symbol::new(name.to_string(), SymbolKind::Variable, uri.clone(), range)
                            .with_detail("variable".to_string()),
                    );
                }
            }
        }

        // Basic syntax validation
        for (line_idx, line) in lines.iter().enumerate() {
            // Check for unmatched braces
            let open_braces = line.matches('{').count();
            let close_braces = line.matches('}').count();
            if open_braces < close_braces {
                diagnostics.push(Diagnostic {
                    range: to_lsp_range(line_idx, 0, line_idx, line.len()),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: "Unmatched closing brace".to_string(),
                    ..Default::default()
                });
            }

            // Check for missing semicolons (simple heuristic)
            let trimmed = line.trim();
            if !trimmed.is_empty()
                && !trimmed.starts_with("//")
                && !trimmed.ends_with('{')
                && !trimmed.ends_with('}')
                && !trimmed.ends_with(';')
                && trimmed.contains("const ")
                && !trimmed.contains("//")
            {
                // This is a basic check - might have false positives
                // diagnostics.push(Diagnostic {
                //     range: to_lsp_range(line_idx, 0, line_idx, line.len()),
                //     severity: Some(DiagnosticSeverity::WARNING),
                //     message: "Statement might be missing a semicolon".to_string(),
                //     ..Default::default()
                // });
            }
        }

        ParseResult {
            is_valid: diagnostics.is_empty(),
            symbols,
            diagnostics,
        }
    }

    fn validate(&self, doc: &Document) -> Vec<Diagnostic> {
        let result = self.parse(&doc.text, &doc.uri);
        result.diagnostics
    }
}

/// Extract symbols from document
pub fn extract_symbols(doc: &Document) -> Vec<Symbol> {
    let parser = DefaultParser::new();
    parser.parse(&doc.text, &doc.uri).symbols
}

/// Validate document and return diagnostics
pub fn validate_document(doc: &Document) -> Vec<Diagnostic> {
    let parser = DefaultParser::new();
    parser.validate(doc)
}

/// Check if a file is a t27 specification file
pub fn is_t27_file(uri: &Url) -> bool {
    uri.path()
        .to_lowercase()
        .ends_with(".t27")
}

/// Check if a file is a tri specification file
pub fn is_tri_file(uri: &Url) -> bool {
    uri.path()
        .to_lowercase()
        .ends_with(".tri")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_module() {
        let text = "module test {\n}";
        let uri = Url::parse("file:///test.t27").unwrap();
        let parser = DefaultParser::new();
        let result = parser.parse(text, &uri);

        assert_eq!(result.symbols.len(), 1);
        assert_eq!(result.symbols[0].name, "test");
        assert_eq!(result.symbols[0].kind, SymbolKind::Module);
    }

    #[test]
    fn test_parse_const() {
        let text = "const PHI: GF16 = 1.618;";
        let uri = Url::parse("file:///test.t27").unwrap();
        let parser = DefaultParser::new();
        let result = parser.parse(text, &uri);

        assert_eq!(result.symbols.len(), 1);
        assert_eq!(result.symbols[0].name, "PHI");
        assert_eq!(result.symbols[0].kind, SymbolKind::Constant);
    }

    #[test]
    fn test_parse_function() {
        let text = "fn test(a: Int) -> Int {\n    a\n}";
        let uri = Url::parse("file:///test.t27").unwrap();
        let parser = DefaultParser::new();
        let result = parser.parse(text, &uri);

        assert_eq!(result.symbols.len(), 1);
        assert_eq!(result.symbols[0].name, "test");
        assert_eq!(result.symbols[0].kind, SymbolKind::Function);
    }

    #[test]
    fn test_is_t27_file() {
        let uri = Url::parse("file:///test.t27").unwrap();
        assert!(is_t27_file(&uri));
    }

    #[test]
    fn test_is_tri_file() {
        let uri = Url::parse("file:///test.tri").unwrap();
        assert!(is_tri_file(&uri));
    }
}
