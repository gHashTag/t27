//! Navigation support for t27 (go-to-definition, references)

use tower_lsp::lsp_types::*;
use std::collections::HashMap;
use std::path::Path;

/// Location of a symbol
pub struct Location {
    pub uri: String,
    pub range: Range,
}

impl Location {
    pub fn new(uri: String, range: Range) -> Self {
        Self { uri, range }
    }

    pub fn to_lsp(&self) -> lsp_types::Location {
        lsp_types::Location {
            uri: Url::parse(&self.uri).unwrap_or_else(|_| Url::parse("file://").unwrap()),
            range: convert_range(&self.range),
        }
    }
}

/// Reference to a symbol
pub struct Reference {
    pub location: Location,
    pub is_definition: bool,
}

/// Symbol reference store
pub struct ReferenceStore {
    definitions: HashMap<String, Vec<Location>>,
    references: HashMap<String, Vec<Reference>>,
}

impl ReferenceStore {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            references: HashMap::new(),
        }
    }

    /// Add a symbol definition
    pub fn add_definition(&mut self, name: String, uri: String, range: Range) {
        self.definitions
            .entry(name.clone())
            .or_insert_with(Vec::new)
            .push(Location::new(uri, range));
    }

    /// Add a symbol reference
    pub fn add_reference(&mut self, name: String, uri: String, range: Range) {
        self.references
            .entry(name.clone())
            .or_insert_with(Vec::new)
            .push(Reference {
                location: Location::new(uri, range),
                is_definition: false,
            });
    }

    /// Find definition for a symbol
    pub fn find_definition(&self, name: &str) -> Option<&Location> {
        self.definitions.get(name).and_then(|locs| locs.first())
    }

    /// Find all references to a symbol
    pub fn find_references(&self, name: &str) -> Vec<Location> {
        let mut refs = Vec::new();

        // Add definition as a reference
        if let Some(def) = self.find_definition(name) {
            refs.push(def.clone());
        }

        // Add all references
        if let Some(refs) = self.references.get(name) {
            for r in refs {
                refs.push(r.location.clone());
            }
        }

        refs
    }

    /// Update reference store from a parsed spec
    pub fn update_from_spec(&mut self, uri: &str, symbols: &[crate::parser::Symbol]) {
        // Extract symbols and their ranges
        for symbol in symbols {
            self.add_definition(
                symbol.name.clone(),
                uri.to_string(),
                symbol.range.clone(),
            );
        }
    }

    /// Update reference store from content (find all identifiers)
    pub fn update_from_content(&mut self, uri: &str, content: &str) {
        // Parse content and find all identifiers
        let mut lines = content.lines().enumerate();
        let mut in_comment = false;
        let mut in_string = false;

        while let Some((line_num, line)) = lines.next() {
            let mut chars = line.chars().peekable();

            while let Some(&ch) = chars.peek() {
                match ch {
                    '/' if !in_string && !in_comment => {
                        chars.next();
                        if let Some(&'/') = chars.peek() {
                            chars.next();
                            in_comment = true;
                        }
                    }
                    '\n' | '\r' => break,
                    '"' => {
                        chars.next();
                        if !in_comment {
                            in_string = !in_string;
                        }
                    }
                    _ if in_string || in_comment => {
                        chars.next();
                    }
                    'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                        let mut ident = String::new();
                        while let Some(&c) = chars.peek() {
                            if c.is_alphanumeric() || c == '_' {
                                ident.push(chars.next().unwrap());
                            } else {
                                break;
                            }
                        }

                        // Skip keywords
                        let is_keyword = matches!(
                            ident.as_str(),
                            "module" | "fn" | "const" | "let" | "if" | "else" | "return" |
                            "test" | "invariant" | "bench" | "given" | "then" | "expect" |
                            "phi" | "gf4" | "gf8" | "gf12" | "gf16" | "gf20" | "gf24" | "gf32" |
                            "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" |
                            "f32" | "f64" | "bool" | "str"
                        );

                        if !is_keyword && !ident.is_empty() {
                            let char_pos = line.find(&ident).unwrap_or(0);
                            let start = Position::new(line_num as u32, char_pos as u32);
                            let end = Position::new(line_num as u32, (char_pos + ident.len()) as u32);

                            self.add_reference(
                                ident.clone(),
                                uri.to_string(),
                                Range::new(start, end),
                            );
                        }
                    }
                    _ => {
                        chars.next();
                    }
                }
            }
        }
    }
}

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