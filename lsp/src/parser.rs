//! t27 Syntax Parser
//!
//! Parses .t27 spec files and extracts syntactic information.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Syntax kind for t27 tokens
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum T27SyntaxKind {
    /// Keywords
    Module,
    Fn,
    Const,
    Let,
    If,
    Else,
    Return,
    Test,
    Invariant,
    Bench,
    Given,
    Then,
    Expect,

    /// Types
    Phi,
    Gf4,
    Gf8,
    Gf12,
    Gf16,
    Gf20,
    Gf24,
    Gf32,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Bool,
    Str,
    Vec,
    Array,
    Option,
    Result,

    /// Literals
    StringLiteral,
    NumberLiteral,
    BooleanLiteral,
    PhiLiteral,

    /// Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Less,
    Greater,
    Equal,
    BangEqual,
    LessEqual,
    GreaterEqual,
    And,
    Or,
    Not,

    /// Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    LeftAngle,
    RightAngle,
    Comma,
    Colon,
    Semicolon,
    Dot,
    Arrow,
    FatArrow,
    DoubleColon,

    /// Comments
    LineComment,
    BlockComment,

    /// Other
    Identifier,
    Whitespace,
    Unknown,
}

/// Parsed token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub kind: T27SyntaxKind,
    pub text: String,
    pub start: Position,
    pub end: Position,
}

/// Position in a document
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl Position {
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }
}

/// Range in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }
}

/// Parse result for a t27 spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSpec {
    pub uri: String,
    pub tokens: Vec<Token>,
    pub symbols: Vec<Symbol>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Symbol in the spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub range: Range,
    pub detail: Option<String>,
}

/// Symbol kind
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolKind {
    Module,
    Function,
    Const,
    Variable,
    Type,
    Test,
    Invariant,
    Benchmark,
}

/// Diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Parse a t27 spec file
pub fn parse_t27_spec(uri: &str, content: &str) -> ParsedSpec {
    let tokens = tokenize(content);
    let symbols = extract_symbols(&tokens);
    let diagnostics = validate_syntax(&tokens);

    ParsedSpec {
        uri: uri.to_string(),
        tokens,
        symbols,
        diagnostics,
    }
}

/// Tokenize content into tokens
fn tokenize(content: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut line = 0u32;
    let mut character = 0u32;
    let mut chars = content.chars().peekable();

    while let Some(&ch) = chars.peek() {
        let start = Position::new(line, character);

        match ch {
            // Whitespace
            ' ' | '\t' => {
                chars.next();
                character += 1;
                tokens.push(Token {
                    kind: T27SyntaxKind::Whitespace,
                    text: ch.to_string(),
                    start,
                    end: Position::new(line, character),
                });
            }
            '\n' => {
                chars.next();
                line += 1;
                character = 0;
                tokens.push(Token {
                    kind: T27SyntaxKind::Whitespace,
                    text: "\n".to_string(),
                    start,
                    end: Position::new(line, character),
                });
            }
            // Comments
            '/' => {
                chars.next();
                character += 1;
                if let Some(&'/') = chars.peek() {
                    chars.next();
                    character += 1;
                    let mut comment = "//".to_string();
                    while let Some(&c) = chars.peek() {
                        if c == '\n' {
                            break;
                        }
                        comment.push(chars.next().unwrap());
                        character += 1;
                    }
                    tokens.push(Token {
                        kind: T27SyntaxKind::LineComment,
                        text: comment,
                        start,
                        end: Position::new(line, character),
                    });
                }
            }
            // Strings
            '"' => {
                chars.next();
                character += 1;
                let mut string = '"'.to_string();
                let mut escaped = false;
                while let Some(&c) = chars.peek() {
                    if escaped {
                        string.push(chars.next().unwrap());
                        character += 1;
                        escaped = false;
                        continue;
                    }
                    if c == '\\' {
                        string.push(chars.next().unwrap());
                        character += 1;
                        escaped = true;
                        continue;
                    }
                    if c == '"' {
                        string.push(chars.next().unwrap());
                        character += 1;
                        break;
                    }
                    string.push(chars.next().unwrap());
                    character += 1;
                }
                tokens.push(Token {
                    kind: T27SyntaxKind::StringLiteral,
                    text: string,
                    start,
                    end: Position::new(line, character),
                });
            }
            // Numbers
            '0'..='9' => {
                let mut num = String::new();
                let mut has_dot = false;
                while let Some(&c) = chars.peek() {
                    if c.is_digit(10) {
                        num.push(chars.next().unwrap());
                        character += 1;
                    } else if c == '.' && !has_dot {
                        num.push(chars.next().unwrap());
                        character += 1;
                        has_dot = true;
                    } else {
                        break;
                    }
                }
                tokens.push(Token {
                    kind: if has_dot {
                        T27SyntaxKind::NumberLiteral
                    } else {
                        T27SyntaxKind::PhiLiteral
                    },
                    text: num,
                    start,
                    end: Position::new(line, character),
                });
            }
            // Identifiers and keywords
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(chars.next().unwrap());
                        character += 1;
                    } else {
                        break;
                    }
                }
                let kind = match ident.as_str() {
                    "module" => T27SyntaxKind::Module,
                    "fn" => T27SyntaxKind::Fn,
                    "const" => T27SyntaxKind::Const,
                    "let" => T27SyntaxKind::Let,
                    "if" => T27SyntaxKind::If,
                    "else" => T27SyntaxKind::Else,
                    "return" => T27SyntaxKind::Return,
                    "test" => T27SyntaxKind::Test,
                    "invariant" => T27SyntaxKind::Invariant,
                    "bench" => T27SyntaxKind::Bench,
                    "given" => T27SyntaxKind::Given,
                    "then" => T27SyntaxKind::Then,
                    "expect" => T27SyntaxKind::Expect,
                    "phi" => T27SyntaxKind::Phi,
                    "gf4" => T27SyntaxKind::Gf4,
                    "gf8" => T27SyntaxKind::Gf8,
                    "gf12" => T27SyntaxKind::Gf12,
                    "gf16" => T27SyntaxKind::Gf16,
                    "gf20" => T27SyntaxKind::Gf20,
                    "gf24" => T27SyntaxKind::Gf24,
                    "gf32" => T27SyntaxKind::Gf32,
                    "u8" => T27SyntaxKind::U8,
                    "u16" => T27SyntaxKind::U16,
                    "u32" => T27SyntaxKind::U32,
                    "u64" => T27SyntaxKind::U64,
                    "i8" => T27SyntaxKind::I8,
                    "i16" => T27SyntaxKind::I16,
                    "i32" => T27SyntaxKind::I32,
                    "i64" => T27SyntaxKind::I64,
                    "f32" => T27SyntaxKind::F32,
                    "f64" => T27SyntaxKind::F64,
                    "bool" => T27SyntaxKind::Bool,
                    "str" => T27SyntaxKind::Str,
                    "vec" => T27SyntaxKind::Vec,
                    "array" => T27SyntaxKind::Array,
                    "option" => T27SyntaxKind::Option,
                    "result" => T27SyntaxKind::Result,
                    "true" | "false" => T27SyntaxKind::BooleanLiteral,
                    _ => T27SyntaxKind::Identifier,
                };
                tokens.push(Token {
                    kind,
                    text: ident,
                    start,
                    end: Position::new(line, character),
                });
            }
            // Operators and delimiters
            '+' => {
                chars.next();
                character += 1;
                tokens.push(Token {
                    kind: T27SyntaxKind::Plus,
                    text: "+".to_string(),
                    start,
                    end: Position::new(line, character),
                });
            }
            '-' => {
                chars.next();
                character += 1;
                if let Some(&'>') = chars.peek() {
                    chars.next();
                    character += 1;
                    tokens.push(Token {
                        kind: T27SyntaxKind::Arrow,
                        text: "->".to_string(),
                        start,
                        end: Position::new(line, character),
                    });
                } else {
                    tokens.push(Token {
                        kind: T27SyntaxKind::Minus,
                        text: "-".to_string(),
                        start,
                        end: Position::new(line, character),
                    });
                }
            }
            '*' => {
                chars.next();
                character += 1;
                tokens.push(Token {
                    kind: T27SyntaxKind::Star,
                    text: "*".to_string(),
                    start,
                    end: Position::new(line, character),
                });
            }
            '/' => {
                chars.next();
                character += 1;
                tokens.push(Token {
                    kind: T27SyntaxKind::Slash,
                    text: "/".to_string(),
                    start,
                    end: Position::new(line, character),
                });
            }
            '%' => {
                chars.next();
                character += 1;
                tokens.push(Token {
                    kind: T27SyntaxKind::Percent,
                    text: "%".to_string(),
                    start,
                    end: Position::new(line, character),
                });
            }
            '^' => {
                chars.next();
                character += 1;
                tokens.push(Token {
                    kind: T27SyntaxKind::Caret,
                    text: "^".to_string(),
                    start,
                    end: Position::new(line, character),
                });
            }
            '<' => {
                chars.next();
                character += 1;
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    character += 1;
                    tokens.push(Token {
                        kind: T27SyntaxKind::LessEqual,
                        text: "<=".to_string(),
                        start,
                        end: Position::new(line, character),
                    });
                } else {
                    tokens.push(Token {
                        kind: T27SyntaxKind::Less,
                        text: "<".to_string(),
                        start,
                        end: Position::new(line, character),
                    });
                }
            }
            '>' => {
                chars.next();
                character += 1;
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    character += 1;
                    tokens.push(Token {
                        kind: T27SyntaxKind::GreaterEqual,
                        text: ">=".to_string(),
                        start,
                        end: Position::new(line, character),
                    });
                } else {
                    tokens.push(Token {
                        kind: T27SyntaxKind::Greater,
                        text: ">".to_string(),
                        start,
                        end: Position::new(line, character),
                    });
                }
            }
            '=' => {
                chars.next();
                character += 1;
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    character += 1;
                    tokens.push(Token {
                        kind: T27SyntaxKind::Equal,
                        text: "==".to_string(),
                        start,
                        end: Position::new(line, character),
                    });
                } else {
                    tokens.push(Token {
                        kind: T27SyntaxKind::Colon,
                        text: "=".to_string(),
                        start,
                        end: Position::new(line, character),
                    });
                }
            }
            '!' => {
                chars.next();
                character += 1;
                if let Some(&'=') = chars.peek() {
                    chars.next();
                    character += 1;
                    tokens.push(Token {
                        kind: T27SyntaxKind::BangEqual,
                        text: "!=".to_string(),
                        start,
                        end: Position::new(line, character),
                    });
                } else {
                    tokens.push(Token {
                        kind: T27SyntaxKind::Not,
                        text: "!".to_string(),
                        start,
                        end: Position::new(line, character),
                    });
                }
            }
            '&' => {
                chars.next();
                character += 1;
                if let Some(&'&') = chars.peek() {
                    chars.next();
                    character += 1;
                    tokens.push(Token {
                        kind: T27SyntaxKind::And,
                        text: "&&".to_string(),
                        start,
                        end: Position::new(line, character),
                    });
                }
            }
            '|' => {
                chars.next();
                character += 1;
                if let Some(&'|') = chars.peek() {
                    chars.next();
                    character += 1;
                    tokens.push(Token {
                        kind: T27SyntaxKind::Or,
                        text: "||".to_string(),
                        start,
                        end: Position::new(line, character),
                    });
                }
            }
            '(' => {
                chars.next();
                character += 1;
                tokens.push(Token {
                    kind: T27SyntaxKind::LeftParen,
                    text: "(".to_string(),
                    start,
                    end: Position::new(line, character),
                });
            }
            ')' => {
                chars.next();
                character += 1;
                tokens.push(Token {
                    kind: T27SyntaxKind::RightParen,
                    text: ")".to_string(),
                    start,
                    end: Position::new(line, character),
                });
            }
            '{' => {
                chars.next();
                character += 1;
                tokens.push(Token {
                    kind: T27SyntaxKind::LeftBrace,
                    text: "{".to_string(),
                    start,
                    end: Position::new(line, character),
                });
            }
            '}' => {
                chars.next();
                character += 1;
                tokens.push(Token {
                    kind: T27SyntaxKind::RightBrace,
                    text: "}".to_string(),
                    start,
                    end: Position::new(line, character),
                });
            }
            '[' => {
                chars.next();
                character += 1;
                tokens.push(Token {
                    kind: T27SyntaxKind::LeftBracket,
                    text: "[".to_string(),
                    start,
                    end: Position::new(line, character),
                });
            }
            ']' => {
                chars.next();
                character += 1;
                tokens.push(Token {
                    kind: T27SyntaxKind::RightBracket,
                    text: "]".to_string(),
                    start,
                    end: Position::new(line, character),
                });
            }
            ':' => {
                chars.next();
                character += 1;
                if let Some(&':') = chars.peek() {
                    chars.next();
                    character += 1;
                    tokens.push(Token {
                        kind: T27SyntaxKind::DoubleColon,
                        text: "::".to_string(),
                        start,
                        end: Position::new(line, character),
                    });
                } else {
                    tokens.push(Token {
                        kind: T27SyntaxKind::Colon,
                        text: ":".to_string(),
                        start,
                        end: Position::new(line, character),
                    });
                }
            }
            ';' => {
                chars.next();
                character += 1;
                tokens.push(Token {
                    kind: T27SyntaxKind::Semicolon,
                    text: ";".to_string(),
                    start,
                    end: Position::new(line, character),
                });
            }
            ',' => {
                chars.next();
                character += 1;
                tokens.push(Token {
                    kind: T27SyntaxKind::Comma,
                    text: ",".to_string(),
                    start,
                    end: Position::new(line, character),
                });
            }
            '.' => {
                chars.next();
                character += 1;
                tokens.push(Token {
                    kind: T27SyntaxKind::Dot,
                    text: ".".to_string(),
                    start,
                    end: Position::new(line, character),
                });
            }
            _ => {
                chars.next();
                character += 1;
                tokens.push(Token {
                    kind: T27SyntaxKind::Unknown,
                    text: ch.to_string(),
                    start,
                    end: Position::new(line, character),
                });
            }
        }
    }

    tokens
}

/// Extract symbols from tokens
fn extract_symbols(tokens: &[Token]) -> Vec<Symbol> {
    let mut symbols = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if tokens[i].kind == T27SyntaxKind::Module && i + 1 < tokens.len() {
            if tokens[i + 1].kind == T27SyntaxKind::Identifier {
                symbols.push(Symbol {
                    name: tokens[i + 1].text.clone(),
                    kind: SymbolKind::Module,
                    range: Range::new(tokens[i].start, tokens[i + 1].end),
                    detail: Some("Module definition".to_string()),
                });
                i += 2;
                continue;
            }
        }

        if tokens[i].kind == T27SyntaxKind::Fn && i + 1 < tokens.len() {
            if tokens[i + 1].kind == T27SyntaxKind::Identifier {
                symbols.push(Symbol {
                    name: tokens[i + 1].text.clone(),
                    kind: SymbolKind::Function,
                    range: Range::new(tokens[i].start, tokens[i + 1].end),
                    detail: Some("Function definition".to_string()),
                });
                i += 2;
                continue;
            }
        }

        if tokens[i].kind == T27SyntaxKind::Const && i + 2 < tokens.len() {
            if tokens[i + 1].kind == T27SyntaxKind::Identifier && tokens[i + 2].kind == T27SyntaxKind::Colon {
                symbols.push(Symbol {
                    name: tokens[i + 1].text.clone(),
                    kind: SymbolKind::Const,
                    range: Range::new(tokens[i].start, tokens[i + 2].end),
                    detail: extract_type_annotation(tokens, i + 3),
                });
                i += 3;
                continue;
            }
        }

        if tokens[i].kind == T27SyntaxKind::Test {
            // Find next identifier as test name
            let mut j = i + 1;
            while j < tokens.len() {
                if tokens[j].kind == T27SyntaxKind::Identifier {
                    symbols.push(Symbol {
                        name: tokens[j].text.clone(),
                        kind: SymbolKind::Test,
                        range: Range::new(tokens[i].start, tokens[j].end),
                        detail: None,
                    });
                    break;
                }
                j += 1;
            }
            i = j.max(i + 1);
            continue;
        }

        i += 1;
    }

    symbols
}

/// Extract type annotation from tokens
fn extract_type_annotation(tokens: &[Token], start: usize) -> Option<String> {
    let mut types = Vec::new();
    let mut i = start;

    while i < tokens.len() {
        match tokens[i].kind {
            T27SyntaxKind::Phi => types.push("phi"),
            T27SyntaxKind::Gf16 => types.push("gf16"),
            T27SyntaxKind::Gf32 => types.push("gf32"),
            T27SyntaxKind::U8 => types.push("u8"),
            T27SyntaxKind::U32 => types.push("u32"),
            T27SyntaxKind::I32 => types.push("i32"),
            T27SyntaxKind::F32 => types.push("f32"),
            T27SyntaxKind::F64 => types.push("f64"),
            T27SyntaxKind::Bool => types.push("bool"),
            T27SyntaxKind::Str => types.push("str"),
            T27SyntaxKind::LeftBracket => {
                // Array type
                let mut array_type = String::new();
                let mut depth = 0;
                while i < tokens.len() {
                    array_type.push_str(&tokens[i].text);
                    if tokens[i].kind == T27SyntaxKind::LeftBracket {
                        depth += 1;
                    } else if tokens[i].kind == T27SyntaxKind::RightBracket {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    i += 1;
                }
                types.push(&array_type);
            }
            _ => break,
        }
        i += 1;
    }

    if types.is_empty() {
        None
    } else {
        Some(types.join(" | "))
    }
}

/// Validate syntax and generate diagnostics
fn validate_syntax(tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut paren_depth = 0;
    let mut brace_depth = 0;
    let mut bracket_depth = 0;

    for (i, token) in tokens.iter().enumerate() {
        match token.kind {
            T27SyntaxKind::LeftParen => {
                paren_depth += 1;
                if paren_depth > 10 {
                    diagnostics.push(Diagnostic {
                        range: Range::new(token.start, token.end),
                        severity: DiagnosticSeverity::Warning,
                        message: "Deeply nested parentheses (depth > 10)".to_string(),
                        code: Some("PAREN_DEPTH".to_string()),
                    });
                }
            }
            T27SyntaxKind::RightParen => {
                paren_depth -= 1;
                if paren_depth < 0 {
                    diagnostics.push(Diagnostic {
                        range: Range::new(token.start, token.end),
                        severity: DiagnosticSeverity::Error,
                        message: "Unmatched closing parenthesis".to_string(),
                        code: Some("UNMATCHED_PAREN".to_string()),
                    });
                }
            }
            T27SyntaxKind::LeftBrace => {
                brace_depth += 1;
                if brace_depth > 10 {
                    diagnostics.push(Diagnostic {
                        range: Range::new(token.start, token.end),
                        severity: DiagnosticSeverity::Warning,
                        message: "Deeply nested braces (depth > 10)".to_string(),
                        code: Some("BRACE_DEPTH".to_string()),
                    });
                }
            }
            T27SyntaxKind::RightBrace => {
                brace_depth -= 1;
                if brace_depth < 0 {
                    diagnostics.push(Diagnostic {
                        range: Range::new(token.start, token.end),
                        severity: DiagnosticSeverity::Error,
                        message: "Unmatched closing brace".to_string(),
                        code: Some("UNMATCHED_BRACE".to_string()),
                    });
                }
            }
            T27SyntaxKind::LeftBracket => {
                bracket_depth += 1;
            }
            T27SyntaxKind::RightBracket => {
                bracket_depth -= 1;
                if bracket_depth < 0 {
                    diagnostics.push(Diagnostic {
                        range: Range::new(token.start, token.end),
                        severity: DiagnosticSeverity::Error,
                        message: "Unmatched closing bracket".to_string(),
                        code: Some("UNMATCHED_BRACKET".to_string()),
                    });
                }
            }
            _ => {}
        }

        // Check for phi identity (φ² + 1/φ² = 3)
        if i + 6 < tokens.len() {
            if tokens[i].text == "phi"
                && tokens[i + 1].kind == T27SyntaxKind::Star
                && tokens[i + 2].text == "2"
                && tokens[i + 3].kind == T27SyntaxKind::Plus
                && tokens[i + 4].kind == T27SyntaxKind::Digit
                && tokens[i + 5].text == "/phi"
                && tokens[i + 6].kind == T27SyntaxKind::Star
                && tokens[i + 7].text == "2"
                && tokens[i + 8].kind == T27SyntaxKind::Equal
            {
                diagnostics.push(Diagnostic {
                    range: Range::new(tokens[i].start, tokens[i + 8].end),
                    severity: DiagnosticSeverity::Hint,
                    message: "Trinity Identity: φ² + φ⁻² = 3 (exact in f64 precision)".to_string(),
                    code: Some("TRINITY_IDENTITY".to_string()),
                });
            }
        }
    }

    // Check for unmatched delimiters
    if paren_depth > 0 {
        diagnostics.push(Diagnostic {
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(tokens.last().map(|t| t.end.line).unwrap_or(0), 0),
            },
            severity: DiagnosticSeverity::Error,
            message: format!("{} unclosed parentheses", paren_depth),
            code: Some("UNCLOSED_PAREN".to_string()),
        });
    }

    if brace_depth > 0 {
        diagnostics.push(Diagnostic {
            range: Range {
                start: Position::new(0, 0),
                end: Position::new(tokens.last().map(|t| t.end.line).unwrap_or(0), 0),
            },
            severity: DiagnosticSeverity::Error,
            message: format!("{} unclosed braces", brace_depth),
            code: Some("UNCLOSED_BRACE".to_string()),
        });
    }

    diagnostics
}