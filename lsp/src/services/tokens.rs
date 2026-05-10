// Semantic tokens service for t27 Language Server

use crate::types::Document;
use crate::types::position::to_lsp_position;
use tower_lsp::lsp_types::{
    Position, Range, SemanticToken, SemanticTokens, SemanticTokensFullOptions,
    SemanticTokensLegend, SemanticTokensOptions, SemanticTokensParams,
    SemanticTokensResult, SemanticTokenType, SemanticTokenModifier,
    SemanticTokensServerCapabilities, ServerCapabilities,
};
use std::sync::Arc;

/// Semantic tokens service
pub struct SemanticTokensService {
    legend: SemanticTokensLegend,
}

impl SemanticTokensService {
    pub fn new() -> Self {
        // Define token types for t27 language
        // Note: CONSTANT is not available in this version of tower_lsp
        let token_types = vec![
            SemanticTokenType::FUNCTION,
            SemanticTokenType::VARIABLE,
            SemanticTokenType::TYPE,
            SemanticTokenType::KEYWORD,
            SemanticTokenType::STRING,
            SemanticTokenType::NUMBER,
            SemanticTokenType::COMMENT,
            SemanticTokenType::OPERATOR,
            SemanticTokenType::NAMESPACE,
        ];

        let token_modifiers = vec![
            SemanticTokenModifier::DOCUMENTATION,
            SemanticTokenModifier::DECLARATION,
            SemanticTokenModifier::DEFINITION,
            SemanticTokenModifier::READONLY,
            SemanticTokenModifier::STATIC,
            SemanticTokenModifier::DEPRECATED,
            SemanticTokenModifier::ABSTRACT,
            SemanticTokenModifier::ASYNC,
            SemanticTokenModifier::MODIFICATION,
            SemanticTokenModifier::DEFAULT_LIBRARY,
        ];

        Self {
            legend: SemanticTokensLegend {
                token_types,
                token_modifiers,
            },
        }
    }

    /// Get the legend for semantic tokens
    pub fn legend(&self) -> SemanticTokensLegend {
        self.legend.clone()
    }

    /// Register semantic tokens capability
    pub fn register_capability(&self) -> ServerCapabilities {
        ServerCapabilities {
            semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
                SemanticTokensOptions {
                    legend: self.legend(),
                    range: Some(false),
                    full: Some(SemanticTokensFullOptions::Bool(true)),
                    work_done_progress_options: Default::default(),
                },
            )),
            ..Default::default()
        }
    }

    /// Generate semantic tokens for a document
    pub fn full(&self, doc: &Document) -> Result<SemanticTokens, String> {
        let mut tokens = Vec::new();

        for (line_idx, line) in doc.text.lines().enumerate() {
            let mut pos = 0;

            while pos < line.len() {
                // Skip whitespace
                while pos < line.len() && line.chars().nth(pos).map_or(false, |c| c.is_whitespace()) {
                    pos += 1;
                }

                if pos >= line.len() {
                    break;
                }

                // Get the remaining part of the line
                let remaining = &line[pos..];

                // Check for comments
                if remaining.starts_with("//") {
                    let token = self.create_token(
                        line_idx,
                        pos,
                        line.len() - pos,
                        SemanticTokenType::COMMENT,
                        0,
                    );
                    tokens.push(token);
                    break;
                }

                // Check for strings
                if remaining.starts_with('"') {
                    if let Some(end) = self.find_string_end(&line[pos..]) {
                        let token = self.create_token(
                            line_idx,
                            pos,
                            end + 1,
                            SemanticTokenType::STRING,
                            0,
                        );
                        tokens.push(token);
                        pos += end + 1;
                        continue;
                    }
                }

                // Check for keywords
                let keywords = ["module", "import", "export", "pub", "const", "type", "fn",
                                "test", "invariant", "bench", "let", "return", "if", "else",
                                "match", "for", "while", "break", "continue", "true", "false"];

                let mut found_keyword = false;
                for keyword in &keywords {
                    if remaining.starts_with(keyword) {
                        // Check if it's followed by a non-alphanumeric character
                        let next_char = line.chars().nth(pos + keyword.len());
                        if next_char.map_or(true, |c| !c.is_alphanumeric() && c != '_') {
                            let token = self.create_token(
                                line_idx,
                                pos,
                                keyword.len(),
                                SemanticTokenType::KEYWORD,
                                0,
                            );
                            tokens.push(token);
                            pos += keyword.len();
                            found_keyword = true;
                            break;
                        }
                    }
                }

                if found_keyword {
                    continue;
                }

                // Check for numbers
                if let Some(num_len) = self.parse_number(&line[pos..]) {
                    let token = self.create_token(
                        line_idx,
                        pos,
                        num_len,
                        SemanticTokenType::NUMBER,
                        0,
                    );
                    tokens.push(token);
                    pos += num_len;
                    continue;
                }

                // Check for operators
                if let Some(op_len) = self.parse_operator(&line[pos..]) {
                    let token = self.create_token(
                        line_idx,
                        pos,
                        op_len,
                        SemanticTokenType::OPERATOR,
                        0,
                    );
                    tokens.push(token);
                    pos += op_len;
                    continue;
                }

                // Check for symbols (variables, functions, types)
                if let Some(ident_len) = self.parse_identifier(&line[pos..]) {
                    let identifier = &line[pos..pos + ident_len];

                    // Determine token type based on context
                    let token_type = self.classify_identifier(identifier);

                    let token = self.create_token(
                        line_idx,
                        pos,
                        ident_len,
                        token_type,
                        0,
                    );
                    tokens.push(token);
                    pos += ident_len;
                    continue;
                }

                // Unknown token, skip one character
                pos += 1;
            }
        }

        Ok(SemanticTokens {
            result_id: None,
            data: tokens,
        })
    }

    /// Find the end of a string literal
    fn find_string_end(&self, s: &str) -> Option<usize> {
        let mut pos = 1; // Skip opening quote
        let chars: Vec<char> = s.chars().collect();

        while pos < chars.len() {
            if chars[pos] == '"' && chars[pos - 1] != '\\' {
                return Some(pos);
            }
            pos += 1;
        }

        None // Unclosed string
    }

    /// Parse a number literal
    fn parse_number(&self, s: &str) -> Option<usize> {
        let mut pos = 0;
        let chars: Vec<char> = s.chars().collect();

        // Optional sign
        if pos < chars.len() && (chars[pos] == '-' || chars[pos] == '+') {
            pos += 1;
        }

        // Integer part
        while pos < chars.len() && chars[pos].is_ascii_digit() {
            pos += 1;
        }

        // Decimal point and fractional part
        if pos < chars.len() && chars[pos] == '.' {
            pos += 1;
            while pos < chars.len() && chars[pos].is_ascii_digit() {
                pos += 1;
            }
        }

        // Exponent
        if pos < chars.len() && (chars[pos] == 'e' || chars[pos] == 'E') {
            pos += 1;
            if pos < chars.len() && (chars[pos] == '+' || chars[pos] == '-') {
                pos += 1;
            }
            while pos < chars.len() && chars[pos].is_ascii_digit() {
                pos += 1;
            }
        }

        if pos > 0 {
            Some(pos)
        } else {
            None
        }
    }

    /// Parse an operator
    fn parse_operator(&self, s: &str) -> Option<usize> {
        let operators = [
            "->", "=>", "==", "!=", "<=", ">=", "&&", "||", "++", "--", "+=", "-=", "*=", "/=",
            "%=", "&=", "|=", "^=", "<<=", ">>=", ">>>=",
            "+", "-", "*", "/", "%", "=", "<", ">", "!", "&", "|", "^", "~", ":", ";", ",",
            "(", ")", "{", "}", "[", "]", ".", "?",
        ];

        for op in &operators {
            if s.starts_with(op) {
                return Some(op.len());
            }
        }

        None
    }

    /// Parse an identifier
    fn parse_identifier(&self, s: &str) -> Option<usize> {
        let chars: Vec<char> = s.chars().collect();

        if chars.is_empty() || !chars[0].is_alphabetic() && chars[0] != '_' {
            return None;
        }

        let mut pos = 1;
        while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
            pos += 1;
        }

        if pos > 0 {
            Some(pos)
        } else {
            None
        }
    }

    /// Classify an identifier based on its name
    fn classify_identifier(&self, identifier: &str) -> SemanticTokenType {
        // Check for built-in types
        let built_in_types = [
            "Int", "Int8", "Int16", "Int32", "Int64",
            "UInt", "UInt8", "UInt16", "UInt32", "UInt64",
            "Float", "Float32", "Float64",
            "Bool", "String", "Array", "Vec",
            "GF4", "GF8", "GF12", "GF16", "GF20", "GF24", "GF32",
            "Trit", "Trit4", "Trit8", "Trit12", "Trit16",
        ];

        // Check for constants (all caps) first
        if identifier.chars().all(|c| c.is_uppercase() || c == '_') && !identifier.is_empty() {
            // All caps is usually a constant - use VARIABLE since CONSTANT not available
            SemanticTokenType::VARIABLE
        } else if built_in_types.contains(&identifier) || identifier.chars().next().map_or(false, |c| c.is_uppercase()) {
            SemanticTokenType::TYPE
        } else {
            SemanticTokenType::VARIABLE
        }
    }

    /// Create a semantic token
    fn create_token(
        &self,
        line: usize,
        char_offset: usize,
        length: usize,
        token_type: SemanticTokenType,
        modifiers: u32,
    ) -> SemanticToken {
        SemanticToken {
            delta_line: 0,
            delta_start: char_offset as u32,
            length: length as u32,
            token_type: self.token_type_index(token_type) as u32,
            token_modifiers_bitset: modifiers,
        }
    }

    /// Get the index of a token type in the legend
    fn token_type_index(&self, token_type: SemanticTokenType) -> usize {
        self.legend
            .token_types
            .iter()
            .position(|t| *t == token_type)
            .unwrap_or(0)
    }
}

impl Default for SemanticTokensService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_identifier() {
        let service = SemanticTokensService::new();

        assert_eq!(service.parse_identifier("test"), Some(4));
        assert_eq!(service.parse_identifier("_test"), Some(5));
        assert_eq!(service.parse_identifier("test123"), Some(7));
        assert_eq!(service.parse_identifier("123test"), None);
    }

    #[test]
    fn test_parse_number() {
        let service = SemanticTokensService::new();

        assert_eq!(service.parse_number("123"), Some(3));
        assert_eq!(service.parse_number("123.456"), Some(7));
        assert_eq!(service.parse_number("-123"), Some(4));
        assert_eq!(service.parse_number("1.23e-10"), Some(8));
        assert_eq!(service.parse_number("abc"), None);
    }

    #[test]
    fn test_parse_operator() {
        let service = SemanticTokensService::new();

        assert_eq!(service.parse_operator("=="), Some(2));
        assert_eq!(service.parse_operator("+="), Some(2));
        assert_eq!(service.parse_operator("+"), Some(1));
        assert_eq!(service.parse_operator("abc"), None);
    }

    #[test]
    fn test_classify_identifier() {
        let service = SemanticTokensService::new();

        assert_eq!(service.classify_identifier("Int"), SemanticTokenType::TYPE);
        assert_eq!(service.classify_identifier("GF16"), SemanticTokenType::TYPE);
        assert_eq!(service.classify_identifier("MY_CONST"), SemanticTokenType::VARIABLE); // CONSTANT not available
        assert_eq!(service.classify_identifier("myVariable"), SemanticTokenType::VARIABLE);
    }
}

