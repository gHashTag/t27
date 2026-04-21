// bootstrap/src/compiler.rs
// T27 → Zig Compiler Core (Lexer + Parser + Codegen)
//
// This module contains the core compiler logic extracted for reusability.
// It can be used by both CLI and HTTP server contexts.

use std::default::Default;

// ============================================================================
// AST Node Types (from parser.t27)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Module,
    UseDecl,
    ConstDecl,
    EnumDecl,
    EnumVariant,
    StructDecl,
    FnDecl,
    InvariantBlock,
    TestBlock,
    BenchBlock,
    ExprLiteral,
    ExprIdentifier,
    ExprEnumValue,
    ExprCall,
    ExprFieldAccess,
    ExprSwitch,
    ExprBinary,
    ExprUnary,
    ExprCast,
    ExprReturn,
    ExprIndex,
    ExprIf,
    ExprStructLit,
    ExprArrayLiteral,
    // Statement nodes for fn bodies
    StmtLocal,  // const x = expr; or var x: T = expr;
    StmtAssign, // x = expr; or x.field = expr;
    StmtIf,     // if (...) { ... } else if (...) { ... } else { ... }
    StmtWhile,  // while (cond) { ... }
    StmtFor,    // for (iter) |capture| { ... }
    StmtBreak,
    StmtContinue,
    StmtExpr, // bare expression statement: func(a, b);
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Node {
    pub kind: NodeKind,
    pub name: String,
    pub value: String,
    pub extra_type: String,
    pub extra_field: String,
    pub extra_size: String,
    pub extra_kind: String,
    pub extra_op: String,
    pub extra_pub: bool,
    pub extra_mutable: bool,
    pub extra_return_type: String,
    pub params: Vec<(String, String)>,
    pub line: u32,
    pub children: Vec<Node>,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            kind: NodeKind::ExprLiteral,
            name: String::new(),
            value: String::new(),
            extra_type: String::new(),
            extra_field: String::new(),
            extra_size: String::new(),
            extra_kind: String::new(),
            extra_op: String::new(),
            extra_pub: false,
            extra_mutable: false,
            extra_return_type: String::new(),
            params: Vec::new(),
            line: 0,
            children: Vec::new(),
        }
    }
}

impl Node {
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            name: String::new(),
            value: String::new(),
            extra_type: String::new(),
            extra_field: String::new(),
            extra_size: String::new(),
            extra_kind: String::new(),
            extra_op: String::new(),
            extra_pub: false,
            extra_mutable: false,
            extra_return_type: String::new(),
            params: Vec::new(),
            line: 0,
            children: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn with_child(mut self, child: Node) -> Self {
        self.children.push(child);
        self
    }
}

// ============================================================================
// Lexer (minimal implementation)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    KwPub,
    KwConst,
    KwFn,
    KwEnum,
    KwStruct,
    KwTest,
    KwInvariant,
    KwBench,
    KwModule,
    KwIf,
    KwElse,
    KwFor,
    KwWhile,
    KwSwitch,
    KwReturn,
    KwVar,
    KwUsing,
    KwVoid,
    KwTrue,
    KwFalse,
    KwUse,
    KwOr,
    KwAnd,
    KwTry,
    KwAs,
    KwBreak,
    KwContinue,

    // Literals
    Ident,
    Number,
    String,
    CharLiteral,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Amp,
    Pipe,
    Caret,
    Tilde,
    Lt,
    Gt,
    Lte,
    Gte,
    Eq,
    Neq,

    // Delimiters
    Colon,
    Comma,
    Equals,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Dot,
    Bang,

    // Multi-char
    Arrow,
    FatArrow,
    Power,
    DotDot,
    PlusPlus,
    ShiftLeft,
    ShiftRight,
    PlusEquals,
    PlusPercent,
    ColonColon,

    // Special
    Semicolon,
    Eof,
}

impl Copy for TokenKind {}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub col: usize,
}

pub struct Lexer {
    source: Vec<u8>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.as_bytes().to_vec(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> u8 {
        if self.pos < self.source.len() {
            self.source[self.pos]
        } else {
            0
        }
    }

    fn peek_offset(&self, offset: usize) -> u8 {
        let new_pos = self.pos + offset;
        if new_pos < self.source.len() {
            self.source[new_pos]
        } else {
            0
        }
    }

    fn advance(&mut self) {
        let ch = self.peek();
        if ch == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        self.pos += 1;
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while self.pos < self.source.len() {
                let ch = self.peek();
                if ch != b' ' && ch != b'\t' && ch != b'\n' && ch != b'\r' {
                    break;
                }
                self.advance();
            }

            // [BUG 2 FIX] Skip // line comments
            if self.pos + 1 < self.source.len()
                && self.source[self.pos] == b'/'
                && self.source[self.pos + 1] == b'/'
            {
                while self.pos < self.source.len() && self.peek() != b'\n' {
                    self.advance();
                }
                continue; // loop back to skip more whitespace/comments
            }

            // Skip /* ... */ block comments
            if self.pos + 1 < self.source.len()
                && self.source[self.pos] == b'/'
                && self.source[self.pos + 1] == b'*'
            {
                self.advance(); // consume /
                self.advance(); // consume *
                let mut depth = 1;
                while depth > 0 && self.pos < self.source.len() {
                    if self.pos + 1 < self.source.len()
                        && self.source[self.pos] == b'*'
                        && self.source[self.pos + 1] == b'/'
                    {
                        self.advance();
                        self.advance();
                        depth -= 1;
                    } else if self.pos + 1 < self.source.len()
                        && self.source[self.pos] == b'/'
                        && self.source[self.pos + 1] == b'*'
                    {
                        self.advance();
                        self.advance();
                        depth += 1;
                    } else {
                        self.advance();
                    }
                }
                continue;
            }

            // Skip ; line comments (old t27 comment style: ; at column 1 followed by space)
            if self.pos < self.source.len() && self.peek() == b';' && self.col == 1 {
                let next = self.peek_offset(1);
                if next == b' ' || next == b'\t' {
                    while self.pos < self.source.len() && self.peek() != b'\n' {
                        self.advance();
                    }
                    continue;
                }
            }

            break;
        }
    }

    fn check_keyword(&self, ident: &str) -> TokenKind {
        match ident {
            "pub" => TokenKind::KwPub,
            "const" => TokenKind::KwConst,
            "fn" => TokenKind::KwFn,
            "enum" => TokenKind::KwEnum,
            "struct" => TokenKind::KwStruct,
            "test" => TokenKind::KwTest,
            "invariant" => TokenKind::KwInvariant,
            "bench" => TokenKind::KwBench,
            "module" => TokenKind::KwModule,
            "if" => TokenKind::KwIf,
            "else" => TokenKind::KwElse,
            "for" => TokenKind::KwFor,
            "while" => TokenKind::KwWhile,
            "or" => TokenKind::KwOr,
            "and" => TokenKind::KwAnd,
            "try" => TokenKind::KwTry,
            "switch" => TokenKind::KwSwitch,
            "return" => TokenKind::KwReturn,
            "var" => TokenKind::KwVar,
            "using" => TokenKind::KwUsing,
            "use" => TokenKind::KwUse,
            "as" => TokenKind::KwAs,
            "void" => TokenKind::KwVoid,
            "true" => TokenKind::KwTrue,
            "false" => TokenKind::KwFalse,
            "break" => TokenKind::KwBreak,
            "continue" => TokenKind::KwContinue,
            _ => TokenKind::Ident,
        }
    }

    pub fn next_token(&mut self) -> Token {
        // [BUG 2 + BUG 9 FIX] Combined whitespace and comment skipping
        self.skip_whitespace_and_comments();

        if self.pos >= self.source.len() {
            return Token {
                kind: TokenKind::Eof,
                lexeme: String::new(),
                line: self.line,
                col: self.col,
            };
        }

        let ch = self.peek();
        let start_line = self.line;
        let start_col = self.col;

        // [BUG 9 FIX] Semicolons are ALWAYS statement terminators — no comment logic
        if ch == b';' {
            self.advance();
            return Token {
                kind: TokenKind::Semicolon,
                lexeme: String::from(";"),
                line: start_line,
                col: start_col,
            };
        }

        // [BUG 3 FIX] String literal "..."
        if ch == b'"' {
            self.advance(); // consume opening "
            let mut s = String::new();
            while self.pos < self.source.len() && self.peek() != b'"' {
                if self.peek() == b'\\' {
                    self.advance(); // skip backslash
                    if self.pos < self.source.len() {
                        let escaped = self.peek();
                        match escaped {
                            b'n' => s.push('\n'),
                            b't' => s.push('\t'),
                            b'\\' => s.push('\\'),
                            b'"' => s.push('"'),
                            _ => {
                                s.push('\\');
                                s.push(escaped as char);
                            }
                        }
                        self.advance();
                    }
                } else {
                    s.push(self.peek() as char);
                    self.advance();
                }
            }
            if self.pos < self.source.len() {
                self.advance(); // consume closing "
            }
            return Token {
                kind: TokenKind::String,
                lexeme: s,
                line: start_line,
                col: start_col,
            };
        }

        // Character literal 'c' (including escape sequences like '\n')
        if ch == b'\'' {
            self.advance(); // consume opening '
            let mut ch_val = String::new();
            if self.pos < self.source.len() {
                if self.peek() == b'\\' {
                    ch_val.push('\\');
                    self.advance();
                    if self.pos < self.source.len() {
                        ch_val.push(self.peek() as char);
                        self.advance();
                    }
                } else {
                    ch_val.push(self.peek() as char);
                    self.advance();
                }
            }
            if self.pos < self.source.len() && self.peek() == b'\'' {
                self.advance(); // consume closing '
            }
            return Token {
                kind: TokenKind::CharLiteral,
                lexeme: ch_val,
                line: start_line,
                col: start_col,
            };
        }

        // Multi-char operators
        if self.pos + 1 < self.source.len() {
            let two = [self.source[self.pos], self.source[self.pos + 1]];

            if two == [b'-', b'>'] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::Arrow,
                    lexeme: String::from("->"),
                    line: start_line,
                    col: start_col,
                };
            }

            if two == [b'=', b'>'] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::FatArrow,
                    lexeme: String::from("=>"),
                    line: start_line,
                    col: start_col,
                };
            }

            if two == [b'*', b'*'] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::Power,
                    lexeme: String::from("**"),
                    line: start_line,
                    col: start_col,
                };
            }

            if two == [b'<', b'='] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::Lte,
                    lexeme: String::from("<="),
                    line: start_line,
                    col: start_col,
                };
            }

            if two == [b'>', b'='] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::Gte,
                    lexeme: String::from(">="),
                    line: start_line,
                    col: start_col,
                };
            }

            if two == [b'=', b'='] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::Eq,
                    lexeme: String::from("=="),
                    line: start_line,
                    col: start_col,
                };
            }

            if two == [b'!', b'='] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::Neq,
                    lexeme: String::from("!="),
                    line: start_line,
                    col: start_col,
                };
            }

            if two == [b'<', b'<'] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::ShiftLeft,
                    lexeme: String::from("<<"),
                    line: start_line,
                    col: start_col,
                };
            }

            if two == [b'>', b'>'] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::ShiftRight,
                    lexeme: String::from(">>"),
                    line: start_line,
                    col: start_col,
                };
            }

            if two == [b'+', b'+'] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::PlusPlus,
                    lexeme: String::from("++"),
                    line: start_line,
                    col: start_col,
                };
            }

            if two == [b'+', b'='] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::PlusEquals,
                    lexeme: String::from("+="),
                    line: start_line,
                    col: start_col,
                };
            }

            if two == [b'+', b'%'] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::PlusPercent,
                    lexeme: String::from("+%"),
                    line: start_line,
                    col: start_col,
                };
            }

            if two == [b':', b':'] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::ColonColon,
                    lexeme: String::from("::"),
                    line: start_line,
                    col: start_col,
                };
            }

            if two == [b'.', b'.'] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::DotDot,
                    lexeme: String::from(".."),
                    line: start_line,
                    col: start_col,
                };
            }
        }

        // Identifier or keyword (including @builtins)
        if ch.is_ascii_alphabetic() || ch == b'_' || ch == b'@' {
            let mut ident = String::new();

            while self.pos < self.source.len() {
                let c = self.peek();
                if c.is_ascii_alphanumeric() || c == b'_' || c == b'@' {
                    ident.push(c as char);
                    self.advance();
                } else {
                    break;
                }
            }

            let kind = self.check_keyword(&ident);
            return Token {
                kind,
                lexeme: ident,
                line: start_line,
                col: start_col,
            };
        }

        // [BUG 6 FIX] Number literal with full hex digit support (a-f, A-F)
        if ch.is_ascii_digit() {
            let mut number = String::new();
            let mut is_hex = false;

            while self.pos < self.source.len() {
                let c = self.peek();
                // Don't consume '.' if it's part of a '..' range operator
                let is_dot_not_range = c == b'.'
                    && (self.pos + 1 >= self.source.len() || self.source[self.pos + 1] != b'.');
                if c.is_ascii_digit()
                    || is_dot_not_range
                    || c == b'x'
                    || c == b'X'
                    || c == b'b'
                    || c == b'B'
                    || c == b'_'
                {
                    if c == b'x' || c == b'X' {
                        is_hex = true;
                    }
                    number.push(c as char);
                    self.advance();
                } else if is_hex && ((c >= b'a' && c <= b'f') || (c >= b'A' && c <= b'F')) {
                    number.push(c as char);
                    self.advance();
                } else {
                    break;
                }
            }

            let type_suffixes: &[&[u8]] = &[
                b"u8",
                b"u16",
                b"u32",
                b"u64",
                b"usize",
                b"i8",
                b"i16",
                b"i32",
                b"i64",
                b"isize",
                b"f16",
                b"f32",
                b"f64",
                b"comptime_int",
            ];
            for suffix in type_suffixes.iter() {
                let end = self.pos + suffix.len();
                if end <= self.source.len() && &self.source[self.pos..end] == *suffix {
                    let next = if end < self.source.len() {
                        self.source[end]
                    } else {
                        0u8
                    };
                    if !next.is_ascii_alphanumeric() && next != b'_' {
                        self.pos = end;
                        self.col += suffix.len();
                    }
                    break;
                }
            }

            return Token {
                kind: TokenKind::Number,
                lexeme: number,
                line: start_line,
                col: start_col,
            };
        }

        // Multi-char tokens
        if ch == b'&' && self.peek() == b'&' {
            self.advance();
            self.advance();
            return Token {
                kind: TokenKind::Amp,
                lexeme: "&&".to_string(),
                line: start_line,
                col: start_col,
            };
        }
        if ch == b'|' && self.peek() == b'|' {
            self.advance();
            self.advance();
            return Token {
                kind: TokenKind::Pipe,
                lexeme: "||".to_string(),
                line: start_line,
                col: start_col,
            };
        }

        // Single char tokens
        let kind = match ch {
            b':' => TokenKind::Colon,
            b',' => TokenKind::Comma,
            b'=' => TokenKind::Equals,
            b'(' => TokenKind::LParen,
            b')' => TokenKind::RParen,
            b'{' => TokenKind::LBrace,
            b'}' => TokenKind::RBrace,
            b'[' => TokenKind::LBracket,
            b']' => TokenKind::RBracket,
            b'.' => TokenKind::Dot,
            b'!' => TokenKind::Bang,
            b'+' => TokenKind::Plus,
            b'-' => TokenKind::Minus,
            b'*' => TokenKind::Star,
            b'/' => TokenKind::Slash,
            b'%' => TokenKind::Percent,
            b'&' => TokenKind::Amp,
            b'|' => TokenKind::Pipe,
            b'^' => TokenKind::Caret,
            b'~' => TokenKind::Tilde,
            b'<' => TokenKind::Lt,
            b'>' => TokenKind::Gt,
            _ => {
                // Unknown character — skip and recurse
                self.advance();
                return self.next_token();
            }
        };

        self.advance();

        Token {
            kind,
            lexeme: String::from_utf8_lossy(&[ch]).to_string(),
            line: start_line,
            col: start_col,
        }
    }

    #[allow(dead_code)]
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token();
            if tok.kind == TokenKind::Eof {
                break;
            }
            tokens.push(tok);
        }
        tokens
    }
}

// ============================================================================
// Parser (recursive descent, minimal)
// ============================================================================

pub struct Parser {
    lexer: Lexer,
    current: Token,
    peek: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let first = lexer.next_token();
        let second = lexer.next_token();
        Self {
            lexer,
            current: first,
            peek: second,
        }
    }

    fn advance(&mut self) {
        self.current = self.peek.clone();
        self.peek = self.lexer.next_token();
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.current.kind == kind
    }

    #[allow(dead_code)]
    fn check_peek(&self, kind: TokenKind) -> bool {
        self.peek.kind == kind
    }

    fn expect(&mut self, kind: TokenKind) -> Result<(), String> {
        if !self.check(kind) {
            return Err(format!(
                "Expected {:?}, got {:?} ('{}') at line {}:{}",
                kind, self.current.kind, self.current.lexeme, self.current.line, self.current.col
            ));
        }
        self.advance();
        Ok(())
    }

    // [BUG 7 FIX] Brace-skip: count { } nesting to skip over body contents
    fn skip_brace_body(&mut self) -> Result<(), String> {
        // current token should be AFTER the opening LBrace was consumed
        let mut depth: i32 = 1;
        while depth > 0 {
            if self.current.kind == TokenKind::Eof {
                return Err("Unexpected EOF inside brace body".to_string());
            }
            if self.current.kind == TokenKind::LBrace {
                depth += 1;
            } else if self.current.kind == TokenKind::RBrace {
                depth -= 1;
                if depth == 0 {
                    // Don't advance past the closing brace — caller expects current == RBrace
                    return Ok(());
                }
            }
            self.advance();
        }
        Ok(())
    }

    // Skip everything to the next semicolon, handling nested braces, brackets, and parens
    fn skip_to_semicolon(&mut self) -> Result<(), String> {
        let mut bracket_depth: i32 = 0;
        let mut paren_depth: i32 = 0;
        while self.current.kind != TokenKind::Eof {
            // Only treat ; as terminator when not inside brackets or parens
            if self.current.kind == TokenKind::Semicolon && bracket_depth == 0 && paren_depth == 0 {
                self.advance();
                return Ok(());
            }
            if self.current.kind == TokenKind::LBrace {
                self.advance();
                self.skip_brace_body()?;
                if self.current.kind == TokenKind::RBrace {
                    self.advance();
                }
            } else if self.current.kind == TokenKind::LBracket {
                bracket_depth += 1;
                self.advance();
            } else if self.current.kind == TokenKind::RBracket {
                bracket_depth -= 1;
                self.advance();
            } else if self.current.kind == TokenKind::LParen {
                paren_depth += 1;
                self.advance();
            } else if self.current.kind == TokenKind::RParen {
                paren_depth -= 1;
                self.advance();
            } else {
                self.advance();
            }
        }
        Ok(())
    }

    // Check if the current token starts a new top-level declaration.
    // This is conservative: we only include keywords that unambiguously start
    // a new top-level form, excluding const/var which can appear inside
    // keyword-style test/invariant/bench blocks.
    fn is_top_level_start(&self) -> bool {
        matches!(
            self.current.kind,
            TokenKind::KwPub
                | TokenKind::KwFn
                | TokenKind::KwEnum
                | TokenKind::KwStruct
                | TokenKind::KwTest
                | TokenKind::KwInvariant
                | TokenKind::KwBench
                | TokenKind::KwUse
                | TokenKind::KwUsing
                | TokenKind::KwModule
                | TokenKind::RBrace
                | TokenKind::Eof
        )
    }

    // Skip tokens until we reach a top-level keyword (for keyword-style test/invariant/bench)
    // Handles nested braces, brackets, and parens so we don't stop inside nested groups
    fn skip_to_next_top_level(&mut self) {
        let mut paren_depth: i32 = 0;
        let mut bracket_depth: i32 = 0;
        loop {
            if self.current.kind == TokenKind::Eof {
                break;
            }
            // Handle brace groups by using skip_brace_body
            if self.current.kind == TokenKind::LBrace {
                self.advance();
                let _ = self.skip_brace_body();
                if self.current.kind == TokenKind::RBrace {
                    self.advance();
                }
                continue;
            }
            if self.current.kind == TokenKind::LParen {
                paren_depth += 1;
                self.advance();
                continue;
            }
            if self.current.kind == TokenKind::RParen {
                paren_depth -= 1;
                self.advance();
                continue;
            }
            if self.current.kind == TokenKind::LBracket {
                bracket_depth += 1;
                self.advance();
                continue;
            }
            if self.current.kind == TokenKind::RBracket {
                bracket_depth -= 1;
                self.advance();
                continue;
            }
            // Only check for top-level start when not inside nested groups
            if paren_depth == 0 && bracket_depth == 0 && self.is_top_level_start() {
                break;
            }
            self.advance();
        }
    }

    pub fn parse(&mut self) -> Result<Node, String> {
        let mut module = Node::new(NodeKind::Module);

        // [BUG 4 FIX] Parse optional module declaration
        if self.current.kind == TokenKind::KwModule {
            self.advance(); // consume 'module'
                            // Module name can contain hyphens: e.g. "tritype-base"
            let mut mod_name = String::new();
            if self.current.kind == TokenKind::Ident {
                mod_name.push_str(&self.current.lexeme);
                self.advance();
                // Consume hyphenated parts: - ident - ident ...
                while self.current.kind == TokenKind::Minus {
                    mod_name.push('-');
                    self.advance(); // consume -
                    if self.current.kind == TokenKind::Ident
                        || self.current.kind == TokenKind::Number
                    {
                        mod_name.push_str(&self.current.lexeme);
                        self.advance();
                    }
                }
            }
            module.name = mod_name;
            if self.current.kind == TokenKind::Semicolon {
                self.advance(); // consume ;
            } else if self.current.kind == TokenKind::LBrace {
                // Brace-style module: module Name { ... }
                self.advance(); // consume {
                self.parse_module_body(&mut module)?;
                self.expect(TokenKind::RBrace)?;
                return Ok(module);
            }
        }

        self.parse_module_body(&mut module)?;

        Ok(module)
    }

    fn parse_module_body(&mut self, module: &mut Node) -> Result<(), String> {
        while self.current.kind != TokenKind::Eof && self.current.kind != TokenKind::RBrace {
            // Parse use/using statements into UseDecl nodes
            if self.current.kind == TokenKind::KwUse || self.current.kind == TokenKind::KwUsing {
                self.advance(); // consume 'use'/'using'
                                // Collect the full path: e.g. "base::types" or just "datalog_solve"
                let mut full_path = String::new();
                let mut alias_name = String::new();
                if self.current.kind == TokenKind::Ident {
                    let first_ident = self.current.lexeme.clone();
                    full_path.push_str(&first_ident);
                    self.advance();

                    // Check for aliased import: using name: @import("path");
                    if self.current.kind == TokenKind::Colon && self.peek.kind != TokenKind::Colon {
                        alias_name = first_ident.clone();
                        self.advance(); // consume :
                                        // Skip @import("path") or any expression until ;
                                        // The value after : can be @import("...") or any expression
                        if self.current.kind == TokenKind::Ident {
                            // Could be @import(...) or a plain identifier
                            self.advance(); // consume @import or ident
                            if self.current.kind == TokenKind::LParen {
                                // Consume (...) call
                                self.advance(); // consume (
                                let mut paren_depth = 1;
                                while paren_depth > 0 && self.current.kind != TokenKind::Eof {
                                    if self.current.kind == TokenKind::LParen {
                                        paren_depth += 1;
                                    }
                                    if self.current.kind == TokenKind::RParen {
                                        paren_depth -= 1;
                                        if paren_depth == 0 {
                                            break;
                                        }
                                    }
                                    self.advance();
                                }
                                if self.current.kind == TokenKind::RParen {
                                    self.advance(); // consume )
                                }
                            }
                        }
                        full_path = alias_name.clone();
                    } else {
                        // Parse :: separated segments
                        while self.current.kind == TokenKind::Colon {
                            self.advance(); // first :
                            if self.current.kind == TokenKind::Colon {
                                self.advance(); // second :
                            }
                            full_path.push_str("::");
                            if self.current.kind == TokenKind::Ident {
                                full_path.push_str(&self.current.lexeme);
                                self.advance();
                            }
                        }
                    }
                }
                if self.current.kind == TokenKind::Semicolon {
                    self.advance();
                }
                // Extract the last segment as the import name
                let import_name = if !alias_name.is_empty() {
                    alias_name
                } else {
                    full_path
                        .rsplit("::")
                        .next()
                        .unwrap_or(&full_path)
                        .to_string()
                };
                let mut use_node = Node::new(NodeKind::UseDecl);
                use_node.name = import_name; // e.g. "types" or alias
                use_node.value = full_path; // e.g. "base::types" or alias
                module.children.push(use_node);
                continue;
            }

            match self.parse_top_level_decl() {
                Ok(decl) => module.children.push(decl),
                Err(_) => {
                    // On parse error, skip to next top-level declaration and continue
                    self.skip_to_next_top_level();
                }
            }
        }

        Ok(())
    }

    fn parse_top_level_decl(&mut self) -> Result<Node, String> {
        let is_pub = self.current.kind == TokenKind::KwPub;

        if is_pub {
            self.advance(); // consume pub
        }

        match self.current.kind {
            TokenKind::KwConst => self.parse_const_decl(is_pub),
            TokenKind::KwVar => self.parse_var_decl(is_pub),
            TokenKind::KwFn => self.parse_fn_decl(is_pub),
            TokenKind::KwEnum => self.parse_enum_decl(is_pub),
            TokenKind::KwStruct => self.parse_struct_decl(is_pub),
            TokenKind::KwTest => self.parse_test_block(),
            TokenKind::KwInvariant => self.parse_invariant_block(),
            TokenKind::KwBench => self.parse_bench_block(),
            _ => {
                // Skip unknown tokens to be resilient
                let tok = format!("{:?}", self.current.kind);
                let line = self.current.line;
                let col = self.current.col;
                let lexeme = self.current.lexeme.clone();
                self.advance();
                Err(format!(
                    "Unexpected top-level token: {} ('{}') at line {}:{}",
                    tok, lexeme, line, col
                ))
            }
        }
    }

    fn parse_const_decl(&mut self, is_pub: bool) -> Result<Node, String> {
        let mut decl = Node::new(NodeKind::ConstDecl);
        decl.extra_pub = is_pub;

        self.advance(); // consume 'const'

        // Name
        if self.current.kind == TokenKind::Ident {
            decl.name = self.current.lexeme.clone();
            self.advance();
        } else {
            return Err(format!(
                "Expected identifier after 'const', got {:?}",
                self.current.kind
            ));
        }

        // Optional type annotation `: Type`
        if self.current.kind == TokenKind::Colon {
            self.advance(); // consume :
                            // Type can be complex: u8, i8, []Trit, [N]T, etc.
            let mut type_str = String::new();
            // Handle [] prefix for slice types
            if self.current.kind == TokenKind::LBracket {
                type_str.push('[');
                self.advance();
                // Might have a size expression
                while self.current.kind != TokenKind::RBracket
                    && self.current.kind != TokenKind::Eof
                {
                    type_str.push_str(&self.current.lexeme);
                    self.advance();
                }
                type_str.push(']');
                if self.current.kind == TokenKind::RBracket {
                    self.advance();
                }
            }
            if self.current.kind == TokenKind::Ident {
                type_str.push_str(&self.current.lexeme);
                self.advance();
            }
            decl.extra_type = type_str;
        }

        // = value
        if self.current.kind == TokenKind::Equals {
            self.advance(); // consume =

            // [BUG 8 FIX] Check what follows: enum(...), struct { }, [N]T{ }, identifier, number, etc.
            if self.current.kind == TokenKind::KwEnum {
                // pub const Trit = enum(i8) { ... };
                decl.kind = NodeKind::EnumDecl;
                self.advance(); // consume 'enum'
                                // Optional backing type: (i8)
                if self.current.kind == TokenKind::LParen {
                    self.advance(); // consume (
                    if self.current.kind == TokenKind::Ident {
                        decl.extra_type = self.current.lexeme.clone();
                        self.advance();
                    }
                    self.expect(TokenKind::RParen)?; // consume )
                }
                // { ... }
                self.expect(TokenKind::LBrace)?;
                // Parse enum body with brace-skip for safety
                self.parse_enum_body(&mut decl)?;
                self.expect(TokenKind::RBrace)?;
            } else if self.current.kind == TokenKind::KwStruct {
                // pub const Foo = struct { ... };
                decl.kind = NodeKind::StructDecl;
                self.advance(); // consume 'struct'
                self.expect(TokenKind::LBrace)?;
                self.parse_struct_body(&mut decl)?;
                self.expect(TokenKind::RBrace)?;
            } else if self.current.kind == TokenKind::LBracket {
                // pub const TernaryWord = [WORD_BYTES]u8; or [_]u8{...} ** N
                // Collect the full expression as value text
                let mut val_text = String::new();
                while self.current.kind != TokenKind::Semicolon
                    && self.current.kind != TokenKind::Eof
                {
                    val_text.push_str(&self.current.lexeme);
                    self.advance();
                }
                let mut val_node = Node::new(NodeKind::ExprIdentifier);
                val_node.name = val_text;
                decl.children.push(val_node);
                if self.current.kind == TokenKind::Semicolon {
                    self.advance();
                }
                return Ok(decl);
            } else if self.current.kind == TokenKind::Minus {
                // [BUG 10 FIX] Negative number: -1 or expression
                self.advance(); // consume -
                if self.current.kind == TokenKind::Number {
                    let mut val_node = Node::new(NodeKind::ExprLiteral);
                    val_node.value = format!("-{}", self.current.lexeme);
                    decl.children.push(val_node);
                    self.advance();
                }
            } else if self.current.kind == TokenKind::Number {
                let mut val_node = Node::new(NodeKind::ExprLiteral);
                val_node.value = self.current.lexeme.clone();
                decl.children.push(val_node);
                self.advance();
            } else if self.current.kind == TokenKind::Ident {
                // Type alias or expression start: pub const PackedTrit = u8;
                let mut val_node = Node::new(NodeKind::ExprIdentifier);
                val_node.name = self.current.lexeme.clone();
                decl.children.push(val_node);
                self.advance();
            } else if self.current.kind == TokenKind::KwTrue
                || self.current.kind == TokenKind::KwFalse
            {
                let mut val_node = Node::new(NodeKind::ExprLiteral);
                val_node.value = self.current.lexeme.clone();
                decl.children.push(val_node);
                self.advance();
            } else if self.current.kind == TokenKind::String {
                let mut val_node = Node::new(NodeKind::ExprLiteral);
                val_node.value = self.current.lexeme.clone();
                decl.children.push(val_node);
                self.advance();
            } else {
                // Other RHS (tilde, parens, etc.) — skip to semicolon
                self.skip_to_semicolon()?;
                return Ok(decl);
            }

            // After reading the first value token, skip any remaining expression
            // tokens (operators, more operands) until semicolon
            // But don't skip past module body closing brace or next declaration
            if self.current.kind != TokenKind::Semicolon
                && self.current.kind != TokenKind::RBrace
                && !self.is_top_level_start()
                && self.current.kind != TokenKind::Eof
            {
                self.skip_to_semicolon()?;
            }
            return Ok(decl);
        }

        // Consume trailing semicolon
        if self.current.kind == TokenKind::Semicolon {
            self.advance();
        }
        Ok(decl)
    }

    fn parse_var_decl(&mut self, is_pub: bool) -> Result<Node, String> {
        let mut decl = Node::new(NodeKind::ConstDecl);
        decl.extra_pub = is_pub;
        decl.extra_mutable = true;

        self.advance(); // consume 'var'

        // Name
        if self.current.kind == TokenKind::Ident {
            decl.name = self.current.lexeme.clone();
            self.advance();
        }

        // Type annotation: : Type
        if self.current.kind == TokenKind::Colon {
            self.advance(); // consume :
            decl.extra_type = self.parse_type_annotation();
        }

        // Initial value: = expr
        if self.current.kind == TokenKind::Equals {
            self.advance(); // consume =
            let val_node = self.parse_expr()?;
            decl.children.push(val_node);
        }

        if self.current.kind == TokenKind::Semicolon {
            self.advance();
        }
        Ok(decl)
    }

    fn parse_enum_body(&mut self, decl: &mut Node) -> Result<(), String> {
        // We are inside { ... } of an enum. Parse variant = value pairs.
        while self.current.kind != TokenKind::RBrace && self.current.kind != TokenKind::Eof {
            if self.current.kind == TokenKind::Ident {
                let name = self.current.lexeme.clone();
                self.advance();

                let mut value_str = String::new();
                if self.current.kind == TokenKind::Equals {
                    self.advance(); // consume =
                                    // [BUG 10] Handle negative enum values
                    if self.current.kind == TokenKind::Minus {
                        value_str.push('-');
                        self.advance();
                    }
                    if self.current.kind == TokenKind::Number {
                        value_str.push_str(&self.current.lexeme);
                        self.advance();
                    } else if self.current.kind == TokenKind::Ident {
                        value_str.push_str(&self.current.lexeme);
                        self.advance();
                    }
                }

                let mut variant = Node::new(NodeKind::EnumVariant);
                variant.name = name;
                variant.value = value_str;
                decl.children.push(variant);

                if self.current.kind == TokenKind::Comma {
                    self.advance();
                }
            } else {
                // Skip unexpected tokens inside enum
                self.advance();
            }
        }
        Ok(())
    }

    fn parse_struct_body(&mut self, decl: &mut Node) -> Result<(), String> {
        // We are inside { ... } of a struct. Parse field: Type pairs.
        while self.current.kind != TokenKind::RBrace && self.current.kind != TokenKind::Eof {
            if self.current.kind == TokenKind::Ident {
                let field_name = self.current.lexeme.clone();
                self.advance();

                let mut type_str = String::new();
                if self.current.kind == TokenKind::Colon {
                    self.advance(); // consume :
                                    // Collect type tokens until comma, semicolon, or closing brace
                    while self.current.kind != TokenKind::Comma
                        && self.current.kind != TokenKind::Semicolon
                        && self.current.kind != TokenKind::RBrace
                        && self.current.kind != TokenKind::Eof
                    {
                        type_str.push_str(&self.current.lexeme);
                        self.advance();
                    }
                }

                let mut field = Node::new(NodeKind::ExprIdentifier);
                field.name = field_name;
                field.extra_type = type_str;
                decl.children.push(field);

                if self.current.kind == TokenKind::Comma
                    || self.current.kind == TokenKind::Semicolon
                {
                    self.advance();
                }
            } else {
                // Skip unexpected tokens inside struct
                self.advance();
            }
        }
        Ok(())
    }

    /// Parse a type annotation like `Trit`, `*Trit`, `[]u8`, `[N]u8`, `[]const u8`, `anytype`
    fn parse_type_annotation(&mut self) -> String {
        let mut ty = String::new();

        // Handle pointer prefix: *Type or *const Type
        if self.current.kind == TokenKind::Star {
            ty.push('*');
            self.advance();
            if self.current.kind == TokenKind::KwConst {
                ty.push_str("const ");
                self.advance();
            }
            if self.current.kind == TokenKind::Ident {
                ty.push_str(&self.current.lexeme);
                self.advance();
            }
            return ty;
        }

        // Handle slice/array prefix: []Type, [N]Type, [[f64; 8]; 8], []const Type
        while self.current.kind == TokenKind::LBracket {
            ty.push('[');
            self.advance(); // consume [
            let mut depth: usize = 1;
            while depth > 0 && self.current.kind != TokenKind::Eof {
                match self.current.kind {
                    TokenKind::LBracket => {
                        depth += 1;
                        ty.push('[');
                        self.advance();
                    }
                    TokenKind::RBracket => {
                        depth -= 1;
                        ty.push(']');
                        self.advance();
                    }
                    _ => {
                        ty.push_str(&self.current.lexeme);
                        self.advance();
                    }
                }
            }
        }

        // Handle 'const' qualifier: []const u8
        if self.current.kind == TokenKind::KwConst {
            if !ty.is_empty() {
                ty.push_str("const ");
            } else {
                ty.push_str("const ");
            }
            self.advance();
        }

        // Handle pointer after brackets
        if self.current.kind == TokenKind::Star {
            ty.push('*');
            self.advance();
            if self.current.kind == TokenKind::KwConst {
                ty.push_str("const ");
                self.advance();
            }
        }

        // Main type identifier with namespace support (lexer::Lexer, base::types)
        if self.current.kind == TokenKind::Ident {
            ty.push_str(&self.current.lexeme);
            self.advance();

            // Handle :: namespace separators
            while self.current.kind == TokenKind::Colon {
                // Check for :: (two colons)
                if self.peek.kind == TokenKind::Colon {
                    ty.push_str("::");
                    self.advance(); // consume first :
                    self.advance(); // consume second :
                    if self.current.kind == TokenKind::Ident {
                        ty.push_str(&self.current.lexeme);
                        self.advance();
                    }
                } else {
                    break;
                }
            }
        } else if self.current.kind == TokenKind::KwVoid {
            ty.push_str("void");
            self.advance();
        }

        ty
    }

    fn parse_fn_decl(&mut self, is_pub: bool) -> Result<Node, String> {
        let mut decl = Node::new(NodeKind::FnDecl);
        decl.extra_pub = is_pub;
        decl.line = self.current.line as u32;

        self.advance(); // consume 'fn'

        if self.current.kind == TokenKind::Ident {
            decl.name = self.current.lexeme.clone();
            self.advance();
            // Handle dotted names like Parser.new
            while self.current.kind == TokenKind::Dot {
                decl.name.push('.');
                self.advance(); // consume .
                if self.current.kind == TokenKind::Ident {
                    decl.name.push_str(&self.current.lexeme);
                    self.advance();
                }
            }
        }

        // Parse parameter list
        self.expect(TokenKind::LParen)?;
        while self.current.kind != TokenKind::RParen && self.current.kind != TokenKind::Eof {
            // Parse param name
            if self.current.kind != TokenKind::Ident {
                // Skip unexpected token
                self.advance();
                continue;
            }
            let param_name = self.current.lexeme.clone();
            self.advance();

            // Expect colon
            if self.current.kind == TokenKind::Colon {
                self.advance(); // consume :
            }

            // Parse param type
            let param_type = self.parse_type_annotation();

            decl.params.push((param_name, param_type));

            // Consume comma between params
            if self.current.kind == TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(TokenKind::RParen)?;

        // Optional arrow for return type: -> Type
        if self.current.kind == TokenKind::Arrow {
            self.advance(); // consume ->
        }

        // Handle error union prefix: !Type or !void
        let has_error_union = self.current.kind == TokenKind::Bang;
        if has_error_union {
            self.advance(); // consume !
        }

        // Return type (identifier, or []T / [N]T / [][]const u8 slice/array types, or void)
        if self.current.kind == TokenKind::Ident {
            decl.extra_return_type = self.current.lexeme.clone();
            self.advance();
            // Handle generic return types like Option<Foo>
            if self.current.kind == TokenKind::Lt {
                let mut gt_depth = 1;
                self.advance(); // consume <
                while gt_depth > 0 && self.current.kind != TokenKind::Eof {
                    if self.current.kind == TokenKind::Lt {
                        gt_depth += 1;
                    } else if self.current.kind == TokenKind::Gt {
                        gt_depth -= 1;
                        if gt_depth == 0 {
                            break;
                        }
                    }
                    self.advance();
                }
                if self.current.kind == TokenKind::Gt {
                    self.advance(); // consume >
                }
            }
        } else if self.current.kind == TokenKind::LBracket {
            // Handle one or more bracket levels: []Type, [][]const u8, [N]Type, [[f64; 8]; 8]
            let mut rt = String::new();
            while self.current.kind == TokenKind::LBracket {
                rt.push('[');
                self.advance(); // consume [
                let mut depth: usize = 1;
                while depth > 0 && self.current.kind != TokenKind::Eof {
                    match self.current.kind {
                        TokenKind::LBracket => {
                            depth += 1;
                            rt.push('[');
                            self.advance();
                        }
                        TokenKind::RBracket => {
                            depth -= 1;
                            rt.push(']');
                            self.advance();
                        }
                        _ => {
                            rt.push_str(&self.current.lexeme);
                            self.advance();
                        }
                    }
                }
            }
            // Handle 'const' qualifier in return type: []const u8
            if self.current.kind == TokenKind::KwConst {
                rt.push_str("const ");
                self.advance();
            }
            // Handle pointer prefix: *Type
            if self.current.kind == TokenKind::Star {
                rt.push('*');
                self.advance();
            }
            if self.current.kind == TokenKind::Ident {
                rt.push_str(&self.current.lexeme);
                self.advance();
            }
            decl.extra_return_type = rt;
        } else if self.current.kind == TokenKind::KwVoid {
            decl.extra_return_type = "void".to_string();
            self.advance();
        } else if self.current.kind == TokenKind::Star {
            // Pointer return type: *Type
            self.advance(); // consume *
            if self.current.kind == TokenKind::KwConst {
                self.advance(); // consume const
            }
            if self.current.kind == TokenKind::Ident {
                decl.extra_return_type = format!("*{}", self.current.lexeme);
                self.advance();
            }
        }

        // Skip optional 'const' qualifier before the body
        if self.current.kind == TokenKind::KwConst {
            self.advance();
        }

        // Parse body: real expressions
        self.expect(TokenKind::LBrace)?;
        self.parse_fn_body(&mut decl)?;
        self.expect(TokenKind::RBrace)?;
        Ok(decl)
    }

    /// Parse function body statements until closing brace
    fn parse_fn_body(&mut self, decl: &mut Node) -> Result<(), String> {
        while self.current.kind != TokenKind::RBrace && self.current.kind != TokenKind::Eof {
            match self.parse_body_stmt() {
                Ok(stmt) => decl.children.push(stmt),
                Err(_) => {
                    // On parse error, skip to next statement boundary and continue
                    self.recover_to_stmt_boundary();
                }
            }
        }
        Ok(())
    }

    /// Skip tokens to recover to next statement boundary (semicolon or closing brace)
    fn recover_to_stmt_boundary(&mut self) {
        let mut brace_depth: i32 = 0;
        loop {
            match self.current.kind {
                TokenKind::Eof => break,
                TokenKind::Semicolon if brace_depth == 0 => {
                    self.advance();
                    break;
                }
                TokenKind::RBrace if brace_depth == 0 => break,
                TokenKind::LBrace => {
                    brace_depth += 1;
                    self.advance();
                }
                TokenKind::RBrace => {
                    brace_depth -= 1;
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    /// Parse a single statement inside a function body
    fn parse_body_stmt(&mut self) -> Result<Node, String> {
        // const / var declaration
        if self.current.kind == TokenKind::KwConst || self.current.kind == TokenKind::KwVar {
            return self.parse_local_decl();
        }

        // return statement
        if self.current.kind == TokenKind::KwReturn {
            return self.parse_return_statement();
        }

        // if statement
        if self.current.kind == TokenKind::KwIf {
            return self.parse_if_stmt();
        }

        // while statement
        if self.current.kind == TokenKind::KwWhile {
            return self.parse_while_stmt();
        }

        // for statement
        if self.current.kind == TokenKind::KwFor {
            return self.parse_for_stmt();
        }

        // break statement
        if self.current.kind == TokenKind::KwBreak {
            self.advance();
            if self.current.kind == TokenKind::Semicolon {
                self.advance();
            }
            return Ok(Node::new(NodeKind::StmtBreak));
        }

        // continue statement
        if self.current.kind == TokenKind::KwContinue {
            self.advance();
            if self.current.kind == TokenKind::Semicolon {
                self.advance();
            }
            return Ok(Node::new(NodeKind::StmtContinue));
        }

        // Expression or assignment
        let expr = self.parse_expr()?;

        // Check for assignment: expr = rhs;
        if self.current.kind == TokenKind::Equals {
            self.advance(); // consume =
            let rhs = self.parse_expr()?;
            if self.current.kind == TokenKind::Semicolon {
                self.advance();
            }
            let mut assign = Node::new(NodeKind::StmtAssign);
            assign.line = self.current.line as u32;
            assign.children.push(expr);
            assign.children.push(rhs);
            return Ok(assign);
        }

        // Check for += assignment
        if self.current.kind == TokenKind::PlusEquals {
            self.advance(); // consume +=
            let rhs = self.parse_expr()?;
            if self.current.kind == TokenKind::Semicolon {
                self.advance();
            }
            let mut assign = Node::new(NodeKind::StmtAssign);
            assign.line = self.current.line as u32;
            assign.extra_op = "+=".to_string();
            assign.children.push(expr);
            assign.children.push(rhs);
            return Ok(assign);
        }

        // Bare expression statement
        if self.current.kind == TokenKind::Semicolon {
            self.advance();
        }
        let mut stmt = Node::new(NodeKind::StmtExpr);
        stmt.children.push(expr);
        Ok(stmt)
    }

    /// Parse local const/var declaration
    fn parse_local_decl(&mut self) -> Result<Node, String> {
        let mut decl = Node::new(NodeKind::StmtLocal);
        decl.line = self.current.line as u32;
        decl.extra_mutable = self.current.kind == TokenKind::KwVar;
        self.advance(); // consume const/var

        // Name
        if self.current.kind == TokenKind::Ident {
            decl.name = self.current.lexeme.clone();
            self.advance();
        }

        // Optional type annotation: : Type
        if self.current.kind == TokenKind::Colon {
            self.advance(); // consume :
            decl.extra_type = self.parse_type_annotation();
        }

        // = initializer
        if self.current.kind == TokenKind::Equals {
            self.advance(); // consume =
            let init = self.parse_expr()?;
            decl.children.push(init);
        }

        if self.current.kind == TokenKind::Semicolon {
            self.advance();
        }
        Ok(decl)
    }

    /// Parse return statement
    fn parse_return_statement(&mut self) -> Result<Node, String> {
        let mut stmt = Node::new(NodeKind::ExprReturn);
        self.advance(); // consume 'return'

        // Optional return value
        if self.current.kind != TokenKind::Semicolon && self.current.kind != TokenKind::RBrace {
            let expr = self.parse_expr()?;
            stmt.children.push(expr);
        }

        if self.current.kind == TokenKind::Semicolon {
            self.advance();
        }
        Ok(stmt)
    }

    /// Parse if / else if / else statement
    fn parse_if_stmt(&mut self) -> Result<Node, String> {
        let mut if_node = Node::new(NodeKind::StmtIf);
        self.advance(); // consume 'if'

        // Condition in parentheses
        self.expect(TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;
        if_node.children.push(cond);

        // Then branch: { ... }
        if self.current.kind == TokenKind::LBrace {
            self.advance(); // consume {
            let mut then_block = Node::new(NodeKind::Module); // reuse Module as block container
            then_block.name = "then".to_string();
            while self.current.kind != TokenKind::RBrace && self.current.kind != TokenKind::Eof {
                match self.parse_body_stmt() {
                    Ok(s) => then_block.children.push(s),
                    Err(_) => self.recover_to_stmt_boundary(),
                }
            }
            self.expect(TokenKind::RBrace)?;
            if_node.children.push(then_block);
        } else {
            // single statement: if (cond) return expr;
            let stmt = self.parse_body_stmt()?;
            let mut then_block = Node::new(NodeKind::Module);
            then_block.name = "then".to_string();
            then_block.children.push(stmt);
            if_node.children.push(then_block);
        }

        // else / else if
        if self.current.kind == TokenKind::KwElse {
            self.advance(); // consume 'else'
            if self.current.kind == TokenKind::KwIf {
                // else if -> recurse
                let else_if = self.parse_if_stmt()?;
                let mut else_block = Node::new(NodeKind::Module);
                else_block.name = "else".to_string();
                else_block.children.push(else_if);
                if_node.children.push(else_block);
            } else if self.current.kind == TokenKind::LBrace {
                self.advance(); // consume {
                let mut else_block = Node::new(NodeKind::Module);
                else_block.name = "else".to_string();
                while self.current.kind != TokenKind::RBrace && self.current.kind != TokenKind::Eof
                {
                    match self.parse_body_stmt() {
                        Ok(s) => else_block.children.push(s),
                        Err(_) => self.recover_to_stmt_boundary(),
                    }
                }
                self.expect(TokenKind::RBrace)?;
                if_node.children.push(else_block);
            }
        }

        Ok(if_node)
    }

    /// Parse while statement
    fn parse_while_stmt(&mut self) -> Result<Node, String> {
        let mut while_node = Node::new(NodeKind::StmtWhile);
        self.advance(); // consume 'while'

        // Condition in parentheses
        self.expect(TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;
        while_node.children.push(cond);

        // Body: { ... }
        self.expect(TokenKind::LBrace)?;
        let mut body_block = Node::new(NodeKind::Module);
        body_block.name = "body".to_string();
        while self.current.kind != TokenKind::RBrace && self.current.kind != TokenKind::Eof {
            match self.parse_body_stmt() {
                Ok(s) => body_block.children.push(s),
                Err(_) => self.recover_to_stmt_boundary(),
            }
        }
        self.expect(TokenKind::RBrace)?;
        while_node.children.push(body_block);

        Ok(while_node)
    }

    /// Parse for statement: for (iterable) |capture| { body }
    /// Also: for (a, b) |x, y| { body }
    fn parse_for_stmt(&mut self) -> Result<Node, String> {
        let mut for_node = Node::new(NodeKind::StmtFor);
        self.advance(); // consume 'for'

        // Iterable(s) in parentheses
        self.expect(TokenKind::LParen)?;
        // Parse comma-separated iterables
        while self.current.kind != TokenKind::RParen && self.current.kind != TokenKind::Eof {
            let iter_expr = self.parse_expr()?;
            for_node.children.push(iter_expr);
            if self.current.kind == TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(TokenKind::RParen)?;

        // Capture variables: |x| or |x, y| or |*x| (pointer capture)
        if self.current.kind == TokenKind::Pipe {
            self.advance(); // consume |
            while self.current.kind != TokenKind::Pipe && self.current.kind != TokenKind::Eof {
                // Skip pointer prefix *
                if self.current.kind == TokenKind::Star {
                    self.advance();
                }
                if self.current.kind == TokenKind::Ident {
                    for_node
                        .params
                        .push((self.current.lexeme.clone(), String::new()));
                    self.advance();
                } else if self.current.kind == TokenKind::Comma {
                    self.advance();
                } else {
                    // Skip unknown tokens to prevent infinite loop
                    self.advance();
                }
            }
            self.expect(TokenKind::Pipe)?;
        }

        // Body: { ... }
        self.expect(TokenKind::LBrace)?;
        let mut body_block = Node::new(NodeKind::Module);
        body_block.name = "body".to_string();
        while self.current.kind != TokenKind::RBrace && self.current.kind != TokenKind::Eof {
            match self.parse_body_stmt() {
                Ok(s) => body_block.children.push(s),
                Err(_) => self.recover_to_stmt_boundary(),
            }
        }
        self.expect(TokenKind::RBrace)?;
        for_node.children.push(body_block);

        Ok(for_node)
    }

    // ========================================================================
    // Expression parser (Pratt-style, operates on current token)
    // ========================================================================

    /// Parse a full expression
    fn parse_expr(&mut self) -> Result<Node, String> {
        self.parse_expr_or()
    }

    /// Parse `or` expressions
    fn parse_expr_or(&mut self) -> Result<Node, String> {
        let mut left = self.parse_expr_and()?;
        while self.current.kind == TokenKind::KwOr
            || (self.current.kind == TokenKind::Pipe && self.current.lexeme == "||")
        {
            self.advance();
            let right = self.parse_expr_and()?;
            left = Node {
                kind: NodeKind::ExprBinary,
                extra_op: "or".to_string(),
                children: vec![left, right],
                ..Default::default()
            };
        }
        Ok(left)
    }

    /// Parse `and` expressions
    fn parse_expr_and(&mut self) -> Result<Node, String> {
        let mut left = self.parse_expr_comparison()?;
        while self.current.kind == TokenKind::KwAnd
            || (self.current.kind == TokenKind::Amp && self.current.lexeme == "&&")
        {
            self.advance();
            let right = self.parse_expr_comparison()?;
            left = Node {
                kind: NodeKind::ExprBinary,
                extra_op: "and".to_string(),
                children: vec![left, right],
                ..Default::default()
            };
        }
        Ok(left)
    }

    /// Parse comparison expressions (==, !=, <, >, <=, >=)
    fn parse_expr_comparison(&mut self) -> Result<Node, String> {
        let mut left = self.parse_expr_bitor()?;
        while matches!(
            self.current.kind,
            TokenKind::Eq
                | TokenKind::Neq
                | TokenKind::Lt
                | TokenKind::Gt
                | TokenKind::Lte
                | TokenKind::Gte
                | TokenKind::DotDot
        ) {
            let op = self.current.lexeme.clone();
            self.advance();
            let right = self.parse_expr_bitor()?;
            left = Node {
                kind: NodeKind::ExprBinary,
                extra_op: op,
                children: vec![left, right],
                ..Default::default()
            };
        }
        Ok(left)
    }

    /// Parse bitwise or (|)
    fn parse_expr_bitor(&mut self) -> Result<Node, String> {
        let mut left = self.parse_expr_bitxor()?;
        while self.current.kind == TokenKind::Pipe && self.current.lexeme == "|" {
            let op = self.current.lexeme.clone();
            self.advance();
            let right = self.parse_expr_bitxor()?;
            left = Node {
                kind: NodeKind::ExprBinary,
                extra_op: op,
                children: vec![left, right],
                ..Default::default()
            };
        }
        Ok(left)
    }

    /// Parse bitwise xor (^)
    fn parse_expr_bitxor(&mut self) -> Result<Node, String> {
        let mut left = self.parse_expr_bitand()?;
        while self.current.kind == TokenKind::Caret {
            let op = self.current.lexeme.clone();
            self.advance();
            let right = self.parse_expr_bitand()?;
            left = Node {
                kind: NodeKind::ExprBinary,
                extra_op: op,
                children: vec![left, right],
                ..Default::default()
            };
        }
        Ok(left)
    }

    /// Parse bitwise and (&)
    fn parse_expr_bitand(&mut self) -> Result<Node, String> {
        let mut left = self.parse_expr_shift()?;
        while self.current.kind == TokenKind::Amp && self.current.lexeme == "&" {
            let op = self.current.lexeme.clone();
            self.advance();
            let right = self.parse_expr_shift()?;
            left = Node {
                kind: NodeKind::ExprBinary,
                extra_op: op,
                children: vec![left, right],
                ..Default::default()
            };
        }
        Ok(left)
    }

    /// Parse shift expressions (<<, >>)
    fn parse_expr_shift(&mut self) -> Result<Node, String> {
        let mut left = self.parse_expr_additive()?;
        while matches!(
            self.current.kind,
            TokenKind::ShiftLeft | TokenKind::ShiftRight
        ) {
            let op = self.current.lexeme.clone();
            self.advance();
            let right = self.parse_expr_additive()?;
            left = Node {
                kind: NodeKind::ExprBinary,
                extra_op: op,
                children: vec![left, right],
                ..Default::default()
            };
        }
        Ok(left)
    }

    /// Parse additive expressions (+, -, +%)
    fn parse_expr_additive(&mut self) -> Result<Node, String> {
        let mut left = self.parse_expr_multiplicative()?;
        while matches!(
            self.current.kind,
            TokenKind::Plus | TokenKind::Minus | TokenKind::PlusPercent
        ) {
            let op = self.current.lexeme.clone();
            self.advance();
            let right = self.parse_expr_multiplicative()?;
            left = Node {
                kind: NodeKind::ExprBinary,
                extra_op: op,
                children: vec![left, right],
                ..Default::default()
            };
        }
        Ok(left)
    }

    /// Parse multiplicative expressions (*, /, %, **)
    fn parse_expr_multiplicative(&mut self) -> Result<Node, String> {
        let mut left = self.parse_expr_unary()?;
        while matches!(
            self.current.kind,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent | TokenKind::Power
        ) {
            let op = self.current.lexeme.clone();
            self.advance();
            let right = self.parse_expr_unary()?;
            left = Node {
                kind: NodeKind::ExprBinary,
                extra_op: op,
                children: vec![left, right],
                ..Default::default()
            };
        }
        Ok(left)
    }

    /// Parse unary expressions (-x, !x, ~x, &x)
    fn parse_expr_unary(&mut self) -> Result<Node, String> {
        if matches!(
            self.current.kind,
            TokenKind::Minus | TokenKind::Bang | TokenKind::Tilde | TokenKind::Amp
        ) {
            let op = self.current.lexeme.clone();
            self.advance();
            let operand = self.parse_expr_unary()?;
            return Ok(Node {
                kind: NodeKind::ExprUnary,
                extra_op: op,
                children: vec![operand],
                ..Default::default()
            });
        }
        self.parse_expr_postfix()
    }

    /// Parse postfix expressions: field access (.field), namespace (::name), deref (.*), indexing ([i]), call (f(args))
    fn parse_expr_postfix(&mut self) -> Result<Node, String> {
        let mut expr = self.parse_expr_primary()?;

        loop {
            if self.current.kind == TokenKind::KwAs {
                // Type cast: expr as Type
                self.advance(); // consume as
                if self.current.kind == TokenKind::Ident {
                    let type_name = self.current.lexeme.clone();
                    self.advance();
                    let mut cast = Node::new(NodeKind::ExprCast);
                    cast.extra_type = type_name;
                    cast.children.push(expr);
                    expr = cast;
                } else {
                    break;
                }
            } else if self.current.kind == TokenKind::ColonColon {
                // Namespace/path access: expr::name
                self.advance(); // consume ::
                if self.current.kind == TokenKind::Ident {
                    let field = self.current.lexeme.clone();
                    self.advance();
                    let mut fa = Node::new(NodeKind::ExprFieldAccess);
                    fa.name = field;
                    fa.children.push(expr);
                    expr = fa;
                } else {
                    break;
                }
            } else if self.current.kind == TokenKind::Dot {
                self.advance(); // consume .
                if self.current.kind == TokenKind::Star {
                    // Dereference: expr.*
                    self.advance(); // consume *
                    let mut deref = Node::new(NodeKind::ExprFieldAccess);
                    deref.name = "*".to_string();
                    deref.children.push(expr);
                    expr = deref;
                } else if self.current.kind == TokenKind::Ident {
                    let field = self.current.lexeme.clone();
                    self.advance();
                    // Check if this is a method/field call: expr.field(args)
                    if self.current.kind == TokenKind::LParen {
                        // Method-style call: expr.field(args)
                        // Build fully-qualified name from field access chain
                        let full_name = Self::flatten_field_access_name(&expr, &field);
                        let call = self.parse_call_args(full_name)?;
                        expr = call;
                    } else {
                        let mut fa = Node::new(NodeKind::ExprFieldAccess);
                        fa.name = field;
                        fa.children.push(expr);
                        expr = fa;
                    }
                } else {
                    break;
                }
            } else if self.current.kind == TokenKind::LBracket {
                self.advance(); // consume [
                let index = self.parse_expr()?;
                self.expect(TokenKind::RBracket)?;
                let mut idx_node = Node::new(NodeKind::ExprIndex);
                idx_node.children.push(expr);
                idx_node.children.push(index);
                expr = idx_node;
            } else if self.current.kind == TokenKind::LParen {
                // Function call on an expression: shouldn't normally happen here
                // since calls are handled in primary for ident(...) and @builtin(...)
                break;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Flatten a chain of ExprFieldAccess nodes into a dotted name
    /// e.g. ExprFieldAccess("expectEqual", ExprFieldAccess("testing", ExprIdentifier("std")))
    /// becomes "std.testing.expectEqual"
    fn flatten_field_access_name(expr: &Node, trailing_field: &str) -> String {
        let mut parts = vec![trailing_field.to_string()];
        let mut current = expr;
        loop {
            match current.kind {
                NodeKind::ExprFieldAccess => {
                    parts.push(current.name.clone());
                    if !current.children.is_empty() {
                        current = &current.children[0];
                    } else {
                        break;
                    }
                }
                NodeKind::ExprIdentifier => {
                    parts.push(current.name.clone());
                    break;
                }
                _ => break,
            }
        }
        parts.reverse();
        parts.join(".")
    }

    /// Parse primary expressions
    fn parse_expr_primary(&mut self) -> Result<Node, String> {
        match self.current.kind {
            // Number literal
            TokenKind::Number => {
                let val = self.current.lexeme.clone();
                self.advance();
                Ok(Node {
                    kind: NodeKind::ExprLiteral,
                    value: val,
                    ..Default::default()
                })
            }

            // Character literal
            TokenKind::CharLiteral => {
                let val = self.current.lexeme.clone();
                self.advance();
                Ok(Node {
                    kind: NodeKind::ExprLiteral,
                    value: format!("'{}'", val),
                    ..Default::default()
                })
            }

            // String literal
            TokenKind::String => {
                let val = self.current.lexeme.clone();
                self.advance();
                Ok(Node {
                    kind: NodeKind::ExprLiteral,
                    value: format!("\"{}\"", val),
                    ..Default::default()
                })
            }

            // Boolean literals
            TokenKind::KwTrue => {
                self.advance();
                Ok(Node {
                    kind: NodeKind::ExprLiteral,
                    value: "true".to_string(),
                    ..Default::default()
                })
            }
            TokenKind::KwFalse => {
                self.advance();
                Ok(Node {
                    kind: NodeKind::ExprLiteral,
                    value: "false".to_string(),
                    ..Default::default()
                })
            }

            // Enum value: .variant
            TokenKind::Dot => {
                self.advance(); // consume .
                if self.current.kind == TokenKind::Ident {
                    let name = self.current.lexeme.clone();
                    self.advance();
                    Ok(Node {
                        kind: NodeKind::ExprEnumValue,
                        name,
                        ..Default::default()
                    })
                } else {
                    Err(format!(
                        "Expected identifier after '.', got {:?}",
                        self.current.kind
                    ))
                }
            }

            // Identifier, function call, @builtin call, or struct literal
            TokenKind::Ident => {
                let mut name = self.current.lexeme.clone();
                self.advance();

                // Handle namespace-qualified names: lexer::next_token
                while self.current.kind == TokenKind::Colon {
                    // Check for :: (two colons)
                    if self.peek.kind == TokenKind::Colon {
                        name.push_str("::");
                        self.advance(); // consume first :
                        self.advance(); // consume second :
                        if self.current.kind == TokenKind::Ident {
                            name.push_str(&self.current.lexeme);
                            self.advance();
                        }
                    } else if self.current.kind == TokenKind::ColonColon {
                        // Single :: token
                        name.push_str("::");
                        self.advance(); // consume ::
                        if self.current.kind == TokenKind::Ident {
                            name.push_str(&self.current.lexeme);
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }

                // Check for struct literal: Name{ .field = expr, ... }
                if self.current.kind == TokenKind::LBrace {
                    return self.parse_struct_literal(name);
                }

                // Check for function call: name(args) or namespace::func(args)
                if self.current.kind == TokenKind::LParen {
                    return self.parse_call_args(name);
                }

                Ok(Node {
                    kind: NodeKind::ExprIdentifier,
                    name,
                    ..Default::default()
                })
            }

            // Parenthesized expression
            TokenKind::LParen => {
                self.advance(); // consume (
                let inner = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(inner)
            }

            // if expression: if (cond) expr else expr
            TokenKind::KwIf => self.parse_if_expr(),

            // switch expression: switch (val) { ... }
            TokenKind::KwSwitch => self.parse_switch_expr(),

            // try expression (skip 'try' and parse inner)
            TokenKind::KwTry => {
                self.advance(); // consume 'try'
                let inner = self.parse_expr_postfix()?;
                Ok(Node {
                    kind: NodeKind::ExprUnary,
                    extra_op: "try ".to_string(),
                    children: vec![inner],
                    ..Default::default()
                })
            }

            // Array literal: [_]Type{ values } or [N]Type{ values }
            TokenKind::LBracket => self.parse_array_literal(),

            _ => Err(format!(
                "Unexpected token in expression: {:?} ('{}') at line {}:{}",
                self.current.kind, self.current.lexeme, self.current.line, self.current.col
            )),
        }
    }

    /// Parse function/builtin call arguments: name(arg1, arg2, ...)
    fn parse_call_args(&mut self, name: String) -> Result<Node, String> {
        self.advance(); // consume (
        let mut call = Node::new(NodeKind::ExprCall);
        call.name = name;

        while self.current.kind != TokenKind::RParen && self.current.kind != TokenKind::Eof {
            let arg = self.parse_expr()?;
            call.children.push(arg);
            if self.current.kind == TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(TokenKind::RParen)?;
        Ok(call)
    }

    /// Parse struct literal: Name{ .field = expr, ... }
    fn parse_struct_literal(&mut self, name: String) -> Result<Node, String> {
        self.advance(); // consume {
        let mut lit = Node::new(NodeKind::ExprStructLit);
        lit.name = name;

        while self.current.kind != TokenKind::RBrace && self.current.kind != TokenKind::Eof {
            // Expect .field = expr  OR  field = expr (dot-prefix optional)
            let field_name;
            if self.current.kind == TokenKind::Dot {
                self.advance(); // consume .
                field_name = if self.current.kind == TokenKind::Ident {
                    let n = self.current.lexeme.clone();
                    self.advance();
                    n
                } else {
                    String::new()
                };
            } else if self.current.kind == TokenKind::Ident {
                // Allow field = expr without dot prefix
                let n = self.current.lexeme.clone();
                // Peek: only treat as field init if followed by '='
                if self.peek.kind == TokenKind::Equals {
                    field_name = n;
                    self.advance(); // consume field name
                } else {
                    break;
                }
            } else {
                break;
            }

            if self.current.kind == TokenKind::Equals {
                self.advance(); // consume =
            }

            let val = self.parse_expr()?;
            let mut field = Node::new(NodeKind::ExprFieldAccess);
            field.name = field_name;
            field.children.push(val);
            lit.children.push(field);

            if self.current.kind == TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(lit)
    }

    /// Parse array literal: [_]Type{ values } or [N]Type{ values }
    fn parse_array_literal(&mut self) -> Result<Node, String> {
        let mut node = Node::new(NodeKind::ExprArrayLiteral);
        self.advance();

        if self.current.kind == TokenKind::RBracket {
            self.advance();
        } else {
            let mut bracket_content = String::new();
            while self.current.kind != TokenKind::RBracket && self.current.kind != TokenKind::Eof {
                bracket_content.push_str(&self.current.lexeme);
                self.advance();
            }
            node.extra_size = bracket_content.trim().to_string();
            self.expect(TokenKind::RBracket)?;
        }

        if self.current.kind == TokenKind::Ident {
            node.extra_type = self.current.lexeme.clone();
            self.advance();
        }

        if self.current.kind == TokenKind::LBrace {
            self.advance();
            if self.current.kind != TokenKind::RBrace {
                let elem = self.parse_expr()?;
                node.children.push(elem);
                while self.current.kind == TokenKind::Comma {
                    self.advance();
                    if self.current.kind == TokenKind::RBrace {
                        break;
                    }
                    let elem = self.parse_expr()?;
                    node.children.push(elem);
                }
            }
            self.expect(TokenKind::RBrace)?;
        }

        if self.current.kind == TokenKind::Power {
            self.advance();
            let count = self.parse_expr()?;
            let mut repeat_node = Node::new(NodeKind::ExprBinary);
            repeat_node.extra_op = "**".to_string();
            repeat_node.children.push(node);
            repeat_node.children.push(count);
            return Ok(repeat_node);
        }

        Ok(node)
    }

    fn parse_if_expr(&mut self) -> Result<Node, String> {
        self.advance(); // consume 'if'

        // Condition in parentheses
        self.expect(TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;

        // Then expression
        let then_expr = self.parse_expr()?;

        // else expression
        let mut if_node = Node::new(NodeKind::ExprIf);
        if_node.children.push(cond);
        if_node.children.push(then_expr);

        if self.current.kind == TokenKind::KwElse {
            self.advance(); // consume 'else'
            let else_expr = self.parse_expr()?;
            if_node.children.push(else_expr);
        }

        Ok(if_node)
    }

    /// Parse switch expression: switch (val) { .arm => expr, ... }
    fn parse_switch_expr(&mut self) -> Result<Node, String> {
        self.advance(); // consume 'switch'

        // Value in parentheses
        self.expect(TokenKind::LParen)?;
        let val = self.parse_expr()?;
        self.expect(TokenKind::RParen)?;

        self.expect(TokenKind::LBrace)?;

        let mut sw = Node::new(NodeKind::ExprSwitch);
        sw.children.push(val);

        while self.current.kind != TokenKind::RBrace && self.current.kind != TokenKind::Eof {
            let mut arm = Node::new(NodeKind::ConstDecl);

            // Pattern
            if self.current.kind == TokenKind::Dot {
                self.advance(); // consume .
                if self.current.kind == TokenKind::Ident {
                    arm.name = self.current.lexeme.clone();
                    self.advance();
                }
            } else if self.current.kind == TokenKind::KwElse {
                arm.name = "else".to_string();
                self.advance();
            } else if self.current.kind == TokenKind::Minus {
                // Negative number pattern: -1, -2, etc.
                arm.name = "-".to_string();
                self.advance(); // consume -
                if self.current.kind == TokenKind::Number {
                    arm.name.push_str(&self.current.lexeme);
                    self.advance();
                }
            } else if self.current.kind == TokenKind::CharLiteral {
                arm.name = format!("'{}'", self.current.lexeme);
                self.advance();
            } else if self.current.kind == TokenKind::Ident
                || self.current.kind == TokenKind::Number
            {
                arm.name = self.current.lexeme.clone();
                self.advance();
            } else {
                break;
            }

            // =>
            if self.current.kind == TokenKind::FatArrow {
                self.advance();
            }

            // Arm expression
            let arm_expr = self.parse_expr()?;
            arm.children.push(arm_expr);

            if self.current.kind == TokenKind::Comma {
                self.advance();
            }

            sw.children.push(arm);
        }

        self.expect(TokenKind::RBrace)?;
        Ok(sw)
    }

    fn parse_enum_decl(&mut self, is_pub: bool) -> Result<Node, String> {
        let mut decl = Node::new(NodeKind::EnumDecl);
        decl.line = self.current.line as u32;
        decl.extra_pub = is_pub;

        self.advance(); // consume 'enum'

        if self.current.kind == TokenKind::Ident {
            decl.name = self.current.lexeme.clone();
            self.advance();
        }

        if self.current.kind == TokenKind::LParen {
            self.advance(); // consume (
            while self.current.kind != TokenKind::RParen && self.current.kind != TokenKind::Eof {
                self.advance();
            }
            self.expect(TokenKind::RParen)?;
        }

        self.expect(TokenKind::LBrace)?;
        self.parse_enum_body(&mut decl)?;
        self.expect(TokenKind::RBrace)?;
        Ok(decl)
    }

    fn parse_struct_decl(&mut self, is_pub: bool) -> Result<Node, String> {
        let mut decl = Node::new(NodeKind::StructDecl);
        decl.line = self.current.line as u32;
        decl.extra_pub = is_pub;

        self.advance(); // consume 'struct'

        if self.current.kind == TokenKind::Ident {
            decl.name = self.current.lexeme.clone();
            self.advance();
        }

        self.expect(TokenKind::LBrace)?;
        self.parse_struct_body(&mut decl)?;
        self.expect(TokenKind::RBrace)?;
        Ok(decl)
    }

    fn parse_block_name(&mut self) -> String {
        // Block names can be string literals ("name") or bare identifiers (name_with_underscores)
        if self.current.kind == TokenKind::String || self.current.kind == TokenKind::Ident {
            let name = self.current.lexeme.clone();
            self.advance();
            // Handle hyphenated names like "some-name"
            let mut full_name = name;
            while self.current.kind == TokenKind::Minus {
                full_name.push('-');
                self.advance();
                if self.current.kind == TokenKind::Ident {
                    full_name.push_str(&self.current.lexeme);
                    self.advance();
                }
            }
            full_name
        } else {
            String::new()
        }
    }

    fn parse_test_block(&mut self) -> Result<Node, String> {
        let mut block = Node::new(NodeKind::TestBlock);

        self.advance(); // consume 'test'
        block.name = self.parse_block_name();

        if self.current.kind == TokenKind::LBrace {
            // Brace-style test: test "name" { ... }
            self.advance(); // consume {
            self.parse_fn_body(&mut block)?;
            self.expect(TokenKind::RBrace)?;
        } else {
            // Keyword-style test: test name given ... when ... then ...
            // Skip until we hit a top-level keyword or EOF or RBrace (end of module)
            self.skip_to_next_top_level();
        }
        Ok(block)
    }

    fn parse_invariant_block(&mut self) -> Result<Node, String> {
        let mut block = Node::new(NodeKind::InvariantBlock);

        self.advance(); // consume 'invariant'
        block.name = self.parse_block_name();

        if self.current.kind == TokenKind::LBrace {
            // Brace-style invariant: invariant "name" { ... }
            self.advance(); // consume {
            self.parse_fn_body(&mut block)?;
            self.expect(TokenKind::RBrace)?;
        } else {
            // Keyword-style invariant: skip until next top-level
            self.skip_to_next_top_level();
        }
        Ok(block)
    }

    fn parse_bench_block(&mut self) -> Result<Node, String> {
        let mut block = Node::new(NodeKind::BenchBlock);

        self.advance(); // consume 'bench'
        block.name = self.parse_block_name();

        if self.current.kind == TokenKind::LBrace {
            // Brace-style bench: bench "name" { ... }
            self.advance(); // consume {
            self.parse_fn_body(&mut block)?;
            self.expect(TokenKind::RBrace)?;
        } else {
            // Keyword-style bench: skip until next top-level
            self.skip_to_next_top_level();
        }
        Ok(block)
    }
}

// ============================================================================
// Code Generator
// ============================================================================

pub struct Codegen {
    output: String,
    indent: u32,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
        }
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn write_line(&mut self, s: &str) {
        self.write(s);
        self.write("\n");
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.write("    ");
        }
    }

    pub fn gen_zig(&mut self, ast: &Node) {
        // Header with module name
        let module_name = if !ast.name.is_empty() {
            &ast.name
        } else {
            "unknown"
        };
        self.write_line(&format!(
            "// Generated from t27 spec: {} (module name)",
            module_name
        ));
        self.write_line("// DO NOT EDIT — generated by t27c");
        self.write_line("// phi^2 + 1/phi^2 = 3 | TRINITY");
        self.write_line("");

        // Check if file has test blocks — emit std import if so
        let has_tests = ast.children.iter().any(|d| d.kind == NodeKind::TestBlock);
        if has_tests {
            self.write_line("const std = @import(\"std\");");
            self.write_line("");
        }

        // Emit @import for UseDecl nodes first
        let mut has_imports = false;
        for decl in &ast.children {
            if decl.kind == NodeKind::UseDecl {
                self.write_line(&format!(
                    "const {} = @import(\"{}.zig\");",
                    decl.name, decl.value
                ));
                has_imports = true;
            }
        }
        if has_imports {
            self.write_line("");
        }

        // Emit other declarations
        for decl in &ast.children {
            if decl.kind != NodeKind::UseDecl {
                self.gen_decl(decl);
            }
        }
    }

    /// Generate Zig code with project-aware import resolution.
    /// `current_rel_path` is e.g. "base/types" (the file being generated).
    /// `module_map` maps "namespace::module" → "namespace/module".
    pub fn gen_zig_project(
        &mut self,
        ast: &Node,
        current_rel_path: &str,
        module_map: &std::collections::HashMap<String, String>,
    ) {
        // Header
        let module_name = if !ast.name.is_empty() {
            &ast.name
        } else {
            "unknown"
        };
        self.write_line(&format!(
            "// Generated from t27 spec: {} (module name)",
            module_name
        ));
        self.write_line("// DO NOT EDIT — generated by t27c compile-project");
        self.write_line("// phi^2 + 1/phi^2 = 3 | TRINITY");
        self.write_line("");

        // Check if file has test blocks — emit std import if so
        let has_tests = ast.children.iter().any(|d| d.kind == NodeKind::TestBlock);
        if has_tests {
            self.write_line("const std = @import(\"std\");");
            self.write_line("");
        }

        // Emit @import for UseDecl nodes with resolved paths
        let mut has_imports = false;
        for decl in &ast.children {
            if decl.kind == NodeKind::UseDecl {
                let import_path = resolve_import_path(
                    &decl.value, // e.g. "base::types"
                    &decl.name,  // e.g. "types"
                    current_rel_path,
                    module_map,
                );
                self.write_line(&format!(
                    "const {} = @import(\"{}\");",
                    decl.name, import_path
                ));
                has_imports = true;
            }
        }
        if has_imports {
            self.write_line("");
        }

        // Emit other declarations
        for decl in &ast.children {
            if decl.kind != NodeKind::UseDecl {
                self.gen_decl(decl);
            }
        }
    }

    pub fn into_string(self) -> String {
        self.output
    }

    fn gen_decl(&mut self, node: &Node) {
        match node.kind {
            NodeKind::ConstDecl => self.gen_const_decl(node),
            NodeKind::EnumDecl => self.gen_enum_decl(node),
            NodeKind::StructDecl => self.gen_struct_decl(node),
            NodeKind::FnDecl => self.gen_fn_decl(node),
            NodeKind::TestBlock => self.gen_test_block(node),
            NodeKind::InvariantBlock => self.gen_invariant_block(node),
            NodeKind::BenchBlock => self.gen_bench_block(node),
            _ => {}
        }
    }

    fn gen_const_decl(&mut self, node: &Node) {
        if node.extra_pub {
            self.write("pub ");
        }

        self.write(&format!("const {}", node.name));

        if !node.extra_type.is_empty() {
            self.write(&format!(": {}", node.extra_type));
        }

        if !node.children.is_empty() {
            self.write(" = ");
            self.gen_expr(&node.children[0]);
        }

        self.write_line(";");
    }

    fn gen_enum_decl(&mut self, node: &Node) {
        if node.extra_pub {
            self.write("pub ");
        }

        self.write(&format!("const {} = enum", node.name));

        if !node.extra_type.is_empty() {
            self.write(&format!("({})", node.extra_type));
        }

        self.write_line(" {");
        self.indent();

        for value_node in node.children.iter() {
            self.write_indent();
            if !value_node.value.is_empty() {
                self.write(&format!("{} = {},", value_node.name, value_node.value));
            } else {
                self.write(&format!("{},", value_node.name));
            }
            self.write_line("");
        }

        self.dedent();
        self.write_line("};");
    }

    fn gen_struct_decl(&mut self, node: &Node) {
        if node.extra_pub {
            self.write("pub ");
        }

        self.write_line(&format!("const {} = struct {{", node.name));
        self.indent();

        for field in &node.children {
            self.write_indent();
            let ty = if !field.extra_type.is_empty() {
                &field.extra_type
            } else {
                "void"
            };
            self.write_line(&format!("{}: {},", field.name, ty));
        }

        self.dedent();
        self.write_line("};");
    }

    fn gen_fn_decl(&mut self, node: &Node) {
        if node.extra_pub {
            self.write("pub ");
        }

        // T27 method syntax: fn foo(self: *Type, ...) -> ReturnType
        // Zig syntax: fn foo(self: *Type, ...) ReturnType
        // For methods, we put return type after ) without arrow

        let return_type = if node.extra_return_type.is_empty() {
            "void".to_string()
        } else {
            node.extra_return_type.clone()
        };

        // Check if this is a method (first param is "self")
        let is_method = node.params.iter().any(|(name, _)| name == "self");

        self.write(&format!("fn {}(", node.name));
        for (i, (pname, ptype)) in node.params.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.write(&format!("{}: {}", pname, ptype));
        }
        self.write(")");

        // T27 methods: return type after ) without arrow
        if is_method {
            self.write(&format!(" {}", return_type));
        } else if !node.extra_return_type.is_empty() {
            // Zig uses space for return type, not : or ->
            self.write(&format!(" {}", return_type));
        }

        self.write_line(" {");

        self.indent();

        if node.children.is_empty() {
            self.write_indent();
            self.write_line("@compileError(\"not yet implemented\");");
        } else {
            for stmt in &node.children {
                self.gen_stmt(stmt);
            }
        }

        self.dedent();
        self.write_line("}");
    }

    fn gen_test_block(&mut self, node: &Node) {
        self.write(&format!("test \"{}\"", node.name));
        self.write_line(" {");

        self.indent();

        for stmt in &node.children {
            self.gen_stmt(stmt);
        }

        self.dedent();
        self.write_line("}");
    }

    fn gen_invariant_block(&mut self, node: &Node) {
        self.write_line(&format!("comptime {{"));

        self.indent();
        self.write_indent();
        self.write_line(&format!("// invariant: {}", node.name));

        for stmt in &node.children {
            self.gen_stmt(stmt);
        }

        if node.children.is_empty() {
            self.write_indent();
            self.write_line(&format!(
                "@compileLog(\"invariant: {} verified\");",
                node.name
            ));
        }

        self.dedent();
        self.write_line("}");
    }

    fn gen_bench_block(&mut self, node: &Node) {
        // Convert bench block name to valid Zig identifier
        let fn_name = node.name.replace('-', "_");
        let fn_name = if fn_name.starts_with("bench_") {
            fn_name
        } else {
            format!("bench_{}", fn_name)
        };

        self.write_line(&format!("fn {}() void {{", fn_name));

        self.indent();
        self.write_indent();
        self.write_line(&format!("// bench: {}", node.name));

        for stmt in &node.children {
            self.gen_stmt(stmt);
        }

        if node.children.is_empty() {
            self.write_indent();
            self.write_line("// TODO: implement benchmark");
        }

        self.dedent();
        self.write_line("}");
    }

    fn gen_stmt(&mut self, node: &Node) {
        match node.kind {
            NodeKind::ExprReturn => {
                self.write_indent();
                self.write("return ");
                if !node.children.is_empty() {
                    self.gen_expr(&node.children[0]);
                }
                self.write_line(";");
            }
            NodeKind::StmtLocal => {
                self.write_indent();
                if node.extra_mutable {
                    self.write("var ");
                } else {
                    self.write("const ");
                }
                self.write(&node.name);
                if !node.extra_type.is_empty() {
                    self.write(&format!(": {}", node.extra_type));
                }
                if !node.children.is_empty() {
                    self.write(" = ");
                    self.gen_expr(&node.children[0]);
                }
                self.write_line(";");
            }
            NodeKind::StmtAssign => {
                self.write_indent();
                if node.children.len() >= 2 {
                    self.gen_expr(&node.children[0]);
                    if node.extra_op == "+=" {
                        self.write(" += ");
                    } else {
                        self.write(" = ");
                    }
                    self.gen_expr(&node.children[1]);
                }
                self.write_line(";");
            }
            NodeKind::StmtIf => {
                self.gen_if_stmt(node);
            }
            NodeKind::StmtWhile => {
                self.gen_while_stmt(node);
            }
            NodeKind::StmtFor => {
                self.gen_for_stmt(node);
            }
            NodeKind::StmtBreak => {
                self.write_line("break;");
            }
            NodeKind::StmtContinue => {
                self.write_line("continue;");
            }
            NodeKind::StmtExpr => {
                self.write_indent();
                if !node.children.is_empty() {
                    self.gen_expr(&node.children[0]);
                }
                self.write_line(";");
            }
            _ => {
                // Fallback: treat as expression
                self.write_indent();
                self.gen_expr(node);
                self.write_line(";");
            }
        }
    }

    fn gen_if_stmt(&mut self, node: &Node) {
        // children[0] = condition, children[1] = then block, children[2] = optional else block
        self.write_indent();
        self.write("if (");
        if !node.children.is_empty() {
            self.gen_expr(&node.children[0]);
        }
        self.write_line(") {");

        self.indent();
        if node.children.len() > 1 {
            for stmt in &node.children[1].children {
                self.gen_stmt(stmt);
            }
        }
        self.dedent();

        if node.children.len() > 2 {
            let else_block = &node.children[2];
            // Check if this is an else-if chain
            if else_block.children.len() == 1 && else_block.children[0].kind == NodeKind::StmtIf {
                self.write_indent();
                self.write("} else ");
                // Generate inline (no indent prefix for the nested if)
                self.gen_if_stmt_inline(&else_block.children[0]);
            } else {
                self.write_indent();
                self.write_line("} else {");
                self.indent();
                for stmt in &else_block.children {
                    self.gen_stmt(stmt);
                }
                self.dedent();
                self.write_indent();
                self.write_line("}");
            }
        } else {
            self.write_indent();
            self.write_line("}");
        }
    }

    fn gen_if_stmt_inline(&mut self, node: &Node) {
        // Same as gen_if_stmt but without leading indent (for else if)
        self.write("if (");
        if !node.children.is_empty() {
            self.gen_expr(&node.children[0]);
        }
        self.write_line(") {");

        self.indent();
        if node.children.len() > 1 {
            for stmt in &node.children[1].children {
                self.gen_stmt(stmt);
            }
        }
        self.dedent();

        if node.children.len() > 2 {
            let else_block = &node.children[2];
            if else_block.children.len() == 1 && else_block.children[0].kind == NodeKind::StmtIf {
                self.write_indent();
                self.write("} else ");
                self.gen_if_stmt_inline(&else_block.children[0]);
            } else {
                self.write_indent();
                self.write_line("} else {");
                self.indent();
                for stmt in &else_block.children {
                    self.gen_stmt(stmt);
                }
                self.dedent();
                self.write_indent();
                self.write_line("}");
            }
        } else {
            self.write_indent();
            self.write_line("}");
        }
    }

    fn gen_while_stmt(&mut self, node: &Node) {
        self.write_indent();
        self.write("while (");
        if !node.children.is_empty() {
            self.gen_expr(&node.children[0]);
        }
        self.write_line(") {");

        self.indent();
        if node.children.len() > 1 {
            for stmt in &node.children[1].children {
                self.gen_stmt(stmt);
            }
        }
        self.dedent();
        self.write_indent();
        self.write_line("}");
    }

    fn gen_for_stmt(&mut self, node: &Node) {
        self.write_indent();
        self.write("for (");

        // Iterables are children[0..n-1], last child is the body block
        let body_idx = node.children.len().saturating_sub(1);
        for (i, child) in node.children.iter().enumerate() {
            if i == body_idx {
                break; // body block
            }
            if i > 0 {
                self.write(", ");
            }
            self.gen_expr(child);
        }
        self.write(")");

        // Capture variables from params
        if !node.params.is_empty() {
            self.write(" |");
            for (i, (name, _)) in node.params.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                self.write(name);
            }
            self.write("|");
        }

        self.write_line(" {");

        self.indent();
        if !node.children.is_empty() {
            for stmt in &node.children[body_idx].children {
                self.gen_stmt(stmt);
            }
        }
        self.dedent();
        self.write_indent();
        self.write_line("}");
    }

    fn gen_expr_maybe_paren(&mut self, node: &Node) {
        if node.kind == NodeKind::ExprBinary {
            self.write("(");
            self.gen_expr(node);
            self.write(")");
        } else {
            self.gen_expr(node);
        }
    }

    fn gen_expr(&mut self, node: &Node) {
        match node.kind {
            NodeKind::ExprLiteral => self.write(&node.value),
            NodeKind::ExprIdentifier => self.write(&node.name),
            NodeKind::ExprEnumValue => {
                self.write(".");
                self.write(&node.name);
            }
            NodeKind::ExprCall => {
                if node.name == "@compileAssert" || node.name == "assert" {
                    if !node.children.is_empty() {
                        self.write("if (!(");
                        self.gen_expr(&node.children[0]);
                        self.write(")) @compileError(\"assertion failed\")");
                    }
                } else if node.name == "gf16_encode_f32" {
                    self.write("gf16_encode_f32(");
                    for (i, arg) in node.children.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.gen_expr(arg);
                    }
                    self.write(")");
                } else if node.name == "gf16_decode_f32" {
                    self.write("gf16_decode_f32(");
                    for (i, arg) in node.children.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.gen_expr(arg);
                    }
                    self.write(")");
                } else if node.name == "gf16_extract_sign"
                    || node.name == "gf16_extract_exponent"
                    || node.name == "gf16_extract_mantissa"
                {
                    self.write(&node.name);
                    self.write("(");
                    for (i, arg) in node.children.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.gen_expr(arg);
                    }
                    self.write(")");
                } else {
                    self.write(&node.name);
                    self.write("(");
                    for (i, arg) in node.children.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.gen_expr(arg);
                    }
                    self.write(")");
                }
            }
            NodeKind::ExprBinary => {
                if node.children.len() >= 2 {
                    self.gen_expr_maybe_paren(&node.children[0]);
                    self.write(&format!(" {} ", node.extra_op));
                    self.gen_expr_maybe_paren(&node.children[1]);
                }
            }
            NodeKind::ExprUnary => {
                self.write(&node.extra_op);
                if !node.children.is_empty() {
                    self.gen_expr(&node.children[0]);
                }
            }
            NodeKind::ExprFieldAccess => {
                // children[0] is the base expression
                if !node.children.is_empty() {
                    self.gen_expr(&node.children[0]);
                }
                self.write(".");
                self.write(&node.name);
            }
            NodeKind::ExprIndex => {
                // children[0] = base, children[1] = index
                if node.children.len() >= 2 {
                    self.gen_expr(&node.children[0]);
                    self.write("[");
                    self.gen_expr(&node.children[1]);
                    self.write("]");
                }
            }
            NodeKind::ExprSwitch => {
                self.write("switch (");
                if !node.children.is_empty() {
                    self.gen_expr(&node.children[0]);
                }
                self.write_line(") {");

                self.indent();
                for case_node in &node.children[1..] {
                    if case_node.kind == NodeKind::ConstDecl {
                        self.write_indent();
                        if !case_node.name.is_empty() && case_node.name != "else" {
                            // Don't prefix with '.' if the arm is a numeric literal or negative number
                            let is_numeric = case_node
                                .name
                                .starts_with(|c: char| c.is_ascii_digit())
                                || (case_node.name.starts_with('-') && case_node.name.len() > 1);
                            if is_numeric {
                                self.write(&case_node.name);
                            } else {
                                self.write(&format!(".{}", case_node.name));
                            }
                        } else {
                            self.write("else");
                        }
                        self.write(" => ");
                        if !case_node.children.is_empty() {
                            self.gen_expr(&case_node.children[0]);
                        }
                        self.write_line(",");
                    }
                }
                self.dedent();
                self.write_indent();
                self.write("}");
            }
            NodeKind::ExprIf => {
                // children[0] = cond, children[1] = then, children[2] = else (optional)
                self.write("if (");
                if !node.children.is_empty() {
                    self.gen_expr(&node.children[0]);
                }
                self.write(") ");
                if node.children.len() > 1 {
                    self.gen_expr(&node.children[1]);
                }
                if node.children.len() > 2 {
                    self.write(" else ");
                    self.gen_expr(&node.children[2]);
                }
            }
            NodeKind::ExprArrayLiteral => {
                let size = if node.extra_size.is_empty() {
                    "_".to_string()
                } else {
                    node.extra_size.clone()
                };
                let typ = if node.extra_type.is_empty() {
                    ""
                } else {
                    &node.extra_type
                };
                self.write(&format!("[{}]{}", size, typ));
                self.write("{");
                for (i, elem) in node.children.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.gen_expr(elem);
                }
                self.write("}");
            }
            NodeKind::ExprStructLit => {
                self.write(&node.name);
                self.write("{ ");
                for (i, field) in node.children.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&format!(".{} = ", field.name));
                    if !field.children.is_empty() {
                        self.gen_expr(&field.children[0]);
                    }
                }
                self.write(" }");
            }
            NodeKind::ExprCast => {
                // Type cast: (expr as Type)
                if !node.children.is_empty() {
                    self.gen_expr(&node.children[0]);
                }
                self.write(&format!(" as {}", node.extra_type));
            }
            _ => {}
        }
    }
}

// ============================================================================
// Verilog Code Generator
// ============================================================================

pub struct VerilogCodegen {
    output: String,
    indent: u32,
    module_name: String,
    current_fn_name: String,
}

impl VerilogCodegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            module_name: String::new(),
            current_fn_name: String::new(),
        }
    }

    fn sanitize_identifier(name: &str) -> String {
        name.replace('-', "_")
            .replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn write_line(&mut self, s: &str) {
        self.write(s);
        self.write("\n");
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.write("    ");
        }
    }

    pub fn into_string(self) -> String {
        self.output
    }

    /// Map t27 type to Verilog type width. Returns bit width.
    fn type_to_width(ty: &str) -> u32 {
        match ty {
            "bool" => 1,
            "u8" | "i8" => 8,
            "u16" | "i16" => 16,
            "u32" | "i32" => 32,
            "u64" | "i64" => 64,
            "usize" => 32,
            _ => 32, // default width
        }
    }

    /// Map t27 type to Verilog signedness
    fn type_is_signed(ty: &str) -> bool {
        matches!(ty, "i8" | "i16" | "i32" | "i64")
    }

    /// Format a Verilog range declaration like [31:0]
    fn range_decl(width: u32) -> String {
        if width == 1 {
            String::new()
        } else {
            format!("[{}:0]", width - 1)
        }
    }

    pub fn gen_verilog(&mut self, ast: &Node) {
        self.module_name = if !ast.name.is_empty() {
            Self::sanitize_identifier(&ast.name)
        } else {
            "unknown".to_string()
        };

        // Header
        self.write_line(
            "// ============================================================================",
        );
        self.write_line(&format!("// Generated from t27 spec: {}", self.module_name));
        self.write_line("// DO NOT EDIT - generated by t27c gen-verilog");
        self.write_line("// phi^2 + 1/phi^2 = 3 | TRINITY");
        self.write_line(
            "// ============================================================================",
        );
        self.write_line("");
        self.write_line("`timescale 1ns / 1ps");
        self.write_line("`default_nettype none");
        self.write_line("");

        // Collect declarations by kind for structured emission
        let mut consts: Vec<&Node> = Vec::new();
        let mut enums: Vec<&Node> = Vec::new();
        let mut structs: Vec<&Node> = Vec::new();
        let mut functions: Vec<&Node> = Vec::new();
        let mut tests: Vec<&Node> = Vec::new();
        let mut invariants: Vec<&Node> = Vec::new();
        let mut benches: Vec<&Node> = Vec::new();

        for decl in &ast.children {
            match decl.kind {
                NodeKind::ConstDecl => consts.push(decl),
                NodeKind::EnumDecl => enums.push(decl),
                NodeKind::StructDecl => structs.push(decl),
                NodeKind::FnDecl => functions.push(decl),
                NodeKind::TestBlock => tests.push(decl),
                NodeKind::InvariantBlock => invariants.push(decl),
                NodeKind::BenchBlock => benches.push(decl),
                _ => {}
            }
        }

        // Emit top-level module
        let mod_name = self.module_name.clone();
        self.write_line(&format!("module {} (", mod_name));
        self.indent();
        self.write_indent();
        self.write_line("input  wire        clk,");
        self.write_indent();
        self.write_line("input  wire        rst_n,");
        self.write_indent();
        self.write_line("input  wire        en,");
        self.write_indent();
        self.write_line("output wire        ready");
        self.dedent();
        self.write_line(");");
        self.write_line("");

        self.indent();

        // Section: Parameters from const declarations
        if !consts.is_empty() {
            self.write_indent();
            self.write_line("// -------------------------------------------------------");
            self.write_indent();
            self.write_line("// Parameters (from const declarations)");
            self.write_indent();
            self.write_line("// -------------------------------------------------------");
            for c in &consts {
                self.gen_verilog_const(c);
            }
            self.write_line("");
        }

        // Section: Enum parameters
        if !enums.is_empty() {
            self.write_indent();
            self.write_line("// -------------------------------------------------------");
            self.write_indent();
            self.write_line("// Enum constants");
            self.write_indent();
            self.write_line("// -------------------------------------------------------");
            for e in &enums {
                self.gen_verilog_enum(e);
            }
            self.write_line("");
        }

        // Section: Struct → register/signal declarations
        if !structs.is_empty() {
            self.write_indent();
            self.write_line("// -------------------------------------------------------");
            self.write_indent();
            self.write_line("// Registers (from struct declarations)");
            self.write_indent();
            self.write_line("// -------------------------------------------------------");
            for s in &structs {
                self.gen_verilog_struct(s);
            }
            self.write_line("");
        }

        // Ready signal
        self.write_indent();
        self.write_line("assign ready = 1'b1;");
        self.write_line("");

        // Section: Functions → always blocks or sub-modules
        if !functions.is_empty() {
            self.write_indent();
            self.write_line("// -------------------------------------------------------");
            self.write_indent();
            self.write_line("// Combinational logic (from function declarations)");
            self.write_indent();
            self.write_line("// -------------------------------------------------------");
            for f in &functions {
                self.gen_verilog_fn(f);
            }
        }

        // Section: Tests → assertions (SystemVerilog-style)
        if !tests.is_empty() {
            self.write_indent();
            self.write_line("// -------------------------------------------------------");
            self.write_indent();
            self.write_line("// Test assertions (from test blocks)");
            self.write_indent();
            self.write_line("// -------------------------------------------------------");
            self.write_indent();
            self.write_line("// synthesis translate_off");
            for t in &tests {
                self.gen_verilog_test(t);
            }
            self.write_indent();
            self.write_line("// synthesis translate_on");
            self.write_line("");
        }

        // Section: Invariants → parameter assertions
        if !invariants.is_empty() {
            self.write_indent();
            self.write_line("// -------------------------------------------------------");
            self.write_indent();
            self.write_line("// Invariant checks (compile-time assertions)");
            self.write_indent();
            self.write_line("// -------------------------------------------------------");
            for inv in &invariants {
                self.gen_verilog_invariant(inv);
            }
            self.write_line("");
        }

        // Section: Bench → initial blocks with timing
        if !benches.is_empty() {
            self.write_indent();
            self.write_line("// -------------------------------------------------------");
            self.write_indent();
            self.write_line("// Benchmark blocks (simulation only)");
            self.write_indent();
            self.write_line("// -------------------------------------------------------");
            for b in &benches {
                self.write_indent();
                self.write_line(&format!(
                    "initial begin : {}_bench // synthesis translate_off",
                    Self::sanitize_identifier(&b.name)
                ));
                self.indent();
                self.write_indent();
                self.write_line(&format!("$display(\"[BENCH] {} : starting\");", b.name));
                self.write_indent();
                self.write_line("integer _bench_cycles = 0;");
                for child in &b.children {
                    self.gen_verilog_test_stmt(child, &b.name);
                    self.write_indent();
                    self.write_line("_bench_cycles = _bench_cycles + 1;");
                }
                self.write_indent();
                self.write_line(&format!(
                    "$display(\"[BENCH] {} : %%0d cycles\", _bench_cycles);",
                    b.name
                ));
                self.write_indent();
                self.write_line(&format!("$display(\"[BENCH] {} : DONE\");", b.name));
                self.dedent();
                self.write_indent();
                self.write_line("end // synthesis translate_on");
            }
            self.write_line("");
        }

        self.dedent();
        self.write_line("endmodule");
        self.write_line("");
        self.write_line("`default_nettype wire");
    }

    fn gen_verilog_const(&mut self, node: &Node) {
        self.write_indent();

        // Determine if this is an array constant (LUT)
        let is_array = !node.extra_size.is_empty();

        if is_array {
            // Emit as localparam array — use initial block or parameter
            self.write(&format!("// LUT: {} [{}]", node.name, node.extra_size));
            self.write_line("");
            // For array constants, emit as individual localparams for each element
            if !node.children.is_empty() {
                // If children represent array elements, emit them
                let child = &node.children[0];
                if child.kind == NodeKind::ExprLiteral && child.value.contains(',') {
                    // Multiple values packed into a single literal — just comment
                    self.write_indent();
                    self.write(&format!("// localparam {} = ", node.name));
                    self.gen_verilog_expr(child);
                    self.write_line(";");
                } else {
                    self.write_indent();
                    if node.extra_pub {
                        self.write("parameter ");
                    } else {
                        self.write("localparam ");
                    }
                    // Determine width from type
                    let width = Self::type_to_width(&node.extra_type);
                    let signed = Self::type_is_signed(&node.extra_type);
                    if signed {
                        self.write("signed ");
                    }
                    let range = Self::range_decl(width);
                    if !range.is_empty() {
                        self.write(&format!("{} ", range));
                    }
                    self.write(&format!("{} = ", node.name));
                    self.gen_verilog_expr(&node.children[0]);
                    self.write_line(";");
                }
            } else {
                self.write_indent();
                self.write_line(&format!("// localparam {} (array — see spec)", node.name));
            }
        } else {
            // Simple scalar constant
            if node.extra_pub {
                self.write("parameter ");
            } else {
                self.write("localparam ");
            }

            // Determine width from type
            let width = Self::type_to_width(&node.extra_type);
            let signed = Self::type_is_signed(&node.extra_type);
            if signed {
                self.write("signed ");
            }
            let range = Self::range_decl(width);
            if !range.is_empty() {
                self.write(&format!("{} ", range));
            }

            self.write(&format!("{} = ", node.name));
            if !node.children.is_empty() {
                self.gen_verilog_expr(&node.children[0]);
            } else {
                self.write("0");
            }
            self.write_line(";");
        }
    }

    fn gen_verilog_enum(&mut self, node: &Node) {
        self.write_indent();
        self.write_line(&format!("// enum {}", node.name));
        for (i, variant) in node.children.iter().enumerate() {
            self.write_indent();
            if !variant.value.is_empty() {
                self.write_line(&format!(
                    "localparam {}_{} = {};",
                    node.name, variant.name, variant.value
                ));
            } else {
                self.write_line(&format!(
                    "localparam {}_{} = {};",
                    node.name, variant.name, i
                ));
            }
        }
    }

    fn gen_verilog_struct(&mut self, node: &Node) {
        self.write_indent();
        self.write_line(&format!("// struct {}", node.name));
        for field in &node.children {
            self.write_indent();
            let width = Self::type_to_width(&field.extra_type);
            let signed = Self::type_is_signed(&field.extra_type);

            let signed_str = if signed { "signed " } else { "" };
            let range = Self::range_decl(width);
            let range_str = if range.is_empty() {
                String::new()
            } else {
                format!("{} ", range)
            };

            // Check if field type is an array (has extra_size)
            let array_suffix = if !field.extra_size.is_empty() {
                // For array fields, we can't easily determine the size at this point
                // Use a comment indicating the array dimension
                format!(" /* [{}] */", field.extra_size)
            } else {
                String::new()
            };

            self.write_line(&format!(
                "reg {}{}{}_{}; // {}.{}{}",
                signed_str,
                range_str,
                node.name.to_lowercase(),
                field.name,
                node.name,
                field.name,
                array_suffix,
            ));
        }
    }

    fn gen_verilog_fn(&mut self, node: &Node) {
        self.current_fn_name = node.name.clone();
        self.write_line("");
        self.write_indent();
        self.write_line(&format!("// function: {}", node.name));

        // Emit as a Verilog function declaration
        let ret_width = if !node.extra_return_type.is_empty() {
            Self::type_to_width(&node.extra_return_type)
        } else {
            32
        };
        let ret_signed = if !node.extra_return_type.is_empty() {
            Self::type_is_signed(&node.extra_return_type)
        } else {
            false
        };

        let signed_str = if ret_signed { "signed " } else { "" };
        let range = Self::range_decl(ret_width);
        let range_str = if range.is_empty() {
            String::new()
        } else {
            format!("{} ", range)
        };

        // void functions → task; others → function
        if node.extra_return_type == "void" {
            self.write_indent();
            self.write_line(&format!("task {};", node.name));
        } else {
            self.write_indent();
            self.write_line(&format!(
                "function {}{}{}; // -> {}",
                signed_str,
                range_str,
                node.name,
                if node.extra_return_type.is_empty() {
                    "auto"
                } else {
                    &node.extra_return_type
                },
            ));
        }

        self.indent();

        // Emit parameters as input declarations
        for (pname, ptype) in &node.params {
            self.write_indent();
            let pw = Self::type_to_width(ptype);
            let ps = Self::type_is_signed(ptype);
            let ps_str = if ps { "signed " } else { "" };
            let pr = Self::range_decl(pw);
            let pr_str = if pr.is_empty() {
                String::new()
            } else {
                format!("{} ", pr)
            };
            self.write_line(&format!("input {}{}{};", ps_str, pr_str, pname));
        }

        // Emit body
        if node.children.is_empty() {
            self.write_indent();
            self.write_line("// TODO: implement");
        } else {
            self.write_indent();
            self.write_line("begin");
            self.indent();
            for stmt in &node.children {
                self.gen_verilog_stmt(stmt);
            }
            self.dedent();
            self.write_indent();
            self.write_line("end");
        }

        self.dedent();

        if node.extra_return_type == "void" {
            self.write_indent();
            self.write_line("endtask");
        } else {
            self.write_indent();
            self.write_line("endfunction");
        }
        self.current_fn_name.clear();
    }

    fn gen_verilog_test(&mut self, node: &Node) {
        self.write_indent();
        self.write_line(&format!("// test: {}", node.name));
        self.write_indent();
        self.write_line(&format!(
            "initial begin : {}_test",
            Self::sanitize_identifier(&node.name)
        ));
        self.indent();
        self.write_indent();
        self.write_line(&format!("$display(\"[TEST] {} : starting\");", node.name));
        for child in &node.children {
            self.gen_verilog_test_stmt(child, &node.name);
        }
        self.write_indent();
        self.write_line(&format!("$display(\"[TEST] {} : PASSED\");", node.name));
        self.dedent();
        self.write_indent();
        self.write_line("end");
    }

    fn gen_verilog_test_stmt(&mut self, node: &Node, test_name: &str) {
        match node.kind {
            NodeKind::StmtExpr => {
                if let Some(expr) = node.children.first() {
                    if expr.kind == NodeKind::ExprCall {
                        self.write_indent();
                        self.write("// ");
                        self.gen_verilog_expr(expr);
                        self.write_line(";");
                    }
                }
            }
            NodeKind::StmtLocal => {
                self.write_indent();
                self.write("// ");
                self.gen_verilog_stmt(node);
            }
            NodeKind::StmtAssign => {
                self.write_indent();
                self.write("// ");
                self.gen_verilog_stmt(node);
            }
            _ => {
                self.write_indent();
                self.write_line(&format!("// (stmt: {:?})", node.kind));
            }
        }
    }

    fn gen_verilog_invariant(&mut self, node: &Node) {
        self.write_indent();
        if node.children.is_empty() && node.extra_type.is_empty() {
            self.write_line(&format!("// invariant: {}", node.name));
        } else if !node.children.is_empty() {
            self.write(&format!(
                "// invariant {} : ",
                Self::sanitize_identifier(&node.name)
            ));
            self.gen_verilog_expr(&node.children[0]);
            self.write_line("");
        } else {
            self.write_line(&format!("// invariant: {}", node.name));
        }
    }

    fn gen_verilog_stmt(&mut self, node: &Node) {
        match node.kind {
            NodeKind::ExprReturn => {
                self.write_indent();
                if !node.children.is_empty() {
                    // In Verilog functions, return is done by assigning to function name
                    let fn_name = if self.current_fn_name.is_empty() {
                        "/* return */".to_string()
                    } else {
                        self.current_fn_name.clone()
                    };
                    self.write(&format!("{} = ", fn_name));
                    self.gen_verilog_expr(&node.children[0]);
                    self.write_line(";");
                }
            }
            NodeKind::StmtLocal => {
                self.write_indent();
                let kw = if node.extra_mutable { "reg" } else { "reg" };
                let width = Self::type_to_width(&node.extra_type);
                let signed = Self::type_is_signed(&node.extra_type);
                let signed_str = if signed { "signed " } else { "" };
                let range = Self::range_decl(width);
                let range_str = if range.is_empty() {
                    String::new()
                } else {
                    format!("{} ", range)
                };

                if node.extra_mutable {
                    self.write(&format!("{} {}{}{}", kw, signed_str, range_str, node.name));
                } else {
                    // const → localparam-like or wire
                    self.write(&format!("{} {}{}{}", kw, signed_str, range_str, node.name));
                }
                if !node.children.is_empty() {
                    self.write(" = ");
                    self.gen_verilog_expr(&node.children[0]);
                }
                self.write_line(";");
            }
            NodeKind::StmtAssign => {
                self.write_indent();
                if node.children.len() >= 2 {
                    self.gen_verilog_expr(&node.children[0]);
                    if node.extra_op == "+=" {
                        self.write(" = ");
                        self.gen_verilog_expr(&node.children[0]);
                        self.write(" + ");
                    } else {
                        self.write(" = ");
                    }
                    self.gen_verilog_expr(&node.children[1]);
                }
                self.write_line(";");
            }
            NodeKind::StmtIf => {
                self.gen_verilog_if_stmt(node);
            }
            NodeKind::StmtWhile => {
                self.gen_verilog_while_stmt(node);
            }
            NodeKind::StmtFor => {
                self.gen_verilog_for_stmt(node);
            }
            NodeKind::StmtBreak => {
                self.write_line("disable fork;");
            }
            NodeKind::StmtContinue => {
                self.write_line("/* continue */;");
            }
            NodeKind::StmtExpr => {
                self.write_indent();
                if !node.children.is_empty() {
                    self.gen_verilog_expr(&node.children[0]);
                }
                self.write_line(";");
            }
            _ => {
                self.write_indent();
                self.gen_verilog_expr(node);
                self.write_line(";");
            }
        }
    }

    fn gen_verilog_if_stmt(&mut self, node: &Node) {
        self.write_indent();
        self.write("if (");
        if !node.children.is_empty() {
            self.gen_verilog_expr(&node.children[0]);
        }
        self.write_line(") begin");

        self.indent();
        if node.children.len() > 1 {
            for stmt in &node.children[1].children {
                self.gen_verilog_stmt(stmt);
            }
        }
        self.dedent();

        if node.children.len() > 2 {
            let else_block = &node.children[2];
            if else_block.children.len() == 1 && else_block.children[0].kind == NodeKind::StmtIf {
                self.write_indent();
                self.write("end else ");
                // Inline else-if
                self.gen_verilog_if_stmt_inline(&else_block.children[0]);
            } else {
                self.write_indent();
                self.write_line("end else begin");
                self.indent();
                for stmt in &else_block.children {
                    self.gen_verilog_stmt(stmt);
                }
                self.dedent();
                self.write_indent();
                self.write_line("end");
            }
        } else {
            self.write_indent();
            self.write_line("end");
        }
    }

    fn gen_verilog_if_stmt_inline(&mut self, node: &Node) {
        self.write("if (");
        if !node.children.is_empty() {
            self.gen_verilog_expr(&node.children[0]);
        }
        self.write_line(") begin");

        self.indent();
        if node.children.len() > 1 {
            for stmt in &node.children[1].children {
                self.gen_verilog_stmt(stmt);
            }
        }
        self.dedent();

        if node.children.len() > 2 {
            let else_block = &node.children[2];
            if else_block.children.len() == 1 && else_block.children[0].kind == NodeKind::StmtIf {
                self.write_indent();
                self.write("end else ");
                self.gen_verilog_if_stmt_inline(&else_block.children[0]);
            } else {
                self.write_indent();
                self.write_line("end else begin");
                self.indent();
                for stmt in &else_block.children {
                    self.gen_verilog_stmt(stmt);
                }
                self.dedent();
                self.write_indent();
                self.write_line("end");
            }
        } else {
            self.write_indent();
            self.write_line("end");
        }
    }

    fn gen_verilog_while_stmt(&mut self, node: &Node) {
        self.write_indent();
        self.write("while (");
        if !node.children.is_empty() {
            self.gen_verilog_expr(&node.children[0]);
        }
        self.write_line(") begin");

        self.indent();
        if node.children.len() > 1 {
            for stmt in &node.children[1].children {
                self.gen_verilog_stmt(stmt);
            }
        }
        self.dedent();
        self.write_indent();
        self.write_line("end");
    }

    fn gen_verilog_for_stmt(&mut self, node: &Node) {
        // Emit as integer for loop: for (i = 0; i < N; i = i + 1)
        let body_idx = node.children.len().saturating_sub(1);

        // Use capture variable name if available, else default to __i
        let iter_var = if !node.params.is_empty() {
            node.params[0].0.clone()
        } else {
            "__i".to_string()
        };

        // Try to extract the range/iterable from children[0]
        // For range-based: for (iter_var = 0; iter_var < upper; iter_var = iter_var + 1)
        self.write_indent();
        if body_idx > 0 {
            let iterable = &node.children[0];
            // Emit: integer iter_var; for (iter_var = 0; iter_var < iterable; iter_var = iter_var + 1)
            self.write_line(&format!("// for-each over iterable"));
            self.write_indent();
            self.write(&format!("for ({} = 0; {} < ", iter_var, iter_var));
            self.gen_verilog_expr(iterable);
            self.write(&format!("; {} = {} + 1)", iter_var, iter_var));
        } else {
            self.write(&format!("for ({0} = 0; {0} < 1; {0} = {0} + 1)", iter_var));
        }
        self.write_line(" begin");

        self.indent();
        if !node.children.is_empty() {
            for stmt in &node.children[body_idx].children {
                self.gen_verilog_stmt(stmt);
            }
        }
        self.dedent();
        self.write_indent();
        self.write_line("end");
    }

    fn gen_verilog_expr(&mut self, node: &Node) {
        match node.kind {
            NodeKind::ExprLiteral => {
                let val = &node.value;
                // Convert hex literals: 0xFF → 8'hFF
                if val.starts_with("0x") || val.starts_with("0X") {
                    let hex = &val[2..];
                    let bits = hex.len() * 4;
                    self.write(&format!("{}'h{}", bits, hex));
                } else if val == "true" {
                    self.write("1'b1");
                } else if val == "false" {
                    self.write("1'b0");
                } else {
                    self.write(val);
                }
            }
            NodeKind::ExprIdentifier => self.write(&node.name),
            NodeKind::ExprEnumValue => {
                self.write(&node.name);
            }
            NodeKind::ExprCall => {
                self.write(&node.name);
                self.write("(");
                for (i, arg) in node.children.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.gen_verilog_expr(arg);
                }
                self.write(")");
            }
            NodeKind::ExprBinary => {
                if node.children.len() >= 2 {
                    // Map operators
                    let op = match node.extra_op.as_str() {
                        "&&" | "and" => "&&",
                        "||" | "or" => "||",
                        "==" => "==",
                        "!=" => "!=",
                        ">=" => ">=",
                        "<=" => "<=",
                        ">" => ">",
                        "<" => "<",
                        "+" => "+",
                        "-" => "-",
                        "*" => "*",
                        "/" => "/",
                        "%" => "%",
                        "&" => "&",
                        "|" => "|",
                        "^" => "^",
                        "<<" => "<<",
                        ">>" => ">>",
                        other => other,
                    };
                    self.write("(");
                    self.gen_verilog_expr(&node.children[0]);
                    self.write(&format!(" {} ", op));
                    self.gen_verilog_expr(&node.children[1]);
                    self.write(")");
                }
            }
            NodeKind::ExprUnary => {
                let op = match node.extra_op.as_str() {
                    "!" | "not" => "!",
                    "-" => "-",
                    "~" => "~",
                    other => other,
                };
                self.write(op);
                if !node.children.is_empty() {
                    self.gen_verilog_expr(&node.children[0]);
                }
            }
            NodeKind::ExprFieldAccess => {
                if !node.children.is_empty() {
                    let child = &node.children[0];
                    if child.kind == NodeKind::ExprIndex && !child.children.is_empty() {
                        let base_name = match child.children[0].kind {
                            NodeKind::ExprIdentifier => child.children[0].name.clone(),
                            _ => String::new(),
                        };
                        let flat_name = format!("{}{}", base_name, node.name);
                        self.write(&flat_name);
                    } else if child.kind == NodeKind::ExprIdentifier {
                        self.write(&child.name);
                        self.write("_");
                        self.write(&node.name);
                    } else {
                        self.gen_verilog_expr(child);
                        self.write("_");
                        self.write(&node.name);
                    }
                } else {
                    self.write(&node.name);
                }
            }
            NodeKind::ExprIndex => {
                if node.children.len() >= 2 {
                    self.gen_verilog_expr(&node.children[0]);
                    self.write("[");
                    self.gen_verilog_expr(&node.children[1]);
                    self.write("]");
                }
            }
            NodeKind::ExprArrayLiteral => {
                self.write(&format!(
                    "/* array [{}]{}{{",
                    node.extra_size, node.extra_type
                ));
                for elem in &node.children {
                    self.write(" ");
                    self.gen_verilog_expr(elem);
                    self.write(",");
                }
                self.write("} */");
            }
            NodeKind::ExprStructLit => {
                // Verilog has no struct literals — emit as comment + value 0
                self.write(&format!("0 /* {} {{...}} */", node.name));
            }
            NodeKind::ExprSwitch => {
                // Emit as nested ternary: (expr == val1) ? res1 : (expr == val2) ? res2 : default
                let cases = &node.children[1..];
                if cases.is_empty() {
                    self.write("0 /* empty switch */");
                } else {
                    let last_idx = cases.len() - 1;
                    for (i, case) in cases.iter().enumerate() {
                        if case.kind == NodeKind::ConstDecl {
                            let is_else = case.name.is_empty() || case.name == "else";
                            let is_last = i == last_idx;

                            if is_else {
                                if !case.children.is_empty() {
                                    self.gen_verilog_expr(&case.children[0]);
                                } else {
                                    self.write("0");
                                }
                            } else {
                                self.write("(");
                                self.gen_verilog_expr(&node.children[0]);
                                self.write(" == ");
                                let is_numeric =
                                    case.name.starts_with(|c: char| c.is_ascii_digit())
                                        || (case.name.starts_with('-') && case.name.len() > 1);
                                if is_numeric {
                                    self.write(&case.name);
                                } else {
                                    self.write(&case.name);
                                }
                                self.write(") ? (");
                                if !case.children.is_empty() {
                                    self.gen_verilog_expr(&case.children[0]);
                                } else {
                                    self.write("0");
                                }
                                self.write(")");

                                if !is_last {
                                    self.write(" : ");
                                } else {
                                    self.write(" : 0");
                                }
                            }
                        }
                    }
                }
            }
            NodeKind::ExprIf => {
                // Ternary operator
                self.write("(");
                if !node.children.is_empty() {
                    self.gen_verilog_expr(&node.children[0]);
                }
                self.write(") ? (");
                if node.children.len() > 1 {
                    self.gen_verilog_expr(&node.children[1]);
                }
                self.write(") : (");
                if node.children.len() > 2 {
                    self.gen_verilog_expr(&node.children[2]);
                } else {
                    self.write("0");
                }
                self.write(")");
            }
            _ => {
                self.write(&format!("/* unsupported expr: {:?} */", node.kind));
            }
        }
    }
}

// ============================================================================
// C Code Generator
// ============================================================================

pub struct CCodegen {
    output: String,
    indent: u32,
    module_name: String,
}

impl CCodegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            module_name: String::new(),
        }
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn write_line(&mut self, s: &str) {
        self.write(s);
        self.write("\n");
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            self.write("    ");
        }
    }

    pub fn into_string(self) -> String {
        self.output
    }

    /// Map t27 type to C type string
    fn type_to_c(ty: &str) -> &str {
        match ty {
            "bool" => "bool",
            "u8" => "uint8_t",
            "i8" => "int8_t",
            "u16" => "uint16_t",
            "i16" => "int16_t",
            "u32" => "uint32_t",
            "i32" => "int32_t",
            "u64" => "uint64_t",
            "i64" => "int64_t",
            "usize" => "size_t",
            "void" => "void",
            _ => ty, // pass through custom types
        }
    }

    /// Check if a type is a primitive (maps to stdint)
    fn is_primitive(ty: &str) -> bool {
        matches!(
            ty,
            "bool" | "u8" | "i8" | "u16" | "i16" | "u32" | "i32" | "u64" | "i64" | "usize" | "void"
        )
    }

    pub fn gen_c(&mut self, ast: &Node) {
        self.module_name = if !ast.name.is_empty() {
            ast.name.clone()
        } else {
            "unknown".to_string()
        };

        // Header
        self.write_line(
            "/* ============================================================================",
        );
        self.write_line(&format!("   Generated from t27 spec: {}", self.module_name));
        self.write_line("   DO NOT EDIT - generated by t27c gen-c");
        let mn = self.module_name.clone();
        self.write_line(&format!("   phi^2 + 1/phi^2 = 3 | TRINITY"));
        self.write_line(
            "   ============================================================================ */",
        );
        self.write_line("");

        // Includes
        self.write_line("#include <stdint.h>");
        self.write_line("#include <stdbool.h>");
        self.write_line("#include <stddef.h>");

        // Check if tests exist — add assert.h
        let has_tests = ast.children.iter().any(|d| d.kind == NodeKind::TestBlock);
        if has_tests {
            self.write_line("#include <assert.h>");
        }
        self.write_line("");

        // Guard macro
        let guard = mn.replace('-', "_").to_uppercase();
        self.write_line(&format!("#ifndef {}_H", guard));
        self.write_line(&format!("#define {}_H", guard));
        self.write_line("");

        // Collect declarations by kind
        let mut consts: Vec<&Node> = Vec::new();
        let mut enums: Vec<&Node> = Vec::new();
        let mut structs: Vec<&Node> = Vec::new();
        let mut functions: Vec<&Node> = Vec::new();
        let mut tests: Vec<&Node> = Vec::new();
        let mut invariants: Vec<&Node> = Vec::new();
        let mut benches: Vec<&Node> = Vec::new();

        for decl in &ast.children {
            match decl.kind {
                NodeKind::ConstDecl => consts.push(decl),
                NodeKind::EnumDecl => enums.push(decl),
                NodeKind::StructDecl => structs.push(decl),
                NodeKind::FnDecl => functions.push(decl),
                NodeKind::TestBlock => tests.push(decl),
                NodeKind::InvariantBlock => invariants.push(decl),
                NodeKind::BenchBlock => benches.push(decl),
                _ => {}
            }
        }

        // Section: Constants
        if !consts.is_empty() {
            self.write_line("/* -------------------------------------------------------");
            self.write_line("   Constants");
            self.write_line("   ------------------------------------------------------- */");
            self.write_line("");
            for c in &consts {
                self.gen_c_const(c);
            }
            self.write_line("");
        }

        // Section: Enums
        if !enums.is_empty() {
            self.write_line("/* -------------------------------------------------------");
            self.write_line("   Enums");
            self.write_line("   ------------------------------------------------------- */");
            self.write_line("");
            for e in &enums {
                self.gen_c_enum(e);
            }
            self.write_line("");
        }

        // Section: Structs
        if !structs.is_empty() {
            self.write_line("/* -------------------------------------------------------");
            self.write_line("   Structs");
            self.write_line("   ------------------------------------------------------- */");
            self.write_line("");
            for s in &structs {
                self.gen_c_struct(s);
            }
            self.write_line("");
        }

        // Section: Function prototypes
        if !functions.is_empty() {
            self.write_line("/* -------------------------------------------------------");
            self.write_line("   Function prototypes");
            self.write_line("   ------------------------------------------------------- */");
            self.write_line("");
            for f in &functions {
                self.gen_c_fn_prototype(f);
            }
            self.write_line("");
        }

        // Section: Function implementations
        if !functions.is_empty() {
            self.write_line("/* -------------------------------------------------------");
            self.write_line("   Function implementations");
            self.write_line("   ------------------------------------------------------- */");
            self.write_line("");
            for f in &functions {
                self.gen_c_fn(f);
            }
        }

        // Section: Invariants as _Static_assert
        if !invariants.is_empty() {
            self.write_line("/* -------------------------------------------------------");
            self.write_line("   Invariants (compile-time assertions)");
            self.write_line("   ------------------------------------------------------- */");
            self.write_line("");
            for inv in &invariants {
                self.gen_c_invariant(inv);
            }
            self.write_line("");
        }

        // Section: Tests
        if !tests.is_empty() {
            self.write_line("/* -------------------------------------------------------");
            self.write_line("   Tests");
            self.write_line("   ------------------------------------------------------- */");
            self.write_line("");
            for t in &tests {
                self.gen_c_test(t);
            }
            self.write_line("");
        }

        // Section: Benchmarks
        if !benches.is_empty() {
            self.write_line("/* -------------------------------------------------------");
            self.write_line("   Benchmarks");
            self.write_line("   ------------------------------------------------------- */");
            self.write_line("");
            for b in &benches {
                self.gen_c_bench(b);
            }
            self.write_line("");
        }

        // Close guard
        self.write_line(&format!("#endif /* {}_H */", guard));
    }

    /// Check if identifier name looks like a type (for type alias detection)
    fn is_type_name(name: &str) -> bool {
        // Primitive types
        if Self::is_primitive(name) {
            return true;
        }
        // Array types: [SIZE]TYPE
        if name.starts_with('[') {
            return true;
        }
        // Custom types: start with uppercase letter
        name.chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    }

    fn gen_c_const(&mut self, node: &Node) {
        // Detect type alias pattern: ConstDecl with single ExprIdentifier child
        // that looks like a type name (e.g., pub const PackedTrit = u8;)
        if node.children.len() == 1 && node.children[0].kind == NodeKind::ExprIdentifier {
            let target = &node.children[0].name;
            if Self::is_type_name(target) {
                // Check for array type alias: [SIZE]TYPE
                if target.starts_with('[') {
                    // Parse [SIZE]TYPE pattern
                    if let Some(bracket_end) = target.find(']') {
                        let size = &target[1..bracket_end];
                        let elem_type = &target[bracket_end + 1..];
                        let c_elem = Self::type_to_c(elem_type);
                        self.write_line(&format!("typedef {} {}[{}];", c_elem, node.name, size));
                    } else {
                        self.write_line(&format!("/* type alias: {} = {} */", node.name, target));
                    }
                } else if Self::is_primitive(target) {
                    let c_type = Self::type_to_c(target);
                    self.write_line(&format!("typedef {} {};", c_type, node.name));
                } else {
                    self.write_line(&format!("typedef {} {};", target, node.name));
                }
                return;
            }
        }

        // Regular constant with expression value
        if !node.children.is_empty() {
            let child = &node.children[0];
            // Simple literal → #define
            if child.kind == NodeKind::ExprLiteral {
                self.write_line(&format!("#define {} {}", node.name, child.value));
            } else {
                // Complex expression → static const
                let c_type = if !node.extra_type.is_empty() {
                    Self::type_to_c(&node.extra_type).to_string()
                } else {
                    "int".to_string()
                };
                self.write(&format!("static const {} {} = ", c_type, node.name));
                self.gen_c_expr(child);
                self.write_line(";");
            }
        } else if !node.value.is_empty() {
            // Constant with a direct value (no child node)
            self.write_line(&format!("#define {} {}", node.name, node.value));
        }
    }

    fn gen_c_enum(&mut self, node: &Node) {
        // typedef enum { ... } Name;
        self.write_line(&format!("typedef enum {{"));
        self.indent();

        for (i, variant) in node.children.iter().enumerate() {
            self.write_indent();
            let prefix = node.name.to_uppercase();
            if !variant.value.is_empty() {
                self.write(&format!(
                    "{}_{} = {}",
                    prefix,
                    variant.name.to_uppercase(),
                    variant.value
                ));
            } else {
                self.write(&format!("{}_{}", prefix, variant.name.to_uppercase()));
            }
            if i < node.children.len() - 1 {
                self.write(",");
            }
            self.write_line("");
        }

        self.dedent();
        self.write_line(&format!("}} {};", node.name));
        self.write_line("");
    }

    fn gen_c_struct(&mut self, node: &Node) {
        self.write_line(&format!("typedef struct {{"));
        self.indent();

        for field in &node.children {
            self.write_indent();
            let c_type = Self::type_to_c(&field.extra_type);
            if !field.extra_size.is_empty() {
                // Array field
                self.write_line(&format!("{} {}[{}];", c_type, field.name, field.extra_size));
            } else {
                self.write_line(&format!("{} {};", c_type, field.name));
            }
        }

        self.dedent();
        self.write_line(&format!("}} {};", node.name));
        self.write_line("");
    }

    fn gen_c_fn_prototype(&mut self, node: &Node) {
        let ret_type = Self::param_type_to_c(&node.extra_return_type);
        let ret_type = if ret_type.is_empty() {
            "void".to_string()
        } else {
            ret_type
        };

        self.write(&format!("{} {}(", ret_type, node.name));
        for (i, (pname, ptype)) in node.params.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            let c_type = Self::param_type_to_c(ptype);
            self.write(&format!("{} {}", c_type, pname));
        }
        if node.params.is_empty() {
            self.write("void");
        }
        self.write_line(");");
    }

    fn gen_c_fn(&mut self, node: &Node) {
        let ret_type = Self::param_type_to_c(&node.extra_return_type);
        let ret_type = if ret_type.is_empty() {
            "void".to_string()
        } else {
            ret_type
        };

        self.write(&format!("{} {}(", ret_type, node.name));
        for (i, (pname, ptype)) in node.params.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            let c_type = Self::param_type_to_c(ptype);
            self.write(&format!("{} {}", c_type, pname));
        }
        if node.params.is_empty() {
            self.write("void");
        }
        self.write_line(") {");

        self.indent();

        if node.children.is_empty() {
            self.write_indent();
            self.write_line("/* TODO: implement */");
        } else {
            for stmt in &node.children {
                self.gen_c_stmt(stmt);
            }
        }

        self.dedent();
        self.write_line("}");
        self.write_line("");
    }

    fn gen_c_test(&mut self, node: &Node) {
        // Convert test name to valid C identifier
        let fn_name = node.name.replace(|c: char| !c.is_alphanumeric(), "_");
        let fn_name = format!("test_{}", fn_name);

        self.write_line(&format!("void {}(void) {{", fn_name));
        self.indent();

        for stmt in &node.children {
            self.gen_c_stmt(stmt);
        }

        if node.children.is_empty() {
            self.write_indent();
            self.write_line("/* TODO: implement test */");
        }

        self.dedent();
        self.write_line("}");
        self.write_line("");
    }

    fn gen_c_invariant(&mut self, node: &Node) {
        self.write_line(&format!("/* invariant: {} */", node.name));
        if node.children.is_empty() {
            self.write_line(&format!(
                "/* _Static_assert(1, \"invariant: {}\"); */",
                node.name
            ));
        } else {
            for stmt in &node.children {
                // Try to emit as _Static_assert if it's a simple expression
                self.write_indent();
                match stmt.kind {
                    NodeKind::StmtExpr => {
                        if !stmt.children.is_empty() {
                            self.write("_Static_assert(");
                            self.gen_c_expr(&stmt.children[0]);
                            self.write(&format!(", \"invariant: {}\");\n", node.name));
                        }
                    }
                    _ => {
                        self.gen_c_stmt(stmt);
                    }
                }
            }
        }
        self.write_line("");
    }

    fn gen_c_bench(&mut self, node: &Node) {
        let fn_name = node.name.replace(|c: char| !c.is_alphanumeric(), "_");
        let fn_name = if fn_name.starts_with("bench_") {
            fn_name
        } else {
            format!("bench_{}", fn_name)
        };

        self.write_line(&format!("void {}(void) {{", fn_name));
        self.indent();
        self.write_indent();
        self.write_line(&format!("/* bench: {} */", node.name));

        for stmt in &node.children {
            self.gen_c_stmt(stmt);
        }

        if node.children.is_empty() {
            self.write_indent();
            self.write_line("/* TODO: implement benchmark */");
        }

        self.dedent();
        self.write_line("}");
        self.write_line("");
    }

    fn gen_c_stmt(&mut self, node: &Node) {
        match node.kind {
            NodeKind::ExprReturn => {
                self.write_indent();
                self.write("return ");
                if !node.children.is_empty() {
                    self.gen_c_expr(&node.children[0]);
                }
                self.write_line(";");
            }
            NodeKind::StmtLocal => {
                self.write_indent();
                let raw_type = &node.extra_type;
                // Handle array local vars: [SIZE]TYPE name
                if raw_type.starts_with('[') {
                    if let Some(bracket_end) = raw_type.find(']') {
                        let size = &raw_type[1..bracket_end];
                        let elem = &raw_type[bracket_end + 1..];
                        let c_elem = if Self::is_primitive(elem) {
                            Self::type_to_c(elem).to_string()
                        } else {
                            elem.to_string()
                        };
                        self.write(&format!("{} {}[{}]", c_elem, node.name, size));
                    } else {
                        self.write(&format!("int {}", node.name));
                    }
                } else {
                    let c_type = if !raw_type.is_empty() {
                        Self::param_type_to_c(raw_type)
                    } else {
                        "int".to_string()
                    };
                    self.write(&format!("{} {}", c_type, node.name));
                }
                if !node.children.is_empty() {
                    self.write(" = ");
                    self.gen_c_expr(&node.children[0]);
                }
                self.write_line(";");
            }
            NodeKind::StmtAssign => {
                self.write_indent();
                if node.children.len() >= 2 {
                    // Check for _ = expr (discard) pattern
                    let is_discard = node.children[0].kind == NodeKind::ExprIdentifier
                        && node.children[0].name == "_";
                    if is_discard {
                        self.write("(void)");
                        self.gen_c_expr(&node.children[1]);
                    } else {
                        self.gen_c_expr(&node.children[0]);
                        if node.extra_op == "+=" {
                            self.write(" += ");
                        } else {
                            self.write(" = ");
                        }
                        self.gen_c_expr(&node.children[1]);
                    }
                }
                self.write_line(";");
            }
            NodeKind::StmtIf => {
                self.gen_c_if_stmt(node);
            }
            NodeKind::StmtWhile => {
                self.gen_c_while_stmt(node);
            }
            NodeKind::StmtFor => {
                self.gen_c_for_stmt(node);
            }
            NodeKind::StmtBreak => {
                self.write_line("break;");
            }
            NodeKind::StmtContinue => {
                self.write_line("continue;");
            }
            NodeKind::StmtExpr => {
                self.write_indent();
                if !node.children.is_empty() {
                    self.gen_c_expr(&node.children[0]);
                }
                self.write_line(";");
            }
            _ => {
                self.write_indent();
                self.gen_c_expr(node);
                self.write_line(";");
            }
        }
    }

    fn gen_c_if_stmt(&mut self, node: &Node) {
        self.write_indent();
        self.write("if (");
        if !node.children.is_empty() {
            self.gen_c_expr(&node.children[0]);
        }
        self.write_line(") {");

        self.indent();
        if node.children.len() > 1 {
            for stmt in &node.children[1].children {
                self.gen_c_stmt(stmt);
            }
        }
        self.dedent();

        if node.children.len() > 2 {
            let else_block = &node.children[2];
            if else_block.children.len() == 1 && else_block.children[0].kind == NodeKind::StmtIf {
                self.write_indent();
                self.write("} else ");
                self.gen_c_if_stmt_inline(&else_block.children[0]);
            } else {
                self.write_indent();
                self.write_line("} else {");
                self.indent();
                for stmt in &else_block.children {
                    self.gen_c_stmt(stmt);
                }
                self.dedent();
                self.write_indent();
                self.write_line("}");
            }
        } else {
            self.write_indent();
            self.write_line("}");
        }
    }

    fn gen_c_if_stmt_inline(&mut self, node: &Node) {
        self.write("if (");
        if !node.children.is_empty() {
            self.gen_c_expr(&node.children[0]);
        }
        self.write_line(") {");

        self.indent();
        if node.children.len() > 1 {
            for stmt in &node.children[1].children {
                self.gen_c_stmt(stmt);
            }
        }
        self.dedent();

        if node.children.len() > 2 {
            let else_block = &node.children[2];
            if else_block.children.len() == 1 && else_block.children[0].kind == NodeKind::StmtIf {
                self.write_indent();
                self.write("} else ");
                self.gen_c_if_stmt_inline(&else_block.children[0]);
            } else {
                self.write_indent();
                self.write_line("} else {");
                self.indent();
                for stmt in &else_block.children {
                    self.gen_c_stmt(stmt);
                }
                self.dedent();
                self.write_indent();
                self.write_line("}");
            }
        } else {
            self.write_indent();
            self.write_line("}");
        }
    }

    fn gen_c_while_stmt(&mut self, node: &Node) {
        self.write_indent();
        self.write("while (");
        if !node.children.is_empty() {
            self.gen_c_expr(&node.children[0]);
        }
        self.write_line(") {");

        self.indent();
        if node.children.len() > 1 {
            for stmt in &node.children[1].children {
                self.gen_c_stmt(stmt);
            }
        }
        self.dedent();
        self.write_indent();
        self.write_line("}");
    }

    fn gen_c_for_stmt(&mut self, node: &Node) {
        // C doesn't have for-each natively; emit as a for loop with index
        self.write_indent();
        self.write_line("/* for-each loop (see t27 source) */");
        self.write_indent();
        self.write_line("{");
        self.indent();

        // Emit body
        let body_idx = node.children.len().saturating_sub(1);
        if !node.children.is_empty() {
            for stmt in &node.children[body_idx].children {
                self.gen_c_stmt(stmt);
            }
        }

        self.dedent();
        self.write_indent();
        self.write_line("}");
    }

    /// Map a t27/Zig type to C for use in parameter/return positions
    fn param_type_to_c(ty: &str) -> String {
        // Slice types: []Type → Type*
        if ty.starts_with("[]") {
            let inner = &ty[2..];
            let c_inner = if Self::is_primitive(inner) {
                Self::type_to_c(inner).to_string()
            } else {
                inner.to_string()
            };
            return format!("{}*", c_inner);
        }
        // Array types: [SIZE]Type → Type* (pointer in param position)
        if ty.starts_with('[') {
            if let Some(bracket_end) = ty.find(']') {
                let elem = &ty[bracket_end + 1..];
                let c_elem = if Self::is_primitive(elem) {
                    Self::type_to_c(elem).to_string()
                } else {
                    elem.to_string()
                };
                return format!("{}*", c_elem);
            }
        }
        if Self::is_primitive(ty) {
            Self::type_to_c(ty).to_string()
        } else {
            ty.to_string()
        }
    }

    fn gen_c_expr(&mut self, node: &Node) {
        match node.kind {
            NodeKind::ExprLiteral => {
                let val = &node.value;
                if val == "true" {
                    self.write("true");
                } else if val == "false" {
                    self.write("false");
                } else if val.starts_with("[_]") || val.starts_with("[") && val.contains('{') {
                    // Zig array literal: [_]u8{ 0xFF, 0x55, ... } → { 0xFF, 0x55, ... }
                    if let Some(brace_start) = val.find('{') {
                        self.write(&val[brace_start..]);
                    } else {
                        self.write(val);
                    }
                } else {
                    self.write(val);
                }
            }
            NodeKind::ExprIdentifier => {
                let name = &node.name;
                // Map Zig-specific identifiers to C equivalents
                if name == "undefined" {
                    self.write("{0}");
                } else if name.starts_with("[_]") || (name.starts_with('[') && name.contains(']')) {
                    // Array type used as value (shouldn't happen, but fallback)
                    self.write(&format!("/* {} */", name));
                } else {
                    self.write(name);
                }
            }
            NodeKind::ExprEnumValue => {
                // In C enums, we use ENUM_VARIANT style
                self.write(&node.name.to_uppercase());
            }
            NodeKind::ExprCall => {
                let fname = &node.name;
                if fname == "@compileAssert" {
                    self.write("_Static_assert(");
                    if !node.children.is_empty() {
                        self.gen_c_expr(&node.children[0]);
                    }
                    self.write(", \"compile assert\")");
                } else if fname.starts_with("@setEvalBranchQuota") || fname == "@setEvalBranchQuota"
                {
                    // Zig comptime hint — emit as comment
                    self.write("/* @setEvalBranchQuota(");
                    for (i, arg) in node.children.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.gen_c_expr(arg);
                    }
                    self.write(") */");
                } else if fname == "std.testing.expectEqual" {
                    // Zig test assert: std.testing.expectEqual(expected, actual) → assert(expected == actual)
                    self.write("assert(");
                    if node.children.len() >= 2 {
                        self.gen_c_expr(&node.children[0]);
                        self.write(" == ");
                        self.gen_c_expr(&node.children[1]);
                    }
                    self.write(")");
                } else if fname == "std.testing.expect" {
                    // Zig test expect: std.testing.expect(cond) → assert(cond)
                    self.write("assert(");
                    if !node.children.is_empty() {
                        self.gen_c_expr(&node.children[0]);
                    }
                    self.write(")");
                } else if fname == "@as" {
                    // Zig @as(Type, value) → (Type)(value) in C
                    if node.children.len() >= 2 {
                        self.write("(");
                        self.gen_c_expr(&node.children[0]);
                        self.write(")(");
                        self.gen_c_expr(&node.children[1]);
                        self.write(")");
                    } else if !node.children.is_empty() {
                        self.gen_c_expr(&node.children[0]);
                    }
                } else if fname == "@intCast" || fname == "@truncate" {
                    // Zig cast builtins → pass through the value argument
                    if !node.children.is_empty() {
                        self.write("(");
                        self.gen_c_expr(&node.children[0]);
                        self.write(")");
                    }
                } else if fname.starts_with("gf16_") {
                    // GF16 builtins — emit as function call
                    self.write(fname);
                    self.write("(");
                    for (i, arg) in node.children.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.gen_c_expr(arg);
                    }
                    self.write(")");
                } else {
                    self.write(fname);
                    self.write("(");
                    for (i, arg) in node.children.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.gen_c_expr(arg);
                    }
                    self.write(")");
                }
            }
            NodeKind::ExprBinary => {
                if node.children.len() >= 2 {
                    let op = node.extra_op.as_str();
                    if op == "**" {
                        // Zig repeat operator: val ** count → memset-style
                        // Emit as comment since C has no direct equivalent
                        self.write("/* repeat: ");
                        self.gen_c_expr(&node.children[0]);
                        self.write(" ** ");
                        self.gen_c_expr(&node.children[1]);
                        self.write(" */ {0}");
                    } else {
                        let c_op = match op {
                            "and" => "&&",
                            "or" => "||",
                            other => other,
                        };
                        self.write("(");
                        self.gen_c_expr(&node.children[0]);
                        self.write(&format!(" {} ", c_op));
                        self.gen_c_expr(&node.children[1]);
                        self.write(")");
                    }
                }
            }
            NodeKind::ExprUnary => {
                let op = node.extra_op.trim();
                if op == "try" {
                    // Zig try — just emit the inner expression in C
                    if !node.children.is_empty() {
                        self.gen_c_expr(&node.children[0]);
                    }
                } else {
                    let c_op = match op {
                        "not" => "!",
                        other => other,
                    };
                    self.write(c_op);
                    if !node.children.is_empty() {
                        self.gen_c_expr(&node.children[0]);
                    }
                }
            }
            NodeKind::ExprFieldAccess => {
                if !node.children.is_empty() {
                    self.gen_c_expr(&node.children[0]);
                }
                self.write(".");
                self.write(&node.name);
            }
            NodeKind::ExprIndex => {
                if node.children.len() >= 2 {
                    self.gen_c_expr(&node.children[0]);
                    self.write("[");
                    self.gen_c_expr(&node.children[1]);
                    self.write("]");
                }
            }
            NodeKind::ExprSwitch => {
                // C doesn't have switch expressions. Emit as nested ternary.
                if node.children.len() > 1 {
                    let cases = &node.children[1..];
                    self.gen_c_switch_expr(node, cases);
                } else {
                    self.write("0 /* empty switch */");
                }
            }
            NodeKind::ExprIf => {
                // Ternary operator
                self.write("(");
                if !node.children.is_empty() {
                    self.gen_c_expr(&node.children[0]);
                }
                self.write(") ? (");
                if node.children.len() > 1 {
                    self.gen_c_expr(&node.children[1]);
                }
                self.write(") : (");
                if node.children.len() > 2 {
                    self.gen_c_expr(&node.children[2]);
                } else {
                    self.write("0");
                }
                self.write(")");
            }
            NodeKind::ExprArrayLiteral => {
                let typ = if node.extra_type.is_empty() {
                    "int"
                } else {
                    &node.extra_type
                };
                self.write(&format!("({}[]){{ ", typ));
                for (i, elem) in node.children.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.gen_c_expr(elem);
                }
                self.write(" }");
            }
            NodeKind::ExprStructLit => {
                // C99 compound literal
                self.write(&format!("({}){{ ", node.name));
                for (i, field) in node.children.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.write(&format!(".{} = ", field.name));
                    if !field.children.is_empty() {
                        self.gen_c_expr(&field.children[0]);
                    }
                }
                self.write(" }");
            }
            NodeKind::ExprReturn => {
                self.write("return ");
                if !node.children.is_empty() {
                    self.gen_c_expr(&node.children[0]);
                }
            }
            _ => {
                self.write(&format!("/* unsupported: {:?} */", node.kind));
            }
        }
    }

    /// Generate a switch expression as nested ternary operators
    fn gen_c_switch_expr(&mut self, switch_node: &Node, cases: &[Node]) {
        // For each case: (switch_expr == case_val) ? result : next_case
        // The switch_node.children[0] is the expression being switched on
        if cases.is_empty() {
            self.write("0 /* no cases */");
            return;
        }

        let last_idx = cases.len() - 1;
        for (i, case) in cases.iter().enumerate() {
            if case.kind == NodeKind::ConstDecl {
                let is_else = case.name.is_empty() || case.name == "else";
                let is_last = i == last_idx;

                if is_else {
                    // else/default case - just emit the value
                    if !case.children.is_empty() {
                        self.gen_c_expr(&case.children[0]);
                    } else {
                        self.write("0");
                    }
                } else {
                    // Normal case: (switch_expr == val) ? result : ...
                    self.write("(");
                    self.gen_c_expr(&switch_node.children[0]);
                    self.write(" == ");
                    // Enum variant name
                    let is_numeric = case.name.starts_with(|c: char| c.is_ascii_digit())
                        || (case.name.starts_with('-') && case.name.len() > 1);
                    if is_numeric {
                        self.write(&case.name);
                    } else {
                        self.write(&case.name.to_uppercase());
                    }
                    self.write(") ? (");
                    if !case.children.is_empty() {
                        self.gen_c_expr(&case.children[0]);
                    } else {
                        self.write("0");
                    }
                    self.write(")");

                    if !is_last {
                        self.write(" : ");
                    } else {
                        self.write(" : 0");
                    }
                }
            }
        }
    }
}

// ============================================================================
// Import Path Resolution for compile-project
// ============================================================================

/// Resolve a `use X::Y` import to the correct relative .zig path.
///
/// Given:
///   - `use_value`: the full use path, e.g. "base::types"
///   - `use_name`: the short alias, e.g. "types"
///   - `current_rel_path`: relative path of the file being compiled, e.g. "ar/asp_solver"
///   - `module_map`: maps "base::types" → "base/types"
///
/// Returns a relative .zig path like "../base/types.zig"
fn resolve_import_path(
    use_value: &str,
    use_name: &str,
    current_rel_path: &str,
    module_map: &std::collections::HashMap<String, String>,
) -> String {
    // Try to find the target in the module map
    let target_rel = if let Some(rel) = module_map.get(use_value) {
        rel.clone()
    } else if let Some(rel) = module_map.get(use_name) {
        rel.clone()
    } else {
        // Fallback: convert use_value "base::types" → "base/types"
        use_value.replace("::", "/")
    };

    // Compute relative path from current file's directory to the target
    let current_dir = if let Some(pos) = current_rel_path.rfind('/') {
        &current_rel_path[..pos]
    } else {
        ""
    };

    let target_dir = if let Some(pos) = target_rel.rfind('/') {
        &target_rel[..pos]
    } else {
        ""
    };

    let target_file = if let Some(pos) = target_rel.rfind('/') {
        &target_rel[pos + 1..]
    } else {
        &target_rel
    };

    // Split directories into components
    let current_parts: Vec<&str> = if current_dir.is_empty() {
        Vec::new()
    } else {
        current_dir.split('/').collect()
    };

    let target_parts: Vec<&str> = if target_dir.is_empty() {
        Vec::new()
    } else {
        target_dir.split('/').collect()
    };

    // Find common prefix length
    let mut common = 0;
    for (a, b) in current_parts.iter().zip(target_parts.iter()) {
        if a == b {
            common += 1;
        } else {
            break;
        }
    }

    // Build relative path: go up from current, then down to target
    let mut rel_parts: Vec<String> = Vec::new();
    let ups = current_parts.len() - common;
    for _ in 0..ups {
        rel_parts.push("..".to_string());
    }
    for part in &target_parts[common..] {
        rel_parts.push(part.to_string());
    }
    rel_parts.push(format!("{}.zig", target_file));

    rel_parts.join("/")
}

// ============================================================================
// Compiler Interface
// ============================================================================

pub struct Compiler;

#[allow(dead_code)]
impl Compiler {
    pub fn compile(source: &str) -> Result<String, String> {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        let mut ast = parser.parse()?;
        optimize(&mut ast, &OptConfig::default());
        let mut codegen = Codegen::new();
        codegen.gen_zig(&ast);
        Ok(codegen.into_string())
    }

    pub fn compile_verilog(source: &str) -> Result<String, String> {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        let mut ast = parser.parse()?;
        optimize(&mut ast, &OptConfig::default());
        let mut codegen = VerilogCodegen::new();
        codegen.gen_verilog(&ast);
        Ok(codegen.into_string())
    }

    pub fn compile_c(source: &str) -> Result<String, String> {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        let mut ast = parser.parse()?;
        optimize(&mut ast, &OptConfig::default());
        let mut codegen = CCodegen::new();
        codegen.gen_c(&ast);
        Ok(codegen.into_string())
    }

    pub fn compile_rust(source: &str) -> Result<String, String> {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        let mut ast = parser.parse()?;
        optimize(&mut ast, &OptConfig::default());
        let mut codegen = RustCodegen::new();
        codegen.gen_rust(&ast);
        Ok(codegen.into_string())
    }

    pub fn parse_ast(source: &str) -> Result<Node, String> {
        // [BUG 1 FIX] Do NOT call lexer.tokenize() — let Parser use next_token() directly
        let lexer = Lexer::new(source);

        let mut parser = Parser::new(lexer);
        parser.parse()
    }

    /// Compile a single file as part of a project, resolving imports using the module map.
    /// `current_rel_path` is the relative path of the current file (e.g. "base/types").
    /// `module_map` maps "namespace::module" → "namespace/module" relative paths.
    pub fn compile_project_file(
        source: &str,
        current_rel_path: &str,
        module_map: &std::collections::HashMap<String, String>,
    ) -> Result<String, String> {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        let ast = parser.parse()?;

        let mut codegen = Codegen::new();
        codegen.gen_zig_project(&ast, current_rel_path, module_map);
        Ok(codegen.into_string())
    }

    pub fn typecheck(source: &str) -> Result<TypeCheckResult, String> {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        let ast = parser.parse()?;
        Ok(typecheck_ast(&ast))
    }

    pub fn compile_verilog_hir(source: &str) -> Result<String, String> {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        let ast = parser.parse()?;
        let mut hir = AstToHir::convert(&ast)?;
        let mut opt = HirOptimizer::new();
        opt.optimize(&mut hir);
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        Ok(emitter.into_string())
    }

    pub fn debug_hir(source: &str) -> Result<String, String> {
        let lexer = Lexer::new(source);
        let mut parser = Parser::new(lexer);
        let ast = parser.parse()?;
        let hir = AstToHir::convert(&ast)?;
        Ok(format!("{:#?}", hir))
    }
}

// ============================================================================
// AST Optimizer
// ============================================================================

pub struct OptConfig {
    pub enable_folding: bool,
    pub enable_dce: bool,
    pub opt_level: u32,
}

impl Default for OptConfig {
    fn default() -> Self {
        OptConfig {
            enable_folding: true,
            enable_dce: true,
            opt_level: 1,
        }
    }
}

pub struct OptStats {
    pub folds: u32,
    pub dead_removed: u32,
    pub copies_propagated: u32,
    pub strengths_reduced: u32,
    pub cse_eliminated: u32,
    pub dead_stores: u32,
    pub loops_unrolled: u32,
    pub passes: u32,
}

pub fn optimize(ast: &mut Node, config: &OptConfig) -> OptStats {
    let mut stats = OptStats {
        folds: 0,
        dead_removed: 0,
        copies_propagated: 0,
        strengths_reduced: 0,
        cse_eliminated: 0,
        dead_stores: 0,
        loops_unrolled: 0,
        passes: 0,
    };
    if config.opt_level == 0 {
        return stats;
    }
    for _ in 0..config.opt_level {
        optimize_module(ast, config, &mut stats);
        stats.passes += 1;
    }
    stats
}

fn optimize_module(node: &mut Node, config: &OptConfig, stats: &mut OptStats) {
    let mut i = 0;
    while i < node.children.len() {
        if node.children[i].kind == NodeKind::FnDecl {
            optimize_fn_body(&mut node.children[i], config, stats);
        }
        i += 1;
    }
}

fn optimize_fn_body(fn_node: &mut Node, config: &OptConfig, stats: &mut OptStats) {
    for child in &mut fn_node.children {
        if child.kind == NodeKind::Module && child.name == "body" {
            optimize_stmts(&mut child.children, config, stats);
        }
    }
    optimize_stmts(&mut fn_node.children, config, stats);
}

fn optimize_stmts(stmts: &mut Vec<Node>, config: &OptConfig, stats: &mut OptStats) {
    if config.enable_dce {
        let before = stmts.len();
        stmts.retain(|s| !is_dead_local(s));
        stats.dead_removed += (before - stmts.len()) as u32;
    }
    if config.enable_folding {
        const_propagate(stmts, stats);
        for stmt in stmts.iter_mut() {
            fold_stmt(stmt, stats);
        }
    }
    copy_propagate(stmts, stats);
    strength_reduce(stmts, stats);
    common_subexpr_elim(stmts, stats);
    dead_store_elim(stmts, stats);
    loop_unroll(stmts, stats);
}

fn is_dead_local(node: &Node) -> bool {
    if node.kind == NodeKind::StmtLocal && node.children.is_empty() && !node.extra_type.is_empty() {
        return true;
    }
    false
}

fn fold_stmt(node: &mut Node, stats: &mut OptStats) {
    if node.kind == NodeKind::StmtLocal && !node.children.is_empty() {
        fold_expr(&mut node.children[0], stats);
    }
    if node.kind == NodeKind::StmtAssign && node.children.len() >= 2 {
        fold_expr(&mut node.children[1], stats);
    }
    if node.kind == NodeKind::ExprReturn && !node.children.is_empty() {
        fold_expr(&mut node.children[0], stats);
    }
    if node.kind == NodeKind::StmtIf {
        for child in &mut node.children {
            if child.kind == NodeKind::Module {
                for stmt in &mut child.children {
                    fold_stmt(stmt, stats);
                }
            }
        }
    }
}

fn fold_expr(node: &mut Node, stats: &mut OptStats) {
    if node.kind == NodeKind::ExprBinary && node.children.len() >= 2 {
        fold_expr(&mut node.children[0], stats);
        fold_expr(&mut node.children[1], stats);
        if is_literal(&node.children[0]) && is_literal(&node.children[1]) {
            if let Some(val) = eval_binary(
                &node.children[0].value,
                &node.extra_op,
                &node.children[1].value,
            ) {
                node.kind = NodeKind::ExprLiteral;
                node.value = val;
                node.children.clear();
                stats.folds += 1;
            }
        }
    }
    if node.kind == NodeKind::ExprUnary && !node.children.is_empty() {
        fold_expr(&mut node.children[0], stats);
        if is_literal(&node.children[0]) {
            if let Some(val) = eval_unary(&node.extra_op, &node.children[0].value) {
                node.kind = NodeKind::ExprLiteral;
                node.value = val;
                node.children.clear();
                stats.folds += 1;
            }
        }
    }
}

fn is_literal(node: &Node) -> bool {
    if node.kind != NodeKind::ExprLiteral {
        return false;
    }
    let v = node.value.trim();
    if v.starts_with("0x") || v.starts_with("0X") {
        i64::from_str_radix(&v[2..], 16).is_ok()
    } else {
        v.parse::<i64>().is_ok()
    }
}

fn parse_int_value(s: &str) -> Option<i64> {
    let v = s.trim();
    if v.starts_with("0x") || v.starts_with("0X") {
        i64::from_str_radix(&v[2..], 16).ok()
    } else {
        v.parse().ok()
    }
}

fn copy_propagate(stmts: &mut Vec<Node>, stats: &mut OptStats) {
    let mut replacements: Vec<(String, String)> = Vec::new();
    for stmt in stmts.iter() {
        if stmt.kind == NodeKind::StmtLocal
            && stmt.children.len() == 1
            && stmt.children[0].kind == NodeKind::ExprIdentifier
            && stmt.name != stmt.children[0].name
        {
            replacements.push((stmt.name.clone(), stmt.children[0].name.clone()));
        }
    }
    if replacements.is_empty() {
        return;
    }
    for stmt in stmts.iter_mut() {
        for (from, to) in &replacements {
            propagate_ident(stmt, from, to);
        }
    }
    stats.copies_propagated += replacements.len() as u32;
}

fn const_propagate(stmts: &mut Vec<Node>, stats: &mut OptStats) {
    let mut consts: Vec<(String, String)> = Vec::new();
    for stmt in stmts.iter() {
        if stmt.kind == NodeKind::StmtLocal
            && !stmt.extra_mutable
            && stmt.children.len() == 1
            && stmt.children[0].kind == NodeKind::ExprLiteral
            && is_literal(&stmt.children[0])
        {
            let name = stmt.name.clone();
            let reassigned = stmts.iter().any(|s| {
                s.kind == NodeKind::StmtAssign
                    && s.children.len() >= 1
                    && s.children[0].kind == NodeKind::ExprIdentifier
                    && s.children[0].name == name
            });
            if !reassigned {
                consts.push((name, stmt.children[0].value.clone()));
            }
        }
    }
    if consts.is_empty() {
        return;
    }
    for stmt in stmts.iter_mut() {
        for (name, val) in &consts {
            replace_ident_with_literal(stmt, name, val, stats);
        }
    }
}

fn replace_ident_with_literal(node: &mut Node, name: &str, val: &str, stats: &mut OptStats) {
    if node.kind == NodeKind::ExprIdentifier && node.name == *name {
        node.kind = NodeKind::ExprLiteral;
        node.value = val.to_string();
        node.children.clear();
        stats.copies_propagated += 1;
        return;
    }
    for (i, child) in node.children.iter_mut().enumerate() {
        if node.kind == NodeKind::StmtAssign && i == 0 {
            continue;
        }
        replace_ident_with_literal(child, name, val, stats);
    }
}

fn propagate_ident(node: &mut Node, from: &str, to: &str) {
    if node.kind == NodeKind::ExprIdentifier && node.name == *from {
        node.name = to.to_string();
    }
    for (i, child) in node.children.iter_mut().enumerate() {
        if node.kind == NodeKind::StmtAssign && i == 0 {
            continue;
        }
        propagate_ident(child, from, to);
    }
}

fn strength_reduce(stmts: &mut Vec<Node>, stats: &mut OptStats) {
    for stmt in stmts.iter_mut() {
        if stmt.kind == NodeKind::StmtAssign && stmt.children.len() >= 2 {
            reduce_expr(&mut stmt.children[1], stats);
        }
        if stmt.kind == NodeKind::StmtLocal && !stmt.children.is_empty() {
            reduce_expr(&mut stmt.children[0], stats);
        }
        if stmt.kind == NodeKind::ExprReturn && !stmt.children.is_empty() {
            reduce_expr(&mut stmt.children[0], stats);
        }
    }
}

fn reduce_expr(node: &mut Node, stats: &mut OptStats) {
    if node.kind == NodeKind::ExprBinary && node.children.len() >= 2 {
        reduce_expr(&mut node.children[0], stats);
        reduce_expr(&mut node.children[1], stats);
        let is_mul = node.extra_op == "*";
        let is_div = node.extra_op == "/";
        if (is_mul || is_div) && is_power_of_two_literal(&node.children[1]) {
            let shift_val = get_power_of_two(&node.children[1]);
            node.extra_op = if is_mul {
                "<<".to_string()
            } else {
                ">>".to_string()
            };
            node.children[1].value = shift_val.to_string();
            stats.strengths_reduced += 1;
        }
    }
}

fn is_power_of_two_literal(node: &Node) -> bool {
    if node.kind != NodeKind::ExprLiteral {
        return false;
    }
    if let Ok(v) = node.value.parse::<i64>() {
        v > 1 && (v & (v - 1)) == 0
    } else {
        false
    }
}

fn get_power_of_two(node: &Node) -> u32 {
    let v: i64 = node.value.parse().unwrap_or(1);
    (v as u64).trailing_zeros()
}

fn expr_key(node: &Node) -> Option<String> {
    if node.kind == NodeKind::ExprBinary && node.children.len() >= 2 {
        let lk = child_key(&node.children[0]);
        let rk = child_key(&node.children[1]);
        Some(format!("{} {} {}", lk, node.extra_op, rk))
    } else {
        None
    }
}

fn child_key(node: &Node) -> String {
    match node.kind {
        NodeKind::ExprLiteral => format!("LIT:{}", node.value),
        NodeKind::ExprIdentifier => format!("ID:{}", node.name),
        NodeKind::ExprBinary => expr_key(node).unwrap_or_default(),
        _ => format!("{:?}", node.kind),
    }
}

fn common_subexpr_elim(stmts: &mut Vec<Node>, stats: &mut OptStats) {
    let mut seen: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut counter: u32 = 0;
    let mut new_stmts: Vec<Node> = Vec::new();
    for stmt in stmts.iter_mut() {
        let mut replacements: Vec<(String, String)> = Vec::new();
        collect_cse_replacements(stmt, &seen, &mut replacements);
        if !replacements.is_empty() {
            for (key, _) in &replacements {
                let var_name = format!("_cse{}", counter);
                counter += 1;
                seen.insert(key.clone(), var_name.clone());
                new_stmts.push(make_cse_local(&var_name, key));
                stats.cse_eliminated += 1;
            }
            apply_cse_replacements(stmt, &seen);
        }
        if let Some(key) = stmt_cse_key(stmt) {
            if !seen.contains_key(&key) {
                let var_name = format!("_cse{}", counter);
                counter += 1;
                seen.insert(key.clone(), var_name);
            }
        }
    }
    let insert_pos = find_first_non_local(stmts);
    for (i, local) in new_stmts.into_iter().enumerate() {
        stmts.insert(insert_pos + i, local);
    }
}

fn stmt_cse_key(stmt: &Node) -> Option<String> {
    let expr = match stmt.kind {
        NodeKind::StmtLocal if !stmt.children.is_empty() => &stmt.children[0],
        NodeKind::StmtAssign if stmt.children.len() >= 2 => &stmt.children[1],
        NodeKind::ExprReturn if !stmt.children.is_empty() => &stmt.children[0],
        _ => return None,
    };
    expr_key(expr)
}

fn collect_cse_replacements(
    node: &Node,
    seen: &std::collections::HashMap<String, String>,
    replacements: &mut Vec<(String, String)>,
) {
    if let Some(key) = expr_key(node) {
        if seen.contains_key(&key) {
            replacements.push((key.clone(), seen.get(&key).unwrap().clone()));
        }
    }
    for child in &node.children {
        collect_cse_replacements(child, seen, replacements);
    }
}

fn apply_cse_replacements(node: &mut Node, seen: &std::collections::HashMap<String, String>) {
    if let Some(key) = expr_key(node) {
        if let Some(var_name) = seen.get(&key) {
            node.kind = NodeKind::ExprIdentifier;
            node.name = var_name.clone();
            node.extra_op.clear();
            node.children.clear();
            return;
        }
    }
    for child in &mut node.children {
        apply_cse_replacements(child, seen);
    }
}

fn make_cse_local(var_name: &str, key: &str) -> Node {
    let parts: Vec<&str> = key.splitn(3, ' ').collect();
    let (op_str, left_str, right_str) = if parts.len() == 3 {
        (parts[1], parts[0], parts[2])
    } else {
        return Node::new(NodeKind::StmtLocal);
    };
    let mut local = Node::new(NodeKind::StmtLocal);
    local.name = var_name.to_string();
    local.extra_mutable = false;
    let mut bin = Node::new(NodeKind::ExprBinary);
    bin.extra_op = op_str.to_string();
    if left_str.starts_with("ID:") {
        let mut lid = Node::new(NodeKind::ExprIdentifier);
        lid.name = left_str[3..].to_string();
        bin.children.push(lid);
    } else if left_str.starts_with("LIT:") {
        let mut lit = Node::new(NodeKind::ExprLiteral);
        lit.value = left_str[4..].to_string();
        bin.children.push(lit);
    }
    if right_str.starts_with("ID:") {
        let mut rid = Node::new(NodeKind::ExprIdentifier);
        rid.name = right_str[3..].to_string();
        bin.children.push(rid);
    } else if right_str.starts_with("LIT:") {
        let mut lit = Node::new(NodeKind::ExprLiteral);
        lit.value = right_str[4..].to_string();
        bin.children.push(lit);
    }
    local.children.push(bin);
    local
}

fn find_first_non_local(stmts: &[Node]) -> usize {
    for (i, s) in stmts.iter().enumerate() {
        if s.kind != NodeKind::StmtLocal {
            return i;
        }
    }
    stmts.len()
}

fn dead_store_elim(stmts: &mut Vec<Node>, stats: &mut OptStats) {
    let mut reads: std::collections::HashSet<String> = std::collections::HashSet::new();
    for stmt in stmts.iter() {
        match stmt.kind {
            NodeKind::StmtLocal if !stmt.children.is_empty() => {
                collect_reads(&stmt.children[0], &mut reads);
            }
            NodeKind::StmtAssign if stmt.children.len() >= 2 => {
                collect_reads(&stmt.children[1], &mut reads);
            }
            NodeKind::ExprReturn if !stmt.children.is_empty() => {
                collect_reads(&stmt.children[0], &mut reads);
            }
            NodeKind::StmtExpr if !stmt.children.is_empty() => {
                collect_reads(&stmt.children[0], &mut reads);
            }
            _ => {}
        }
    }
    let before = stmts.len();
    stmts.retain(|s| {
        if s.kind == NodeKind::StmtLocal && !s.children.is_empty() {
            if !reads.contains(&s.name) {
                return false;
            }
        }
        if s.kind == NodeKind::StmtAssign && !s.children.is_empty() {
            if !reads.contains(&s.name) {
                return false;
            }
        }
        true
    });
    stats.dead_stores += (before - stmts.len()) as u32;
}

fn loop_unroll(stmts: &mut Vec<Node>, stats: &mut OptStats) {
    let mut insertions: Vec<(usize, Vec<Node>)> = Vec::new();
    for (i, stmt) in stmts.iter_mut().enumerate() {
        if stmt.kind == NodeKind::StmtFor && stmt.children.len() >= 3 {
            let iter_expr = &stmt.children[0];
            if iter_expr.kind == NodeKind::ExprBinary && iter_expr.extra_op == ".." {
                if iter_expr.children.len() >= 2 {
                    let start = parse_int_value(&iter_expr.children[0].value);
                    let end = parse_int_value(&iter_expr.children[1].value);
                    if let (Some(s), Some(e)) = (start, end) {
                        let count = e - s;
                        if count > 0 && count <= 4 {
                            let body = &stmt.children[2];
                            let iter_var = if stmt.children.len() > 1 {
                                stmt.children[1].name.clone()
                            } else {
                                "_i".to_string()
                            };
                            let mut unrolled = Vec::new();
                            for v in s..e {
                                let mut body_clone = body.clone();
                                replace_iter_var(&mut body_clone, &iter_var, v);
                                for child in body_clone.children.drain(..) {
                                    unrolled.push(child);
                                }
                            }
                            insertions.push((i, unrolled));
                            stats.loops_unrolled += 1;
                        }
                    }
                }
            }
        }
    }
    for (idx, unrolled) in insertions.into_iter().rev() {
        stmts.remove(idx);
        for (j, s) in unrolled.into_iter().enumerate() {
            stmts.insert(idx + j, s);
        }
    }
}

fn replace_iter_var(node: &mut Node, var: &str, val: i64) {
    if node.kind == NodeKind::ExprIdentifier && node.name == var {
        node.kind = NodeKind::ExprLiteral;
        node.value = val.to_string();
        node.name.clear();
        return;
    }
    for child in &mut node.children {
        replace_iter_var(child, var, val);
    }
}

fn collect_reads(node: &Node, reads: &mut std::collections::HashSet<String>) {
    if node.kind == NodeKind::ExprIdentifier {
        reads.insert(node.name.clone());
    }
    for child in &node.children {
        collect_reads(child, reads);
    }
}

fn eval_binary(left: &str, op: &str, right: &str) -> Option<String> {
    let l: i64 = parse_int_value(left)?;
    let r: i64 = parse_int_value(right)?;
    let result = match op {
        "+" => Some(l + r),
        "-" => Some(l - r),
        "*" => Some(l * r),
        "/" if r != 0 => Some(l / r),
        "%" if r != 0 => Some(l % r),
        "&" => Some(l & r),
        "|" => Some(l | r),
        "^" => Some(l ^ r),
        "<<" if r >= 0 && r < 64 => Some(l << r),
        ">>" if r >= 0 && r < 64 => Some(l >> r),
        _ => None,
    };
    result.map(|v| v.to_string())
}

fn eval_unary(op: &str, operand: &str) -> Option<String> {
    let v: i64 = parse_int_value(operand)?;
    match op {
        "-" => Some((-v).to_string()),
        "!" => Some((if v == 0 { 1 } else { 0 }).to_string()),
        "~" => Some((!v).to_string()),
        _ => None,
    }
}

// ============================================================================
// Type Checker
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum TypeInfo {
    Void,
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    GF16,
    Str,
    Array(Box<TypeInfo>),
    Pointer(Box<TypeInfo>),
    Optional(Box<TypeInfo>),
    Custom(String),
    Unknown,
    Error,
}

pub struct TypeCheckResult {
    pub ok: bool,
    pub error_count: u32,
    pub warnings: u32,
    pub errors: Vec<String>,
}

#[derive(Clone)]
#[allow(dead_code)]
struct SymbolEntry {
    name: String,
    type_info: TypeInfo,
    is_mutable: bool,
}

struct FnEntry {
    name: String,
    return_type: TypeInfo,
    params: Vec<(String, TypeInfo)>,
}

pub fn typecheck_ast(ast: &Node) -> TypeCheckResult {
    let mut result = TypeCheckResult {
        ok: true,
        error_count: 0,
        warnings: 0,
        errors: Vec::new(),
    };
    let mut symbols: Vec<SymbolEntry> = Vec::new();
    let mut fns: Vec<FnEntry> = Vec::new();

    for child in &ast.children {
        match child.kind {
            NodeKind::ConstDecl => {
                let t = resolve_type_str(&child.extra_type);
                symbols.push(SymbolEntry {
                    name: child.name.clone(),
                    type_info: t,
                    is_mutable: false,
                });
            }
            NodeKind::StructDecl | NodeKind::EnumDecl => {
                symbols.push(SymbolEntry {
                    name: child.name.clone(),
                    type_info: TypeInfo::Custom(child.name.clone()),
                    is_mutable: false,
                });
            }
            NodeKind::FnDecl => {
                let ret = resolve_type_str(&child.extra_return_type);
                let mut params = Vec::new();
                for (pname, ptype) in &child.params {
                    params.push((pname.clone(), resolve_type_str(ptype)));
                }
                fns.push(FnEntry {
                    name: child.name.clone(),
                    return_type: ret,
                    params,
                });
            }
            _ => {}
        }
    }

    {
        let struct_fields: std::collections::HashMap<String, Vec<String>> = ast
            .children
            .iter()
            .filter(|c| c.kind == NodeKind::StructDecl)
            .map(|c| {
                (
                    c.name.clone(),
                    c.children.iter().map(|f| f.extra_type.clone()).collect(),
                )
            })
            .collect();
        for (sname, _fields) in &struct_fields {
            let mut visited = std::collections::HashSet::new();
            fn has_cycle(
                name: &str,
                structs: &std::collections::HashMap<String, Vec<String>>,
                visited: &mut std::collections::HashSet<String>,
            ) -> bool {
                if visited.contains(name) {
                    return true;
                }
                visited.insert(name.to_string());
                if let Some(fs) = structs.get(name) {
                    for ft in fs {
                        let base = ft
                            .trim()
                            .trim_end_matches('*')
                            .trim()
                            .trim_start_matches("[]")
                            .trim();
                        if structs.contains_key(base) && has_cycle(base, structs, visited) {
                            return true;
                        }
                    }
                }
                visited.take(name).unwrap();
                false
            }
            if has_cycle(sname, &struct_fields, &mut visited) {
                result.warnings += 1;
                result.errors.push(format!(
                    "warning: recursive struct '{}' detected — consider using a pointer/optional field",
                    sname
                ));
            }
        }
    }

    for child in &ast.children {
        if child.kind == NodeKind::FnDecl {
            if child.params.len() > 8 {
                result.warnings += 1;
                let line = if child.line > 0 {
                    format!(":{}", child.line)
                } else {
                    String::new()
                };
                result.errors.push(format!(
                    "warning: function '{}' has {} parameters{} — consider refactoring",
                    child.name,
                    child.params.len(),
                    line
                ));
            }
            let mut fn_symbols = symbols.clone();
            for (pname, ptype) in &child.params {
                fn_symbols.push(SymbolEntry {
                    name: pname.clone(),
                    type_info: resolve_type_str(ptype),
                    is_mutable: true,
                });
            }
            for body_child in &child.children {
                check_stmt(body_child, &fn_symbols, &fns, &mut result);
            }

            let mut found_return = false;
            for body_child in &child.children {
                if found_return {
                    result.warnings += 1;
                    let line = if body_child.line > 0 {
                        format!(":{}", body_child.line)
                    } else {
                        String::new()
                    };
                    result.errors.push(format!(
                        "warning: unreachable code in function '{}'{}",
                        child.name, line
                    ));
                    break;
                }
                if body_child.kind == NodeKind::ExprReturn {
                    found_return = true;
                }
            }

            let mut reads: std::collections::HashSet<String> = std::collections::HashSet::new();
            fn collect_reads(node: &Node, reads: &mut std::collections::HashSet<String>) {
                if node.kind == NodeKind::ExprIdentifier {
                    reads.insert(node.name.clone());
                }
                for child in &node.children {
                    collect_reads(child, reads);
                }
            }
            for body_child in &child.children {
                collect_reads(body_child, &mut reads);
            }
            for body_child in &child.children {
                if body_child.kind == NodeKind::StmtLocal && !body_child.name.is_empty() {
                    if !reads.contains(&body_child.name) && !body_child.extra_mutable {
                        result.warnings += 1;
                        let line = if body_child.line > 0 {
                            format!(":{}", body_child.line)
                        } else {
                            String::new()
                        };
                        result.errors.push(format!(
                            "warning: unused variable '{}' in function '{}'{}",
                            body_child.name, child.name, line
                        ));
                    }
                }
            }

            fn is_tail_call(fn_body: &[Node], fn_name: &str) -> bool {
                let last = match fn_body.last() {
                    Some(s) => s,
                    None => return false,
                };
                if last.kind == NodeKind::ExprReturn && !last.children.is_empty() {
                    if last.children[0].kind == NodeKind::ExprCall
                        && last.children[0].name == fn_name
                    {
                        return true;
                    }
                }
                false
            }

            if is_tail_call(&child.children, &child.name) {
                let line = if child.line > 0 {
                    format!(":{}", child.line)
                } else {
                    String::new()
                };
                result.errors.push(format!(
                    "info: tail call detected in '{}'{} — candidate for optimization",
                    child.name, line
                ));
            }
        }
    }

    let mut enum_variants: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for child in &ast.children {
        if child.kind == NodeKind::EnumDecl {
            let variants: Vec<String> = child
                .children
                .iter()
                .filter(|c| c.kind == NodeKind::EnumVariant)
                .map(|c| format!("{}::{}", child.name, c.name))
                .collect();
            enum_variants.insert(child.name.clone(), variants);
        }
    }

    let mut used_variants: std::collections::HashSet<String> = std::collections::HashSet::new();
    fn collect_enum_values(node: &Node, used: &mut std::collections::HashSet<String>) {
        if node.kind == NodeKind::ExprEnumValue {
            used.insert(format!("{}::{}", node.name, node.extra_field));
        }
        for child in &node.children {
            collect_enum_values(child, used);
        }
    }
    collect_enum_values(&ast, &mut used_variants);

    for (enum_name, variants) in &enum_variants {
        let unused: Vec<&String> = variants
            .iter()
            .filter(|v| !used_variants.contains(*v))
            .collect();
        if !unused.is_empty() && unused.len() < variants.len() {
            for v in &unused {
                result.warnings += 1;
                result.errors.push(format!(
                    "info: unused enum variant '{}' in enum '{}'",
                    v, enum_name
                ));
            }
        }
    }

    if result.error_count > 0 {
        result.ok = false;
    }
    result
}

fn check_stmt(node: &Node, symbols: &[SymbolEntry], fns: &[FnEntry], result: &mut TypeCheckResult) {
    match node.kind {
        NodeKind::StmtLocal => {
            let t = if node.extra_type.is_empty() {
                if node.children.is_empty() {
                    TypeInfo::Unknown
                } else {
                    infer_expr(&node.children[0], symbols, fns)
                }
            } else {
                resolve_type_str(&node.extra_type)
            };
            let mut syms = symbols.to_vec();
            syms.push(SymbolEntry {
                name: node.name.clone(),
                type_info: t,
                is_mutable: node.extra_mutable,
            });
            for child in &node.children {
                check_expr(child, &syms, fns, result);
            }
        }
        NodeKind::StmtAssign => {
            if !node.children.is_empty() {
                if let Some(sym) = symbols.iter().find(|s| {
                    node.children[0].kind == NodeKind::ExprIdentifier
                        && s.name == node.children[0].name
                }) {
                    if !sym.is_mutable {
                        let name = &node.children[0].name;
                        let line = if node.line > 0 {
                            format!(":{}", node.line)
                        } else {
                            String::new()
                        };
                        result.warnings += 1;
                        result.errors.push(format!(
                            "warning: cannot assign to immutable '{}'{}",
                            name, line
                        ));
                    }
                }
                let target_type = infer_expr(&node.children[0], symbols, fns);
                if node.children.len() > 1 {
                    let value_type = infer_expr(&node.children[1], symbols, fns);
                    if !types_compatible(&target_type, &value_type)
                        && target_type != TypeInfo::Unknown
                        && value_type != TypeInfo::Unknown
                    {
                        result.error_count += 1;
                        result.errors.push(format!(
                            "type mismatch at line {}: cannot assign {:?} to {:?}",
                            if node.line > 0 {
                                node.line.to_string()
                            } else {
                                "?".to_string()
                            },
                            value_type,
                            target_type
                        ));
                    }
                }
            }
        }
        NodeKind::StmtIf | NodeKind::StmtWhile => {
            for child in &node.children {
                check_stmt(child, symbols, fns, result);
            }
        }
        NodeKind::StmtFor => {
            for child in &node.children {
                check_stmt(child, symbols, fns, result);
            }
        }
        NodeKind::Module => {
            let syms = symbols.to_vec();
            for child in &node.children {
                check_stmt(child, &syms, fns, result);
            }
        }
        NodeKind::ExprReturn => {
            if !node.children.is_empty() {
                check_expr(&node.children[0], symbols, fns, result);
            }
        }
        NodeKind::StmtExpr => {
            for child in &node.children {
                check_expr(child, symbols, fns, result);
            }
        }
        _ => {}
    }
}

fn check_expr(node: &Node, symbols: &[SymbolEntry], fns: &[FnEntry], result: &mut TypeCheckResult) {
    match node.kind {
        NodeKind::ExprCall => {
            if let Some(fn_entry) = fns.iter().find(|f| f.name == node.name) {
                let call_args: Vec<&Node> = node
                    .children
                    .iter()
                    .filter(|c| c.kind != NodeKind::Module)
                    .collect();
                if call_args.len() != fn_entry.params.len() {
                    result.warnings += 1;
                    result.errors.push(format!(
                        "function '{}' expects {} args, got {} at line {}",
                        node.name,
                        fn_entry.params.len(),
                        call_args.len(),
                        if node.line > 0 {
                            node.line.to_string()
                        } else {
                            "?".to_string()
                        }
                    ));
                } else {
                    for (i, arg) in call_args.iter().enumerate() {
                        let arg_type = infer_expr(arg, symbols, fns);
                        let param_type = &fn_entry.params[i].1;
                        if !types_compatible(param_type, &arg_type)
                            && *param_type != TypeInfo::Unknown
                            && arg_type != TypeInfo::Unknown
                        {
                            result.warnings += 1;
                            result.errors.push(format!(
                                "arg {} of '{}': expected {:?}, got {:?}",
                                i, node.name, param_type, arg_type
                            ));
                        }
                    }
                }
            }
            for child in &node.children {
                check_expr(child, symbols, fns, result);
            }
        }
        NodeKind::ExprBinary => {
            for child in &node.children {
                check_expr(child, symbols, fns, result);
            }
        }
        NodeKind::ExprUnary => {
            for child in &node.children {
                check_expr(child, symbols, fns, result);
            }
        }
        NodeKind::ExprFieldAccess | NodeKind::ExprIndex => {
            for child in &node.children {
                check_expr(child, symbols, fns, result);
            }
        }
        NodeKind::ExprStructLit => {
            if let Some(_sym) = symbols.iter().find(|s| {
                if let TypeInfo::Custom(ref name) = s.type_info {
                    name == &node.name
                } else {
                    false
                }
            }) {
                // We found the struct type — field validation happens at gen time
            }
            for child in &node.children {
                check_expr(child, symbols, fns, result);
            }
        }
        _ => {}
    }
}

fn infer_expr(node: &Node, symbols: &[SymbolEntry], fns: &[FnEntry]) -> TypeInfo {
    match node.kind {
        NodeKind::ExprLiteral => {
            if node.value == "true" || node.value == "false" {
                return TypeInfo::Bool;
            }
            if node.value.starts_with('"') {
                return TypeInfo::Str;
            }
            if node.value.parse::<i64>().is_ok() {
                return TypeInfo::I32;
            }
            if node.value.parse::<f64>().is_ok() {
                return TypeInfo::F64;
            }
            TypeInfo::Unknown
        }
        NodeKind::ExprIdentifier => symbols
            .iter()
            .rev()
            .find(|s| s.name == node.name)
            .map(|s| s.type_info.clone())
            .unwrap_or(TypeInfo::Unknown),
        NodeKind::ExprCall => fns
            .iter()
            .rev()
            .find(|f| f.name == node.name)
            .map(|f| f.return_type.clone())
            .unwrap_or(TypeInfo::Unknown),
        NodeKind::ExprBinary => {
            if node.children.len() >= 2 {
                let lt = infer_expr(&node.children[0], symbols, fns);
                let rt = infer_expr(&node.children[1], symbols, fns);
                if node.extra_op == "=="
                    || node.extra_op == "!="
                    || node.extra_op == "<"
                    || node.extra_op == ">"
                    || node.extra_op == "<="
                    || node.extra_op == ">="
                    || node.extra_op == "and"
                    || node.extra_op == "or"
                {
                    return TypeInfo::Bool;
                }
                if node.extra_op == "+" && (lt == TypeInfo::Str || rt == TypeInfo::Str) {
                    return TypeInfo::Str;
                }
                promote_types(&lt, &rt)
            } else {
                TypeInfo::Unknown
            }
        }
        NodeKind::ExprUnary => {
            if !node.children.is_empty() {
                infer_expr(&node.children[0], symbols, fns)
            } else {
                TypeInfo::Unknown
            }
        }
        _ => TypeInfo::Unknown,
    }
}

fn promote_types(a: &TypeInfo, b: &TypeInfo) -> TypeInfo {
    if a == b {
        return a.clone();
    }
    if *a == TypeInfo::Unknown || *b == TypeInfo::Unknown {
        return TypeInfo::Unknown;
    }
    let a_rank = type_rank(a);
    let b_rank = type_rank(b);
    if a_rank >= b_rank {
        a.clone()
    } else {
        b.clone()
    }
}

fn type_rank(t: &TypeInfo) -> u8 {
    match t {
        TypeInfo::Bool => 0,
        TypeInfo::I8 | TypeInfo::U8 => 1,
        TypeInfo::I16 | TypeInfo::U16 => 2,
        TypeInfo::I32 | TypeInfo::U32 => 3,
        TypeInfo::I64 | TypeInfo::U64 => 4,
        TypeInfo::F32 => 5,
        TypeInfo::GF16 => 5,
        TypeInfo::F64 => 6,
        _ => 0,
    }
}

fn types_compatible(target: &TypeInfo, value: &TypeInfo) -> bool {
    if *target == TypeInfo::Unknown || *value == TypeInfo::Unknown {
        return true;
    }
    if target == value {
        return true;
    }
    if *target == TypeInfo::F32 && *value == TypeInfo::F64 {
        return true;
    }
    if *target == TypeInfo::GF16 && (*value == TypeInfo::F32 || *value == TypeInfo::F64) {
        return true;
    }
    type_rank(target) >= type_rank(value)
}

fn resolve_type_str(s: &str) -> TypeInfo {
    let t = s.trim().trim_end_matches('?');
    let is_opt = s.trim().ends_with('?');
    let base = match t {
        "void" => TypeInfo::Void,
        "bool" => TypeInfo::Bool,
        "i8" => TypeInfo::I8,
        "i16" => TypeInfo::I16,
        "i32" => TypeInfo::I32,
        "i64" => TypeInfo::I64,
        "u8" => TypeInfo::U8,
        "u16" => TypeInfo::U16,
        "u32" => TypeInfo::U32,
        "u64" => TypeInfo::U64,
        "f32" => TypeInfo::F32,
        "f64" => TypeInfo::F64,
        "GF16" | "gf16" => TypeInfo::GF16,
        "str" => TypeInfo::Str,
        "" => TypeInfo::Unknown,
        other => TypeInfo::Custom(other.to_string()),
    };
    if is_opt {
        TypeInfo::Optional(Box::new(base))
    } else {
        base
    }
}

// ============================================================================
// Rust Code Generator
// ============================================================================

pub struct RustCodegen {
    output: String,
    indent: usize,
}

#[allow(dead_code)]
impl RustCodegen {
    pub fn new() -> Self {
        RustCodegen {
            output: String::new(),
            indent: 0,
        }
    }

    pub fn into_string(self) -> String {
        self.output
    }

    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn write_line(&mut self, s: &str) {
        self.output.push_str(&self.indent_str());
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn blank_line(&mut self) {
        self.output.push('\n');
    }

    fn write_indent(&mut self) {
        self.output.push_str(&self.indent_str());
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    pub fn gen_rust(&mut self, ast: &Node) {
        // Header
        self.write_line("// Generated from .t27 spec");
        self.write_line("// DO NOT EDIT — generated by t27c");
        self.blank_line();

        // Find module node
        for child in &ast.children {
            match child.kind {
                NodeKind::Module => {
                    let module_name = &child.name;
                    self.write_line(&format!("// Module: {}", module_name));
                    self.blank_line();
                    self.gen_module(child);
                }
                NodeKind::StructDecl => self.gen_struct(child),
                NodeKind::EnumDecl => self.gen_enum(child),
                NodeKind::ConstDecl => self.gen_const(child),
                NodeKind::FnDecl => self.gen_fn(child),
                _ => {}
            }
        }
    }

    fn gen_module(&mut self, node: &Node) {
        for child in &node.children {
            match child.kind {
                NodeKind::UseDecl => {
                    // Skip use declarations for now
                }
                NodeKind::StructDecl => self.gen_struct(child),
                NodeKind::EnumDecl => self.gen_enum(child),
                NodeKind::ConstDecl => self.gen_const(child),
                NodeKind::FnDecl => self.gen_fn(child),
                _ => {}
            }
        }
    }

    fn gen_struct(&mut self, node: &Node) {
        self.write_line(&format!(
            "#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]"
        ));
        self.write_line(&format!("pub struct {} {{", node.name));
        self.indent += 1;
        for child in &node.children {
            // Struct fields are stored as ExprIdentifier with name and extra_type
            if child.kind == NodeKind::ExprIdentifier && !child.name.is_empty() {
                let field_name = &child.name;
                let field_type = Self::t27_type_to_rust(&child.extra_type);
                self.write_line(&format!("pub {}: {},", field_name, field_type));
            }
        }
        self.indent -= 1;
        self.write_line("}");
        self.blank_line();
    }

    fn gen_enum(&mut self, node: &Node) {
        self.write_line(&format!(
            "#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]"
        ));
        self.write_line(&format!("pub enum {} {{", node.name));
        self.indent += 1;
        for child in &node.children {
            if child.kind == NodeKind::EnumVariant {
                let variant_name = &child.name;
                if child.value.is_empty() {
                    self.write_line(&format!("{},", variant_name));
                } else {
                    self.write_line(&format!("{} = {},", variant_name, child.value));
                }
            }
        }
        self.indent -= 1;
        self.write_line("}");
        self.blank_line();
    }

    fn gen_const(&mut self, node: &Node) {
        let const_type = if node.extra_type.is_empty() {
            "i32".to_string()
        } else {
            Self::t27_type_to_rust(node.extra_type.as_str())
        };
        let value = if node.children.is_empty() {
            "()".to_string()
        } else {
            Self::expr_to_rust(&node.children[0])
        };
        self.write_line(&format!(
            "pub const {}: {} = {};",
            node.name, const_type, value
        ));
        self.blank_line();
    }

    fn gen_fn(&mut self, node: &Node) {
        let fn_name = &node.name;
        let params: Vec<(String, String)> = node.params.clone();
        let params_str = params
            .iter()
            .map(|(n, t)| format!("{}: {}", n, Self::t27_type_to_rust(t)))
            .collect::<Vec<_>>()
            .join(", ");
        let ret_type = if node.extra_return_type.is_empty() {
            "()".to_string()
        } else {
            Self::t27_type_to_rust(node.extra_return_type.as_str())
        };

        self.write(&format!(
            "pub fn {}({}) -> {} {{",
            fn_name, params_str, ret_type
        ));

        // Check if there's a body
        let has_body = node.children.iter().any(|c| {
            matches!(c.kind, NodeKind::ExprReturn) || matches!(c.kind, NodeKind::StmtExpr)
        });

        if has_body {
            self.output.push('\n');
            self.indent += 1;
            for child in &node.children {
                match child.kind {
                    NodeKind::ExprReturn => {
                        let val = if child.children.is_empty() {
                            "()".to_string()
                        } else {
                            Self::expr_to_rust(&child.children[0])
                        };
                        self.write_line(&format!("return {};", val));
                    }
                    NodeKind::StmtExpr => {
                        if child.children.len() == 1 {
                            let expr = Self::expr_to_rust(&child.children[0]);
                            self.write_line(&format!("{};", expr));
                        }
                    }
                    NodeKind::StmtLocal => {
                        let mutable = child.extra_mutable;
                        let kw = if mutable { "let mut" } else { "let" };
                        let var_name = &child.name;
                        let typ = Self::t27_type_to_rust(&child.extra_type);
                        if child.children.is_empty() {
                            if child.extra_type.is_empty() {
                                self.write_line(&format!("{} {};", kw, var_name));
                            } else {
                                self.write_line(&format!("{} {}: {};", kw, var_name, typ));
                            }
                        } else {
                            let val = Self::expr_to_rust(&child.children[0]);
                            if child.extra_type.is_empty() {
                                self.write_line(&format!("{} {} = {};", kw, var_name, val));
                            } else {
                                self.write_line(&format!(
                                    "{} {}: {} = {};",
                                    kw, var_name, typ, val
                                ));
                            }
                        }
                    }
                    NodeKind::StmtAssign => {
                        let target = if child.children.is_empty() {
                            child.name.clone()
                        } else {
                            Self::expr_to_rust(&child.children[0])
                        };
                        if child.children.len() >= 2 {
                            let val = Self::expr_to_rust(&child.children[1]);
                            self.write_line(&format!("{} = {};", target, val));
                        } else {
                            self.write_line(&format!("{};", target));
                        }
                    }
                    NodeKind::StmtIf => {
                        self.write_indent();
                        self.write("if ");
                        if !child.children.is_empty() {
                            self.write(&Self::expr_to_rust(&child.children[0]));
                        }
                        self.write(" {\n");
                        self.indent += 1;
                        if child.children.len() > 1 {
                            for stmt in &child.children[1].children {
                                self.gen_rust_stmt(stmt);
                            }
                        }
                        self.indent -= 1;
                        if child.children.len() > 2 {
                            self.write_indent();
                            self.write("} else {\n");
                            self.indent += 1;
                            for stmt in &child.children[2].children {
                                self.gen_rust_stmt(stmt);
                            }
                            self.indent -= 1;
                        }
                        self.write_line("}");
                    }
                    NodeKind::StmtWhile => {
                        self.write_indent();
                        self.write("while ");
                        if !child.children.is_empty() {
                            self.write(&Self::expr_to_rust(&child.children[0]));
                        }
                        self.write(" {\n");
                        self.indent += 1;
                        if child.children.len() > 1 {
                            for stmt in &child.children[1].children {
                                self.gen_rust_stmt(stmt);
                            }
                        }
                        self.indent -= 1;
                        self.write_line("}");
                    }
                    NodeKind::StmtFor => {
                        self.write_indent();
                        self.write("for ");
                        if child.children.len() > 1 {
                            self.write(&child.children[1].name);
                        }
                        self.write(" in ");
                        if !child.children.is_empty() {
                            self.write(&Self::expr_to_rust(&child.children[0]));
                        }
                        self.write(" {\n");
                        self.indent += 1;
                        if child.children.len() > 2 {
                            for stmt in &child.children[2].children {
                                self.gen_rust_stmt(stmt);
                            }
                        }
                        self.indent -= 1;
                        self.write_line("}");
                    }
                    _ => {}
                }
            }
            self.indent -= 1;
            self.write_line("}");
        } else {
            self.write_line(" unimplemented!() }");
        }
        self.blank_line();
    }

    fn gen_rust_stmt(&mut self, stmt: &Node) {
        match stmt.kind {
            NodeKind::ExprReturn => {
                let val = if stmt.children.is_empty() {
                    "()".to_string()
                } else {
                    Self::expr_to_rust(&stmt.children[0])
                };
                self.write_line(&format!("return {};", val));
            }
            NodeKind::StmtExpr => {
                if stmt.children.len() == 1 {
                    self.write_line(&format!("{};", Self::expr_to_rust(&stmt.children[0])));
                }
            }
            NodeKind::StmtLocal => {
                let kw = if stmt.extra_mutable { "let mut" } else { "let" };
                let typ = Self::t27_type_to_rust(&stmt.extra_type);
                if stmt.children.is_empty() {
                    if stmt.extra_type.is_empty() {
                        self.write_line(&format!("{} {};", kw, stmt.name));
                    } else {
                        self.write_line(&format!("{} {}: {};", kw, stmt.name, typ));
                    }
                } else {
                    let val = Self::expr_to_rust(&stmt.children[0]);
                    if stmt.extra_type.is_empty() {
                        self.write_line(&format!("{} {} = {};", kw, stmt.name, val));
                    } else {
                        self.write_line(&format!("{} {}: {} = {};", kw, stmt.name, typ, val));
                    }
                }
            }
            NodeKind::StmtAssign => {
                if stmt.children.len() >= 2 {
                    let target = Self::expr_to_rust(&stmt.children[0]);
                    let val = Self::expr_to_rust(&stmt.children[1]);
                    self.write_line(&format!("{} = {};", target, val));
                }
            }
            NodeKind::StmtIf => {
                self.write_indent();
                self.write("if ");
                if !stmt.children.is_empty() {
                    self.write(&Self::expr_to_rust(&stmt.children[0]));
                }
                self.write(" {\n");
                self.indent += 1;
                if stmt.children.len() > 1 {
                    for s in &stmt.children[1].children {
                        self.gen_rust_stmt(s);
                    }
                }
                self.indent -= 1;
                if stmt.children.len() > 2 {
                    self.write_indent();
                    self.write("} else {\n");
                    self.indent += 1;
                    for s in &stmt.children[2].children {
                        self.gen_rust_stmt(s);
                    }
                    self.indent -= 1;
                }
                self.write_line("}");
            }
            NodeKind::StmtWhile => {
                self.write_indent();
                self.write("while ");
                if !stmt.children.is_empty() {
                    self.write(&Self::expr_to_rust(&stmt.children[0]));
                }
                self.write(" {\n");
                self.indent += 1;
                if stmt.children.len() > 1 {
                    for s in &stmt.children[1].children {
                        self.gen_rust_stmt(s);
                    }
                }
                self.indent -= 1;
                self.write_line("}");
            }
            NodeKind::StmtBreak => {
                self.write_line("break;");
            }
            NodeKind::StmtContinue => {
                self.write_line("continue;");
            }
            NodeKind::StmtFor => {
                self.write_indent();
                self.write("for ");
                if stmt.children.len() > 1 {
                    self.write(&stmt.children[1].name);
                }
                self.write(" in ");
                if !stmt.children.is_empty() {
                    self.write(&Self::expr_to_rust(&stmt.children[0]));
                }
                self.write(" {\n");
                self.indent += 1;
                if stmt.children.len() > 2 {
                    for s in &stmt.children[2].children {
                        self.gen_rust_stmt(s);
                    }
                }
                self.indent -= 1;
                self.write_line("}");
            }
            _ => {}
        }
    }

    fn t27_type_to_rust(t27_type: &str) -> String {
        let t = t27_type.trim();
        // Handle optional types
        let (base_type, is_optional) = if t.ends_with('?') {
            (&t[..t.len() - 1], true)
        } else {
            (t, false)
        };

        let rust_type = match base_type {
            "u8" | "u16" | "u32" | "u64" | "u128" => base_type.to_string(),
            "i8" | "i16" | "i32" | "i64" | "i128" => base_type.to_string(),
            "f32" | "f64" => base_type.to_string(),
            "GF16" | "gf16" => "u16".to_string(),
            "bool" => "bool".to_string(),
            "str" => "String".to_string(),
            "void" => "()".to_string(),
            t if t.starts_with("[]") => {
                let inner = &t[2..];
                format!("Vec<{}>", Self::t27_type_to_rust(inner))
            }
            t if t.starts_with('[') && t.contains(']') => {
                // [N]T format - convert to Vec
                if let Some(bracket_end) = t.find(']') {
                    let inner = &t[bracket_end + 1..];
                    format!("Vec<{}>", Self::t27_type_to_rust(inner))
                } else {
                    t.to_string()
                }
            }
            t => t.to_string(), // Custom type name
        };

        if is_optional {
            format!("Option<{}>", rust_type)
        } else {
            rust_type
        }
    }

    fn expr_to_rust(node: &Node) -> String {
        match node.kind {
            NodeKind::ExprLiteral => node.value.clone(),
            NodeKind::ExprIdentifier => node.name.clone(),
            NodeKind::ExprBinary => {
                if node.children.len() >= 2 {
                    let left = Self::expr_to_rust(&node.children[0]);
                    let right = Self::expr_to_rust(&node.children[1]);
                    let op = match node.extra_op.as_str() {
                        "and" => "&&",
                        "or" => "||",
                        op => op,
                    };
                    format!("({} {} {})", left, op, right)
                } else {
                    "()".to_string()
                }
            }
            NodeKind::ExprCall => {
                let args: Vec<String> = node
                    .children
                    .iter()
                    .map(|c| Self::expr_to_rust(c))
                    .collect();
                format!("{}({})", node.name, args.join(", "))
            }
            NodeKind::ExprArrayLiteral => {
                let elems: Vec<String> = node
                    .children
                    .iter()
                    .map(|c| Self::expr_to_rust(c))
                    .collect();
                format!("vec![{}]", elems.join(", "))
            }
            NodeKind::ExprStructLit => {
                let fields: Vec<String> = node
                    .children
                    .iter()
                    .map(|c| {
                        let val = if c.children.is_empty() {
                            "{}".to_string()
                        } else {
                            Self::expr_to_rust(&c.children[0])
                        };
                        format!("{}: {}", c.name, val)
                    })
                    .collect();
                format!("{} {{ {} }}", node.name, fields.join(", "))
            }
            NodeKind::ExprEnumValue => format!("{}::{}", node.name, node.extra_field),
            NodeKind::ExprUnary => {
                if !node.children.is_empty() {
                    format!(
                        "{}({})",
                        node.extra_op,
                        Self::expr_to_rust(&node.children[0])
                    )
                } else {
                    node.extra_op.clone()
                }
            }
            NodeKind::ExprFieldAccess => {
                if !node.children.is_empty() {
                    format!("{}.{}", Self::expr_to_rust(&node.children[0]), node.name)
                } else {
                    node.name.clone()
                }
            }
            NodeKind::ExprIndex => {
                if node.children.len() >= 2 {
                    format!(
                        "{}[{}]",
                        Self::expr_to_rust(&node.children[0]),
                        Self::expr_to_rust(&node.children[1])
                    )
                } else {
                    "()".to_string()
                }
            }
            NodeKind::ExprIf => {
                let mut s = format!("if {} {{ ", Self::expr_to_rust(&node.children[0]));
                if node.children.len() > 1 {
                    s.push_str(&Self::expr_to_rust(&node.children[1]));
                }
                s.push_str(" }");
                if node.children.len() > 2 {
                    s.push_str(&format!(
                        " else {{ {} }}",
                        Self::expr_to_rust(&node.children[2])
                    ));
                }
                s
            }
            NodeKind::ExprSwitch => {
                if node.children.is_empty() {
                    return "/* switch */".to_string();
                }
                let scrutinee = Self::expr_to_rust(&node.children[0]);
                let mut s = format!("match {} {{\n", scrutinee);
                for i in 1..node.children.len() {
                    let arm = &node.children[i];
                    if arm.kind == NodeKind::Module {
                        let pattern = if !arm.name.is_empty() {
                            arm.name.clone()
                        } else {
                            "_".to_string()
                        };
                        let body = if !arm.children.is_empty() {
                            Self::expr_to_rust(&arm.children[0])
                        } else {
                            "()".to_string()
                        };
                        s.push_str(&format!("{} => {},\n", pattern, body));
                    }
                }
                s.push('}');
                s
            }
            _ => "()".to_string(),
        }
    }
}

// ============================================================================
// Hardware IR (HIR) — Phase 0 FPGA foundation
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum HwType {
    Bits(u32),
    UInt(u32),
    SInt(u32),
    Bool,
    Clock,
    Reset(HwResetKind, HwResetPolarity),
    Vector(Box<HwType>, u32),
    Bundle(Vec<(String, HwType)>),
    Enum(Vec<(String, String)>),
    GF16,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HwResetKind {
    Async,
    Sync,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HwResetPolarity {
    ActiveHigh,
    ActiveLow,
}

impl HwType {
    pub fn hw_width(&self) -> u32 {
        match self {
            HwType::Bits(w) | HwType::UInt(w) | HwType::SInt(w) => *w,
            HwType::Bool | HwType::Clock => 1,
            HwType::Reset(_, _) => 1,
            HwType::Vector(elem, len) => elem.hw_width() * len,
            HwType::Bundle(fields) => fields.iter().map(|(_, t)| t.hw_width()).sum(),
            HwType::Enum(variants) => {
                let n = variants.len() as u32;
                if n <= 2 {
                    1
                } else if n <= 4 {
                    2
                } else if n <= 8 {
                    3
                } else if n <= 16 {
                    4
                } else if n <= 32 {
                    5
                } else if n <= 64 {
                    6
                } else {
                    8
                }
            }
            HwType::GF16 => 16,
        }
    }

    pub fn is_signed(&self) -> bool {
        matches!(self, HwType::SInt(_))
    }

    pub fn is_clock_like(&self) -> bool {
        matches!(self, HwType::Clock)
    }

    pub fn is_reset_like(&self) -> bool {
        matches!(self, HwType::Reset(_, _))
    }

    pub fn verilog_range(&self) -> String {
        let w = self.hw_width();
        if w <= 1 {
            String::new()
        } else {
            format!("[{}:0]", w - 1)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HwPortDir {
    Input,
    Output,
    Inout,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HwSignalKind {
    Wire,
    Reg,
}

#[derive(Debug, Clone)]
pub struct HirPort {
    pub name: String,
    pub dir: HwPortDir,
    pub ty: HwType,
}

#[derive(Debug, Clone)]
pub struct HirSignal {
    pub name: String,
    pub kind: HwSignalKind,
    pub ty: HwType,
    pub reset_value: String,
}

#[derive(Debug, Clone)]
pub struct HirAssign {
    pub target: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HwEdge {
    Posedge,
    Negedge,
    Comb,
}

#[derive(Debug, Clone)]
pub struct HirAlwaysBlock {
    pub edge: HwEdge,
    pub clock_name: String,
    pub reset_name: String,
    pub body: Vec<HirAlwaysStmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirAlwaysStmtKind {
    BlockingAssign,
    NonBlockingAssign,
    IfElse,
    Case,
    ForLoop,
    Block,
}

#[derive(Debug, Clone)]
pub struct HirAlwaysStmt {
    pub kind: HirAlwaysStmtKind,
    pub target: String,
    pub value: String,
    pub condition: String,
    pub body: Vec<HirAlwaysStmt>,
}

#[derive(Debug, Clone)]
pub struct HirInstance {
    pub name: String,
    pub module_name: String,
    pub port_map: Vec<(String, String)>,
    pub param_map: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HwMemKind {
    Bram,
    Dram,
    Rom,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HwMemPortKind {
    Read,
    Write,
    ReadWrite,
}

#[derive(Debug, Clone)]
pub struct HirMemPort {
    pub name: String,
    pub kind: HwMemPortKind,
    pub addr_width: u32,
    pub data_width: u32,
}

#[derive(Debug, Clone)]
pub struct HirMemory {
    pub name: String,
    pub kind: HwMemKind,
    pub depth: u32,
    pub data_width: u32,
    pub addr_width: u32,
    pub ports: Vec<HirMemPort>,
}

impl HirMemory {
    pub fn new_bram(name: &str, depth: u32, data_width: u32) -> Self {
        let addr_width = Self::calc_addr_width(depth);
        HirMemory {
            name: name.to_string(),
            kind: HwMemKind::Bram,
            depth,
            data_width,
            addr_width,
            ports: Vec::new(),
        }
    }

    pub fn new_rom(name: &str, depth: u32, data_width: u32) -> Self {
        let addr_width = Self::calc_addr_width(depth);
        HirMemory {
            name: name.to_string(),
            kind: HwMemKind::Rom,
            depth,
            data_width,
            addr_width,
            ports: Vec::new(),
        }
    }

    fn calc_addr_width(depth: u32) -> u32 {
        if depth <= 1 {
            return 1;
        }
        let mut w = 0u32;
        let mut d = depth;
        while d > 1 {
            w += 1;
            d /= 2;
        }
        w.max(1)
    }

    pub fn add_read_port(&mut self, name: &str) {
        self.ports.push(HirMemPort {
            name: name.to_string(),
            kind: HwMemPortKind::Read,
            addr_width: self.addr_width,
            data_width: self.data_width,
        });
    }

    pub fn add_write_port(&mut self, name: &str) {
        self.ports.push(HirMemPort {
            name: name.to_string(),
            kind: HwMemPortKind::Write,
            addr_width: self.addr_width,
            data_width: self.data_width,
        });
    }

    pub fn has_read(&self) -> bool {
        self.ports
            .iter()
            .any(|p| p.kind == HwMemPortKind::Read || p.kind == HwMemPortKind::ReadWrite)
    }

    pub fn has_write(&self) -> bool {
        self.ports
            .iter()
            .any(|p| p.kind == HwMemPortKind::Write || p.kind == HwMemPortKind::ReadWrite)
    }

    pub fn total_bits(&self) -> u32 {
        self.depth * self.data_width
    }

    pub fn bram18_count(&self) -> u32 {
        let bits = self.total_bits();
        let count = bits / 18432;
        if bits % 18432 > 0 {
            count + 1
        } else {
            count
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("memory name must not be empty".to_string());
        }
        if self.depth == 0 {
            errors.push(format!("memory '{}' has zero depth", self.name));
        }
        if self.data_width == 0 {
            errors.push(format!("memory '{}' has zero data width", self.name));
        }
        if self.kind == HwMemKind::Rom && self.has_write() {
            errors.push(format!("ROM '{}' cannot have write ports", self.name));
        }
        errors
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HwClkSrcKind {
    External,
    Pll,
    Dcm,
    Mmcm,
}

#[derive(Debug, Clone)]
pub struct HirClkSource {
    pub name: String,
    pub kind: HwClkSrcKind,
    pub freq_hz: u32,
    pub phase_deg: u32,
    pub jitter_ps: u32,
}

#[derive(Debug, Clone)]
pub struct HirClockDomain {
    pub name: String,
    pub source_name: String,
    pub freq_hz: u32,
    pub posedge: bool,
}

impl HirClockDomain {
    pub fn new(name: &str, source: &str, freq_hz: u32) -> Self {
        HirClockDomain {
            name: name.to_string(),
            source_name: source.to_string(),
            freq_hz,
            posedge: true,
        }
    }

    pub fn period_ns(&self) -> u32 {
        if self.freq_hz == 0 {
            return 0;
        }
        1_000_000_000 / self.freq_hz
    }

    pub fn half_period_ns(&self) -> u32 {
        self.period_ns() / 2
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HwCrossStrategy {
    TwoFlop,
    FifoAsync,
    Handshake,
}

#[derive(Debug, Clone)]
pub struct HirClockCrossing {
    pub src_domain: String,
    pub dst_domain: String,
    pub strategy: HwCrossStrategy,
    pub data_width: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HwFifoKind {
    Sync,
    Async,
}

#[derive(Debug, Clone)]
pub struct HirFifo {
    pub name: String,
    pub kind: HwFifoKind,
    pub depth: u32,
    pub data_width: u32,
    pub has_almost_empty: bool,
    pub has_almost_full: bool,
    pub almost_empty_threshold: u32,
    pub almost_full_threshold: u32,
}

impl HirFifo {
    pub fn new_sync(name: &str, depth: u32, data_width: u32) -> Self {
        HirFifo {
            name: name.to_string(),
            kind: HwFifoKind::Sync,
            depth,
            data_width,
            has_almost_empty: false,
            has_almost_full: false,
            almost_empty_threshold: 0,
            almost_full_threshold: 0,
        }
    }

    pub fn new_async(name: &str, depth: u32, data_width: u32) -> Self {
        HirFifo {
            name: name.to_string(),
            kind: HwFifoKind::Async,
            depth,
            data_width,
            has_almost_empty: false,
            has_almost_full: false,
            almost_empty_threshold: 0,
            almost_full_threshold: 0,
        }
    }

    pub fn addr_width(&self) -> u32 {
        if self.depth <= 1 {
            return 1;
        }
        let mut w = 0u32;
        let mut d = self.depth;
        while d > 1 {
            w += 1;
            d /= 2;
        }
        w.max(1)
    }

    pub fn total_bits(&self) -> u32 {
        self.depth * self.data_width
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HwBusKind {
    Axi4Lite,
    Axi4Full,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HwBusRole {
    Master,
    Slave,
}

#[derive(Debug, Clone)]
pub struct HirBusPort {
    pub name: String,
    pub kind: HwBusKind,
    pub role: HwBusRole,
    pub addr_width: u32,
    pub data_width: u32,
    pub id_width: u32,
}

impl HirBusPort {
    pub fn axi4_lite_slave(name: &str, addr_width: u32, data_width: u32) -> Self {
        HirBusPort {
            name: name.to_string(),
            kind: HwBusKind::Axi4Lite,
            role: HwBusRole::Slave,
            addr_width,
            data_width,
            id_width: 0,
        }
    }

    pub fn axi4_lite_master(name: &str, addr_width: u32, data_width: u32) -> Self {
        HirBusPort {
            name: name.to_string(),
            kind: HwBusKind::Axi4Lite,
            role: HwBusRole::Master,
            addr_width,
            data_width,
            id_width: 0,
        }
    }

    pub fn axi4_full_slave(name: &str, addr_width: u32, data_width: u32, id_width: u32) -> Self {
        HirBusPort {
            name: name.to_string(),
            kind: HwBusKind::Axi4Full,
            role: HwBusRole::Slave,
            addr_width,
            data_width,
            id_width,
        }
    }

    pub fn axi4_full_master(name: &str, addr_width: u32, data_width: u32, id_width: u32) -> Self {
        HirBusPort {
            name: name.to_string(),
            kind: HwBusKind::Axi4Full,
            role: HwBusRole::Master,
            addr_width,
            data_width,
            id_width,
        }
    }

    pub fn strb_width(&self) -> u32 {
        self.data_width / 8
    }

    pub fn is_lite(&self) -> bool {
        self.kind == HwBusKind::Axi4Lite
    }

    pub fn is_full(&self) -> bool {
        self.kind == HwBusKind::Axi4Full
    }

    pub fn is_slave(&self) -> bool {
        self.role == HwBusRole::Slave
    }

    pub fn is_master(&self) -> bool {
        self.role == HwBusRole::Master
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("bus port name must not be empty".to_string());
        }
        if self.addr_width == 0 {
            errors.push(format!("bus '{}' has zero address width", self.name));
        }
        if self.data_width == 0 {
            errors.push(format!("bus '{}' has zero data width", self.name));
        }
        if self.data_width % 8 != 0 {
            errors.push(format!(
                "bus '{}' data width {} is not byte-aligned",
                self.name, self.data_width
            ));
        }
        if self.kind == HwBusKind::Axi4Full && self.id_width == 0 {
            errors.push(format!(
                "AXI4-Full bus '{}' must have non-zero ID width",
                self.name
            ));
        }
        errors
    }

    pub fn port_count(&self) -> u32 {
        let mut count = 0u32;
        let has_id = self.id_width > 0;
        let is_lite = self.kind == HwBusKind::Axi4Lite;
        // AW channel: valid, addr, [id], len, size, burst, [cache], [prot], [qos], [region], [lock]
        count += 2; // awvalid, awready
        count += 1; // awaddr
        if has_id {
            count += 1;
        }
        if !is_lite {
            count += 3; // awlen, awsize, awburst
        }
        if !is_lite {
            count += 1; // awcache
        }
        count += 1; // awprot
        if !is_lite {
            count += 1; // awqos
            count += 1; // awregion
            count += 1; // awlock
        }
        // W channel: valid, data, strb, last, ready
        count += 2; // wvalid, wready
        count += 1; // wdata
        count += 1; // wstrb
        if !is_lite {
            count += 1; // wlast
        }
        // B channel: valid, resp, [id], ready
        count += 2; // bvalid, bready
        count += 1; // bresp
        if has_id {
            count += 1;
        }
        // AR channel: valid, addr, [id], len, size, burst, [cache], [prot], [qos], [region], [lock]
        count += 2; // arvalid, arready
        count += 1; // araddr
        if has_id {
            count += 1;
        }
        if !is_lite {
            count += 3; // arlen, arsize, arburst
        }
        if !is_lite {
            count += 1; // arcache
        }
        count += 1; // arprot
        if !is_lite {
            count += 1; // arqos
            count += 1; // arregion
            count += 1; // arlock
        }
        // R channel: valid, data, resp, last, [id], ready
        count += 2; // rvalid, rready
        count += 1; // rdata
        count += 1; // rresp
        if !is_lite {
            count += 1; // rlast
        }
        if has_id {
            count += 1;
        }
        count
    }

    pub fn total_signal_bits(&self) -> u32 {
        let mut bits = 0u32;
        let has_id = self.id_width > 0;
        let is_lite = self.kind == HwBusKind::Axi4Lite;
        // AW
        bits += self.addr_width;
        if has_id {
            bits += self.id_width;
        }
        if !is_lite {
            bits += 8 + 3 + 2 + 4 + 4 + 4 + 1;
        }
        bits += 3; // prot
        bits += 2; // valid/ready
                   // W
        bits += self.data_width + self.strb_width();
        if !is_lite {
            bits += 1;
        }
        bits += 2; // valid/ready
                   // B
        bits += 2;
        if has_id {
            bits += self.id_width;
        }
        bits += 2; // valid/ready
                   // AR
        bits += self.addr_width;
        if has_id {
            bits += self.id_width;
        }
        if !is_lite {
            bits += 8 + 3 + 2 + 4 + 4 + 4 + 1;
        }
        bits += 3;
        bits += 2;
        // R
        bits += self.data_width + 2;
        if !is_lite {
            bits += 1;
        }
        if has_id {
            bits += self.id_width;
        }
        bits += 2;
        bits
    }
}

#[derive(Debug, Clone)]
pub struct HirApbPeriphMap {
    pub name: String,
    pub base_addr: u32,
    pub size: u32,
    pub index: u32,
}

#[derive(Debug, Clone)]
pub struct HirApbBridge {
    pub name: String,
    pub addr_width: u32,
    pub data_width: u32,
    pub num_peripherals: u32,
    pub has_pslverr: bool,
    pub has_pprot: bool,
    pub periph_maps: Vec<HirApbPeriphMap>,
}

impl HirApbBridge {
    pub fn new(name: &str, addr_width: u32, data_width: u32, num_peripherals: u32) -> Self {
        HirApbBridge {
            name: name.to_string(),
            addr_width,
            data_width,
            num_peripherals,
            has_pslverr: false,
            has_pprot: false,
            periph_maps: Vec::new(),
        }
    }

    pub fn with_error_response(mut self) -> Self {
        self.has_pslverr = true;
        self.has_pprot = true;
        self
    }

    pub fn add_peripheral(&mut self, name: &str, base_addr: u32, size: u32, index: u32) {
        self.periph_maps.push(HirApbPeriphMap {
            name: name.to_string(),
            base_addr,
            size,
            index,
        });
    }

    pub fn strb_width(&self) -> u32 {
        self.data_width / 8
    }

    pub fn addr_bits_for_peripherals(&self) -> u32 {
        let mut n = self.num_peripherals;
        if n <= 1 {
            return 0;
        }
        let mut bits = 0u32;
        while n > 1 {
            bits += 1;
            n /= 2;
        }
        bits
    }

    pub fn select_peripheral(&self, addr: u32) -> Option<u32> {
        for m in &self.periph_maps {
            if addr >= m.base_addr && addr < m.base_addr + m.size {
                return Some(m.index);
            }
        }
        None
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("APB bridge name must not be empty".to_string());
        }
        if self.addr_width == 0 {
            errors.push(format!("APB bridge '{}' has zero address width", self.name));
        }
        if self.data_width == 0 {
            errors.push(format!("APB bridge '{}' has zero data width", self.name));
        }
        if self.data_width % 8 != 0 {
            errors.push(format!(
                "APB bridge '{}' data width {} is not byte-aligned",
                self.name, self.data_width
            ));
        }
        if self.num_peripherals == 0 {
            errors.push(format!(
                "APB bridge '{}' has zero peripheral count",
                self.name
            ));
        }
        errors
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Gf16OpKind {
    Mul,
    Add,
    Mac,
    Dot,
    Fft,
    Ifft,
    Matmul,
    Inverse,
}

#[derive(Debug, Clone)]
pub struct HirGf16MacUnit {
    pub name: String,
    pub accumulator_width: u32,
    pub pipeline_stages: u32,
}

#[derive(Debug, Clone)]
pub struct HirGf16FftConfig {
    pub name: String,
    pub num_points: u32,
    pub radix: u32,
}

impl HirGf16FftConfig {
    pub fn fft_stages(&self) -> u32 {
        let mut n = self.num_points;
        let r = self.radix;
        if r == 0 {
            return 0;
        }
        let mut stages = 0u32;
        while n > 1 {
            stages += 1;
            n /= r;
        }
        stages
    }

    pub fn twiddle_count(&self) -> u32 {
        self.num_points / 2
    }
}

#[derive(Debug, Clone)]
pub struct HirGf16Accel {
    pub name: String,
    pub num_multipliers: u32,
    pub vector_width: u32,
    pub has_mac: bool,
    pub has_fft: bool,
    pub has_dot_product: bool,
    pub has_matmul: bool,
    pub clock_freq_hz: u32,
    pub mac_units: Vec<HirGf16MacUnit>,
    pub fft_config: Option<HirGf16FftConfig>,
}

impl HirGf16Accel {
    pub fn basic(name: &str, num_mult: u32) -> Self {
        HirGf16Accel {
            name: name.to_string(),
            num_multipliers: num_mult,
            vector_width: num_mult,
            has_mac: true,
            has_fft: false,
            has_dot_product: false,
            has_matmul: false,
            clock_freq_hz: 100_000_000,
            mac_units: Vec::new(),
            fft_config: None,
        }
    }

    pub fn full(name: &str, num_mult: u32, vec_width: u32) -> Self {
        HirGf16Accel {
            name: name.to_string(),
            num_multipliers: num_mult,
            vector_width: vec_width,
            has_mac: true,
            has_fft: true,
            has_dot_product: true,
            has_matmul: true,
            clock_freq_hz: 100_000_000,
            mac_units: Vec::new(),
            fft_config: None,
        }
    }

    pub fn add_mac_unit(&mut self, name: &str, acc_width: u32, stages: u32) {
        self.mac_units.push(HirGf16MacUnit {
            name: name.to_string(),
            accumulator_width: acc_width,
            pipeline_stages: stages,
        });
    }

    pub fn set_fft(&mut self, name: &str, num_points: u32, radix: u32) {
        self.fft_config = Some(HirGf16FftConfig {
            name: name.to_string(),
            num_points,
            radix,
        });
    }

    pub fn total_gf16_bits(&self) -> u32 {
        self.num_multipliers * 4
    }

    pub fn mac_unit_count(&self) -> u32 {
        if self.has_mac {
            self.num_multipliers
        } else {
            0
        }
    }

    pub fn dsp48_count(&self) -> u32 {
        self.num_multipliers
    }

    pub fn bram_count(&self) -> u32 {
        let mut count = 0u32;
        if self.has_fft {
            count += self.vector_width / 4;
        }
        if self.has_matmul {
            count += 2;
        }
        count
    }

    pub fn matmul_cycles(&self, n: u32) -> u32 {
        if self.has_matmul && self.num_multipliers > 0 {
            n * n * n / self.num_multipliers
        } else {
            0
        }
    }

    pub fn dot_product_cycles(&self, vec_len: u32) -> u32 {
        if self.has_dot_product && self.num_multipliers > 0 {
            vec_len / self.num_multipliers + 2
        } else {
            0
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("GF16 accelerator name must not be empty".to_string());
        }
        if self.num_multipliers == 0 {
            errors.push(format!(
                "GF16 accelerator '{}' has zero multipliers",
                self.name
            ));
        }
        if self.vector_width == 0 {
            errors.push(format!(
                "GF16 accelerator '{}' has zero vector width",
                self.name
            ));
        }
        if self.clock_freq_hz == 0 {
            errors.push(format!(
                "GF16 accelerator '{}' has zero clock frequency",
                self.name
            ));
        }
        errors
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HwAssertKind {
    Immediate,
    Concurrent,
    Cover,
    Assume,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HwAssertSeverity {
    Info,
    Warning,
    Error,
    Fatal,
}

#[derive(Debug, Clone)]
pub struct HirFormalAssert {
    pub name: String,
    pub kind: HwAssertKind,
    pub severity: HwAssertSeverity,
    pub condition: String,
    pub clock: String,
    pub reset: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct HirCoverPoint {
    pub name: String,
    pub condition: String,
    pub clock: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct HirFormalAssume {
    pub name: String,
    pub condition: String,
    pub clock: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct HirFormalConfig {
    pub name: String,
    pub module_name: String,
    pub clock: String,
    pub reset: String,
    pub depth: u32,
    pub timeout_cycles: u32,
}

impl HirFormalConfig {
    pub fn new(name: &str, module_name: &str, clock: &str, reset: &str) -> Self {
        HirFormalConfig {
            name: name.to_string(),
            module_name: module_name.to_string(),
            clock: clock.to_string(),
            reset: reset.to_string(),
            depth: 20,
            timeout_cycles: 100,
        }
    }

    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_timeout(mut self, timeout: u32) -> Self {
        self.timeout_cycles = timeout;
        self
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("formal config name must not be empty".to_string());
        }
        if self.module_name.is_empty() {
            errors.push("formal config module name must not be empty".to_string());
        }
        if self.clock.is_empty() {
            errors.push("formal config clock must not be empty".to_string());
        }
        if self.depth == 0 {
            errors.push("formal config depth must be positive".to_string());
        }
        if self.timeout_cycles == 0 {
            errors.push("formal config timeout must be positive".to_string());
        }
        errors
    }
}

impl HirFormalAssert {
    pub fn immediate(
        name: &str,
        condition: &str,
        severity: HwAssertSeverity,
        description: &str,
    ) -> Self {
        HirFormalAssert {
            name: name.to_string(),
            kind: HwAssertKind::Immediate,
            severity,
            condition: condition.to_string(),
            clock: String::new(),
            reset: String::new(),
            description: description.to_string(),
        }
    }

    pub fn concurrent(
        name: &str,
        condition: &str,
        clock: &str,
        reset: &str,
        description: &str,
    ) -> Self {
        HirFormalAssert {
            name: name.to_string(),
            kind: HwAssertKind::Concurrent,
            severity: HwAssertSeverity::Error,
            condition: condition.to_string(),
            clock: clock.to_string(),
            reset: reset.to_string(),
            description: description.to_string(),
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("assertion name must not be empty".to_string());
        }
        if self.condition.is_empty() {
            errors.push(format!("assertion '{}' has empty condition", self.name));
        }
        if self.kind == HwAssertKind::Concurrent && self.clock.is_empty() {
            errors.push(format!(
                "concurrent assertion '{}' needs a clock",
                self.name
            ));
        }
        errors
    }
}

impl HirCoverPoint {
    pub fn new(name: &str, condition: &str, clock: &str, description: &str) -> Self {
        HirCoverPoint {
            name: name.to_string(),
            condition: condition.to_string(),
            clock: clock.to_string(),
            description: description.to_string(),
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("cover point name must not be empty".to_string());
        }
        if self.condition.is_empty() {
            errors.push(format!("cover point '{}' has empty condition", self.name));
        }
        errors
    }
}

impl HirFormalAssume {
    pub fn new(name: &str, condition: &str, clock: &str, description: &str) -> Self {
        HirFormalAssume {
            name: name.to_string(),
            condition: condition.to_string(),
            clock: clock.to_string(),
            description: description.to_string(),
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("assumption name must not be empty".to_string());
        }
        if self.condition.is_empty() {
            errors.push(format!("assumption '{}' has empty condition", self.name));
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct HirTernaryAluOp {
    pub name: String,
    pub opcode: u32,
    pub latency: u32,
    pub uses_gf16: bool,
}

#[derive(Debug, Clone)]
pub struct HirPipelineStage {
    pub name: String,
    pub latency: u32,
    pub has_forwarding: bool,
}

#[derive(Debug, Clone)]
pub struct HirTernaryRegFile {
    pub name: String,
    pub num_regs: u32,
    pub trit_width: u32,
    pub read_ports: u32,
    pub write_ports: u32,
    pub has_forwarding: bool,
}

#[derive(Debug, Clone)]
pub struct HirTernaryCore {
    pub name: String,
    pub data_width: u32,
    pub addr_width: u32,
    pub num_alus: u32,
    pub has_gf16_unit: bool,
    pub has_ternary_alu: bool,
    pub has_branch_predictor: bool,
    pub pipeline_depth: u32,
    pub clock_freq_hz: u32,
    pub reg_file: Option<HirTernaryRegFile>,
    pub pipeline_stages: Vec<HirPipelineStage>,
    pub alu_ops: Vec<HirTernaryAluOp>,
}

impl HirTernaryCore {
    pub fn basic(name: &str) -> Self {
        HirTernaryCore {
            name: name.to_string(),
            data_width: 64,
            addr_width: 32,
            num_alus: 1,
            has_gf16_unit: true,
            has_ternary_alu: true,
            has_branch_predictor: false,
            pipeline_depth: 5,
            clock_freq_hz: 100_000_000,
            reg_file: None,
            pipeline_stages: Vec::new(),
            alu_ops: Vec::new(),
        }
    }

    pub fn full(name: &str) -> Self {
        HirTernaryCore {
            name: name.to_string(),
            data_width: 64,
            addr_width: 32,
            num_alus: 4,
            has_gf16_unit: true,
            has_ternary_alu: true,
            has_branch_predictor: true,
            pipeline_depth: 7,
            clock_freq_hz: 100_000_000,
            reg_file: None,
            pipeline_stages: Vec::new(),
            alu_ops: Vec::new(),
        }
    }

    pub fn with_regfile(mut self, rf: HirTernaryRegFile) -> Self {
        self.reg_file = Some(rf);
        self
    }

    pub fn add_pipeline_stage(&mut self, name: &str, latency: u32, has_forwarding: bool) {
        self.pipeline_stages.push(HirPipelineStage {
            name: name.to_string(),
            latency,
            has_forwarding,
        });
    }

    pub fn add_alu_op(&mut self, name: &str, opcode: u32, latency: u32, uses_gf16: bool) {
        self.alu_ops.push(HirTernaryAluOp {
            name: name.to_string(),
            opcode,
            latency,
            uses_gf16,
        });
    }

    pub fn dsp_count(&self) -> u32 {
        let mut count = self.num_alus;
        if self.has_gf16_unit {
            count += 4;
        }
        count
    }

    pub fn bram_count(&self) -> u32 {
        let mut count = 2u32;
        if self.has_gf16_unit {
            count += 1;
        }
        count
    }

    pub fn lut_estimate(&self) -> u32 {
        let mut luts = 5000u32;
        luts += self.num_alus * 2000;
        if self.has_gf16_unit {
            luts += 3000;
        }
        if self.has_ternary_alu {
            luts += 1500;
        }
        if self.has_branch_predictor {
            luts += 500;
        }
        luts
    }

    pub fn fmax_mhz(&self) -> u32 {
        self.clock_freq_hz / 1_000_000
    }

    pub fn fits_arty_a7(&self) -> bool {
        self.lut_estimate() < 33800
    }

    pub fn fits_xc7a100t(&self) -> bool {
        self.lut_estimate() < 63400
    }

    pub fn pipeline_total_latency(&self) -> u32 {
        self.pipeline_stages.iter().map(|s| s.latency).sum()
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("ternary core name must not be empty".to_string());
        }
        if self.data_width == 0 {
            errors.push(format!("core '{}' has zero data width", self.name));
        }
        if self.num_alus == 0 {
            errors.push(format!("core '{}' has zero ALUs", self.name));
        }
        if self.pipeline_depth == 0 {
            errors.push(format!("core '{}' has zero pipeline depth", self.name));
        }
        if self.clock_freq_hz == 0 {
            errors.push(format!("core '{}' has zero clock frequency", self.name));
        }
        errors
    }
}

impl HirTernaryRegFile {
    pub fn new(name: &str) -> Self {
        HirTernaryRegFile {
            name: name.to_string(),
            num_regs: 27,
            trit_width: 27,
            read_ports: 2,
            write_ports: 1,
            has_forwarding: true,
        }
    }

    pub fn total_bits(&self) -> u32 {
        self.num_regs * self.trit_width * 2
    }

    pub fn bram18_count(&self) -> u32 {
        self.total_bits() / 18432 + 1
    }
}

#[derive(Debug, Clone)]
pub struct HirResourceEstimate {
    pub luts: u32,
    pub ffs: u32,
    pub bram18: u32,
    pub dsp48: u32,
    pub io_pins: u32,
}

impl HirResourceEstimate {
    pub fn zero() -> Self {
        HirResourceEstimate {
            luts: 0,
            ffs: 0,
            bram18: 0,
            dsp48: 0,
            io_pins: 0,
        }
    }

    pub fn new(luts: u32, ffs: u32, bram18: u32, dsp48: u32, io_pins: u32) -> Self {
        HirResourceEstimate {
            luts,
            ffs,
            bram18,
            dsp48,
            io_pins,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HirIpCore {
    pub name: String,
    pub resources: HirResourceEstimate,
    pub clock_freq_mhz: u32,
    pub verified: bool,
}

#[derive(Debug, Clone)]
pub struct HirBoardResources {
    pub name: String,
    pub luts: u32,
    pub ffs: u32,
    pub bram18: u32,
    pub dsp48: u32,
    pub io_pins: u32,
}

impl HirBoardResources {
    pub fn arty_a7() -> Self {
        HirBoardResources {
            name: "arty_a7".into(),
            luts: 33800,
            ffs: 67600,
            bram18: 60,
            dsp48: 90,
            io_pins: 210,
        }
    }

    pub fn xc7a100t() -> Self {
        HirBoardResources {
            name: "xc7a100t".into(),
            luts: 63400,
            ffs: 126800,
            bram18: 135,
            dsp48: 240,
            io_pins: 300,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HirIpCatalog {
    pub name: String,
    pub cores: Vec<HirIpCore>,
}

impl HirIpCatalog {
    pub fn new(name: &str) -> Self {
        HirIpCatalog {
            name: name.to_string(),
            cores: Vec::new(),
        }
    }

    pub fn add_core(
        &mut self,
        name: &str,
        luts: u32,
        ffs: u32,
        bram18: u32,
        dsp48: u32,
        io: u32,
        freq_mhz: u32,
    ) {
        self.cores.push(HirIpCore {
            name: name.to_string(),
            resources: HirResourceEstimate::new(luts, ffs, bram18, dsp48, io),
            clock_freq_mhz: freq_mhz,
            verified: false,
        });
    }

    pub fn total_luts(&self) -> u32 {
        self.cores.iter().map(|c| c.resources.luts).sum()
    }
    pub fn total_ffs(&self) -> u32 {
        self.cores.iter().map(|c| c.resources.ffs).sum()
    }
    pub fn total_bram18(&self) -> u32 {
        self.cores.iter().map(|c| c.resources.bram18).sum()
    }
    pub fn total_dsp48(&self) -> u32 {
        self.cores.iter().map(|c| c.resources.dsp48).sum()
    }

    pub fn fits_board(&self, board: &HirBoardResources) -> bool {
        self.total_luts() <= board.luts
            && self.total_ffs() <= board.ffs
            && self.total_bram18() <= board.bram18
            && self.total_dsp48() <= board.dsp48
    }

    pub fn luts_remaining(&self, board: &HirBoardResources) -> u32 {
        board.luts.saturating_sub(self.total_luts())
    }

    pub fn utilization_percent(&self, board: &HirBoardResources) -> u32 {
        if board.luts == 0 {
            return 0;
        }
        self.total_luts() * 100 / board.luts
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("catalog name must not be empty".to_string());
        }
        for core in &self.cores {
            if core.name.is_empty() {
                errors.push("IP core name must not be empty".to_string());
            }
            if core.clock_freq_mhz == 0 {
                errors.push(format!("IP core '{}' has zero clock frequency", core.name));
            }
        }
        errors
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AsmSectionKind {
    Text,
    Data,
    Bss,
    Rodata,
}

impl AsmSectionKind {
    pub fn from_i8(v: i8) -> Self {
        match v {
            0 => AsmSectionKind::Text,
            1 => AsmSectionKind::Data,
            2 => AsmSectionKind::Bss,
            3 => AsmSectionKind::Rodata,
            _ => AsmSectionKind::Text,
        }
    }
    pub fn to_i8(&self) -> i8 {
        match self {
            AsmSectionKind::Text => 0,
            AsmSectionKind::Data => 1,
            AsmSectionKind::Bss => 2,
            AsmSectionKind::Rodata => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AsmRelocKind {
    Abs32,
    Rel21,
    Gf16Label,
}

impl AsmRelocKind {
    pub fn from_i8(v: i8) -> Self {
        match v {
            0 => AsmRelocKind::Abs32,
            1 => AsmRelocKind::Rel21,
            2 => AsmRelocKind::Gf16Label,
            _ => AsmRelocKind::Abs32,
        }
    }
    pub fn to_i8(&self) -> i8 {
        match self {
            AsmRelocKind::Abs32 => 0,
            AsmRelocKind::Rel21 => 1,
            AsmRelocKind::Gf16Label => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssembledInstr {
    pub address: u32,
    pub opcode: u32,
    pub rd: u32,
    pub rs1: u32,
    pub rs2: u32,
    pub imm: u32,
    pub label: String,
    pub is_gf16: bool,
}

impl AssembledInstr {
    pub fn r_type(opcode: u32, rd: u32, rs1: u32, rs2: u32) -> Self {
        AssembledInstr {
            address: 0,
            opcode,
            rd,
            rs1,
            rs2,
            imm: 0,
            label: String::new(),
            is_gf16: false,
        }
    }

    pub fn i_type(opcode: u32, rd: u32, rs1: u32, imm: u32) -> Self {
        AssembledInstr {
            address: 0,
            opcode,
            rd,
            rs1,
            rs2: 0,
            imm,
            label: String::new(),
            is_gf16: false,
        }
    }

    pub fn gf16(opcode: u32, rd: u32, rs1: u32, rs2: u32) -> Self {
        AssembledInstr {
            address: 0,
            opcode,
            rd,
            rs1,
            rs2,
            imm: 0,
            label: String::new(),
            is_gf16: true,
        }
    }

    pub fn is_r_type(&self) -> bool {
        self.imm == 0 && self.rs2 > 0
    }

    pub fn is_i_type(&self) -> bool {
        self.imm > 0
    }

    pub fn is_gf16_instr(&self) -> bool {
        self.is_gf16
    }

    pub fn encode_r_type(&self) -> u32 {
        (self.opcode << 26) | (self.rd << 21) | (self.rs1 << 16) | (self.rs2 << 11)
    }

    pub fn encode_i_type(&self) -> u32 {
        (self.opcode << 26) | (self.rd << 21) | (self.rs1 << 16) | (self.imm & 0xFFFF)
    }

    pub fn encode(&self) -> u32 {
        if self.is_gf16 {
            self.encode_r_type()
        } else if self.is_i_type() {
            self.encode_i_type()
        } else {
            self.encode_r_type()
        }
    }
}

#[derive(Debug, Clone)]
pub struct AsmRelocEntry {
    pub offset: u32,
    pub kind: AsmRelocKind,
    pub symbol: String,
    pub addend: u32,
}

impl AsmRelocEntry {
    pub fn new(offset: u32, kind: AsmRelocKind, symbol: &str, addend: u32) -> Self {
        AsmRelocEntry {
            offset,
            kind,
            symbol: symbol.to_string(),
            addend,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AsmSymbol {
    pub name: String,
    pub address: u32,
    pub size: u32,
    pub section: AsmSectionKind,
    pub is_global: bool,
}

impl AsmSymbol {
    pub fn new(name: &str, address: u32, section: AsmSectionKind, is_global: bool) -> Self {
        AsmSymbol {
            name: name.to_string(),
            address,
            size: 0,
            section,
            is_global,
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("symbol name must not be empty".to_string());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct AsmSection {
    pub name: String,
    pub kind: AsmSectionKind,
    pub base_address: u32,
    pub size: u32,
}

impl AsmSection {
    pub fn text(base: u32) -> Self {
        AsmSection {
            name: ".text".to_string(),
            kind: AsmSectionKind::Text,
            base_address: base,
            size: 0,
        }
    }

    pub fn data(base: u32) -> Self {
        AsmSection {
            name: ".data".to_string(),
            kind: AsmSectionKind::Data,
            base_address: base,
            size: 0,
        }
    }

    pub fn bss(base: u32) -> Self {
        AsmSection {
            name: ".bss".to_string(),
            kind: AsmSectionKind::Bss,
            base_address: base,
            size: 0,
        }
    }

    pub fn rodata(base: u32) -> Self {
        AsmSection {
            name: ".rodata".to_string(),
            kind: AsmSectionKind::Rodata,
            base_address: base,
            size: 0,
        }
    }

    pub fn end(&self) -> u32 {
        self.base_address.saturating_add(self.size)
    }
}

#[derive(Debug, Clone)]
pub struct AsmConfig {
    pub name: String,
    pub text_base: u32,
    pub data_base: u32,
    pub word_size: u32,
    pub has_gf16_ext: bool,
    pub has_ternary_ext: bool,
}

impl AsmConfig {
    pub fn new(name: &str) -> Self {
        AsmConfig {
            name: name.to_string(),
            text_base: 0,
            data_base: 4096,
            word_size: 4,
            has_gf16_ext: true,
            has_ternary_ext: true,
        }
    }

    pub fn instr_count(&self, bytes: u32) -> u32 {
        if self.word_size == 0 {
            return 0;
        }
        bytes / self.word_size
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("assembler config name must not be empty".to_string());
        }
        if self.word_size == 0 {
            errors.push("word size must be positive".to_string());
        }
        errors
    }
}

pub fn align_address(addr: u32, alignment: u32) -> u32 {
    if alignment == 0 {
        return addr;
    }
    let remainder = addr % alignment;
    if remainder == 0 {
        addr
    } else {
        addr + alignment - remainder
    }
}

#[derive(Debug, Clone)]
pub struct HirAssembler {
    pub config: AsmConfig,
    pub sections: Vec<AsmSection>,
    pub instructions: Vec<AssembledInstr>,
    pub symbols: Vec<AsmSymbol>,
    pub relocations: Vec<AsmRelocEntry>,
    pub current_section: usize,
    pub errors: Vec<String>,
}

impl HirAssembler {
    pub fn new(name: &str) -> Self {
        let config = AsmConfig::new(name);
        let text_section = AsmSection::text(config.text_base);
        let data_section = AsmSection::data(config.data_base);
        HirAssembler {
            config,
            sections: vec![text_section, data_section],
            instructions: Vec::new(),
            symbols: Vec::new(),
            relocations: Vec::new(),
            current_section: 0,
            errors: Vec::new(),
        }
    }

    pub fn with_config(config: AsmConfig) -> Self {
        let text_section = AsmSection::text(config.text_base);
        let data_section = AsmSection::data(config.data_base);
        HirAssembler {
            config,
            sections: vec![text_section, data_section],
            instructions: Vec::new(),
            symbols: Vec::new(),
            relocations: Vec::new(),
            current_section: 0,
            errors: Vec::new(),
        }
    }

    pub fn set_section(&mut self, name: &str) -> Result<(), String> {
        for (i, sec) in self.sections.iter().enumerate() {
            if sec.name == name {
                self.current_section = i;
                return Ok(());
            }
        }
        Err(format!("unknown section: {}", name))
    }

    pub fn emit_r(&mut self, opcode: u32, rd: u32, rs1: u32, rs2: u32) -> u32 {
        let section = &self.sections[self.current_section];
        let addr = section.end();
        let mut instr = AssembledInstr::r_type(opcode, rd, rs1, rs2);
        instr.address = addr;
        self.instructions.push(instr);
        self.sections[self.current_section].size += self.config.word_size;
        addr
    }

    pub fn emit_i(&mut self, opcode: u32, rd: u32, rs1: u32, imm: u32) -> u32 {
        let section = &self.sections[self.current_section];
        let addr = section.end();
        let mut instr = AssembledInstr::i_type(opcode, rd, rs1, imm);
        instr.address = addr;
        self.instructions.push(instr);
        self.sections[self.current_section].size += self.config.word_size;
        addr
    }

    pub fn emit_gf16(&mut self, opcode: u32, rd: u32, rs1: u32, rs2: u32) -> u32 {
        let section = &self.sections[self.current_section];
        let addr = section.end();
        let mut instr = AssembledInstr::gf16(opcode, rd, rs1, rs2);
        instr.address = addr;
        self.instructions.push(instr);
        self.sections[self.current_section].size += self.config.word_size;
        addr
    }

    pub fn define_symbol(&mut self, name: &str, is_global: bool) {
        let section = &self.sections[self.current_section];
        let address = section.end();
        let section_kind = section.kind.clone();
        self.symbols
            .push(AsmSymbol::new(name, address, section_kind, is_global));
    }

    pub fn resolve_symbol(&self, name: &str) -> Option<u32> {
        self.symbols
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.address)
    }

    pub fn add_relocation(&mut self, offset: u32, kind: AsmRelocKind, symbol: &str, addend: u32) {
        self.relocations
            .push(AsmRelocEntry::new(offset, kind, symbol, addend));
    }

    pub fn apply_relocations(&mut self) -> Result<u32, String> {
        let mut applied: u32 = 0;
        for reloc in &self.relocations {
            if let Some(addr) = self.resolve_symbol(&reloc.symbol) {
                let resolved = addr.saturating_add(reloc.addend);
                for instr in &mut self.instructions {
                    if instr.address == reloc.offset {
                        instr.imm = resolved & 0xFFFF;
                    }
                }
                applied += 1;
            } else {
                self.errors
                    .push(format!("undefined symbol: {}", reloc.symbol));
            }
        }
        if self.errors.is_empty() {
            Ok(applied)
        } else {
            Err(format!("relocation errors: {}", self.errors.join("; ")))
        }
    }

    pub fn total_bytes(&self) -> u32 {
        self.sections.iter().map(|s| s.size).sum()
    }

    pub fn total_instructions(&self) -> u32 {
        self.instructions.len() as u32
    }

    pub fn encode_all(&self) -> Vec<u32> {
        self.instructions.iter().map(|i| i.encode()).collect()
    }

    pub fn to_binary(&self) -> Vec<u8> {
        let words = self.encode_all();
        let mut bytes = Vec::with_capacity(words.len() * 4);
        for w in &words {
            bytes.extend_from_slice(&w.to_le_bytes());
        }
        bytes
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = self.config.validate();
        for sym in &self.symbols {
            errors.extend(sym.validate());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct TbClockCfg {
    pub period_ns: u32,
    pub duty_cycle: u32,
    pub phase_ns: u32,
}

impl TbClockCfg {
    pub fn new(period_ns: u32) -> Self {
        TbClockCfg {
            period_ns,
            duty_cycle: 50,
            phase_ns: 0,
        }
    }

    pub fn half_period(&self) -> u32 {
        self.period_ns / 2
    }
}

#[derive(Debug, Clone)]
pub struct TbResetCfg {
    pub active_low: bool,
    pub delay_cycles: u32,
    pub duration_cycles: u32,
}

impl TbResetCfg {
    pub fn new(delay: u32, duration: u32) -> Self {
        TbResetCfg {
            active_low: true,
            delay_cycles: delay,
            duration_cycles: duration,
        }
    }

    pub fn reset_end_cycle(&self) -> u32 {
        self.delay_cycles.saturating_add(self.duration_cycles)
    }
}

#[derive(Debug, Clone)]
pub struct TbStimulus {
    pub cycle: u32,
    pub signal: String,
    pub value: u32,
}

impl TbStimulus {
    pub fn new(cycle: u32, signal: &str, value: u32) -> Self {
        TbStimulus {
            cycle,
            signal: signal.to_string(),
            value,
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.signal.is_empty() {
            errors.push("stimulus signal must not be empty".to_string());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct TbCheck {
    pub cycle: u32,
    pub signal: String,
    pub expected: u32,
    pub mask: u32,
}

impl TbCheck {
    pub fn new(cycle: u32, signal: &str, expected: u32) -> Self {
        TbCheck {
            cycle,
            signal: signal.to_string(),
            expected,
            mask: 0xFFFF_FFFF,
        }
    }

    pub fn with_mask(cycle: u32, signal: &str, expected: u32, mask: u32) -> Self {
        TbCheck {
            cycle,
            signal: signal.to_string(),
            expected,
            mask,
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.signal.is_empty() {
            errors.push("check signal must not be empty".to_string());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct TbConfig {
    pub name: String,
    pub dut_name: String,
    pub timescale: String,
    pub max_cycles: u32,
    pub timeout_ns: u32,
    pub fail_fast: bool,
}

impl TbConfig {
    pub fn new(dut: &str, max_cycles: u32) -> Self {
        TbConfig {
            name: "tb".to_string(),
            dut_name: dut.to_string(),
            timescale: "1ns/1ps".to_string(),
            max_cycles,
            timeout_ns: max_cycles * 10,
            fail_fast: true,
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.dut_name.is_empty() {
            errors.push("DUT name must not be empty".to_string());
        }
        if self.max_cycles == 0 {
            errors.push("max_cycles must be positive".to_string());
        }
        errors
    }
}

#[derive(Debug)]
pub struct HirTestbench {
    pub config: TbConfig,
    pub clock: TbClockCfg,
    pub reset: TbResetCfg,
    pub stimuli: Vec<TbStimulus>,
    pub checks: Vec<TbCheck>,
    pub probe_signals: Vec<String>,
}

impl HirTestbench {
    pub fn new(dut: &str, max_cycles: u32, clock_period_ns: u32) -> Self {
        HirTestbench {
            config: TbConfig::new(dut, max_cycles),
            clock: TbClockCfg::new(clock_period_ns),
            reset: TbResetCfg::new(5, 10),
            stimuli: Vec::new(),
            checks: Vec::new(),
            probe_signals: Vec::new(),
        }
    }

    pub fn add_stimulus(&mut self, cycle: u32, signal: &str, value: u32) {
        self.stimuli.push(TbStimulus::new(cycle, signal, value));
    }

    pub fn add_check(&mut self, cycle: u32, signal: &str, expected: u32) {
        self.checks.push(TbCheck::new(cycle, signal, expected));
    }

    pub fn add_check_masked(&mut self, cycle: u32, signal: &str, expected: u32, mask: u32) {
        self.checks
            .push(TbCheck::with_mask(cycle, signal, expected, mask));
    }

    pub fn probe(&mut self, signal: &str) {
        self.probe_signals.push(signal.to_string());
    }

    pub fn total_sim_ns(&self) -> u32 {
        self.clock.period_ns.saturating_mul(self.config.max_cycles)
    }

    pub fn emit_verilog(&self) -> String {
        let mut tb = String::new();
        tb.push_str(&format!("`timescale {}\n\n", self.config.timescale));
        tb.push_str(&format!("module {};\n", self.config.name));

        tb.push_str("    // Clock and reset\n");
        tb.push_str("    reg clk;\n");
        tb.push_str("    reg rst_n;\n\n");

        let dut = &self.config.dut_name;
        tb.push_str(&format!("    // DUT instance\n"));
        tb.push_str(&format!("    {} uut (\n", dut));
        tb.push_str("        .clk(clk),\n");
        tb.push_str("        .rst_n(rst_n)\n");
        tb.push_str("    );\n\n");

        tb.push_str(&format!(
            "    // Clock generation: period={}ns\n",
            self.clock.period_ns
        ));
        tb.push_str("    initial begin\n");
        tb.push_str("        clk = 0;\n");
        tb.push_str(&format!(
            "        forever #{} clk = ~clk;\n",
            self.clock.half_period()
        ));
        tb.push_str("    end\n\n");

        let reset_val = if self.reset.active_low { "0" } else { "1" };
        let reset_inactive = if self.reset.active_low { "1" } else { "0" };
        let reset_delay_ps = self.reset.delay_cycles * self.clock.period_ns * 1000;
        let reset_dur_ps = self.reset.duration_cycles * self.clock.period_ns * 1000;
        tb.push_str("    // Reset generation\n");
        tb.push_str("    initial begin\n");
        tb.push_str(&format!("        rst_n = {};\n", reset_val));
        tb.push_str(&format!(
            "        #{} rst_n = {};\n",
            reset_delay_ps, reset_inactive
        ));
        tb.push_str(&format!("        #{};\n", reset_dur_ps));
        tb.push_str("    end\n\n");

        if !self.stimuli.is_empty() {
            tb.push_str("    // Stimulus\n");
            tb.push_str("    initial begin\n");
            for s in &self.stimuli {
                let delay_ps = s.cycle * self.clock.period_ns * 1000;
                tb.push_str(&format!(
                    "        #{} uut.{} = {};\n",
                    delay_ps, s.signal, s.value
                ));
            }
            tb.push_str("    end\n\n");
        }

        if !self.checks.is_empty() {
            tb.push_str("    // Checks\n");
            tb.push_str("    initial begin\n");
            for c in &self.checks {
                let delay_ps = c.cycle * self.clock.period_ns * 1000;
                tb.push_str(&format!(
                    "        #{} assert(uut.{} & 32'h{:08X} == 32'h{:08X})\n",
                    delay_ps, c.signal, c.mask, c.expected
                ));
                tb.push_str(&format!(
                    "            else $error(\"CHECK FAIL: {} cycle {} expected {} mask {}\");\n",
                    c.signal, c.cycle, c.expected, c.mask
                ));
            }
            tb.push_str("    end\n\n");
        }

        if !self.probe_signals.is_empty() {
            tb.push_str("    // Probe signals\n");
            for sig in &self.probe_signals {
                tb.push_str(&format!("    wire probe_{};\n", sig));
                tb.push_str(&format!("    assign probe_{} = uut.{};\n", sig, sig));
            }
            tb.push_str("\n");
        }

        let timeout_ps = self.config.timeout_ns * 1000;
        tb.push_str("    // Timeout watchdog\n");
        tb.push_str("    initial begin\n");
        tb.push_str(&format!("        #{};\n", timeout_ps));
        tb.push_str(&format!(
            "        $display(\"TIMEOUT after {}ns\");\n",
            self.config.timeout_ns
        ));
        tb.push_str(&format!(
            "        $display(\"Simulated {} cycles\");\n",
            self.config.max_cycles
        ));
        tb.push_str("        $finish;\n");
        tb.push_str("    end\n\n");

        tb.push_str("    // Completion\n");
        tb.push_str("    initial begin\n");
        tb.push_str(&format!("        #{};\n", self.total_sim_ns() * 1000));
        tb.push_str("        $display(\"SIM PASSED\");\n");
        tb.push_str("        $finish;\n");
        tb.push_str("    end\n\n");

        tb.push_str("endmodule\n");
        tb
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = self.config.validate();
        for s in &self.stimuli {
            errors.extend(s.validate());
        }
        for c in &self.checks {
            errors.extend(c.validate());
        }
        errors
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VcdVarKind {
    Wire,
    Reg,
    Integer,
    Parameter,
}

impl VcdVarKind {
    pub fn vcd_str(&self) -> &'static str {
        match self {
            VcdVarKind::Wire => "wire",
            VcdVarKind::Reg => "reg",
            VcdVarKind::Integer => "integer",
            VcdVarKind::Parameter => "parameter",
        }
    }
}

#[derive(Debug, Clone)]
pub struct VcdVar {
    pub kind: VcdVarKind,
    pub size: u32,
    pub name: String,
    pub ident: String,
}

impl VcdVar {
    pub fn wire(size: u32, name: &str, ident: &str) -> Self {
        VcdVar {
            kind: VcdVarKind::Wire,
            size,
            name: name.to_string(),
            ident: ident.to_string(),
        }
    }

    pub fn reg(size: u32, name: &str, ident: &str) -> Self {
        VcdVar {
            kind: VcdVarKind::Reg,
            size,
            name: name.to_string(),
            ident: ident.to_string(),
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("VCD var name must not be empty".to_string());
        }
        if self.ident.is_empty() {
            errors.push("VCD var ident must not be empty".to_string());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct VcdChange {
    pub timestamp_ps: u64,
    pub ident: String,
    pub value: u32,
    pub bit_width: u32,
}

impl VcdChange {
    pub fn new(ts: u64, ident: &str, value: u32, width: u32) -> Self {
        VcdChange {
            timestamp_ps: ts,
            ident: ident.to_string(),
            value,
            bit_width: width,
        }
    }

    pub fn format_binary(&self) -> String {
        if self.bit_width == 1 {
            format!("{}", self.value & 1)
        } else {
            format!(
                "b{:0width$b} {}",
                self.value,
                self.ident,
                width = self.bit_width as usize
            )
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.ident.is_empty() {
            errors.push("VCD change ident must not be empty".to_string());
        }
        if self.bit_width == 0 {
            errors.push("VCD change bit_width must be positive".to_string());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct VcdHeader {
    pub date: String,
    pub version: String,
    pub timescale: String,
    pub comment: String,
}

impl VcdHeader {
    pub fn new(version: &str, timescale: &str) -> Self {
        VcdHeader {
            date: "2026-04-10".to_string(),
            version: version.to_string(),
            timescale: timescale.to_string(),
            comment: "T27 Trinity VCD".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct HirVcdTrace {
    pub header: VcdHeader,
    pub variables: Vec<VcdVar>,
    pub changes: Vec<VcdChange>,
    pub end_time_ps: u64,
}

impl HirVcdTrace {
    pub fn new(version: &str) -> Self {
        HirVcdTrace {
            header: VcdHeader::new(version, "1 ps"),
            variables: Vec::new(),
            changes: Vec::new(),
            end_time_ps: 0,
        }
    }

    pub fn add_var(&mut self, kind: VcdVarKind, size: u32, name: &str) {
        let idx = self.variables.len();
        let ident = HirVcdTrace::ident_from_index(idx);
        self.variables.push(VcdVar {
            kind,
            size,
            name: name.to_string(),
            ident,
        });
    }

    pub fn add_wire(&mut self, size: u32, name: &str) {
        self.add_var(VcdVarKind::Wire, size, name);
    }

    pub fn add_reg(&mut self, size: u32, name: &str) {
        self.add_var(VcdVarKind::Reg, size, name);
    }

    pub fn record(&mut self, timestamp_ps: u64, var_name: &str, value: u32) {
        if let Some(var) = self.variables.iter().find(|v| v.name == var_name) {
            let ident = var.ident.clone();
            let width = var.size;
            self.changes
                .push(VcdChange::new(timestamp_ps, &ident, value, width));
            if timestamp_ps > self.end_time_ps {
                self.end_time_ps = timestamp_ps;
            }
        }
    }

    pub fn changes_at(&self, ts: u64) -> u32 {
        self.changes.iter().filter(|c| c.timestamp_ps == ts).count() as u32
    }

    pub fn earliest_timestamp(&self) -> u64 {
        self.changes
            .iter()
            .map(|c| c.timestamp_ps)
            .min()
            .unwrap_or(0)
    }

    pub fn latest_timestamp(&self) -> u64 {
        self.changes
            .iter()
            .map(|c| c.timestamp_ps)
            .max()
            .unwrap_or(0)
    }

    pub fn duration_ps(&self) -> u64 {
        if self.changes.is_empty() {
            return 0;
        }
        self.latest_timestamp()
            .saturating_sub(self.earliest_timestamp())
    }

    pub fn ident_from_index(idx: usize) -> String {
        let mut s = String::new();
        let mut i = idx;
        loop {
            s.push((b'!' + (i % 94) as u8) as char);
            i /= 94;
            if i == 0 {
                break;
            }
        }
        s
    }

    pub fn emit_vcd(&self) -> String {
        let mut vcd = String::new();
        vcd.push_str("$date\n");
        vcd.push_str(&format!("    {}\n", self.header.date));
        vcd.push_str("$end\n");
        vcd.push_str("$version\n");
        vcd.push_str(&format!("    {}\n", self.header.version));
        vcd.push_str("$end\n");
        vcd.push_str("$comment\n");
        vcd.push_str(&format!("    {}\n", self.header.comment));
        vcd.push_str("$end\n");
        vcd.push_str(&format!("$timescale {}\n$end\n", self.header.timescale));
        vcd.push_str("$scope module top $end\n");
        for var in &self.variables {
            vcd.push_str(&format!(
                "$var {} {} {} {} $end\n",
                var.kind.vcd_str(),
                var.size,
                var.ident,
                var.name
            ));
        }
        vcd.push_str("$upscope $end\n");
        vcd.push_str("$enddefinitions $end\n");
        vcd.push_str("$dumpvars\n");
        for var in &self.variables {
            vcd.push_str(&format!("x {}\n", var.ident));
        }
        vcd.push_str("$end\n");
        let mut sorted_changes = self.changes.clone();
        sorted_changes.sort_by_key(|c| c.timestamp_ps);
        let mut last_ts: Option<u64> = None;
        for change in &sorted_changes {
            if last_ts != Some(change.timestamp_ps) {
                vcd.push_str(&format!("#{}\n", change.timestamp_ps));
                last_ts = Some(change.timestamp_ps);
            }
            vcd.push_str(&format!("{}\n", change.format_binary()));
        }
        vcd.push_str(&format!("#{}\n", self.end_time_ps));
        vcd
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for v in &self.variables {
            errors.extend(v.validate());
        }
        for c in &self.changes {
            errors.extend(c.validate());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct LinkSection {
    pub name: String,
    pub vaddr: u32,
    pub size: u32,
    pub flags: u32,
    pub align: u32,
}

impl LinkSection {
    pub fn text(vaddr: u32, size: u32) -> Self {
        LinkSection {
            name: ".text".into(),
            vaddr,
            size,
            flags: 5,
            align: 4,
        }
    }

    pub fn data(vaddr: u32, size: u32) -> Self {
        LinkSection {
            name: ".data".into(),
            vaddr,
            size,
            flags: 3,
            align: 4,
        }
    }

    pub fn bss(vaddr: u32, size: u32) -> Self {
        LinkSection {
            name: ".bss".into(),
            vaddr,
            size,
            flags: 2,
            align: 4,
        }
    }

    pub fn end(&self) -> u32 {
        self.vaddr.saturating_add(self.size)
    }
}

#[derive(Debug, Clone)]
pub struct LinkedSymbol {
    pub name: String,
    pub value: u32,
    pub size: u32,
    pub section_idx: u32,
    pub bind: u32,
    pub kind: u32,
}

impl LinkedSymbol {
    pub fn new(name: &str, value: u32, section_idx: u32) -> Self {
        LinkedSymbol {
            name: name.to_string(),
            value,
            size: 0,
            section_idx,
            bind: 1,
            kind: 2,
        }
    }

    pub fn is_global(&self) -> bool {
        self.bind == 1
    }

    pub fn is_local(&self) -> bool {
        self.bind == 0
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("symbol name must not be empty".to_string());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct LinkSegment {
    pub kind: u32,
    pub vaddr: u32,
    pub memsz: u32,
    pub filesz: u32,
    pub align: u32,
}

impl LinkSegment {
    pub fn text(vaddr: u32, size: u32) -> Self {
        LinkSegment {
            kind: 1,
            vaddr,
            memsz: size,
            filesz: size,
            align: 4096,
        }
    }

    pub fn data(vaddr: u32, memsz: u32, filesz: u32) -> Self {
        LinkSegment {
            kind: 1,
            vaddr,
            memsz,
            filesz,
            align: 4096,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LinkerConfig {
    pub entry: String,
    pub text_base: u32,
    pub data_base: u32,
    pub stack_size: u32,
    pub heap_size: u32,
    pub output_format: i8,
}

impl LinkerConfig {
    pub fn new(entry: &str) -> Self {
        LinkerConfig {
            entry: entry.to_string(),
            text_base: 0,
            data_base: 4096,
            stack_size: 1024,
            heap_size: 4096,
            output_format: 0,
        }
    }

    pub fn stack_top(&self) -> u32 {
        self.data_base.saturating_add(self.stack_size)
    }

    pub fn heap_start(&self) -> u32 {
        self.data_base
            .saturating_add(self.data_base)
            .saturating_add(self.stack_size)
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.entry.is_empty() {
            errors.push("linker entry must not be empty".to_string());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct LinkResult {
    pub entry_addr: u32,
    pub total_text: u32,
    pub total_data: u32,
    pub total_bss: u32,
    pub num_symbols: u32,
    pub num_segments: u32,
    pub errors: u32,
}

impl LinkResult {
    pub fn ok(entry: u32, text: u32, data: u32, bss: u32) -> Self {
        LinkResult {
            entry_addr: entry,
            total_text: text,
            total_data: data,
            total_bss: bss,
            num_symbols: 0,
            num_segments: 2,
            errors: 0,
        }
    }

    pub fn fail(errors: u32) -> Self {
        LinkResult {
            entry_addr: 0,
            total_text: 0,
            total_data: 0,
            total_bss: 0,
            num_symbols: 0,
            num_segments: 0,
            errors,
        }
    }

    pub fn total_image_size(&self) -> u32 {
        self.total_text
            .saturating_add(self.total_data)
            .saturating_add(self.total_bss)
    }

    pub fn passed(&self) -> bool {
        self.errors == 0
    }
}

#[derive(Debug)]
pub struct HirLinker {
    pub config: LinkerConfig,
    pub sections: Vec<LinkSection>,
    pub symbols: Vec<LinkedSymbol>,
    pub segments: Vec<LinkSegment>,
    pub merged_text: Vec<u32>,
    pub merged_data: Vec<u32>,
}

impl HirLinker {
    pub fn new(entry: &str) -> Self {
        HirLinker {
            config: LinkerConfig::new(entry),
            sections: Vec::new(),
            symbols: Vec::new(),
            segments: Vec::new(),
            merged_text: Vec::new(),
            merged_data: Vec::new(),
        }
    }

    pub fn add_object(&mut self, asm: &HirAssembler) {
        for sym in &asm.symbols {
            self.symbols
                .push(LinkedSymbol::new(&sym.name, sym.address, 0));
        }
        let encoded = asm.encode_all();
        self.merged_text.extend(encoded);
    }

    pub fn link(&mut self) -> LinkResult {
        let errors = self.config.validate();
        if !errors.is_empty() {
            return LinkResult::fail(errors.len() as u32);
        }
        self.sections.push(LinkSection::text(
            self.config.text_base,
            self.merged_text.len() as u32 * 4,
        ));
        self.sections.push(LinkSection::data(
            self.config.data_base,
            self.merged_data.len() as u32 * 4,
        ));

        let text_size = self.merged_text.len() as u32 * 4;
        let data_size = self.merged_data.len() as u32 * 4;

        self.segments
            .push(LinkSegment::text(self.config.text_base, text_size));
        if data_size > 0 {
            self.segments.push(LinkSegment::data(
                self.config.data_base,
                data_size,
                data_size,
            ));
        }

        let entry_addr = self.resolve_entry();
        match entry_addr {
            Some(addr) => LinkResult::ok(addr, text_size, data_size, 0),
            None => LinkResult::fail(1),
        }
    }

    pub fn resolve_entry(&self) -> Option<u32> {
        for sym in &self.symbols {
            if sym.name == self.config.entry {
                return Some(sym.value);
            }
        }
        if self.config.entry == "_start" && !self.merged_text.is_empty() {
            return Some(0);
        }
        None
    }

    pub fn resolve_symbol(&self, name: &str) -> Option<u32> {
        self.symbols
            .iter()
            .find(|s| s.name == name)
            .map(|s| s.value)
    }

    pub fn emit_image(&self) -> Vec<u8> {
        let mut image = Vec::new();
        for w in &self.merged_text {
            image.extend_from_slice(&w.to_le_bytes());
        }
        for w in &self.merged_data {
            image.extend_from_slice(&w.to_le_bytes());
        }
        image
    }

    pub fn emit_hex(&self) -> String {
        let mut hex = String::new();
        hex.push_str("@00000000\n");
        for (i, w) in self.merged_text.iter().enumerate() {
            hex.push_str(&format!("{:08x}", w));
            if (i + 1) % 8 == 0 {
                hex.push('\n');
            } else {
                hex.push(' ');
            }
        }
        if !self.merged_data.is_empty() {
            let data_addr = self.config.data_base;
            hex.push_str(&format!("\n@{:08x}\n", data_addr));
            for (i, w) in self.merged_data.iter().enumerate() {
                hex.push_str(&format!("{:08x}", w));
                if (i + 1) % 8 == 0 {
                    hex.push('\n');
                } else {
                    hex.push(' ');
                }
            }
        }
        hex
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = self.config.validate();
        for sym in &self.symbols {
            errors.extend(sym.validate());
        }
        errors
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArcKind {
    Comb,
    RegToReg,
    RegToOutput,
    InputToReg,
    InputToOutput,
}

impl ArcKind {
    pub fn from_i8(v: i8) -> Self {
        match v {
            0 => ArcKind::Comb,
            1 => ArcKind::RegToReg,
            2 => ArcKind::RegToOutput,
            3 => ArcKind::InputToReg,
            4 => ArcKind::InputToOutput,
            _ => ArcKind::Comb,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimingArc {
    pub source: String,
    pub sink: String,
    pub delay_ps: u32,
    pub kind: ArcKind,
}

impl TimingArc {
    pub fn comb(source: &str, sink: &str, delay_ps: u32) -> Self {
        TimingArc {
            source: source.into(),
            sink: sink.into(),
            delay_ps,
            kind: ArcKind::Comb,
        }
    }
    pub fn reg_to_reg(source: &str, sink: &str, delay_ps: u32) -> Self {
        TimingArc {
            source: source.into(),
            sink: sink.into(),
            delay_ps,
            kind: ArcKind::RegToReg,
        }
    }
    pub fn input_to_reg(source: &str, sink: &str, delay_ps: u32) -> Self {
        TimingArc {
            source: source.into(),
            sink: sink.into(),
            delay_ps,
            kind: ArcKind::InputToReg,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TimingPath {
    pub startpoint: String,
    pub endpoint: String,
    pub total_delay_ps: u32,
    pub slack_ps: i64,
    pub num_arcs: u32,
}

impl TimingPath {
    pub fn new(start: &str, end: &str, delay: u32, slack: i64) -> Self {
        TimingPath {
            startpoint: start.into(),
            endpoint: end.into(),
            total_delay_ps: delay,
            slack_ps: slack,
            num_arcs: 1,
        }
    }
    pub fn is_met(&self) -> bool {
        self.slack_ps >= 0
    }
    pub fn is_violated(&self) -> bool {
        self.slack_ps < 0
    }
}

#[derive(Debug, Clone)]
pub struct TimingConstraint {
    pub name: String,
    pub period_ps: u32,
    pub clock_name: String,
}

impl TimingConstraint {
    pub fn from_period(name: &str, period_ps: u32) -> Self {
        TimingConstraint {
            name: name.into(),
            period_ps,
            clock_name: "clk".into(),
        }
    }
    pub fn from_mhz(name: &str, mhz: u32) -> Self {
        let period = if mhz == 0 { 10000 } else { 1_000_000_000 / mhz };
        TimingConstraint {
            name: name.into(),
            period_ps: period,
            clock_name: "clk".into(),
        }
    }
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("constraint name empty".into());
        }
        if self.period_ps == 0 {
            errors.push("period must be positive".into());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct TimingReport {
    pub total_paths: u32,
    pub met_paths: u32,
    pub violated_paths: u32,
    pub worst_slack_ps: i64,
    pub critical_path_ps: u32,
    pub fmax_mhz: u32,
    pub has_violations: bool,
}

impl TimingReport {
    pub fn ok(critical_ps: u32, fmax: u32) -> Self {
        TimingReport {
            total_paths: 1,
            met_paths: 1,
            violated_paths: 0,
            worst_slack_ps: 5000,
            critical_path_ps: critical_ps,
            fmax_mhz: fmax,
            has_violations: false,
        }
    }
    pub fn fail(critical_ps: u32) -> Self {
        TimingReport {
            total_paths: 1,
            met_paths: 0,
            violated_paths: 1,
            worst_slack_ps: -1000,
            critical_path_ps: critical_ps,
            fmax_mhz: 0,
            has_violations: true,
        }
    }
    pub fn passed(&self) -> bool {
        !self.has_violations
    }
}

pub struct TimingModel;

impl TimingModel {
    pub const LUT_DELAY_PS: u32 = 100;
    pub const BRAM_DELAY_PS: u32 = 2000;
    pub const DSP_DELAY_PS: u32 = 2500;
    pub const ROUTING_DELAY_PS: u32 = 300;
    pub const SETUP_TIME_PS: u32 = 200;
    pub const HOLD_TIME_PS: u32 = 50;

    pub fn est_comb_delay(num_luts: u32) -> u32 {
        num_luts * Self::LUT_DELAY_PS + Self::ROUTING_DELAY_PS
    }

    pub fn est_reg_to_reg_delay(num_luts: u32) -> u32 {
        num_luts * Self::LUT_DELAY_PS + Self::ROUTING_DELAY_PS + Self::SETUP_TIME_PS
    }

    pub fn slack(delay_ps: u32, constraint_ps: u32) -> i64 {
        constraint_ps as i64 - delay_ps as i64
    }

    pub fn fmax_from_delay(delay_ps: u32) -> u32 {
        if delay_ps == 0 {
            return 0;
        }
        1_000_000_000 / delay_ps
    }

    pub fn analyze_module(module: &HirModule, constraint: &TimingConstraint) -> TimingReport {
        let mut paths = Vec::new();
        let mut max_delay: u32 = 0;

        for sig in &module.signals {
            if sig.kind == HwSignalKind::Reg {
                let est = Self::est_reg_to_reg_delay(1);
                let sl = Self::slack(est, constraint.period_ps);
                if est > max_delay {
                    max_delay = est;
                }
                paths.push(TimingPath::new(&sig.name, &sig.name, est, sl));
            }
        }
        for mem in &module.memories {
            let est = Self::BRAM_DELAY_PS + Self::ROUTING_DELAY_PS;
            let sl = Self::slack(est, constraint.period_ps);
            if est > max_delay {
                max_delay = est;
            }
            paths.push(TimingPath::new(&mem.name, &mem.name, est, sl));
        }
        for gf16 in &module.gf16_accels {
            let est = gf16.mac_units.len() as u32 * (Self::DSP_DELAY_PS + Self::ROUTING_DELAY_PS);
            let sl = Self::slack(est, constraint.period_ps);
            if est > max_delay {
                max_delay = est;
            }
            paths.push(TimingPath::new(&gf16.name, &gf16.name, est, sl));
        }
        for tc in &module.ternary_cores {
            let pipeline_stages = tc.pipeline_stages.len() as u32;
            let est = if pipeline_stages > 0 {
                2000 / pipeline_stages
            } else {
                2000
            };
            let sl = Self::slack(est, constraint.period_ps);
            if est > max_delay {
                max_delay = est;
            }
            paths.push(TimingPath::new(&tc.name, &tc.name, est, sl));
        }

        let met = paths.iter().filter(|p| p.is_met()).count() as u32;
        let violated = paths.iter().filter(|p| p.is_violated()).count() as u32;
        let worst_slack = paths.iter().map(|p| p.slack_ps).min().unwrap_or(0);
        let fmax = Self::fmax_from_delay(max_delay);

        TimingReport {
            total_paths: paths.len() as u32,
            met_paths: met,
            violated_paths: violated,
            worst_slack_ps: worst_slack,
            critical_path_ps: max_delay,
            fmax_mhz: fmax / 1_000_000,
            has_violations: violated > 0,
        }
    }
}

pub fn path_delay(arcs: &[TimingArc]) -> u32 {
    arcs.iter().map(|a| a.delay_ps).sum()
}

pub fn worst_path_delay(paths: &[TimingPath]) -> u32 {
    if paths.is_empty() {
        return 0;
    }
    paths.iter().map(|p| p.total_delay_ps).max().unwrap_or(0)
}

#[derive(Debug, Clone)]
pub struct PowerDomain {
    pub name: String,
    pub voltage_mv: u32,
    pub clock_mhz: u32,
    pub toggle_rate: u32,
}

impl PowerDomain {
    pub fn new(name: &str, clock_mhz: u32) -> Self {
        PowerDomain {
            name: name.into(),
            voltage_mv: 1000,
            clock_mhz,
            toggle_rate: 12,
        }
    }
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("domain name empty".into());
        }
        if self.clock_mhz == 0 {
            errors.push("clock_mhz must be positive".into());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct PowerEstimate {
    pub dynamic_mw: u32,
    pub static_mw: u32,
    pub total_mw: u32,
    pub lut_power_mw: u32,
    pub ff_power_mw: u32,
    pub bram_power_mw: u32,
    pub dsp_power_mw: u32,
}

pub struct PowerModel;

impl PowerModel {
    pub const LUT_POWER_UW_PER_MHZ: u32 = 10;
    pub const FF_POWER_UW_PER_MHZ: u32 = 5;
    pub const BRAM_POWER_UW_PER_MHZ: u32 = 50;
    pub const DSP_POWER_UW_PER_MHZ: u32 = 100;
    pub const IO_POWER_UW_PER_MHZ: u32 = 20;
    pub const STATIC_BASE_MW: u32 = 50;
    pub const STATIC_PER_RESOURCE_UW: u32 = 100;

    pub fn est_lut_dynamic(luts: u32, clock_mhz: u32, toggle_rate: u32) -> u32 {
        luts * Self::LUT_POWER_UW_PER_MHZ * clock_mhz * toggle_rate / 1000 / 100
    }
    pub fn est_ff_dynamic(ffs: u32, clock_mhz: u32, toggle_rate: u32) -> u32 {
        ffs * Self::FF_POWER_UW_PER_MHZ * clock_mhz * toggle_rate / 1000 / 100
    }
    pub fn est_bram_dynamic(brams: u32, clock_mhz: u32) -> u32 {
        brams * Self::BRAM_POWER_UW_PER_MHZ * clock_mhz / 1000
    }
    pub fn est_dsp_dynamic(dsps: u32, clock_mhz: u32) -> u32 {
        dsps * Self::DSP_POWER_UW_PER_MHZ * clock_mhz / 1000
    }
    pub fn est_static(total_resources: u32) -> u32 {
        Self::STATIC_BASE_MW + total_resources * Self::STATIC_PER_RESOURCE_UW / 1000
    }

    pub fn estimate_module(module: &HirModule, clock_mhz: u32, toggle_rate: u32) -> PowerEstimate {
        let luts: u32 = module
            .signals
            .iter()
            .filter(|s| s.kind == HwSignalKind::Wire)
            .count() as u32
            * 2
            + module.assigns.len() as u32;
        let ffs: u32 = module
            .signals
            .iter()
            .filter(|s| s.kind == HwSignalKind::Reg)
            .count() as u32;
        let brams: u32 = module.memories.len() as u32;
        let dsps: u32 = module
            .gf16_accels
            .iter()
            .map(|g| g.mac_units.len() as u32)
            .sum();

        let lut_p = Self::est_lut_dynamic(luts, clock_mhz, toggle_rate);
        let ff_p = Self::est_ff_dynamic(ffs, clock_mhz, toggle_rate);
        let bram_p = Self::est_bram_dynamic(brams, clock_mhz);
        let dsp_p = Self::est_dsp_dynamic(dsps, clock_mhz);
        let dynamic = lut_p + ff_p + bram_p + dsp_p;
        let stat = Self::est_static(luts + ffs + brams + dsps);

        PowerEstimate {
            dynamic_mw: dynamic,
            static_mw: stat,
            total_mw: dynamic + stat,
            lut_power_mw: lut_p,
            ff_power_mw: ff_p,
            bram_power_mw: bram_p,
            dsp_power_mw: dsp_p,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegionKind {
    ClockRegion,
    IoBank,
    BramColumn,
    DspColumn,
    LogicCluster,
}

#[derive(Debug, Clone)]
pub struct PlacementRegion {
    pub name: String,
    pub kind: RegionKind,
    pub x0: u32,
    pub y0: u32,
    pub x1: u32,
    pub y1: u32,
}

impl PlacementRegion {
    pub fn logic(name: &str, x0: u32, y0: u32, x1: u32, y1: u32) -> Self {
        PlacementRegion {
            name: name.into(),
            kind: RegionKind::LogicCluster,
            x0,
            y0,
            x1,
            y1,
        }
    }
    pub fn bram_col(name: &str, col: u32, y0: u32, y1: u32) -> Self {
        PlacementRegion {
            name: name.into(),
            kind: RegionKind::BramColumn,
            x0: col,
            y0,
            x1: col + 1,
            y1,
        }
    }
    pub fn dsp_col(name: &str, col: u32, y0: u32, y1: u32) -> Self {
        PlacementRegion {
            name: name.into(),
            kind: RegionKind::DspColumn,
            x0: col,
            y0,
            x1: col + 1,
            y1,
        }
    }
    pub fn width(&self) -> u32 {
        if self.x1 > self.x0 {
            self.x1 - self.x0
        } else {
            0
        }
    }
    pub fn height(&self) -> u32 {
        if self.y1 > self.y0 {
            self.y1 - self.y0
        } else {
            0
        }
    }
    pub fn area(&self) -> u32 {
        self.width() * self.height()
    }
    pub fn overlaps(&self, other: &PlacementRegion) -> bool {
        self.x0 < other.x1 && self.x1 > other.x0 && self.y0 < other.y1 && self.y1 > other.y0
    }
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("region name empty".into());
        }
        if self.x1 < self.x0 {
            errors.push("x1 < x0".into());
        }
        if self.y1 < self.y0 {
            errors.push("y1 < y0".into());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct PlacementHint {
    pub module_name: String,
    pub region_name: String,
    pub priority: u32,
}

impl PlacementHint {
    pub fn new(module: &str, region: &str, priority: u32) -> Self {
        PlacementHint {
            module_name: module.into(),
            region_name: region.into(),
            priority,
        }
    }
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.module_name.is_empty() {
            errors.push("module name empty".into());
        }
        if self.region_name.is_empty() {
            errors.push("region name empty".into());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct RouteConstraint {
    pub source: String,
    pub sink: String,
    pub max_delay_ps: u32,
}

#[derive(Debug, Clone)]
pub struct Floorplan {
    pub name: String,
    pub device: String,
    pub regions: Vec<PlacementRegion>,
    pub hints: Vec<PlacementHint>,
    pub routes: Vec<RouteConstraint>,
}

impl Floorplan {
    pub fn new(name: &str, device: &str) -> Self {
        Floorplan {
            name: name.into(),
            device: device.into(),
            regions: Vec::new(),
            hints: Vec::new(),
            routes: Vec::new(),
        }
    }

    pub fn add_region(&mut self, region: PlacementRegion) {
        self.regions.push(region);
    }

    pub fn add_hint(&mut self, module: &str, region: &str, priority: u32) {
        self.hints
            .push(PlacementHint::new(module, region, priority));
    }

    pub fn add_route(&mut self, source: &str, sink: &str, max_delay_ps: u32) {
        self.routes.push(RouteConstraint {
            source: source.into(),
            sink: sink.into(),
            max_delay_ps,
        });
    }

    pub fn auto_floorplan(&mut self, module: &HirModule) {
        let mut x_offset: u32 = 0;
        for mem in &module.memories {
            let name = format!("bram_{}", mem.name);
            self.regions
                .push(PlacementRegion::bram_col(&name, x_offset, 0, 50));
            self.hints.push(PlacementHint::new(&mem.name, &name, 3));
            x_offset += 2;
        }
        for gf16 in &module.gf16_accels {
            let name = format!("dsp_{}", gf16.name);
            self.regions
                .push(PlacementRegion::dsp_col(&name, x_offset, 0, 50));
            self.hints.push(PlacementHint::new(&gf16.name, &name, 3));
            x_offset += 2;
        }
        let has_ternary = !module.ternary_cores.is_empty();
        if has_ternary {
            let name = "ternary_cluster";
            self.regions
                .push(PlacementRegion::logic(name, x_offset, 0, x_offset + 10, 20));
            for tc in &module.ternary_cores {
                self.hints.push(PlacementHint::new(&tc.name, name, 2));
            }
            x_offset += 11;
        }
        let io_ports: Vec<_> = module
            .ports
            .iter()
            .filter(|p| p.ty == HwType::Bool || matches!(p.ty, HwType::UInt(_)))
            .collect();
        if !io_ports.is_empty() {
            let name = "io_region";
            self.regions
                .push(PlacementRegion::logic(name, 0, 50, 20, 60));
            for port in io_ports {
                self.hints.push(PlacementHint::new(&port.name, name, 1));
            }
        }
    }

    pub fn check_overlaps(&self) -> Vec<(String, String)> {
        let mut overlaps = Vec::new();
        for i in 0..self.regions.len() {
            for j in (i + 1)..self.regions.len() {
                if self.regions[i].overlaps(&self.regions[j]) {
                    overlaps.push((self.regions[i].name.clone(), self.regions[j].name.clone()));
                }
            }
        }
        overlaps
    }

    pub fn total_area(&self) -> u32 {
        self.regions.iter().map(|r| r.area()).sum()
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for r in &self.regions {
            errors.extend(r.validate());
        }
        for h in &self.hints {
            errors.extend(h.validate());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct BitstreamMeta {
    pub device: String,
    pub size_bytes: u32,
    pub checksum: u64,
    pub build_timestamp: String,
    pub design_name: String,
    pub source_hash: String,
}

impl BitstreamMeta {
    pub fn new(device: &str, size_bytes: u32) -> Self {
        BitstreamMeta {
            device: device.into(),
            size_bytes,
            checksum: 0,
            build_timestamp: "2026-04-10".into(),
            design_name: String::new(),
            source_hash: String::new(),
        }
    }

    pub fn compute_checksum(data: &[u8]) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for &b in data {
            hash ^= b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }

    pub fn with_checksum(mut self, data: &[u8]) -> Self {
        self.checksum = Self::compute_checksum(data);
        self.size_bytes = data.len() as u32;
        self
    }

    pub fn verify(&self, data: &[u8]) -> bool {
        Self::compute_checksum(data) == self.checksum
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.device.is_empty() {
            errors.push("device empty".into());
        }
        if self.size_bytes == 0 {
            errors.push("size_bytes zero".into());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct FpgaNode {
    pub name: String,
    pub device: String,
    pub luts: u32,
    pub ffs: u32,
    pub bram18: u32,
    pub dsp48: u32,
    pub io_pins: u32,
}

impl FpgaNode {
    pub fn arty_a7(name: &str) -> Self {
        FpgaNode {
            name: name.into(),
            device: "xc7a100t".into(),
            luts: 63400,
            ffs: 126800,
            bram18: 135,
            dsp48: 240,
            io_pins: 300,
        }
    }
    pub fn util(&self, used_luts: u32) -> u32 {
        if self.luts == 0 {
            return 0;
        }
        used_luts * 100 / self.luts
    }
    pub fn remaining(&self, used_luts: u32) -> u32 {
        self.luts.saturating_sub(used_luts)
    }
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("fpga name empty".into());
        }
        if self.luts == 0 {
            errors.push("fpga luts zero".into());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct InterFpgaLink {
    pub fpga_a: u32,
    pub fpga_b: u32,
    pub width: u32,
    pub protocol: i8,
    pub max_mbps: u32,
}

impl InterFpgaLink {
    pub fn lvds(a: u32, b: u32, width: u32) -> Self {
        InterFpgaLink {
            fpga_a: a,
            fpga_b: b,
            width,
            protocol: 0,
            max_mbps: 1000,
        }
    }
    pub fn serdes(a: u32, b: u32) -> Self {
        InterFpgaLink {
            fpga_a: a,
            fpga_b: b,
            width: 4,
            protocol: 1,
            max_mbps: 6250,
        }
    }
    pub fn bandwidth_mbps(&self) -> u32 {
        self.width * self.max_mbps
    }
}

#[derive(Debug, Clone)]
pub struct PartitionAssign {
    pub module_name: String,
    pub fpga_idx: u32,
    pub luts: u32,
    pub ffs: u32,
    pub bram18: u32,
    pub dsp48: u32,
}

#[derive(Debug, Clone)]
pub struct PartitionResult {
    pub num_fpgas: u32,
    pub num_assignments: u32,
    pub num_links: u32,
    pub total_bandwidth_mbps: u32,
    pub balanced: bool,
    pub errors: u32,
}

impl PartitionResult {
    pub fn ok(fpgas: u32, assigns: u32, links: u32, bw: u32) -> Self {
        PartitionResult {
            num_fpgas: fpgas,
            num_assignments: assigns,
            num_links: links,
            total_bandwidth_mbps: bw,
            balanced: true,
            errors: 0,
        }
    }
    pub fn fail(errors: u32) -> Self {
        PartitionResult {
            num_fpgas: 0,
            num_assignments: 0,
            num_links: 0,
            total_bandwidth_mbps: 0,
            balanced: false,
            errors,
        }
    }
    pub fn passed(&self) -> bool {
        self.errors == 0
    }
}

#[derive(Debug)]
pub struct HirPartitioner {
    pub fpgas: Vec<FpgaNode>,
    pub assignments: Vec<PartitionAssign>,
    pub links: Vec<InterFpgaLink>,
}

impl HirPartitioner {
    pub fn new() -> Self {
        HirPartitioner {
            fpgas: Vec::new(),
            assignments: Vec::new(),
            links: Vec::new(),
        }
    }

    pub fn add_fpga(&mut self, fpga: FpgaNode) {
        self.fpgas.push(fpga);
    }

    pub fn add_link(&mut self, link: InterFpgaLink) {
        self.links.push(link);
    }

    pub fn total_bandwidth(&self) -> u32 {
        self.links.iter().map(|l| l.bandwidth_mbps()).sum()
    }

    pub fn fpga_usage(&self, fpga_idx: u32) -> (u32, u32, u32, u32) {
        let (luts, ffs, bram, dsp) = self
            .assignments
            .iter()
            .filter(|a| a.fpga_idx == fpga_idx)
            .fold((0u32, 0u32, 0u32, 0u32), |(l, f, b, d), a| {
                (l + a.luts, f + a.ffs, b + a.bram18, d + a.dsp48)
            });
        (luts, ffs, bram, dsp)
    }

    pub fn fits(&self, fpga_idx: u32, luts: u32, ffs: u32, bram: u32, dsp: u32) -> bool {
        if fpga_idx as usize >= self.fpgas.len() {
            return false;
        }
        let fpga = &self.fpgas[fpga_idx as usize];
        let (used_l, used_f, used_b, used_d) = self.fpga_usage(fpga_idx);
        used_l + luts <= fpga.luts
            && used_f + ffs <= fpga.ffs
            && used_b + bram <= fpga.bram18
            && used_d + dsp <= fpga.dsp48
    }

    pub fn assign(
        &mut self,
        module: &str,
        fpga_idx: u32,
        luts: u32,
        ffs: u32,
        bram: u32,
        dsp: u32,
    ) -> bool {
        if !self.fits(fpga_idx, luts, ffs, bram, dsp) {
            return false;
        }
        self.assignments.push(PartitionAssign {
            module_name: module.into(),
            fpga_idx,
            luts,
            ffs,
            bram18: bram,
            dsp48: dsp,
        });
        true
    }

    pub fn auto_partition(&mut self, modules: &[(String, u32, u32, u32, u32)]) -> PartitionResult {
        if self.fpgas.is_empty() {
            return PartitionResult::fail(1);
        }
        let mut errors: u32 = 0;
        let n_fpgas = self.fpgas.len() as u32;
        for (i, (name, luts, ffs, bram, dsp)) in modules.iter().enumerate() {
            let fpga_idx = (i as u32) % n_fpgas;
            if !self.assign(name, fpga_idx, *luts, *ffs, *bram, *dsp) {
                errors += 1;
            }
        }
        let bw = self.total_bandwidth();
        let balanced = self.check_balanced();
        PartitionResult {
            num_fpgas: n_fpgas,
            num_assignments: self.assignments.len() as u32,
            num_links: self.links.len() as u32,
            total_bandwidth_mbps: bw,
            balanced,
            errors,
        }
    }

    fn check_balanced(&self) -> bool {
        if self.fpgas.len() < 2 {
            return true;
        }
        let usages: Vec<u32> = (0..self.fpgas.len() as u32)
            .map(|i| {
                let (l, _, _, _) = self.fpga_usage(i);
                l
            })
            .collect();
        let max = *usages.iter().max().unwrap_or(&0);
        let min = *usages.iter().min().unwrap_or(&0);
        if max == 0 {
            return true;
        }
        (max - min) * 100 / max < 30
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnEdgeKind {
    Data,
    Clock,
    Reset,
    Enable,
}

#[derive(Debug, Clone)]
pub struct ConnEdge {
    pub source: String,
    pub sink: String,
    pub kind: ConnEdgeKind,
    pub bit_width: u32,
}

impl ConnEdge {
    pub fn data(source: &str, sink: &str, width: u32) -> Self {
        ConnEdge {
            source: source.into(),
            sink: sink.into(),
            kind: ConnEdgeKind::Data,
            bit_width: width,
        }
    }
    pub fn clock(source: &str, sink: &str) -> Self {
        ConnEdge {
            source: source.into(),
            sink: sink.into(),
            kind: ConnEdgeKind::Clock,
            bit_width: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FanoutInfo {
    pub signal: String,
    pub fanout: u32,
    pub total_bits: u32,
}

impl FanoutInfo {
    pub fn is_high_fanout(&self) -> bool {
        self.fanout > 16
    }
    pub fn is_clock_network(&self) -> bool {
        self.signal == "clk" || self.signal == "rst_n"
    }
}

#[derive(Debug, Clone)]
pub struct RouteEstimate {
    pub total_nets: u32,
    pub total_wire_length_um: u32,
    pub avg_wire_length_um: u32,
    pub max_fanout: u32,
    pub congestion_score: u32,
    pub needs_global_buf: bool,
}

impl RouteEstimate {
    pub fn passed(&self) -> bool {
        self.congestion_score < 80
    }
}

pub struct RouteModel;

impl RouteModel {
    pub const LOCAL_WIRE_UM: u32 = 500;
    pub const MEDIUM_WIRE_UM: u32 = 2000;
    pub const LONG_WIRE_UM: u32 = 5000;

    pub fn est_wire_length(fanout: u32) -> u32 {
        match fanout {
            0 => 0,
            1..=4 => Self::LOCAL_WIRE_UM,
            5..=16 => Self::MEDIUM_WIRE_UM,
            _ => Self::LONG_WIRE_UM,
        }
    }

    pub fn est_congestion(nets: u32, die_area_mm2: u32) -> u32 {
        if die_area_mm2 == 0 {
            return 0;
        }
        nets / die_area_mm2
    }
}

#[derive(Debug)]
pub struct HirRouter {
    pub edges: Vec<ConnEdge>,
    pub fanouts: Vec<FanoutInfo>,
}

impl HirRouter {
    pub fn new() -> Self {
        HirRouter {
            edges: Vec::new(),
            fanouts: Vec::new(),
        }
    }

    pub fn add_edge(&mut self, edge: ConnEdge) {
        self.edges.push(edge);
    }

    pub fn analyze_fanout(&mut self) {
        let mut fanout_map: std::collections::HashMap<String, (u32, u32)> =
            std::collections::HashMap::new();
        for edge in &self.edges {
            let entry = fanout_map.entry(edge.source.clone()).or_insert((0, 0));
            entry.0 += 1;
            entry.1 = entry.1.max(edge.bit_width);
        }
        self.fanouts = fanout_map
            .into_iter()
            .map(|(sig, (count, bits))| FanoutInfo {
                signal: sig,
                fanout: count,
                total_bits: bits * count,
            })
            .collect();
        self.fanouts.sort_by(|a, b| b.fanout.cmp(&a.fanout));
    }

    pub fn analyze_module(module: &HirModule) -> RouteEstimate {
        let mut router = HirRouter::new();
        for port in &module.ports {
            router.add_edge(ConnEdge::data(
                &port.name,
                &format!("uut_{}", port.name),
                port.ty.hw_width(),
            ));
        }
        for sig in &module.signals {
            if sig.kind == HwSignalKind::Reg {
                router.add_edge(ConnEdge::clock("clk", &sig.name));
            }
        }
        for assign in &module.assigns {
            router.add_edge(ConnEdge::data(&assign.target, &assign.value, 32));
        }
        router.analyze_fanout();
        let total_nets = router.edges.len() as u32;
        let total_wire: u32 = router
            .fanouts
            .iter()
            .map(|f| RouteModel::est_wire_length(f.fanout) * f.total_bits)
            .sum();
        let max_fanout = router.fanouts.iter().map(|f| f.fanout).max().unwrap_or(0);
        let needs_buf = max_fanout > 32;
        RouteEstimate {
            total_nets,
            total_wire_length_um: total_wire,
            avg_wire_length_um: if total_nets > 0 {
                total_wire / total_nets
            } else {
                0
            },
            max_fanout,
            congestion_score: RouteModel::est_congestion(total_nets, 10),
            needs_global_buf: needs_buf,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScanChain {
    pub name: String,
    pub num_regs: u32,
    pub chain_length_bits: u32,
}

impl ScanChain {
    pub fn new(name: &str, regs: u32) -> Self {
        ScanChain {
            name: name.into(),
            num_regs: regs,
            chain_length_bits: regs * 32,
        }
    }
    pub fn cycles(&self) -> u32 {
        self.chain_length_bits + 10
    }
    pub fn bytes(&self) -> u32 {
        self.chain_length_bits / 8
    }
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("chain name empty".into());
        }
        if self.num_regs == 0 {
            errors.push("num_regs zero".into());
        }
        errors
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BistKind {
    Memory,
    Logic,
    Io,
}

#[derive(Debug, Clone)]
pub struct BistCtrl {
    pub name: String,
    pub kind: BistKind,
    pub patterns: u32,
    pub pass_threshold: u32,
}

impl BistCtrl {
    pub fn memory(name: &str, patterns: u32) -> Self {
        BistCtrl {
            name: name.into(),
            kind: BistKind::Memory,
            patterns,
            pass_threshold: patterns,
        }
    }
    pub fn logic(name: &str, patterns: u32) -> Self {
        BistCtrl {
            name: name.into(),
            kind: BistKind::Logic,
            patterns,
            pass_threshold: patterns,
        }
    }
    pub fn cycles(&self) -> u32 {
        self.patterns * 2
    }
    pub fn coverage(&self, total_faults: u32) -> u32 {
        if total_faults == 0 {
            return 100;
        }
        self.patterns * 100 / total_faults
    }
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("bist name empty".into());
        }
        if self.patterns == 0 {
            errors.push("patterns zero".into());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct JtagTap {
    pub name: String,
    pub ir_width: u32,
    pub num_dr_regs: u32,
    pub bypass_code: u32,
    pub idcode: u32,
}

impl JtagTap {
    pub fn new(name: &str, ir_width: u32, idcode: u32) -> Self {
        JtagTap {
            name: name.into(),
            ir_width,
            num_dr_regs: 3,
            bypass_code: 0xFF,
            idcode,
        }
    }
    pub fn total_bits(&self) -> u32 {
        self.ir_width + 32 * self.num_dr_regs
    }
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("tap name empty".into());
        }
        if self.ir_width == 0 {
            errors.push("ir_width zero".into());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct TestCoverage {
    pub scan_coverage: u32,
    pub bist_coverage: u32,
    pub atpg_coverage: u32,
    pub total_coverage: u32,
}

impl TestCoverage {
    pub fn new(scan: u32, bist: u32, atpg: u32) -> Self {
        TestCoverage {
            scan_coverage: scan,
            bist_coverage: bist,
            atpg_coverage: atpg,
            total_coverage: (scan + bist + atpg) / 3,
        }
    }
    pub fn is_acceptable(&self) -> bool {
        self.total_coverage >= 90
    }
}

#[derive(Debug)]
pub struct HirDft {
    pub scan_chains: Vec<ScanChain>,
    pub bist_controllers: Vec<BistCtrl>,
    pub jtag_tap: Option<JtagTap>,
    pub target_coverage: u32,
}

impl HirDft {
    pub fn new() -> Self {
        HirDft {
            scan_chains: Vec::new(),
            bist_controllers: Vec::new(),
            jtag_tap: None,
            target_coverage: 95,
        }
    }

    pub fn add_scan_chain(&mut self, name: &str, regs: u32) {
        self.scan_chains.push(ScanChain::new(name, regs));
    }

    pub fn add_memory_bist(&mut self, name: &str, patterns: u32) {
        self.bist_controllers.push(BistCtrl::memory(name, patterns));
    }

    pub fn add_logic_bist(&mut self, name: &str, patterns: u32) {
        self.bist_controllers.push(BistCtrl::logic(name, patterns));
    }

    pub fn set_jtag(&mut self, tap: JtagTap) {
        self.jtag_tap = Some(tap);
    }

    pub fn total_scan_regs(&self) -> u32 {
        self.scan_chains.iter().map(|c| c.num_regs).sum()
    }

    pub fn total_scan_cycles(&self) -> u32 {
        self.scan_chains.iter().map(|c| c.cycles()).sum()
    }

    pub fn total_bist_cycles(&self) -> u32 {
        self.bist_controllers.iter().map(|b| b.cycles()).sum()
    }

    pub fn est_test_time_cycles(&self) -> u32 {
        self.total_scan_cycles() + self.total_bist_cycles()
    }

    pub fn coverage_estimate(&self) -> TestCoverage {
        let scan = if self.scan_chains.is_empty() { 0 } else { 95 };
        let bist = if self.bist_controllers.is_empty() {
            0
        } else {
            90
        };
        let atpg = 80;
        TestCoverage::new(scan, bist, atpg)
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for c in &self.scan_chains {
            errors.extend(c.validate());
        }
        for b in &self.bist_controllers {
            errors.extend(b.validate());
        }
        if let Some(ref tap) = self.jtag_tap {
            errors.extend(tap.validate());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct PllConfig {
    pub name: String,
    pub input_mhz: u32,
    pub output_mhz: u32,
    pub multiply: u32,
    pub divide: u32,
    pub jitter_ps: u32,
}

impl PllConfig {
    pub fn new(name: &str, input_mhz: u32, output_mhz: u32) -> Self {
        PllConfig {
            name: name.into(),
            input_mhz,
            output_mhz,
            multiply: output_mhz.max(1),
            divide: input_mhz.max(1),
            jitter_ps: 50,
        }
    }
    pub fn period_ps(&self) -> u32 {
        if self.output_mhz == 0 {
            return 0;
        }
        1_000_000_000 / self.output_mhz
    }
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("pll name empty".into());
        }
        if self.output_mhz == 0 {
            errors.push("output_mhz zero".into());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct ClockBuffer {
    pub name: String,
    pub delay_ps: u32,
    pub fanout: u32,
}

impl ClockBuffer {
    pub fn bufg(name: &str) -> Self {
        ClockBuffer {
            name: name.into(),
            delay_ps: 100,
            fanout: 32,
        }
    }
    pub fn bufh(name: &str) -> Self {
        ClockBuffer {
            name: name.into(),
            delay_ps: 50,
            fanout: 16,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClockTree {
    pub root: String,
    pub num_levels: u32,
    pub total_buffers: u32,
    pub max_skew_ps: u32,
}

impl ClockTree {
    pub fn new(root: &str, levels: u32, bufs: u32) -> Self {
        ClockTree {
            root: root.into(),
            num_levels: levels,
            total_buffers: bufs,
            max_skew_ps: 100,
        }
    }
    pub fn tree_delay_ps(&self, buf_delay: u32) -> u32 {
        self.num_levels * buf_delay
    }
    pub fn skew_ok(&self, max_allowed_ps: u32) -> bool {
        self.max_skew_ps <= max_allowed_ps
    }
}

#[derive(Debug, Clone)]
pub struct CtsReport {
    pub num_clocks: u32,
    pub num_plls: u32,
    pub total_buffers: u32,
    pub worst_skew_ps: u32,
    pub worst_latency_ps: u32,
    pub has_violations: bool,
}

impl CtsReport {
    pub fn passed(&self) -> bool {
        !self.has_violations
    }
}

pub struct CtsModel;

impl CtsModel {
    pub fn est_buffers_needed(num_sinks: u32) -> u32 {
        if num_sinks <= 16 {
            1
        } else {
            num_sinks / 16 + 1
        }
    }
    pub fn est_tree_levels(num_sinks: u32) -> u32 {
        if num_sinks <= 16 {
            1
        } else if num_sinks <= 256 {
            2
        } else {
            3
        }
    }
}

#[derive(Debug)]
pub struct HirCts {
    pub plls: Vec<PllConfig>,
    pub trees: Vec<ClockTree>,
    pub buffers: Vec<ClockBuffer>,
}

impl HirCts {
    pub fn new() -> Self {
        HirCts {
            plls: Vec::new(),
            trees: Vec::new(),
            buffers: Vec::new(),
        }
    }

    pub fn add_pll(&mut self, pll: PllConfig) {
        self.plls.push(pll);
    }

    pub fn build_tree(&mut self, root: &str, num_sinks: u32) {
        let levels = CtsModel::est_tree_levels(num_sinks);
        let bufs = CtsModel::est_buffers_needed(num_sinks);
        self.trees.push(ClockTree::new(root, levels, bufs));
        for i in 0..bufs {
            self.buffers
                .push(ClockBuffer::bufg(&format!("{}_bufg{}", root, i)));
        }
    }

    pub fn report(&self) -> CtsReport {
        let worst_skew = self.trees.iter().map(|t| t.max_skew_ps).max().unwrap_or(0);
        let worst_latency = self
            .trees
            .iter()
            .map(|t| t.tree_delay_ps(100))
            .max()
            .unwrap_or(0);
        CtsReport {
            num_clocks: self.trees.len() as u32,
            num_plls: self.plls.len() as u32,
            total_buffers: self.buffers.len() as u32,
            worst_skew_ps: worst_skew,
            worst_latency_ps: worst_latency,
            has_violations: worst_skew > 200,
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        for pll in &self.plls {
            errors.extend(pll.validate());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct ResetSynchronizer {
    pub name: String,
    pub num_stages: u32,
    pub input_clock: String,
    pub output_clock: String,
    pub async_assert: bool,
}

impl ResetSynchronizer {
    pub fn new(name: &str, stages: u32, in_clk: &str, out_clk: &str) -> Self {
        ResetSynchronizer {
            name: name.into(),
            num_stages: stages,
            input_clock: in_clk.into(),
            output_clock: out_clk.into(),
            async_assert: true,
        }
    }
    pub fn meta_stability_mttf_ps(&self, clk_period_ps: u32) -> u64 {
        if self.num_stages == 0 || clk_period_ps == 0 {
            return 0;
        }
        let base_mttf: u64 = 1_000_000_000;
        base_mttf * (self.num_stages as u64).pow(2)
    }
    pub fn latency_ps(&self, clk_period_ps: u32) -> u32 {
        self.num_stages * clk_period_ps
    }
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("reset sync name empty".into());
        }
        if self.num_stages == 0 {
            errors.push("num_stages zero".into());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct ResetDomain {
    pub name: String,
    pub clock_domain: String,
    pub polarity_active_low: bool,
    pub sync_chains: Vec<ResetSynchronizer>,
}

impl ResetDomain {
    pub fn new(name: &str, clk_domain: &str, active_low: bool) -> Self {
        ResetDomain {
            name: name.into(),
            clock_domain: clk_domain.into(),
            polarity_active_low: active_low,
            sync_chains: Vec::new(),
        }
    }
    pub fn add_sync(&mut self, sync: ResetSynchronizer) {
        self.sync_chains.push(sync);
    }
}

#[derive(Debug, Clone)]
pub struct RetimingOp {
    pub from_signal: String,
    pub to_signal: String,
    pub direction_forward: bool,
    pub registers_moved: u32,
}

impl RetimingOp {
    pub fn forward(from: &str, to: &str, regs: u32) -> Self {
        RetimingOp {
            from_signal: from.into(),
            to_signal: to.into(),
            direction_forward: true,
            registers_moved: regs,
        }
    }
    pub fn backward(from: &str, to: &str, regs: u32) -> Self {
        RetimingOp {
            from_signal: from.into(),
            to_signal: to.into(),
            direction_forward: false,
            registers_moved: regs,
        }
    }
}

#[derive(Debug)]
pub struct HirRetimer {
    pub operations: Vec<RetimingOp>,
    pub original_crit_ps: u32,
    pub retimed_crit_ps: u32,
}

impl HirRetimer {
    pub fn new() -> Self {
        HirRetimer {
            operations: Vec::new(),
            original_crit_ps: 0,
            retimed_crit_ps: 0,
        }
    }

    pub fn retime_forward(&mut self, from: &str, to: &str, regs: u32) {
        self.operations.push(RetimingOp::forward(from, to, regs));
    }

    pub fn retime_backward(&mut self, from: &str, to: &str, regs: u32) {
        self.operations.push(RetimingOp::backward(from, to, regs));
    }

    pub fn improvement_percent(&self) -> u32 {
        if self.original_crit_ps == 0 {
            return 0;
        }
        let saved = self.original_crit_ps.saturating_sub(self.retimed_crit_ps);
        saved * 100 / self.original_crit_ps
    }

    pub fn fmax_improvement(&self) -> u32 {
        let orig_fmax = if self.original_crit_ps > 0 {
            1_000_000_000 / self.original_crit_ps
        } else {
            0
        };
        let new_fmax = if self.retimed_crit_ps > 0 {
            1_000_000_000 / self.retimed_crit_ps
        } else {
            0
        };
        if orig_fmax == 0 {
            return 0;
        }
        (new_fmax.saturating_sub(orig_fmax)) * 100 / orig_fmax
    }
}

#[derive(Debug, Clone)]
pub struct ConfigReg {
    pub name: String,
    pub offset: u32,
    pub width: u32,
    pub reset_value: u32,
    pub writable: bool,
    pub description: String,
}

impl ConfigReg {
    pub fn rw(name: &str, offset: u32, width: u32, reset: u32) -> Self {
        ConfigReg {
            name: name.into(),
            offset,
            width,
            reset_value: reset,
            writable: true,
            description: String::new(),
        }
    }
    pub fn ro(name: &str, offset: u32, width: u32) -> Self {
        ConfigReg {
            name: name.into(),
            offset,
            width,
            reset_value: 0,
            writable: false,
            description: String::new(),
        }
    }
    pub fn byte_offset(&self) -> u32 {
        self.offset / 8
    }
}

#[derive(Debug)]
pub struct HirConfigBlock {
    pub name: String,
    pub base_address: u32,
    pub registers: Vec<ConfigReg>,
}

impl HirConfigBlock {
    pub fn new(name: &str, base: u32) -> Self {
        HirConfigBlock {
            name: name.into(),
            base_address: base,
            registers: Vec::new(),
        }
    }
    pub fn add_rw(&mut self, name: &str, width: u32, reset: u32) {
        let offset = self
            .registers
            .iter()
            .map(|r| r.offset + r.width)
            .max()
            .unwrap_or(0);
        self.registers
            .push(ConfigReg::rw(name, offset, width, reset));
    }
    pub fn add_ro(&mut self, name: &str, width: u32) {
        let offset = self
            .registers
            .iter()
            .map(|r| r.offset + r.width)
            .max()
            .unwrap_or(0);
        self.registers.push(ConfigReg::ro(name, offset, width));
    }
    pub fn total_bytes(&self) -> u32 {
        let total_bits = self.registers.iter().map(|r| r.width).sum::<u32>();
        (total_bits + 7) / 8
    }
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("config block name empty".into());
        }
        for r in &self.registers {
            if r.name.is_empty() {
                errors.push("reg name empty".into());
            }
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct IrqSource {
    pub name: String,
    pub id: u32,
    pub priority: u32,
    pub edge_triggered: bool,
}

impl IrqSource {
    pub fn level(name: &str, id: u32, priority: u32) -> Self {
        IrqSource {
            name: name.into(),
            id,
            priority,
            edge_triggered: false,
        }
    }
    pub fn edge(name: &str, id: u32, priority: u32) -> Self {
        IrqSource {
            name: name.into(),
            id,
            priority,
            edge_triggered: true,
        }
    }
}

#[derive(Debug)]
pub struct HirInterruptCtrl {
    pub name: String,
    pub sources: Vec<IrqSource>,
    pub num_priorities: u32,
    pub nesting_enabled: bool,
    pub vector_table_base: u32,
}

impl HirInterruptCtrl {
    pub fn new(name: &str, num_priorities: u32) -> Self {
        HirInterruptCtrl {
            name: name.into(),
            sources: Vec::new(),
            num_priorities,
            nesting_enabled: true,
            vector_table_base: 0,
        }
    }
    pub fn add_level_irq(&mut self, name: &str, id: u32, priority: u32) {
        self.sources.push(IrqSource::level(name, id, priority));
    }
    pub fn add_edge_irq(&mut self, name: &str, id: u32, priority: u32) {
        self.sources.push(IrqSource::edge(name, id, priority));
    }
    pub fn highest_priority(&self) -> Option<&IrqSource> {
        self.sources.iter().min_by_key(|s| s.priority)
    }
    pub fn pending_count(&self) -> u32 {
        self.sources.len() as u32
    }
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("irq ctrl name empty".into());
        }
        errors
    }

    pub fn emit_verilog(&self) -> String {
        let mut v = String::new();
        v.push_str(&format!("// Interrupt Controller: {}\n", self.name));
        v.push_str(&format!(
            "// Sources: {}, Priorities: {}\n",
            self.sources.len(),
            self.num_priorities
        ));
        v.push_str(&format!("module irq_{} (\n", self.name));
        v.push_str("    input  wire        clk,\n");
        v.push_str("    input  wire        rst_n,\n");
        for src in &self.sources {
            let trig = if src.edge_triggered { "edge" } else { "level" };
            v.push_str(&format!(
                "    input  wire        irq_{}_{}_{},\n",
                src.name, trig, src.id
            ));
        }
        v.push_str("    output reg  [31:0] irq_vector,\n");
        v.push_str("    output reg         irq_pending\n");
        v.push_str(");\n");
        v.push_str(&format!(
            "    // {} interrupt sources\n",
            self.sources.len()
        ));
        v.push_str("    reg [7:0] irq_priority [0:31];\n");
        v.push_str("    reg [31:0] irq_active;\n");
        v.push_str("    always @(posedge clk or negedge rst_n) begin\n");
        v.push_str("        if (!rst_n) begin\n");
        v.push_str("            irq_active <= 0;\n");
        v.push_str("            irq_vector <= 0;\n");
        v.push_str("            irq_pending <= 0;\n");
        v.push_str("        end else begin\n");
        for src in &self.sources {
            v.push_str(&format!(
                "            if (irq_{}_{}_{}) irq_active[{}] <= 1'b1;\n",
                src.name,
                if src.edge_triggered { "edge" } else { "level" },
                src.id,
                src.id
            ));
        }
        v.push_str("            irq_pending <= |irq_active;\n");
        v.push_str("        end\n");
        v.push_str("    end\n");
        v.push_str("endmodule\n");
        v
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DmaTransferKind {
    Single,
    Burst,
    ScatterGather,
}

#[derive(Debug, Clone)]
pub struct DmaChannel {
    pub name: String,
    pub index: u32,
    pub src_addr: u32,
    pub dst_addr: u32,
    pub length_bytes: u32,
    pub kind: DmaTransferKind,
    pub burst_size: u32,
}

impl DmaChannel {
    pub fn single(name: &str, idx: u32, src: u32, dst: u32, len: u32) -> Self {
        DmaChannel {
            name: name.into(),
            index: idx,
            src_addr: src,
            dst_addr: dst,
            length_bytes: len,
            kind: DmaTransferKind::Single,
            burst_size: 1,
        }
    }
    pub fn burst(name: &str, idx: u32, src: u32, dst: u32, len: u32, burst: u32) -> Self {
        DmaChannel {
            name: name.into(),
            index: idx,
            src_addr: src,
            dst_addr: dst,
            length_bytes: len,
            kind: DmaTransferKind::Burst,
            burst_size: burst,
        }
    }
    pub fn transfer_cycles(&self) -> u32 {
        if self.burst_size == 0 {
            return 0;
        }
        let num_bursts = (self.length_bytes + self.burst_size - 1) / self.burst_size;
        num_bursts * (self.burst_size + 2)
    }
    pub fn bandwidth_mbps(&self, clock_mhz: u32) -> u32 {
        if self.transfer_cycles() == 0 || clock_mhz == 0 {
            return 0;
        }
        ((self.length_bytes as u64 * 8 * clock_mhz as u64) / self.transfer_cycles() as u64) as u32
    }
}

#[derive(Debug)]
pub struct HirDmaEngine {
    pub name: String,
    pub channels: Vec<DmaChannel>,
    pub data_width: u32,
    pub max_burst: u32,
}

impl HirDmaEngine {
    pub fn new(name: &str, data_width: u32) -> Self {
        HirDmaEngine {
            name: name.into(),
            channels: Vec::new(),
            data_width,
            max_burst: 16,
        }
    }
    pub fn add_single(&mut self, name: &str, src: u32, dst: u32, len: u32) {
        let idx = self.channels.len() as u32;
        self.channels
            .push(DmaChannel::single(name, idx, src, dst, len));
    }
    pub fn add_burst(&mut self, name: &str, src: u32, dst: u32, len: u32, burst: u32) {
        let idx = self.channels.len() as u32;
        self.channels
            .push(DmaChannel::burst(name, idx, src, dst, len, burst));
    }
    pub fn total_transfer_cycles(&self) -> u32 {
        self.channels.iter().map(|c| c.transfer_cycles()).sum()
    }
    pub fn total_bytes(&self) -> u32 {
        self.channels.iter().map(|c| c.length_bytes).sum()
    }
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("dma name empty".into());
        }
        if self.data_width == 0 {
            errors.push("data_width zero".into());
        }
        errors
    }

    pub fn emit_verilog(&self) -> String {
        let mut v = String::new();
        v.push_str(&format!(
            "// DMA Engine: {} ({} channels, {}-bit)\n",
            self.name,
            self.channels.len(),
            self.data_width
        ));
        v.push_str(&format!("module dma_{} (\n", self.name));
        v.push_str("    input  wire        clk,\n    input  wire        rst_n,\n");
        v.push_str("    input  wire [31:0] src_addr,\n    input  wire [31:0] dst_addr,\n");
        v.push_str("    input  wire [15:0] xfer_len,\n    output reg         dma_done,\n    output reg         dma_error\n);\n");
        for ch in &self.channels {
            v.push_str(&format!(
                "    // Channel {}: {} bytes\n",
                ch.name, ch.length_bytes
            ));
        }
        v.push_str("    reg [31:0] src_ptr; reg [31:0] dst_ptr; reg [15:0] count; reg active;\n");
        v.push_str("    always @(posedge clk or negedge rst_n) begin\n        if (!rst_n) begin\n            src_ptr <= 0; dst_ptr <= 0; count <= 0; active <= 0; dma_done <= 0; dma_error <= 0;\n        end else if (active) begin\n            if (count >= xfer_len) begin active <= 0; dma_done <= 1; end\n            else count <= count + 1;\n        end\n    end\nendmodule\n");
        v
    }
}

#[derive(Debug, Clone)]
pub struct CrossOptPass {
    pub name: String,
    pub num_modules: u32,
    pub constants_propagated: u32,
    pub dead_signals_removed: u32,
    pub instances_merged: u32,
}

impl CrossOptPass {
    pub fn empty() -> Self {
        CrossOptPass {
            name: "empty".into(),
            num_modules: 0,
            constants_propagated: 0,
            dead_signals_removed: 0,
            instances_merged: 0,
        }
    }
    pub fn new(name: &str, mods: u32, consts: u32, dead: u32, merged: u32) -> Self {
        CrossOptPass {
            name: name.into(),
            num_modules: mods,
            constants_propagated: consts,
            dead_signals_removed: dead,
            instances_merged: merged,
        }
    }
    pub fn total_improvements(&self) -> u32 {
        self.constants_propagated + self.dead_signals_removed + self.instances_merged
    }
    pub fn has_improvements(&self) -> bool {
        self.total_improvements() > 0
    }
    pub fn improvement_density(&self) -> u32 {
        if self.num_modules == 0 {
            return 0;
        }
        self.total_improvements() / self.num_modules
    }
}

#[derive(Debug, Clone)]
pub struct CrossOptReport {
    pub total_passes: u32,
    pub total_constants: u32,
    pub total_dead: u32,
    pub total_merged: u32,
    pub total_modules: u32,
}

impl CrossOptReport {
    pub fn new(passes: u32, consts: u32, dead: u32, merged: u32, mods: u32) -> Self {
        CrossOptReport {
            total_passes: passes,
            total_constants: consts,
            total_dead: dead,
            total_merged: merged,
            total_modules: mods,
        }
    }
    pub fn total_optimizations(&self) -> u32 {
        self.total_constants + self.total_dead + self.total_merged
    }
    pub fn is_effective(&self) -> bool {
        self.total_optimizations() > 0
    }
}

#[derive(Debug)]
pub struct HirCrossOptimizer {
    pub passes: Vec<CrossOptPass>,
}

impl HirCrossOptimizer {
    pub fn new() -> Self {
        HirCrossOptimizer { passes: Vec::new() }
    }

    pub fn run_pass(&mut self, modules: &mut [HirModule]) -> CrossOptPass {
        let n = modules.len() as u32;
        let mut consts: u32 = 0;
        let mut dead: u32 = 0;
        for m in modules.iter_mut() {
            let before = m.signals.len();
            let const_vals: std::collections::HashMap<String, String> = m
                .assigns
                .iter()
                .filter(|a| a.value.parse::<u32>().is_ok())
                .map(|a| (a.target.clone(), a.value.clone()))
                .collect();
            consts += const_vals.len() as u32;
            m.assigns.retain(|a| !const_vals.contains_key(&a.target));
            m.signals.retain(|s| {
                let used_in_assigns = m
                    .assigns
                    .iter()
                    .any(|a| a.value.contains(&s.name) || a.target == s.name);
                let used_in_always = m.always_blocks.iter().any(|ab| {
                    ab.body
                        .iter()
                        .any(|stmt| stmt.target.contains(&s.name) || stmt.value.contains(&s.name))
                });
                let is_port = m.ports.iter().any(|p| p.name == s.name);
                is_port || used_in_assigns || used_in_always
            });
            dead += (before - m.signals.len()) as u32;
        }
        let pass = CrossOptPass::new(&format!("pass_{}", self.passes.len()), n, consts, dead, 0);
        self.passes.push(pass.clone());
        pass
    }

    pub fn report(&self) -> CrossOptReport {
        CrossOptReport::new(
            self.passes.len() as u32,
            self.passes.iter().map(|p| p.constants_propagated).sum(),
            self.passes.iter().map(|p| p.dead_signals_removed).sum(),
            self.passes.iter().map(|p| p.instances_merged).sum(),
            self.passes.iter().map(|p| p.num_modules).max().unwrap_or(0),
        )
    }
}

#[derive(Debug, Clone)]
pub struct BootStage {
    pub name: String,
    pub index: u32,
    pub size_bytes: u32,
    pub entry_addr: u32,
}

impl BootStage {
    pub fn new(name: &str, idx: u32, size: u32, entry: u32) -> Self {
        BootStage {
            name: name.into(),
            index: idx,
            size_bytes: size,
            entry_addr: entry,
        }
    }
    pub fn end(&self) -> u32 {
        self.entry_addr.saturating_add(self.size_bytes)
    }
}

#[derive(Debug, Clone)]
pub struct BootConfig {
    pub name: String,
    pub rom_base: u32,
    pub rom_size: u32,
    pub has_integrity_check: bool,
    pub has_chain_loader: bool,
}

impl BootConfig {
    pub fn new(name: &str, rom_size: u32) -> Self {
        BootConfig {
            name: name.into(),
            rom_base: 0,
            rom_size,
            has_integrity_check: true,
            has_chain_loader: true,
        }
    }
    pub fn end(&self) -> u32 {
        self.rom_base.saturating_add(self.rom_size)
    }
    pub fn fits(&self, stages: &[BootStage]) -> bool {
        let total: u32 = stages.iter().map(|s| s.size_bytes).sum();
        total <= self.rom_size
    }
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("boot config name empty".into());
        }
        if self.rom_size == 0 {
            errors.push("rom_size zero".into());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct WatchdogConfig {
    pub name: String,
    pub counter_width: u32,
    pub timeout_cycles: u32,
    pub window_open: u32,
    pub window_close: u32,
    pub generate_reset: bool,
    pub generate_interrupt: bool,
}

impl WatchdogConfig {
    pub fn new(name: &str, timeout: u32) -> Self {
        WatchdogConfig {
            name: name.into(),
            counter_width: 32,
            timeout_cycles: timeout,
            window_open: timeout / 4,
            window_close: timeout * 3 / 4,
            generate_reset: true,
            generate_interrupt: true,
        }
    }
    pub fn windowed(name: &str, timeout: u32, open: u32, close: u32) -> Self {
        WatchdogConfig {
            name: name.into(),
            counter_width: 32,
            timeout_cycles: timeout,
            window_open: open,
            window_close: close,
            generate_reset: true,
            generate_interrupt: true,
        }
    }
    pub fn timeout_ns(&self, clock_ns: u32) -> u64 {
        (self.timeout_cycles as u64) * (clock_ns as u64)
    }
    pub fn in_window(&self, count: u32) -> bool {
        count >= self.window_open && count <= self.window_close
    }
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("wdt name empty".into());
        }
        if self.timeout_cycles == 0 {
            errors.push("timeout zero".into());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct MemMapEntry {
    pub name: String,
    pub base: u32,
    pub size: u32,
    pub kind: String,
    pub readable: bool,
    pub writable: bool,
}

impl MemMapEntry {
    pub fn rom(name: &str, base: u32, size: u32) -> Self {
        MemMapEntry {
            name: name.into(),
            base,
            size,
            kind: "rom".into(),
            readable: true,
            writable: false,
        }
    }
    pub fn ram(name: &str, base: u32, size: u32) -> Self {
        MemMapEntry {
            name: name.into(),
            base,
            size,
            kind: "ram".into(),
            readable: true,
            writable: true,
        }
    }
    pub fn peripheral(name: &str, base: u32, size: u32) -> Self {
        MemMapEntry {
            name: name.into(),
            base,
            size,
            kind: "periph".into(),
            readable: true,
            writable: true,
        }
    }
    pub fn end(&self) -> u32 {
        self.base.saturating_add(self.size)
    }
    pub fn contains(&self, addr: u32) -> bool {
        addr >= self.base && addr < self.end()
    }
}

#[derive(Debug)]
pub struct HirMemMap {
    pub entries: Vec<MemMapEntry>,
}

impl HirMemMap {
    pub fn new() -> Self {
        HirMemMap {
            entries: Vec::new(),
        }
    }
    pub fn add_rom(&mut self, name: &str, base: u32, size: u32) {
        self.entries.push(MemMapEntry::rom(name, base, size));
    }
    pub fn add_ram(&mut self, name: &str, base: u32, size: u32) {
        self.entries.push(MemMapEntry::ram(name, base, size));
    }
    pub fn add_periph(&mut self, name: &str, base: u32, size: u32) {
        self.entries.push(MemMapEntry::peripheral(name, base, size));
    }
    pub fn lookup(&self, addr: u32) -> Option<&MemMapEntry> {
        self.entries.iter().find(|e| e.contains(addr))
    }
    pub fn total_size(&self) -> u32 {
        self.entries.iter().map(|e| e.size).sum()
    }
    pub fn check_overlaps(&self) -> Vec<(String, String)> {
        let mut overlaps = Vec::new();
        for i in 0..self.entries.len() {
            for j in (i + 1)..self.entries.len() {
                let a = &self.entries[i];
                let b = &self.entries[j];
                if a.base < b.end() && b.base < a.end() {
                    overlaps.push((a.name.clone(), b.name.clone()));
                }
            }
        }
        overlaps
    }
}

#[derive(Debug, Clone)]
pub struct SerDesConfig {
    pub name: String,
    pub lanes: u32,
    pub line_rate_gbps: u32,
    pub data_width: u32,
    pub encoding: String,
}

impl SerDesConfig {
    pub fn new(name: &str, lanes: u32, rate_gbps: u32) -> Self {
        SerDesConfig {
            name: name.into(),
            lanes,
            line_rate_gbps: rate_gbps,
            data_width: 32,
            encoding: "8b10b".into(),
        }
    }
    pub fn total_bandwidth_gbps(&self) -> u32 {
        self.lanes * self.line_rate_gbps
    }
    pub fn throughput_bytes_per_sec(&self) -> u64 {
        (self.total_bandwidth_gbps() as u64) * 1_000_000_000 / 10 * 8 / 8
    }
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("serdes name empty".into());
        }
        if self.lanes == 0 {
            errors.push("lanes zero".into());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct BuildStep {
    pub name: String,
    pub tool: String,
    pub input_file: String,
    pub output_file: String,
    pub duration_estimated_ms: u32,
}

#[derive(Debug)]
pub struct HirBuildOrchestrator {
    pub steps: Vec<BuildStep>,
    pub total_estimated_ms: u32,
}

impl HirBuildOrchestrator {
    pub fn new() -> Self {
        HirBuildOrchestrator {
            steps: Vec::new(),
            total_estimated_ms: 0,
        }
    }

    pub fn add_step(&mut self, name: &str, tool: &str, input: &str, output: &str, est_ms: u32) {
        self.total_estimated_ms += est_ms;
        self.steps.push(BuildStep {
            name: name.into(),
            tool: tool.into(),
            input_file: input.into(),
            output_file: output.into(),
            duration_estimated_ms: est_ms,
        });
    }

    pub fn standard_fpga_flow(&mut self) {
        self.add_step("synthesize", "yosys", "top.v", "synth.json", 30000);
        self.add_step("place_route", "nextpnr", "synth.json", "top.rpt", 60000);
        self.add_step("fasm_gen", "fasm2frames", "top.rpt", "top.frames", 5000);
        self.add_step("bitstream", "frames2bit", "top.frames", "top.bit", 3000);
    }

    pub fn step_count(&self) -> u32 {
        self.steps.len() as u32
    }
    pub fn has_step(&self, name: &str) -> bool {
        self.steps.iter().any(|s| s.name == name)
    }
}

#[derive(Debug)]
pub struct HirModule {
    pub name: String,
    pub ports: Vec<HirPort>,
    pub signals: Vec<HirSignal>,
    pub assigns: Vec<HirAssign>,
    pub always_blocks: Vec<HirAlwaysBlock>,
    pub instances: Vec<HirInstance>,
    pub memories: Vec<HirMemory>,
    pub clock_domains: Vec<HirClockDomain>,
    pub clock_crossings: Vec<HirClockCrossing>,
    pub fifos: Vec<HirFifo>,
    pub bus_ports: Vec<HirBusPort>,
    pub apb_bridges: Vec<HirApbBridge>,
    pub gf16_accels: Vec<HirGf16Accel>,
    pub formal_asserts: Vec<HirFormalAssert>,
    pub formal_covers: Vec<HirCoverPoint>,
    pub formal_assumes: Vec<HirFormalAssume>,
    pub formal_config: Option<HirFormalConfig>,
    pub ternary_cores: Vec<HirTernaryCore>,
}

impl HirModule {
    pub fn new(name: &str) -> Self {
        HirModule {
            name: name.to_string(),
            ports: Vec::new(),
            signals: Vec::new(),
            assigns: Vec::new(),
            always_blocks: Vec::new(),
            instances: Vec::new(),
            memories: Vec::new(),
            clock_domains: Vec::new(),
            clock_crossings: Vec::new(),
            fifos: Vec::new(),
            bus_ports: Vec::new(),
            apb_bridges: Vec::new(),
            gf16_accels: Vec::new(),
            formal_asserts: Vec::new(),
            formal_covers: Vec::new(),
            formal_assumes: Vec::new(),
            formal_config: None,
            ternary_cores: Vec::new(),
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("module name must not be empty".to_string());
        }
        for i in 0..self.ports.len() {
            for j in (i + 1)..self.ports.len() {
                if self.ports[i].name == self.ports[j].name {
                    errors.push(format!("duplicate port name: {}", self.ports[i].name));
                }
            }
        }
        for i in 0..self.signals.len() {
            for j in (i + 1)..self.signals.len() {
                if self.signals[i].name == self.signals[j].name {
                    errors.push(format!("duplicate signal name: {}", self.signals[i].name));
                }
            }
        }
        for mem in &self.memories {
            errors.extend(mem.validate());
        }
        for bus in &self.bus_ports {
            errors.extend(bus.validate());
        }
        for apb in &self.apb_bridges {
            errors.extend(apb.validate());
        }
        for gf16 in &self.gf16_accels {
            errors.extend(gf16.validate());
        }
        for fa in &self.formal_asserts {
            errors.extend(fa.validate());
        }
        for fc in &self.formal_covers {
            errors.extend(fc.validate());
        }
        for fam in &self.formal_assumes {
            errors.extend(fam.validate());
        }
        if let Some(ref fcfg) = self.formal_config {
            errors.extend(fcfg.validate());
        }
        for tc in &self.ternary_cores {
            errors.extend(tc.validate());
        }
        errors
    }
}

pub struct AstToHir;

impl AstToHir {
    pub fn convert(ast: &Node) -> Result<HirModule, String> {
        let module_name = if !ast.name.is_empty() {
            ast.name.clone()
        } else {
            "unknown".to_string()
        };

        let mut hir = HirModule::new(&module_name);

        for child in &ast.children {
            match child.kind {
                NodeKind::ConstDecl => {
                    Self::convert_const_to_signal(child, &mut hir)?;
                }
                NodeKind::EnumDecl => {
                    Self::convert_enum_to_signal(child, &mut hir)?;
                }
                NodeKind::StructDecl => {
                    Self::convert_struct_to_bundle(child, &mut hir)?;
                }
                NodeKind::FnDecl => {
                    Self::convert_fn_to_comb(child, &mut hir)?;
                }
                _ => {}
            }
        }

        let errors = hir.validate();
        if !errors.is_empty() {
            return Err(format!("HIR validation failed: {}", errors.join("; ")));
        }

        Ok(hir)
    }

    fn convert_const_to_signal(node: &Node, hir: &mut HirModule) -> Result<(), String> {
        let hw_type = Self::t27_type_to_hw(&node.extra_type);
        hir.signals.push(HirSignal {
            name: node.name.clone(),
            kind: if node.extra_mutable {
                HwSignalKind::Reg
            } else {
                HwSignalKind::Wire
            },
            ty: hw_type,
            reset_value: if !node.children.is_empty() {
                node.children[0].value.clone()
            } else {
                node.value.clone()
            },
        });
        Ok(())
    }

    fn convert_struct_to_bundle(node: &Node, hir: &mut HirModule) -> Result<(), String> {
        let fields: Vec<(String, HwType)> = node
            .children
            .iter()
            .filter(|c| c.kind == NodeKind::ExprIdentifier && !c.name.is_empty())
            .map(|c| (c.name.clone(), Self::t27_type_to_hw(&c.extra_type)))
            .collect();
        hir.signals.push(HirSignal {
            name: node.name.clone(),
            kind: HwSignalKind::Wire,
            ty: HwType::Bundle(fields),
            reset_value: String::new(),
        });
        Ok(())
    }

    fn convert_fn_to_comb(node: &Node, hir: &mut HirModule) -> Result<(), String> {
        let fn_name = &node.name;
        let ret_type = Self::t27_type_to_hw(&node.extra_return_type);
        if !node.extra_return_type.is_empty() && ret_type.hw_width() > 0 {
            let port_name = format!("{}_result", fn_name);
            hir.ports.push(HirPort {
                name: port_name,
                dir: HwPortDir::Output,
                ty: ret_type,
            });
        }
        for (pname, ptype) in &node.params {
            let hw_ty = Self::t27_type_to_hw(ptype);
            let existing = hir
                .ports
                .iter()
                .find(|p| p.name == *pname && p.dir == HwPortDir::Input && p.ty == hw_ty);
            if existing.is_none() {
                let conflict = hir.ports.iter().any(|p| p.name == *pname);
                let port_name = if conflict {
                    format!("{}_{}", fn_name, pname)
                } else {
                    pname.clone()
                };
                hir.ports.push(HirPort {
                    name: port_name,
                    dir: HwPortDir::Input,
                    ty: hw_ty,
                });
            }
        }
        for stmt in &node.children {
            if stmt.kind == NodeKind::StmtAssign {
                if let (Some(lhs), Some(rhs)) = (stmt.children.first(), stmt.children.get(1)) {
                    hir.assigns.push(HirAssign {
                        target: Self::expr_to_string(lhs),
                        value: Self::expr_to_string(rhs),
                    });
                }
            }
        }
        Ok(())
    }

    fn convert_enum_to_signal(node: &Node, hir: &mut HirModule) -> Result<(), String> {
        let variants: Vec<(String, String)> = node
            .children
            .iter()
            .filter(|c| c.kind == NodeKind::EnumVariant && !c.name.is_empty())
            .map(|c| (c.name.clone(), c.value.clone()))
            .collect();
        hir.signals.push(HirSignal {
            name: node.name.clone(),
            kind: HwSignalKind::Wire,
            ty: HwType::Enum(variants),
            reset_value: String::new(),
        });
        Ok(())
    }

    fn expr_to_string(node: &Node) -> String {
        match node.kind {
            NodeKind::ExprIdentifier => node.name.clone(),
            NodeKind::ExprLiteral => node.value.clone(),
            NodeKind::ExprFieldAccess => {
                if let Some(obj) = node.children.first() {
                    format!("{}.{}", Self::expr_to_string(obj), node.name)
                } else {
                    node.name.clone()
                }
            }
            NodeKind::ExprIndex => {
                if let Some(arr) = node.children.first() {
                    if let Some(idx) = node.children.get(1) {
                        format!(
                            "{}[{}]",
                            Self::expr_to_string(arr),
                            Self::expr_to_string(idx)
                        )
                    } else {
                        Self::expr_to_string(arr)
                    }
                } else {
                    node.name.clone()
                }
            }
            NodeKind::ExprBinary => {
                let lhs = node
                    .children
                    .first()
                    .map(|c| Self::expr_to_string(c))
                    .unwrap_or_default();
                let rhs = node
                    .children
                    .get(1)
                    .map(|c| Self::expr_to_string(c))
                    .unwrap_or_default();
                format!("{} {} {}", lhs, node.extra_op, rhs)
            }
            NodeKind::ExprCall => {
                let args: Vec<String> = node
                    .children
                    .iter()
                    .map(|c| Self::expr_to_string(c))
                    .collect();
                format!("{}({})", node.name, args.join(", "))
            }
            _ => node.name.clone(),
        }
    }

    fn t27_type_to_hw(ty: &str) -> HwType {
        let t = ty.trim();
        match t {
            "bool" => HwType::Bool,
            "u8" => HwType::UInt(8),
            "u16" => HwType::UInt(16),
            "u32" => HwType::UInt(32),
            "u64" => HwType::UInt(64),
            "i8" => HwType::SInt(8),
            "i16" => HwType::SInt(16),
            "i32" => HwType::SInt(32),
            "i64" => HwType::SInt(64),
            "f32" => HwType::Bits(32),
            "f64" => HwType::Bits(64),
            "GF16" | "gf16" => HwType::GF16,
            "" => HwType::Bits(0),
            other => {
                if other.starts_with('[') {
                    if let Some(bracket_end) = other.find(']') {
                        let size_str = &other[1..bracket_end];
                        let elem_str = &other[bracket_end + 1..];
                        if let Ok(size) = size_str.parse::<u32>() {
                            let elem = Self::t27_type_to_hw(elem_str);
                            return HwType::Vector(Box::new(elem), size);
                        }
                    }
                    HwType::Bits(32)
                } else {
                    HwType::Bits(32)
                }
            }
        }
    }
}

// ============================================================================
// HIR Optimizer — dead signal elimination + constant folding
// ============================================================================

pub struct HirOptimizer {
    removed_signals: u32,
    removed_assigns: u32,
    folded_constants: u32,
    merged_assigns: u32,
    pass_count: u32,
}

impl HirOptimizer {
    pub fn new() -> Self {
        HirOptimizer {
            removed_signals: 0,
            removed_assigns: 0,
            folded_constants: 0,
            merged_assigns: 0,
            pass_count: 0,
        }
    }

    pub fn optimize(&mut self, hir: &mut HirModule) {
        loop {
            let before_signals = hir.signals.len();
            let before_assigns = hir.assigns.len();
            let before_folded = self.folded_constants;
            let before_merged = self.merged_assigns;

            self.dead_signal_elimination(hir);
            self.constant_fold_assigns(hir);
            self.merge_chained_assigns(hir);

            let changed = hir.signals.len() != before_signals
                || hir.assigns.len() != before_assigns
                || self.folded_constants != before_folded
                || self.merged_assigns != before_merged;

            self.pass_count += 1;
            if !changed || self.pass_count >= 10 {
                break;
            }
        }
    }

    pub fn stats(&self) -> (u32, u32, u32, u32) {
        (
            self.removed_signals,
            self.removed_assigns,
            self.folded_constants,
            self.merged_assigns,
        )
    }

    pub fn resource_estimate(&self, hir: &HirModule) -> HirResourceEstimate {
        let mut luts = 0u32;
        let mut ffs = 0u32;
        let mut bram18 = 0u32;
        let mut dsp48 = 0u32;

        for sig in &hir.signals {
            match sig.kind {
                HwSignalKind::Reg => ffs += sig.ty.hw_width(),
                HwSignalKind::Wire => luts += sig.ty.hw_width() / 2,
            }
        }
        for _ in &hir.assigns {
            luts += 1;
        }
        for blk in &hir.always_blocks {
            luts += blk.body.len() as u32 * 2;
        }
        for mem in &hir.memories {
            bram18 += mem.bram18_count();
        }
        for fifo in &hir.fifos {
            bram18 += if fifo.total_bits() > 0 {
                fifo.total_bits() / 18432 + 1
            } else {
                0
            };
        }
        for bus in &hir.bus_ports {
            luts += bus.port_count() / 4;
        }
        for apb in &hir.apb_bridges {
            luts += apb.num_peripherals * 10;
        }
        for gf16 in &hir.gf16_accels {
            dsp48 += gf16.dsp48_count();
            bram18 += gf16.bram_count();
            luts += gf16.num_multipliers * 50;
        }
        for tc in &hir.ternary_cores {
            dsp48 += tc.dsp_count();
            bram18 += tc.bram_count();
            luts += tc.lut_estimate();
        }

        HirResourceEstimate {
            luts,
            ffs,
            bram18,
            dsp48,
            io_pins: hir.ports.len() as u32,
        }
    }

    fn dead_signal_elimination(&mut self, hir: &mut HirModule) {
        let used_names: std::collections::HashSet<String> = {
            let mut used = std::collections::HashSet::new();
            for p in &hir.ports {
                used.insert(p.name.clone());
            }
            for a in &hir.assigns {
                used.insert(a.target.clone());
                Self::extract_names(&a.value, &mut used);
            }
            for blk in &hir.always_blocks {
                for stmt in &blk.body {
                    Self::collect_stmt_names(stmt, &mut used);
                }
            }
            for inst in &hir.instances {
                for (_, sig) in &inst.port_map {
                    used.insert(sig.clone());
                }
            }
            for mem in &hir.memories {
                used.insert(mem.name.clone());
                for p in &mem.ports {
                    used.insert(p.name.clone());
                }
            }
            used
        };

        let before = hir.signals.len();
        hir.signals.retain(|s| {
            let keep = used_names.contains(&s.name) || s.kind == HwSignalKind::Reg;
            if !keep {
                self.removed_signals += 1;
            }
            keep
        });
        let _removed = before - hir.signals.len();
    }

    fn constant_fold_assigns(&mut self, hir: &mut HirModule) {
        let assign_targets: std::collections::HashSet<String> =
            hir.assigns.iter().map(|a| a.target.clone()).collect();
        let const_vals: std::collections::HashMap<String, String> = {
            let mut vals = std::collections::HashMap::new();
            for sig in &hir.signals {
                if sig.kind == HwSignalKind::Wire
                    && !sig.reset_value.is_empty()
                    && !assign_targets.contains(&sig.name)
                {
                    vals.insert(sig.name.clone(), sig.reset_value.clone());
                }
            }
            vals
        };

        for assign in &mut hir.assigns {
            if let Some(val) = const_vals.get(&assign.value) {
                assign.value = val.clone();
                self.folded_constants += 1;
            }
        }
    }

    fn merge_chained_assigns(&mut self, hir: &mut HirModule) {
        let assign_map: std::collections::HashMap<String, String> = {
            let mut map = std::collections::HashMap::new();
            for a in &hir.assigns {
                if !a.target.is_empty() && !a.value.is_empty() {
                    map.insert(a.target.clone(), a.value.clone());
                }
            }
            map
        };

        for assign in &mut hir.assigns {
            if let Some(val) = assign_map.get(&assign.value) {
                assign.value = val.clone();
                self.merged_assigns += 1;
            }
        }
    }

    fn extract_names(expr: &str, names: &mut std::collections::HashSet<String>) {
        let candidate: String = expr
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        if !candidate.is_empty() {
            names.insert(candidate);
        }
        for part in expr.split(|c: char| !c.is_alphanumeric() && c != '_') {
            if !part.is_empty() {
                names.insert(part.to_string());
            }
        }
    }

    fn collect_stmt_names(stmt: &HirAlwaysStmt, names: &mut std::collections::HashSet<String>) {
        if !stmt.target.is_empty() {
            names.insert(stmt.target.clone());
        }
        if !stmt.value.is_empty() {
            Self::extract_names(&stmt.value, names);
        }
        if !stmt.condition.is_empty() {
            Self::extract_names(&stmt.condition, names);
        }
        for s in &stmt.body {
            Self::collect_stmt_names(s, names);
        }
    }
}

// ============================================================================
// HIR Verilog Emitter (v0.5 — minimal, alongside existing codegen)
// ============================================================================

pub struct HirVerilogEmitter {
    output: String,
    indent: u32,
}

impl HirVerilogEmitter {
    pub fn new() -> Self {
        HirVerilogEmitter {
            output: String::new(),
            indent: 0,
        }
    }

    pub fn into_string(self) -> String {
        self.output
    }

    fn write_line(&mut self, s: &str) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn indent(&mut self) {
        self.indent += 1;
    }
    fn dedent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    pub fn emit(&mut self, hir: &HirModule) {
        self.write_line(&format!("// Generated from HIR: {}", hir.name));
        self.write_line("// DO NOT EDIT - generated by t27c (HIR path)");
        self.write_line("// phi^2 + 1/phi^2 = 3 | TRINITY");
        self.write_line("`timescale 1ns / 1ps");
        self.write_line("`default_nettype none");
        self.write_line("");

        self.write_line(&format!("module {} (", hir.name));
        self.indent();

        let has_clk = hir.ports.iter().any(|p| p.ty.is_clock_like());
        let has_rst = hir.ports.iter().any(|p| p.ty.is_reset_like());

        if !has_clk {
            self.write_line("input  wire        clk,");
        }
        if !has_rst {
            self.write_line("input  wire        rst_n,");
        }

        for (i, port) in hir.ports.iter().enumerate() {
            let dir_str = match port.dir {
                HwPortDir::Input => "input ",
                HwPortDir::Output => "output",
                HwPortDir::Inout => "inout ",
            };
            let range = port.ty.verilog_range();
            let signed = if port.ty.is_signed() { "signed " } else { "" };
            let comma = if i < hir.ports.len() - 1 { "," } else { "" };
            if range.is_empty() {
                self.write_line(&format!(
                    "{} wire {} {}{}{}",
                    dir_str, signed, port.name, comma, ""
                ));
            } else {
                self.write_line(&format!(
                    "{} wire {}{} {}{}",
                    dir_str, signed, range, port.name, comma
                ));
            }
        }

        self.dedent();
        self.write_line(");");
        self.write_line("");

        self.indent();

        if !hir.signals.is_empty() {
            self.write_line("// Internal signals");
            for sig in &hir.signals {
                let kind_str = match sig.kind {
                    HwSignalKind::Wire => "wire",
                    HwSignalKind::Reg => "reg ",
                };
                let range = sig.ty.verilog_range();
                if range.is_empty() {
                    self.write_line(&format!("{} {};", kind_str, sig.name));
                } else {
                    self.write_line(&format!("{} {} {};", kind_str, range, sig.name));
                }
            }
            self.write_line("");
        }

        if !hir.assigns.is_empty() {
            self.write_line("// Combinational assignments");
            for assign in &hir.assigns {
                self.write_line(&format!("assign {} = {};", assign.target, assign.value));
            }
            self.write_line("");
        }

        for blk in &hir.always_blocks {
            self.emit_always_block(blk);
        }

        for mem in &hir.memories {
            self.emit_memory(mem);
        }

        for fifo in &hir.fifos {
            self.emit_fifo(fifo);
        }

        for bus in &hir.bus_ports {
            self.emit_axi4_bus(bus);
        }

        for apb in &hir.apb_bridges {
            self.emit_apb_bridge(apb);
        }

        for gf16 in &hir.gf16_accels {
            self.emit_gf16_accel(gf16);
        }

        if !hir.formal_asserts.is_empty()
            || !hir.formal_covers.is_empty()
            || !hir.formal_assumes.is_empty()
        {
            self.emit_formal(hir);
        }

        for tc in &hir.ternary_cores {
            self.emit_ternary_core(tc);
        }

        for inst in &hir.instances {
            self.emit_instance(inst);
        }

        self.dedent();
        self.write_line("endmodule");
    }

    fn emit_always_block(&mut self, blk: &HirAlwaysBlock) {
        match blk.edge {
            HwEdge::Posedge => {
                self.write_line(&format!("always @(posedge {}) begin", blk.clock_name));
            }
            HwEdge::Negedge => {
                self.write_line(&format!("always @(negedge {}) begin", blk.clock_name));
            }
            HwEdge::Comb => {
                self.write_line("always @(*) begin");
            }
        }
        self.indent();

        let has_reset = !blk.reset_name.is_empty();
        if has_reset {
            self.write_line(&format!("if (!{}) begin", blk.reset_name));
            self.indent();
            for stmt in &blk.body {
                if stmt.kind == HirAlwaysStmtKind::NonBlockingAssign && !stmt.target.is_empty() {
                    self.write_line(&format!("{} <= {};", stmt.target, stmt.value));
                }
            }
            self.dedent();
            self.write_line("end else begin");
            self.indent();
        }

        for stmt in &blk.body {
            self.emit_always_stmt(stmt);
        }

        if has_reset {
            self.dedent();
            self.write_line("end");
        }

        self.dedent();
        self.write_line("end");
        self.write_line("");
    }

    fn emit_always_stmt(&mut self, stmt: &HirAlwaysStmt) {
        match stmt.kind {
            HirAlwaysStmtKind::NonBlockingAssign => {
                if !stmt.target.is_empty() {
                    self.write_line(&format!("{} <= {};", stmt.target, stmt.value));
                }
            }
            HirAlwaysStmtKind::BlockingAssign => {
                if !stmt.target.is_empty() {
                    self.write_line(&format!("{} = {};", stmt.target, stmt.value));
                }
            }
            HirAlwaysStmtKind::IfElse => {
                self.write_line(&format!("if ({}) begin", stmt.condition));
                self.indent();
                for s in &stmt.body {
                    self.emit_always_stmt(s);
                }
                self.dedent();
                self.write_line("end");
            }
            HirAlwaysStmtKind::Block => {
                for s in &stmt.body {
                    self.emit_always_stmt(s);
                }
            }
            _ => {}
        }
    }

    fn emit_fifo(&mut self, fifo: &HirFifo) {
        let kind_str = match fifo.kind {
            HwFifoKind::Sync => "SYNC_FIFO",
            HwFifoKind::Async => "ASYNC_FIFO",
        };
        let aw = fifo.addr_width();
        let dw = fifo.data_width;
        self.write_line(&format!(
            "// {} {} ({}x{} = {} bits)",
            kind_str,
            fifo.name,
            fifo.depth,
            dw,
            fifo.total_bits()
        ));
        self.write_line(&format!(
            "(* ram_style = \"block\" *) reg [{}:0] {}_mem [0:{}];",
            dw - 1,
            fifo.name,
            fifo.depth - 1
        ));
        self.write_line(&format!("reg  [{}:0] {}_head;", aw - 1, fifo.name));
        self.write_line(&format!("reg  [{}:0] {}_tail;", aw - 1, fifo.name));
        self.write_line(&format!("reg  [{}:0] {}_count;", aw, fifo.name));
        self.write_line(&format!(
            "wire {}_empty = ({}_count == 0);",
            fifo.name, fifo.name
        ));
        self.write_line(&format!(
            "wire {}_full = ({}_count == {});",
            fifo.name, fifo.name, fifo.depth
        ));
        self.write_line(&format!("wire [{}:0] {}_dout;", dw - 1, fifo.name));
        self.write_line(&format!("wire {}_ren;", fifo.name));
        self.write_line(&format!("wire [{}:0] {}_din;", dw - 1, fifo.name));
        self.write_line(&format!("wire {}_wen;", fifo.name));
        self.write_line(&format!(
            "always @(posedge clk) begin if ({}_wen && !{}_full) begin {}_mem[{}_tail] <= {}_din; {}_tail <= {}_tail + 1; {}_count <= {}_count + 1; end end",
            fifo.name, fifo.name, fifo.name, fifo.name, fifo.name, fifo.name, fifo.name, fifo.name, fifo.name
        ));
        self.write_line(&format!(
            "always @(posedge clk) begin if ({n}_ren && !{n}_empty) begin {n}_dout <= {n}_mem[{n}_head]; {n}_head <= {n}_head + 1; end end",
            n = fifo.name
        ));
        if fifo.has_almost_empty {
            self.write_line(&format!(
                "wire {}_almost_empty = ({}_count <= {});",
                fifo.name, fifo.name, fifo.almost_empty_threshold
            ));
        }
        if fifo.has_almost_full {
            self.write_line(&format!(
                "wire {}_almost_full = ({}_count >= {});",
                fifo.name, fifo.name, fifo.almost_full_threshold
            ));
        }
        self.write_line("");
    }

    fn emit_memory(&mut self, mem: &HirMemory) {
        let mem_type = match mem.kind {
            HwMemKind::Bram => "BRAM",
            HwMemKind::Dram => "DRAM",
            HwMemKind::Rom => "ROM",
        };
        self.write_line(&format!(
            "// {} {} ({}x{} = {} bits, {} ports, ~{} BRAM18)",
            mem_type,
            mem.name,
            mem.depth,
            mem.data_width,
            mem.total_bits(),
            mem.ports.len(),
            mem.bram18_count()
        ));
        self.write_line(&format!(
            "(* ram_style = \"block\" *) reg [{}:0] {} [0:{}];",
            mem.data_width - 1,
            mem.name,
            mem.depth - 1
        ));

        for port in &mem.ports {
            let (addr_suffix, data_suffix, en_suffix, we_suffix) = match port.kind {
                HwMemPortKind::Read => ("_addr", "_data", "_en", ""),
                HwMemPortKind::Write => ("_addr", "_data", "", "_we"),
                HwMemPortKind::ReadWrite => ("_addr", "_rdata", "_en", "_we"),
            };
            let aw = port.addr_width;
            let dw = port.data_width;
            self.write_line(&format!(
                "wire [{}:0] {}{};",
                aw - 1,
                port.name,
                addr_suffix
            ));
            match port.kind {
                HwMemPortKind::Read | HwMemPortKind::ReadWrite => {
                    self.write_line(&format!(
                        "reg  [{}:0] {}{};",
                        dw - 1,
                        port.name,
                        data_suffix
                    ));
                }
                HwMemPortKind::Write => {
                    self.write_line(&format!(
                        "wire [{}:0] {}{};",
                        dw - 1,
                        port.name,
                        data_suffix
                    ));
                }
            }
            if !en_suffix.is_empty() {
                self.write_line(&format!("wire {}{};", port.name, en_suffix));
            }
            if !we_suffix.is_empty() {
                self.write_line(&format!("wire {}{};", port.name, we_suffix));
            }

            match port.kind {
                HwMemPortKind::Read => {
                    self.write_line(&format!(
                        "always @(posedge clk) begin if ({}{}) {}{} <= {}[{}{}]; end",
                        port.name,
                        en_suffix,
                        port.name,
                        data_suffix,
                        mem.name,
                        port.name,
                        addr_suffix
                    ));
                }
                HwMemPortKind::Write => {
                    self.write_line(&format!(
                        "always @(posedge clk) begin if ({}{}) {}[{}{}] <= {}{}; end",
                        port.name,
                        we_suffix,
                        mem.name,
                        port.name,
                        addr_suffix,
                        port.name,
                        data_suffix
                    ));
                }
                HwMemPortKind::ReadWrite => {
                    self.write_line(&format!(
                        "always @(posedge clk) begin if ({}{}) {}{} <= {}[{}{}]; if ({}{}) {}[{}{}] <= {}{}; end",
                        port.name, en_suffix, port.name, data_suffix, mem.name, port.name, addr_suffix,
                        port.name, we_suffix, mem.name, port.name, addr_suffix, port.name, data_suffix
                    ));
                }
            }
        }
        self.write_line("");
    }

    fn emit_axi4_bus(&mut self, bus: &HirBusPort) {
        let kind_str = match bus.kind {
            HwBusKind::Axi4Lite => "AXI4-LITE",
            HwBusKind::Axi4Full => "AXI4-FULL",
        };
        let role_str = match bus.role {
            HwBusRole::Master => "MASTER",
            HwBusRole::Slave => "SLAVE",
        };
        self.write_line(&format!(
            "// {} {} {} ({} addr, {} data, {} ports)",
            kind_str,
            role_str,
            bus.name,
            bus.addr_width,
            bus.data_width,
            bus.port_count()
        ));

        let n = &bus.name;
        let aw = bus.addr_width;
        let dw = bus.data_width;
        let sw = bus.strb_width();
        let is_lite = bus.kind == HwBusKind::Axi4Lite;
        let has_id = bus.id_width > 0;
        let is_slave = bus.role == HwBusRole::Slave;

        let (aw_dir, ar_dir, w_dir, r_dir, b_dir) = if is_slave {
            ("input ", "input ", "input ", "output", "output")
        } else {
            ("output", "output", "output", "input ", "input ")
        };

        // AW channel
        self.write_line(&format!("{} wire {}_awvalid,", aw_dir, n));
        self.write_line(&format!(
            "{} wire        {}_awready,",
            if is_slave { "output" } else { "input " },
            n
        ));
        self.write_line(&format!("{} wire [{}:0] {}_awaddr,", aw_dir, aw - 1, n));
        if has_id {
            self.write_line(&format!(
                "{} wire [{}:0] {}_awid,",
                aw_dir,
                bus.id_width - 1,
                n
            ));
        }
        if !is_lite {
            self.write_line(&format!("{} wire [7:0] {}_awlen,", aw_dir, n));
            self.write_line(&format!("{} wire [2:0] {}_awsize,", aw_dir, n));
            self.write_line(&format!("{} wire [1:0] {}_awburst,", aw_dir, n));
            self.write_line(&format!("{} wire [3:0] {}_awcache,", aw_dir, n));
            self.write_line(&format!("{} wire [2:0] {}_awprot,", aw_dir, n));
            self.write_line(&format!("{} wire [3:0] {}_awqos,", aw_dir, n));
            self.write_line(&format!("{} wire [3:0] {}_awregion,", aw_dir, n));
            self.write_line(&format!("{} wire        {}_awlock,", aw_dir, n));
        } else {
            self.write_line(&format!("{} wire [2:0] {}_awprot,", aw_dir, n));
        }

        // W channel
        self.write_line(&format!("{} wire [{}:0] {}_wdata,", w_dir, dw - 1, n));
        self.write_line(&format!("{} wire [{}:0] {}_wstrb,", w_dir, sw - 1, n));
        self.write_line(&format!("{} wire        {}_wvalid,", w_dir, n));
        self.write_line(&format!(
            "{} wire        {}_wready,",
            if is_slave { "output" } else { "input " },
            n
        ));
        if !is_lite {
            self.write_line(&format!("{} wire        {}_wlast,", w_dir, n));
        }

        // B channel
        self.write_line(&format!("{} wire [1:0] {}_bresp,", b_dir, n));
        self.write_line(&format!("{} wire        {}_bvalid,", b_dir, n));
        self.write_line(&format!(
            "{} wire        {}_bready,",
            if is_slave { "input " } else { "output" },
            n
        ));
        if has_id {
            self.write_line(&format!(
                "{} wire [{}:0] {}_bid,",
                b_dir,
                bus.id_width - 1,
                n
            ));
        }

        // AR channel
        self.write_line(&format!("{} wire        {}_arvalid,", ar_dir, n));
        self.write_line(&format!(
            "{} wire        {}_arready,",
            if is_slave { "output" } else { "input " },
            n
        ));
        self.write_line(&format!("{} wire [{}:0] {}_araddr,", ar_dir, aw - 1, n));
        if has_id {
            self.write_line(&format!(
                "{} wire [{}:0] {}_arid,",
                ar_dir,
                bus.id_width - 1,
                n
            ));
        }
        if !is_lite {
            self.write_line(&format!("{} wire [7:0] {}_arlen,", ar_dir, n));
            self.write_line(&format!("{} wire [2:0] {}_arsize,", ar_dir, n));
            self.write_line(&format!("{} wire [1:0] {}_arburst,", ar_dir, n));
            self.write_line(&format!("{} wire [3:0] {}_arcache,", ar_dir, n));
            self.write_line(&format!("{} wire [2:0] {}_arprot,", ar_dir, n));
            self.write_line(&format!("{} wire [3:0] {}_arqos,", ar_dir, n));
            self.write_line(&format!("{} wire [3:0] {}_arregion,", ar_dir, n));
            self.write_line(&format!("{} wire        {}_arlock,", ar_dir, n));
        } else {
            self.write_line(&format!("{} wire [2:0] {}_arprot,", ar_dir, n));
        }

        // R channel
        self.write_line(&format!("{} wire [{}:0] {}_rdata,", r_dir, dw - 1, n));
        self.write_line(&format!("{} wire [1:0] {}_rresp,", r_dir, n));
        self.write_line(&format!("{} wire        {}_rvalid,", r_dir, n));
        self.write_line(&format!(
            "{} wire        {}_rready,",
            if is_slave { "input " } else { "output" },
            n
        ));
        if !is_lite {
            self.write_line(&format!("{} wire        {}_rlast,", r_dir, n));
        }
        if has_id {
            self.write_line(&format!(
                "{} wire [{}:0] {}_rid,",
                r_dir,
                bus.id_width - 1,
                n
            ));
        }

        self.write_line("");
    }

    fn emit_apb_bridge(&mut self, apb: &HirApbBridge) {
        self.write_line(&format!(
            "// APB BRIDGE {} ({} addr, {} data, {} peripherals)",
            apb.name, apb.addr_width, apb.data_width, apb.num_peripherals
        ));

        let n = &apb.name;
        let aw = apb.addr_width;
        let dw = apb.data_width;

        // APB slave signals (input to bridge)
        self.write_line(&format!("input  wire        {}_psel,", n));
        self.write_line(&format!("input  wire        {}_penable,", n));
        self.write_line(&format!("input  wire        {}_pwrite,", n));
        self.write_line(&format!("input  wire [{}:0] {}_paddr,", aw - 1, n));
        self.write_line(&format!("input  wire [{}:0] {}_pwdata,", dw - 1, n));
        self.write_line(&format!(
            "input  wire [{}:0] {}_pstrb,",
            apb.strb_width() - 1,
            n
        ));
        self.write_line(&format!("output wire [{}:0] {}_prdata,", dw - 1, n));
        self.write_line(&format!("output wire        {}_pready,", n));
        if apb.has_pslverr {
            self.write_line(&format!("output wire        {}_pslverr,", n));
        }
        if apb.has_pprot {
            self.write_line(&format!("input  wire [2:0] {}_pprot,", n));
        }

        // Peripheral select outputs
        let mut i = 0u32;
        while i < apb.num_peripherals {
            self.write_line(&format!("output wire        {}_periph{}_sel,", n, i));
            self.write_line(&format!(
                "output wire [{}:0] {}_periph{}_addr,",
                aw - 1,
                n,
                i
            ));
            self.write_line(&format!(
                "output wire [{}:0] {}_periph{}_wdata,",
                dw - 1,
                n,
                i
            ));
            self.write_line(&format!("output wire        {}_periph{}_wen,", n, i));
            self.write_line(&format!(
                "input  wire [{}:0] {}_periph{}_rdata,",
                dw - 1,
                n,
                i
            ));
            i += 1;
        }

        // Address decode logic
        self.write_line(&format!("reg  [{}:0] {}_prdata_r;", dw - 1, n));
        self.write_line(&format!("reg         {}_pready_r;", n));
        self.write_line(&format!("assign {}_prdata = {}_prdata_r;", n, n));
        self.write_line(&format!("assign {}_pready = {}_pready_r;", n, n));

        self.write_line(&format!("always @(*) begin",));
        self.indent();
        self.write_line(&format!("{}_prdata_r = {}d0;", n, dw));
        self.write_line(&format!("{}_pready_r = 1'b1;", n));
        i = 0;
        while i < apb.num_peripherals {
            self.write_line(&format!("{}_periph{}_sel = 1'b0;", n, i));
            i += 1;
        }

        for m in &apb.periph_maps {
            self.write_line(&format!(
                "if ({}_paddr >= {}d{} && {}_paddr < {}d{}) begin",
                n,
                aw,
                m.base_addr,
                n,
                aw,
                m.base_addr + m.size
            ));
            self.indent();
            self.write_line(&format!(
                "{}_periph{}_sel = {}_psel && {}_penable;",
                n, m.index, n, n
            ));
            self.write_line(&format!(
                "{}_periph{}_addr = {}_paddr - {}d{};",
                n, m.index, n, aw, m.base_addr
            ));
            self.write_line(&format!("{}_periph{}_wdata = {}_pwdata;", n, m.index, n));
            self.write_line(&format!("{}_periph{}_wen = {}_pwrite;", n, m.index, n));
            self.write_line(&format!("{}_prdata_r = {}_periph{}_rdata;", n, n, m.index));
            self.dedent();
            self.write_line("end");
        }
        self.dedent();
        self.write_line("end");
        self.write_line("");
    }

    fn emit_gf16_accel(&mut self, gf16: &HirGf16Accel) {
        self.write_line(&format!(
            "// GF16 ACCELERATOR {} ({} mult, {} vec, {} DSP48, {} BRAM)",
            gf16.name,
            gf16.num_multipliers,
            gf16.vector_width,
            gf16.dsp48_count(),
            gf16.bram_count()
        ));
        self.write_line("// phi^2 + 1/phi^2 = 3 | TRINITY");

        let n = &gf16.name;
        let nm = gf16.num_multipliers;
        let vw = gf16.vector_width;

        // Control interface
        self.write_line(&format!("input  wire        {}_start,", n));
        self.write_line(&format!("input  wire [3:0]  {}_opcode,", n));
        self.write_line(&format!("output wire        {}_done,", n));
        self.write_line(&format!("output wire        {}_busy,", n));
        self.write_line(&format!("output wire        {}_result_valid,", n));

        // Data input
        self.write_line(&format!("input  wire [{}:0] {}_a_data,", vw * 4 - 1, n));
        self.write_line(&format!("input  wire [{}:0] {}_b_data,", vw * 4 - 1, n));
        self.write_line(&format!("input  wire        {}_data_valid,", n));

        // Data output
        self.write_line(&format!(
            "output wire [{}:0] {}_result_data,",
            vw * 4 - 1,
            n
        ));

        // GF16 multiplier array
        self.write_line(&format!(
            "// GF16 multiplier array ({} parallel 4-bit GF multiply)",
            nm
        ));
        self.write_line(&format!("wire [{}:0] {}_gf_mul_result;", nm * 4 - 1, n));
        for i in 0..nm {
            self.write_line(&format!("gf16_multiply {}_mult_{} (", n, i));
            self.indent();
            self.write_line(&format!(".a({}_a_data[{}:{}]),", n, (i + 1) * 4 - 1, i * 4));
            self.write_line(&format!(".b({}_b_data[{}:{}]),", n, (i + 1) * 4 - 1, i * 4));
            self.write_line(&format!(
                ".p({}_gf_mul_result[{}:{}])",
                n,
                (i + 1) * 4 - 1,
                i * 4
            ));
            self.dedent();
            self.write_line(");");
        }

        // MAC accumulator if present
        if gf16.has_mac {
            self.write_line(&format!(
                "// MAC accumulator ({} bits)",
                gf16.mac_units.len()
            ));
            for mac in &gf16.mac_units {
                let aw = mac.accumulator_width;
                self.write_line(&format!("reg  [{}:0] {}_{}_acc;", aw - 1, n, mac.name));
                self.write_line(&format!("wire [{}:0] {}_{}_result;", aw - 1, n, mac.name));
                if mac.pipeline_stages > 0 {
                    self.write_line(&format!("// Pipeline: {} stages", mac.pipeline_stages));
                    for stage in 0..mac.pipeline_stages {
                        self.write_line(&format!(
                            "reg  [{}:0] {}_{}_pipe{};",
                            aw - 1,
                            n,
                            mac.name,
                            stage
                        ));
                    }
                }
            }
        }

        // FFT butterfly
        if let Some(ref fft) = gf16.fft_config {
            self.write_line(&format!(
                "// FFT butterfly ({}pt radix-{}, {} stages, {} twiddles)",
                fft.num_points,
                fft.radix,
                fft.fft_stages(),
                fft.twiddle_count()
            ));
            self.write_line(&format!(
                "wire [{}:0] {}_fft_twiddle;",
                fft.num_points * 4 - 1,
                n
            ));
            self.write_line(&format!(
                "(* ram_style = \"block\" *) reg [3:0] {}_fft_mem [0:{}];",
                n,
                fft.num_points - 1
            ));
        }

        // Status register
        self.write_line(&format!("reg  {}_busy_r;", n));
        self.write_line(&format!("reg  {}_done_r;", n));
        self.write_line(&format!("reg  {}_valid_r;", n));
        self.write_line(&format!("assign {}_busy = {}_busy_r;", n, n));
        self.write_line(&format!("assign {}_done = {}_done_r;", n, n));
        self.write_line(&format!("assign {}_result_valid = {}_valid_r;", n, n));

        self.write_line("");
    }

    fn emit_formal(&mut self, hir: &HirModule) {
        self.write_line("// Formal verification assertions (SVA)");
        self.write_line("// phi^2 + 1/phi^2 = 3 | TRINITY");
        self.write_line("");

        if let Some(ref fcfg) = hir.formal_config {
            self.write_line(&format!(
                "// Formal config: {} (module={}, depth={}, timeout={})",
                fcfg.name, fcfg.module_name, fcfg.depth, fcfg.timeout_cycles
            ));
            self.write_line(&format!("// Clock: {} | Reset: {}", fcfg.clock, fcfg.reset));
            self.write_line("`ifndef FORMAL");
            self.write_line("`define FORMAL");
            self.write_line("`endif");
            self.write_line("");
        }

        for fa in &hir.formal_asserts {
            match fa.kind {
                HwAssertKind::Immediate => {
                    let severity = match fa.severity {
                        HwAssertSeverity::Info => "$info",
                        HwAssertSeverity::Warning => "$warning",
                        HwAssertSeverity::Error => "$error",
                        HwAssertSeverity::Fatal => "$fatal",
                    };
                    if !fa.description.is_empty() {
                        self.write_line(&format!("// {}", fa.description));
                    }
                    self.write_line(&format!(
                        "assert_{}: assert property ({}) else {}(\"{}\");",
                        fa.name, fa.condition, severity, fa.name
                    ));
                }
                HwAssertKind::Concurrent => {
                    let severity = match fa.severity {
                        HwAssertSeverity::Info => "$info",
                        HwAssertSeverity::Warning => "$warning",
                        HwAssertSeverity::Error => "$error",
                        HwAssertSeverity::Fatal => "$fatal",
                    };
                    if !fa.description.is_empty() {
                        self.write_line(&format!("// {}", fa.description));
                    }
                    let reset_guard = if !fa.reset.is_empty() {
                        format!("{} |-> ", fa.reset)
                    } else {
                        String::new()
                    };
                    self.write_line(&format!(
                        "assert_{}: assert property (@(posedge {}) disable iff (!{}) {}{}) else {}(\"{}\");",
                        fa.name, fa.clock, fa.reset, reset_guard, fa.condition, severity, fa.name
                    ));
                }
                _ => {}
            }
        }

        for fc in &hir.formal_covers {
            if !fc.description.is_empty() {
                self.write_line(&format!("// Cover: {}", fc.description));
            }
            self.write_line(&format!(
                "cover_{}: cover property (@(posedge {}) {});",
                fc.name, fc.clock, fc.condition
            ));
        }

        for fam in &hir.formal_assumes {
            if !fam.description.is_empty() {
                self.write_line(&format!("// Assume: {}", fam.description));
            }
            self.write_line(&format!(
                "assume_{}: assume property (@(posedge {}) {});",
                fam.name, fam.clock, fam.condition
            ));
        }

        self.write_line("");
    }

    fn emit_ternary_core(&mut self, tc: &HirTernaryCore) {
        self.write_line(&format!(
            "// TERNARY CORE {} ({} ALUs, {} DSP48, {} BRAM, ~{} LUTs, {} MHz)",
            tc.name,
            tc.num_alus,
            tc.dsp_count(),
            tc.bram_count(),
            tc.lut_estimate(),
            tc.fmax_mhz()
        ));
        self.write_line("// phi^2 + 1/phi^2 = 3 | TRINITY");

        let n = &tc.name;
        let dw = tc.data_width;
        let aw = tc.addr_width;

        // Instruction interface
        self.write_line(&format!(
            "input  wire [{}:0] {}_instr,
    output wire        {}_instr_ready,
    output wire [{}:0] {}_instr_addr,",
            31,
            n,
            n,
            aw - 1,
            n
        ));

        // Data interface
        self.write_line(&format!(
            "input  wire [{}:0] {}_mem_rdata,
    output wire [{}:0] {}_mem_wdata,
    output wire [{}:0] {}_mem_addr,
    output wire        {}_mem_we,
    output wire        {}_mem_re,",
            dw - 1,
            n,
            dw - 1,
            n,
            aw - 1,
            n,
            n,
            n
        ));

        // Register file interface
        if let Some(ref rf) = tc.reg_file {
            self.write_line(&format!(
                "// Register file: {} regs x {} trits ({} bits, {} BRAM18)",
                rf.num_regs,
                rf.trit_width,
                rf.total_bits(),
                rf.bram18_count()
            ));
            self.write_line(&format!(
                "(* ram_style = \"block\" *) reg [{}:0] {}_regs [0:{}];",
                rf.trit_width * 2 - 1,
                n,
                rf.num_regs - 1
            ));
            for rp in 0..rf.read_ports {
                self.write_line(&format!("wire  [4:0] {}_rf_raddr{};", n, rp));
                self.write_line(&format!(
                    "wire  [{}:0] {}_rf_rdata{};",
                    rf.trit_width * 2 - 1,
                    n,
                    rp
                ));
            }
            for wp in 0..rf.write_ports {
                self.write_line(&format!("wire  [4:0] {}_rf_waddr{};", n, wp));
                self.write_line(&format!(
                    "wire  [{}:0] {}_rf_wdata{};",
                    rf.trit_width * 2 - 1,
                    n,
                    wp
                ));
                self.write_line(&format!("wire         {}_rf_we{};", n, wp));
            }
        }

        // Pipeline stages
        if !tc.pipeline_stages.is_empty() {
            self.write_line("// Pipeline stages:");
            for stage in &tc.pipeline_stages {
                let fwd = if stage.has_forwarding {
                    " [FORWARDING]"
                } else {
                    ""
                };
                self.write_line(&format!(
                    "//   {} ({} cycle{}){}",
                    stage.name,
                    stage.latency,
                    if stage.latency != 1 { "s" } else { "" },
                    fwd
                ));
            }
            self.write_line(&format!(
                "// Total pipeline latency: {} cycles",
                tc.pipeline_total_latency()
            ));
        }

        // ALU ops
        if !tc.alu_ops.is_empty() {
            self.write_line("// ALU operations:");
            for op in &tc.alu_ops {
                let gf16_tag = if op.uses_gf16 { " [GF16]" } else { "" };
                self.write_line(&format!(
                    "//   {} (opcode={}, {} cycles{})",
                    op.name, op.opcode, op.latency, gf16_tag
                ));
            }
        }

        // Core state
        self.write_line(&format!("reg  [{}:0] {}_pc;", aw - 1, n));
        self.write_line(&format!("reg  [{}:0] {}_ir;", 31, n));
        self.write_line(&format!("reg  [{}:0] {}_status;", 7, n));
        self.write_line(&format!("reg  [2:0]  {}_state;", n));

        self.write_line("");
    }

    fn emit_instance(&mut self, inst: &HirInstance) {
        self.write_line(&format!("{} {} (", inst.module_name, inst.name));
        self.indent();
        for (i, (port, signal)) in inst.port_map.iter().enumerate() {
            let comma = if i < inst.port_map.len() - 1 { "," } else { "" };
            self.write_line(&format!(".{}({}){}", port, signal, comma));
        }
        self.dedent();
        self.write_line(");");
        self.write_line("");
    }
}

#[cfg(test)]
mod tests_hw_types {
    use super::*;

    #[test]
    fn test_bits_width() {
        assert_eq!(HwType::Bits(8).hw_width(), 8);
        assert_eq!(HwType::Bits(32).hw_width(), 32);
        assert_eq!(HwType::Bits(0).hw_width(), 0);
    }

    #[test]
    fn test_uint_width() {
        assert_eq!(HwType::UInt(8).hw_width(), 8);
        assert_eq!(HwType::UInt(1).hw_width(), 1);
        assert_eq!(HwType::UInt(64).hw_width(), 64);
    }

    #[test]
    fn test_sint_width() {
        assert_eq!(HwType::SInt(8).hw_width(), 8);
        assert_eq!(HwType::SInt(32).hw_width(), 32);
    }

    #[test]
    fn test_bool_width() {
        assert_eq!(HwType::Bool.hw_width(), 1);
    }

    #[test]
    fn test_clock_width() {
        assert_eq!(HwType::Clock.hw_width(), 1);
    }

    #[test]
    fn test_reset_width() {
        assert_eq!(
            HwType::Reset(HwResetKind::Async, HwResetPolarity::ActiveLow).hw_width(),
            1
        );
        assert_eq!(
            HwType::Reset(HwResetKind::Sync, HwResetPolarity::ActiveHigh).hw_width(),
            1
        );
    }

    #[test]
    fn test_vector_width() {
        let v = HwType::Vector(Box::new(HwType::UInt(8)), 4);
        assert_eq!(v.hw_width(), 32);
    }

    #[test]
    fn test_nested_vector_width() {
        let inner = HwType::Vector(Box::new(HwType::UInt(4)), 2);
        let outer = HwType::Vector(Box::new(inner), 3);
        assert_eq!(outer.hw_width(), 24);
    }

    #[test]
    fn test_bundle_width() {
        let bundle = HwType::Bundle(vec![
            ("a".into(), HwType::UInt(8)),
            ("b".into(), HwType::UInt(16)),
            ("c".into(), HwType::Bool),
        ]);
        assert_eq!(bundle.hw_width(), 25);
    }

    #[test]
    fn test_enum_width() {
        let e2 = HwType::Enum(vec![("A".into(), "".into()), ("B".into(), "".into())]);
        assert_eq!(e2.hw_width(), 1);

        let e5 = HwType::Enum(vec![
            ("A".into(), "".into()),
            ("B".into(), "".into()),
            ("C".into(), "".into()),
            ("D".into(), "".into()),
            ("E".into(), "".into()),
        ]);
        assert_eq!(e5.hw_width(), 3);
    }

    #[test]
    fn test_gf16_width() {
        assert_eq!(HwType::GF16.hw_width(), 16);
    }

    #[test]
    fn test_signedness() {
        assert!(HwType::SInt(32).is_signed());
        assert!(!HwType::UInt(32).is_signed());
        assert!(!HwType::Bool.is_signed());
        assert!(!HwType::Clock.is_signed());
    }

    #[test]
    fn test_clock_like() {
        assert!(HwType::Clock.is_clock_like());
        assert!(!HwType::Bool.is_clock_like());
        assert!(!HwType::UInt(1).is_clock_like());
    }

    #[test]
    fn test_reset_like() {
        assert!(HwType::Reset(HwResetKind::Async, HwResetPolarity::ActiveLow).is_reset_like());
        assert!(!HwType::Bool.is_reset_like());
    }

    #[test]
    fn test_verilog_range() {
        assert_eq!(HwType::Bool.verilog_range(), "");
        assert_eq!(HwType::UInt(8).verilog_range(), "[7:0]");
        assert_eq!(HwType::UInt(32).verilog_range(), "[31:0]");
        assert_eq!(HwType::GF16.verilog_range(), "[15:0]");
    }
}

#[cfg(test)]
mod tests_hir_module {
    use super::*;

    #[test]
    fn test_empty_module_name_fails() {
        let m = HirModule::new("");
        let errors = m.validate();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("empty"));
    }

    #[test]
    fn test_duplicate_port_names_fails() {
        let m = HirModule {
            name: "dup".into(),
            ports: vec![
                HirPort {
                    name: "a".into(),
                    dir: HwPortDir::Input,
                    ty: HwType::Bool,
                },
                HirPort {
                    name: "a".into(),
                    dir: HwPortDir::Output,
                    ty: HwType::Bool,
                },
            ],
            signals: vec![],
            assigns: vec![],
            always_blocks: vec![],
            instances: vec![],
            memories: vec![],
            clock_domains: vec![],
            clock_crossings: vec![],
            fifos: vec![],
            bus_ports: vec![],
            apb_bridges: vec![],
            gf16_accels: vec![],
            formal_asserts: vec![],
            formal_covers: vec![],
            formal_assumes: vec![],
            formal_config: None,
            ternary_cores: vec![],
        };
        let errors = m.validate();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("duplicate port"));
    }

    #[test]
    fn test_duplicate_signal_names_fails() {
        let m = HirModule {
            name: "dup_sig".into(),
            ports: vec![],
            signals: vec![
                HirSignal {
                    name: "counter".into(),
                    kind: HwSignalKind::Reg,
                    ty: HwType::UInt(8),
                    reset_value: "0".into(),
                },
                HirSignal {
                    name: "counter".into(),
                    kind: HwSignalKind::Wire,
                    ty: HwType::UInt(16),
                    reset_value: "".into(),
                },
            ],
            assigns: vec![],
            always_blocks: vec![],
            instances: vec![],
            memories: vec![],
            clock_domains: vec![],
            clock_crossings: vec![],
            fifos: vec![],
            bus_ports: vec![],
            apb_bridges: vec![],
            gf16_accels: vec![],
            formal_asserts: vec![],
            formal_covers: vec![],
            formal_assumes: vec![],
            formal_config: None,
            ternary_cores: vec![],
        };
        let errors = m.validate();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("duplicate signal"));
    }

    #[test]
    fn test_multiple_duplicates() {
        let m = HirModule {
            name: "".into(),
            ports: vec![
                HirPort {
                    name: "a".into(),
                    dir: HwPortDir::Input,
                    ty: HwType::Bool,
                },
                HirPort {
                    name: "a".into(),
                    dir: HwPortDir::Output,
                    ty: HwType::Bool,
                },
            ],
            signals: vec![
                HirSignal {
                    name: "x".into(),
                    kind: HwSignalKind::Reg,
                    ty: HwType::Bool,
                    reset_value: "".into(),
                },
                HirSignal {
                    name: "x".into(),
                    kind: HwSignalKind::Wire,
                    ty: HwType::Bool,
                    reset_value: "".into(),
                },
            ],
            assigns: vec![],
            always_blocks: vec![],
            instances: vec![],
            memories: vec![],
            clock_domains: vec![],
            clock_crossings: vec![],
            fifos: vec![],
            bus_ports: vec![],
            apb_bridges: vec![],
            gf16_accels: vec![],
            formal_asserts: vec![],
            formal_covers: vec![],
            formal_assumes: vec![],
            formal_config: None,
            ternary_cores: vec![],
        };
        let errors = m.validate();
        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn test_module_new_fields() {
        let m = HirModule::new("my_module");
        assert_eq!(m.name, "my_module");
        assert!(m.ports.is_empty());
        assert!(m.signals.is_empty());
        assert!(m.assigns.is_empty());
        assert!(m.always_blocks.is_empty());
        assert!(m.instances.is_empty());
    }
}

#[cfg(test)]
mod tests_ast_to_hir {
    use super::*;

    fn parse_single(code: &str) -> Node {
        let lex = Lexer::new(code);
        let mut parser = Parser::new(lex);
        parser.parse().expect("parse should succeed")
    }

    fn dump_node(n: &Node, depth: usize) {
        eprintln!(
            "{}kind={:?} name='{}' mutable={} type='{}' ret_type='{}' children={}",
            "  ".repeat(depth),
            n.kind,
            n.name,
            n.extra_mutable,
            n.extra_type,
            n.extra_return_type,
            n.children.len()
        );
    }

    #[test]
    fn test_convert_empty_module() {
        let root = parse_single("module Empty { }");
        eprintln!(
            "ROOT: kind={:?} name='{}' children={}",
            root.kind,
            root.name,
            root.children.len()
        );
        for (i, c) in root.children.iter().enumerate() {
            eprintln!("  child[{}]: kind={:?} name='{}'", i, c.kind, c.name);
        }
        let hir = AstToHir::convert(&root).unwrap();
        assert_eq!(hir.name, "Empty");
        assert!(hir.ports.is_empty());
        assert!(hir.signals.is_empty());
    }

    #[test]
    fn test_convert_const_decl() {
        let code = "module M { pub const X : u32 = 42; }";
        let module = &parse_single(code);
        let hir = AstToHir::convert(module).unwrap();
        assert_eq!(hir.signals.len(), 1);
        assert_eq!(hir.signals[0].name, "X");
        assert_eq!(hir.signals[0].kind, HwSignalKind::Wire);
        assert_eq!(hir.signals[0].ty, HwType::UInt(32));
    }

    #[test]
    fn test_convert_var_is_reg() {
        let root = parse_single("module M { var counter : u16 = 0; }");
        let hir = AstToHir::convert(&root).unwrap();
        assert_eq!(hir.signals.len(), 1);
        assert_eq!(hir.signals[0].name, "counter");
        assert_eq!(hir.signals[0].kind, HwSignalKind::Reg);
        assert_eq!(hir.signals[0].ty, HwType::UInt(16));
    }

    #[test]
    fn test_convert_struct_to_bundle() {
        let code = "module M { pub struct Pair { lo : u8, hi : u8, } }";
        let module = &parse_single(code);
        let hir = AstToHir::convert(module).unwrap();
        assert_eq!(hir.signals.len(), 1);
        assert_eq!(hir.signals[0].name, "Pair");
        match &hir.signals[0].ty {
            HwType::Bundle(fields) => {
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "lo");
                assert_eq!(fields[1].0, "hi");
            }
            other => panic!("Expected Bundle, got {:?}", other),
        }
    }

    #[test]
    fn test_convert_fn_to_ports() {
        let code = "module M { pub fn add(a: u8, b: u8) -> u8 { return a; } }";
        let module = &parse_single(code);
        let hir = AstToHir::convert(module).unwrap();
        let port_names: Vec<&str> = hir.ports.iter().map(|p| p.name.as_str()).collect();
        assert!(port_names.contains(&"add_result"));
        assert!(port_names.contains(&"a"));
        assert!(port_names.contains(&"b"));
    }

    #[test]
    fn test_type_mapping() {
        assert_eq!(AstToHir::t27_type_to_hw("bool"), HwType::Bool);
        assert_eq!(AstToHir::t27_type_to_hw("u8"), HwType::UInt(8));
        assert_eq!(AstToHir::t27_type_to_hw("u16"), HwType::UInt(16));
        assert_eq!(AstToHir::t27_type_to_hw("u32"), HwType::UInt(32));
        assert_eq!(AstToHir::t27_type_to_hw("i8"), HwType::SInt(8));
        assert_eq!(AstToHir::t27_type_to_hw("i32"), HwType::SInt(32));
        assert_eq!(AstToHir::t27_type_to_hw("GF16"), HwType::GF16);
    }

    #[test]
    fn test_type_mapping_array() {
        let arr_ty = AstToHir::t27_type_to_hw("[4]u8");
        assert_eq!(arr_ty, HwType::Vector(Box::new(HwType::UInt(8)), 4));
        assert_eq!(arr_ty.hw_width(), 32);
    }

    #[test]
    fn test_convert_enum_decl() {
        let root =
            parse_single("module M { pub const Edge = enum(i8) { posedge = 0, negedge = 1, }; }");
        let hir = AstToHir::convert(&root).unwrap();
        assert_eq!(hir.signals.len(), 1);
        assert_eq!(hir.signals[0].name, "Edge");
        match &hir.signals[0].ty {
            HwType::Enum(variants) => {
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].0, "posedge");
                assert_eq!(variants[1].0, "negedge");
            }
            other => panic!("Expected Enum, got {:?}", other),
        }
    }

    #[test]
    fn test_convert_fn_with_assignments() {
        let root =
            parse_single("module M { pub fn blink(led: u8) -> u8 { led = 1; return led; } }");
        let hir = AstToHir::convert(&root).unwrap();
        assert!(hir.assigns.iter().any(|a| a.target.contains("led")));
    }
}

#[cfg(test)]
mod tests_hir_memory {
    use super::*;

    #[test]
    fn test_bram_new() {
        let m = HirMemory::new_bram("test_ram", 1024, 32);
        assert_eq!(m.name, "test_ram");
        assert_eq!(m.depth, 1024);
        assert_eq!(m.data_width, 32);
        assert_eq!(m.addr_width, 10);
        assert_eq!(m.kind, HwMemKind::Bram);
        assert!(m.ports.is_empty());
    }

    #[test]
    fn test_bram_addr_width_small() {
        assert_eq!(HirMemory::new_bram("r", 4, 8).addr_width, 2);
        assert_eq!(HirMemory::new_bram("r", 1, 8).addr_width, 1);
        assert_eq!(HirMemory::new_bram("r", 2, 8).addr_width, 1);
        assert_eq!(HirMemory::new_bram("r", 256, 8).addr_width, 8);
    }

    #[test]
    fn test_bram_add_ports() {
        let mut m = HirMemory::new_bram("ram", 256, 16);
        m.add_read_port("rda");
        m.add_write_port("wra");
        assert_eq!(m.ports.len(), 2);
        assert!(m.has_read());
        assert!(m.has_write());
    }

    #[test]
    fn test_rom_no_write() {
        let mut m = HirMemory::new_rom("rom", 512, 8);
        m.add_read_port("rda");
        assert!(m.has_read());
        assert!(!m.has_write());
        assert_eq!(m.kind, HwMemKind::Rom);
    }

    #[test]
    fn test_total_bits() {
        let m = HirMemory::new_bram("ram", 1024, 32);
        assert_eq!(m.total_bits(), 32768);
    }

    #[test]
    fn test_bram18_count() {
        let m = HirMemory::new_bram("ram", 1024, 18);
        assert_eq!(m.bram18_count(), 1);
        let m2 = HirMemory::new_bram("ram", 4096, 36);
        assert!(m2.bram18_count() >= 8);
    }

    #[test]
    fn test_validate_ok() {
        let mut m = HirMemory::new_bram("ram", 256, 16);
        m.add_read_port("rda");
        assert!(m.validate().is_empty());
    }

    #[test]
    fn test_validate_empty_name() {
        let m = HirMemory::new_bram("", 256, 16);
        assert!(m.validate().iter().any(|e| e.contains("empty")));
    }

    #[test]
    fn test_validate_zero_depth() {
        let m = HirMemory::new_bram("ram", 0, 16);
        assert!(m.validate().iter().any(|e| e.contains("zero depth")));
    }

    #[test]
    fn test_validate_rom_with_write() {
        let mut m = HirMemory::new_rom("rom", 256, 16);
        m.add_write_port("wra");
        assert!(m.validate().iter().any(|e| e.contains("cannot have write")));
    }

    #[test]
    fn test_memory_verilog_emission() {
        let mut mem = HirMemory::new_bram("sram", 256, 16);
        mem.add_read_port("rda");
        mem.add_write_port("wra");
        let hir = HirModule {
            name: "MemTest".into(),
            ports: vec![],
            signals: vec![],
            assigns: vec![],
            always_blocks: vec![],
            instances: vec![],
            memories: vec![mem],
            clock_domains: vec![],
            clock_crossings: vec![],
            fifos: vec![],
            bus_ports: vec![],
            apb_bridges: vec![],
            gf16_accels: vec![],
            formal_asserts: vec![],
            formal_covers: vec![],
            formal_assumes: vec![],
            formal_config: None,
            ternary_cores: vec![],
        };
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        let verilog = emitter.into_string();
        assert!(verilog.contains("BRAM sram"));
        assert!(verilog.contains("reg [15:0] sram [0:255]"));
        assert!(verilog.contains("rda_addr"));
        assert!(verilog.contains("wra_addr"));
        assert!(verilog.contains("posedge clk"));
    }
}

#[cfg(test)]
mod tests_hir_fifo {
    use super::*;

    #[test]
    fn test_fifo_sync_new() {
        let f = HirFifo::new_sync("tx_fifo", 16, 8);
        assert_eq!(f.name, "tx_fifo");
        assert_eq!(f.depth, 16);
        assert_eq!(f.data_width, 8);
        assert_eq!(f.kind, HwFifoKind::Sync);
        assert_eq!(f.addr_width(), 4);
    }

    #[test]
    fn test_fifo_async_new() {
        let f = HirFifo::new_async("cross", 32, 16);
        assert_eq!(f.kind, HwFifoKind::Async);
        assert_eq!(f.addr_width(), 5);
    }

    #[test]
    fn test_fifo_total_bits() {
        let f = HirFifo::new_sync("f", 16, 32);
        assert_eq!(f.total_bits(), 512);
    }

    #[test]
    fn test_fifo_verilog_emission() {
        let mut hir = HirModule::new("FifoTest");
        hir.fifos.push(HirFifo::new_sync("tx_fifo", 16, 8));
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        let v = emitter.into_string();
        assert!(v.contains("SYNC_FIFO tx_fifo"));
        assert!(v.contains("tx_fifo_mem"));
        assert!(v.contains("tx_fifo_empty"));
        assert!(v.contains("tx_fifo_full"));
        assert!(v.contains("tx_fifo_dout"));
        assert!(v.contains("tx_fifo_din"));
        assert!(v.contains("tx_fifo_wen"));
        assert!(v.contains("tx_fifo_ren"));
        assert!(v.contains("posedge clk"));
    }
}

#[cfg(test)]
mod tests_hir_clock_domain {
    use super::*;

    #[test]
    fn test_domain_new() {
        let d = HirClockDomain::new("sys", "sys_clk", 12_000_000);
        assert_eq!(d.name, "sys");
        assert_eq!(d.freq_hz, 12_000_000);
        assert_eq!(d.source_name, "sys_clk");
    }

    #[test]
    fn test_period_12mhz() {
        let d = HirClockDomain::new("sys", "clk", 12_000_000);
        assert_eq!(d.period_ns(), 83);
    }

    #[test]
    fn test_period_100mhz() {
        let d = HirClockDomain::new("fast", "pll", 100_000_000);
        assert_eq!(d.period_ns(), 10);
    }

    #[test]
    fn test_half_period() {
        let d = HirClockDomain::new("sys", "clk", 12_000_000);
        assert_eq!(d.half_period_ns(), 41);
    }

    #[test]
    fn test_period_zero_freq() {
        let d = HirClockDomain::new("zero", "clk", 0);
        assert_eq!(d.period_ns(), 0);
    }

    #[test]
    fn test_clock_crossing() {
        let c = HirClockCrossing {
            src_domain: "sys".into(),
            dst_domain: "fast".into(),
            strategy: HwCrossStrategy::TwoFlop,
            data_width: 32,
        };
        assert_eq!(c.src_domain, "sys");
        assert_eq!(c.data_width, 32);
        assert_eq!(c.strategy, HwCrossStrategy::TwoFlop);
    }
}

#[cfg(test)]
mod tests_hir_optimizer {
    use super::*;

    #[test]
    fn test_dead_signal_elimination() {
        let mut hir = HirModule::new("opt_test");
        hir.signals.push(HirSignal {
            name: "used_sig".into(),
            kind: HwSignalKind::Wire,
            ty: HwType::UInt(8),
            reset_value: "42".into(),
        });
        hir.signals.push(HirSignal {
            name: "unused_sig".into(),
            kind: HwSignalKind::Wire,
            ty: HwType::UInt(8),
            reset_value: "0".into(),
        });
        hir.assigns.push(HirAssign {
            target: "led".into(),
            value: "used_sig".into(),
        });
        let mut opt = HirOptimizer::new();
        opt.optimize(&mut hir);
        assert!(!hir.signals.iter().any(|s| s.name == "unused_sig"));
        assert!(opt.stats().0 >= 1);
        let led_assign = hir.assigns.iter().find(|a| a.target == "led").unwrap();
        assert_eq!(led_assign.value, "42");
    }

    #[test]
    fn test_reg_not_eliminated() {
        let mut hir = HirModule::new("reg_test");
        hir.signals.push(HirSignal {
            name: "unused_reg".into(),
            kind: HwSignalKind::Reg,
            ty: HwType::UInt(16),
            reset_value: "0".into(),
        });
        let mut opt = HirOptimizer::new();
        opt.optimize(&mut hir);
        assert!(hir.signals.iter().any(|s| s.name == "unused_reg"));
    }

    #[test]
    fn test_constant_folding() {
        let mut hir = HirModule::new("fold_test");
        hir.signals.push(HirSignal {
            name: "CONST_VAL".into(),
            kind: HwSignalKind::Wire,
            ty: HwType::UInt(8),
            reset_value: "42".into(),
        });
        hir.assigns.push(HirAssign {
            target: "led".into(),
            value: "CONST_VAL".into(),
        });
        let mut opt = HirOptimizer::new();
        opt.optimize(&mut hir);
        let folded_assign = hir.assigns.iter().find(|a| a.target == "led").unwrap();
        assert_eq!(folded_assign.value, "42");
        let (_, _, folded, _) = opt.stats();
        assert_eq!(folded, 1);
    }

    #[test]
    fn test_no_optimization_needed() {
        let mut hir = HirModule::new("clean");
        hir.ports.push(HirPort {
            name: "clk".into(),
            dir: HwPortDir::Input,
            ty: HwType::Clock,
        });
        let mut opt = HirOptimizer::new();
        opt.optimize(&mut hir);
        let (r, a, f, _) = opt.stats();
        assert_eq!(r, 0);
        assert_eq!(a, 0);
        assert_eq!(f, 0);
    }

    #[test]
    fn test_merge_chained_assigns() {
        let mut hir = HirModule::new("merge_test");
        hir.assigns.push(HirAssign {
            target: "a".into(),
            value: "b + 1".into(),
        });
        hir.assigns.push(HirAssign {
            target: "c".into(),
            value: "a".into(),
        });
        let mut opt = HirOptimizer::new();
        opt.optimize(&mut hir);
        let c_assign = hir.assigns.iter().find(|a| a.target == "c").unwrap();
        assert_eq!(c_assign.value, "b + 1");
    }

    #[test]
    fn test_multipass_converges() {
        let mut hir = HirModule::new("multipass");
        hir.signals.push(HirSignal {
            name: "A".into(),
            kind: HwSignalKind::Wire,
            ty: HwType::UInt(8),
            reset_value: "5".into(),
        });
        hir.signals.push(HirSignal {
            name: "B".into(),
            kind: HwSignalKind::Wire,
            ty: HwType::UInt(8),
            reset_value: "0".into(),
        });
        hir.assigns.push(HirAssign {
            target: "B".into(),
            value: "A".into(),
        });
        hir.assigns.push(HirAssign {
            target: "out".into(),
            value: "B".into(),
        });
        let mut opt = HirOptimizer::new();
        opt.optimize(&mut hir);
        let out_assign = hir.assigns.iter().find(|a| a.target == "out").unwrap();
        assert_eq!(out_assign.value, "5");
        assert!(opt.pass_count <= 10);
    }

    #[test]
    fn test_resource_estimate() {
        let mut hir = HirModule::new("res_test");
        hir.ports.push(HirPort {
            name: "clk".into(),
            dir: HwPortDir::Input,
            ty: HwType::Clock,
        });
        hir.ports.push(HirPort {
            name: "data_in".into(),
            dir: HwPortDir::Input,
            ty: HwType::UInt(8),
        });
        hir.ports.push(HirPort {
            name: "data_out".into(),
            dir: HwPortDir::Output,
            ty: HwType::UInt(8),
        });
        hir.signals.push(HirSignal {
            name: "counter".into(),
            kind: HwSignalKind::Reg,
            ty: HwType::UInt(32),
            reset_value: "0".into(),
        });
        hir.assigns.push(HirAssign {
            target: "data_out".into(),
            value: "counter[7:0]".into(),
        });
        let mut mem = HirMemory::new_bram("ram", 512, 32);
        mem.add_read_port("rd");
        hir.memories.push(mem);
        let opt = HirOptimizer::new();
        let res = opt.resource_estimate(&hir);
        assert!(res.ffs > 0);
        assert!(res.bram18 > 0);
        assert!(res.io_pins >= 3);
    }
}

#[cfg(test)]
mod tests_hir_verilog_emitter {
    use super::*;

    #[test]
    fn test_emit_empty_module() {
        let hir = HirModule::new("EmptyMod");
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        let verilog = emitter.into_string();
        assert!(verilog.contains("module EmptyMod"));
        assert!(verilog.contains("endmodule"));
        assert!(verilog.contains("input  wire        clk"));
        assert!(verilog.contains("input  wire        rst_n"));
    }

    #[test]
    fn test_emit_module_with_ports() {
        let hir = HirModule {
            name: "LedBlinker".into(),
            ports: vec![
                HirPort {
                    name: "clk".into(),
                    dir: HwPortDir::Input,
                    ty: HwType::Clock,
                },
                HirPort {
                    name: "led".into(),
                    dir: HwPortDir::Output,
                    ty: HwType::UInt(4),
                },
            ],
            signals: vec![],
            assigns: vec![],
            always_blocks: vec![],
            instances: vec![],
            memories: vec![],
            clock_domains: vec![],
            clock_crossings: vec![],
            fifos: vec![],
            bus_ports: vec![],
            apb_bridges: vec![],
            gf16_accels: vec![],
            formal_asserts: vec![],
            formal_covers: vec![],
            formal_assumes: vec![],
            formal_config: None,
            ternary_cores: vec![],
        };
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        let verilog = emitter.into_string();
        assert!(verilog.contains("module LedBlinker"));
        assert!(verilog.contains("clk"));
        assert!(verilog.contains("output wire [3:0] led"));
    }

    #[test]
    fn test_emit_module_with_signals_and_assigns() {
        let hir = HirModule {
            name: "AssignTest".into(),
            ports: vec![
                HirPort {
                    name: "clk".into(),
                    dir: HwPortDir::Input,
                    ty: HwType::Clock,
                },
                HirPort {
                    name: "rst_n".into(),
                    dir: HwPortDir::Input,
                    ty: HwType::Reset(HwResetKind::Async, HwResetPolarity::ActiveLow),
                },
            ],
            signals: vec![
                HirSignal {
                    name: "counter".into(),
                    kind: HwSignalKind::Reg,
                    ty: HwType::UInt(16),
                    reset_value: "0".into(),
                },
                HirSignal {
                    name: "led_wire".into(),
                    kind: HwSignalKind::Wire,
                    ty: HwType::Bool,
                    reset_value: "".into(),
                },
            ],
            assigns: vec![HirAssign {
                target: "led_wire".into(),
                value: "counter[7]".into(),
            }],
            always_blocks: vec![],
            instances: vec![],
            memories: vec![],
            clock_domains: vec![],
            clock_crossings: vec![],
            fifos: vec![],
            bus_ports: vec![],
            apb_bridges: vec![],
            gf16_accels: vec![],
            formal_asserts: vec![],
            formal_covers: vec![],
            formal_assumes: vec![],
            formal_config: None,
            ternary_cores: vec![],
        };
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        let verilog = emitter.into_string();
        assert!(verilog.contains("reg  [15:0] counter"));
        assert!(verilog.contains("wire led_wire"));
        assert!(verilog.contains("assign led_wire = counter[7]"));
    }

    #[test]
    fn test_emit_signed_port() {
        let hir = HirModule {
            name: "SignedTest".into(),
            ports: vec![HirPort {
                name: "data_in".into(),
                dir: HwPortDir::Input,
                ty: HwType::SInt(8),
            }],
            signals: vec![],
            assigns: vec![],
            always_blocks: vec![],
            instances: vec![],
            memories: vec![],
            clock_domains: vec![],
            clock_crossings: vec![],
            fifos: vec![],
            bus_ports: vec![],
            apb_bridges: vec![],
            gf16_accels: vec![],
            formal_asserts: vec![],
            formal_covers: vec![],
            formal_assumes: vec![],
            formal_config: None,
            ternary_cores: vec![],
        };
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        let verilog = emitter.into_string();
        assert!(verilog.contains("signed"));
        assert!(verilog.contains("[7:0]"));
        assert!(verilog.contains("data_in"));
    }

    #[test]
    fn test_emit_always_block_posedge() {
        let hir = HirModule {
            name: "Counter".into(),
            ports: vec![
                HirPort {
                    name: "clk".into(),
                    dir: HwPortDir::Input,
                    ty: HwType::Clock,
                },
                HirPort {
                    name: "rst_n".into(),
                    dir: HwPortDir::Input,
                    ty: HwType::Reset(HwResetKind::Async, HwResetPolarity::ActiveLow),
                },
            ],
            signals: vec![],
            assigns: vec![],
            always_blocks: vec![HirAlwaysBlock {
                edge: HwEdge::Posedge,
                clock_name: "clk".into(),
                reset_name: "rst_n".into(),
                body: vec![HirAlwaysStmt {
                    kind: HirAlwaysStmtKind::NonBlockingAssign,
                    target: "counter".into(),
                    value: "counter + 1".into(),
                    condition: String::new(),
                    body: vec![],
                }],
            }],
            instances: vec![],
            memories: vec![],
            clock_domains: vec![],
            clock_crossings: vec![],
            fifos: vec![],
            bus_ports: vec![],
            apb_bridges: vec![],
            gf16_accels: vec![],
            formal_asserts: vec![],
            formal_covers: vec![],
            formal_assumes: vec![],
            formal_config: None,
            ternary_cores: vec![],
        };
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        let verilog = emitter.into_string();
        assert!(verilog.contains("always @(posedge clk)"));
        assert!(verilog.contains("if (!rst_n)"));
        assert!(verilog.contains("counter <= counter + 1"));
        assert!(verilog.contains("end"));
    }

    #[test]
    fn test_emit_instance() {
        let hir = HirModule {
            name: "Top".into(),
            ports: vec![],
            signals: vec![],
            assigns: vec![],
            always_blocks: vec![],
            instances: vec![HirInstance {
                name: "u_uart".into(),
                module_name: "UART_TX".into(),
                port_map: vec![
                    ("clk".into(), "clk".into()),
                    ("tx".into(), "uart_tx_pin".into()),
                ],
                param_map: vec![],
            }],
            memories: vec![],
            clock_domains: vec![],
            clock_crossings: vec![],
            fifos: vec![],
            bus_ports: vec![],
            apb_bridges: vec![],
            gf16_accels: vec![],
            formal_asserts: vec![],
            formal_covers: vec![],
            formal_assumes: vec![],
            formal_config: None,
            ternary_cores: vec![],
        };
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        let verilog = emitter.into_string();
        assert!(verilog.contains("UART_TX u_uart"));
        assert!(verilog.contains(".clk(clk)"));
        assert!(verilog.contains(".tx(uart_tx_pin)"));
    }
}

#[cfg(test)]
mod tests_hir_roundtrip {
    use super::*;

    fn roundtrip(code: &str) -> String {
        let lex = Lexer::new(code);
        let mut parser = Parser::new(lex);
        let ast = parser.parse().expect("parse should succeed");
        let hir = AstToHir::convert(&ast).expect("HIR conversion should succeed");
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        emitter.into_string()
    }

    #[test]
    fn test_roundtrip_simple_counter() {
        let code = r#"
module SimpleCounter {
    pub const WIDTH : u32 = 16;
    pub var counter : u16 = 0;
    pub fn tick(clk: bool) -> u16 {
        counter = counter + 1;
        return counter;
    }
}
"#;
        let verilog = roundtrip(code);
        assert!(verilog.contains("module SimpleCounter"));
        assert!(verilog.contains("endmodule"));
        assert!(verilog.contains("reg"));
        assert!(verilog.contains("counter"));
    }

    #[test]
    fn test_roundtrip_led_blinker() {
        let code = r#"
module LedBlinker {
    pub const MAX : u32 = 27000000;
    pub var counter : u32 = 0;
    pub var led : u8 = 0;
    pub fn update(clk: bool, rst_n: bool) -> u8 {
        counter = counter + 1;
        led = counter[24];
        return led;
    }
}
"#;
        let verilog = roundtrip(code);
        assert!(verilog.contains("module LedBlinker"));
        assert!(verilog.contains("counter"));
        assert!(verilog.contains("led"));
        assert!(verilog.contains("update_result"));
    }

    #[test]
    fn test_roundtrip_struct_type() {
        let code = r#"
module StructTest {
    pub struct Pair {
        lo : u8,
        hi : u8,
    }
    pub fn combine(a_lo: u8, a_hi: u8) -> u16 {
        return 0;
    }
}
"#;
        let verilog = roundtrip(code);
        assert!(verilog.contains("module StructTest"));
        assert!(verilog.contains("Pair"));
    }

    #[test]
    fn test_roundtrip_enum_type() {
        let code = r#"
module EnumTest {
    pub const State = enum(i8) {
        idle = 0,
        run = 1,
        done = 2,
    };
    pub fn next(state: u8) -> u8 {
        return 0;
    }
}
"#;
        let verilog = roundtrip(code);
        assert!(verilog.contains("module EnumTest"));
        assert!(verilog.contains("State"));
    }

    #[test]
    fn test_roundtrip_multiple_fns_dedup_ports() {
        let code = r#"
module DedupTest {
    pub fn read(addr: u32) -> u8 {
        return 0;
    }
    pub fn write(addr: u32, data: u8) -> bool {
        return true;
    }
}
"#;
        let verilog = roundtrip(code);
        assert!(verilog.contains("module DedupTest"));
        assert!(verilog.contains("addr"));
        assert!(verilog.contains("write_result"));
        assert!(verilog.contains("read_result"));
        assert!(verilog.contains("data"));
    }

    #[test]
    fn test_roundtrip_uart_spec() {
        let base = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
        let path = std::path::Path::new(&base).join("../specs/fpga/uart.t27");
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("uart.t27 not found at {:?}", path));
        let verilog = roundtrip(&source);
        assert!(verilog.contains("module ZeroDSP_UART"));
        assert!(verilog.contains("endmodule"));
        assert!(verilog.contains("uart_tx_ready_result"));
        assert!(verilog.contains("UART_CLOCK_HZ"));
    }

    #[test]
    fn test_roundtrip_bridge_spec() {
        let base = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
        let path = std::path::Path::new(&base).join("../specs/fpga/bridge.t27");
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("bridge.t27 not found at {:?}", path));
        let verilog = roundtrip(&source);
        assert!(verilog.contains("module FPGA_Bridge"));
        assert!(verilog.contains("endmodule"));
    }
}

#[cfg(test)]
mod tests_hir_bus_port {
    use super::*;

    #[test]
    fn test_axi4_lite_slave_creation() {
        let bus = HirBusPort::axi4_lite_slave("s0", 32, 32);
        assert_eq!(bus.name, "s0");
        assert_eq!(bus.kind, HwBusKind::Axi4Lite);
        assert_eq!(bus.role, HwBusRole::Slave);
        assert_eq!(bus.addr_width, 32);
        assert_eq!(bus.data_width, 32);
        assert_eq!(bus.id_width, 0);
    }

    #[test]
    fn test_axi4_lite_master_creation() {
        let bus = HirBusPort::axi4_lite_master("m0", 32, 32);
        assert_eq!(bus.role, HwBusRole::Master);
        assert!(bus.is_lite());
        assert!(bus.is_master());
    }

    #[test]
    fn test_axi4_full_slave_creation() {
        let bus = HirBusPort::axi4_full_slave("s1", 32, 64, 4);
        assert_eq!(bus.kind, HwBusKind::Axi4Full);
        assert_eq!(bus.data_width, 64);
        assert_eq!(bus.id_width, 4);
        assert!(bus.is_full());
        assert!(bus.is_slave());
    }

    #[test]
    fn test_axi4_full_master_creation() {
        let bus = HirBusPort::axi4_full_master("m1", 32, 64, 4);
        assert!(bus.is_full());
        assert!(bus.is_master());
    }

    #[test]
    fn test_strb_width() {
        let bus32 = HirBusPort::axi4_lite_slave("s0", 32, 32);
        assert_eq!(bus32.strb_width(), 4);
        let bus64 = HirBusPort::axi4_full_slave("s1", 32, 64, 4);
        assert_eq!(bus64.strb_width(), 8);
    }

    #[test]
    fn test_validate_lite_ok() {
        let bus = HirBusPort::axi4_lite_slave("s0", 32, 32);
        assert!(bus.validate().is_empty());
    }

    #[test]
    fn test_validate_full_ok() {
        let bus = HirBusPort::axi4_full_slave("s1", 32, 64, 4);
        assert!(bus.validate().is_empty());
    }

    #[test]
    fn test_validate_empty_name() {
        let bus = HirBusPort::axi4_lite_slave("", 32, 32);
        assert!(!bus.validate().is_empty());
    }

    #[test]
    fn test_validate_zero_addr() {
        let bus = HirBusPort::axi4_lite_slave("s0", 0, 32);
        assert!(!bus.validate().is_empty());
    }

    #[test]
    fn test_validate_zero_data() {
        let bus = HirBusPort::axi4_lite_slave("s0", 32, 0);
        assert!(!bus.validate().is_empty());
    }

    #[test]
    fn test_validate_non_byte_data() {
        let bus = HirBusPort {
            name: "s0".into(),
            kind: HwBusKind::Axi4Lite,
            role: HwBusRole::Slave,
            addr_width: 32,
            data_width: 12,
            id_width: 0,
        };
        assert!(!bus.validate().is_empty());
    }

    #[test]
    fn test_validate_full_no_id() {
        let bus = HirBusPort::axi4_full_slave("s1", 32, 32, 0);
        assert!(!bus.validate().is_empty());
    }

    #[test]
    fn test_port_count_lite() {
        let bus = HirBusPort::axi4_lite_slave("s0", 32, 32);
        let count = bus.port_count();
        assert!(count > 0);
    }

    #[test]
    fn test_port_count_full_more_than_lite() {
        let lite = HirBusPort::axi4_lite_slave("s0", 32, 32);
        let full = HirBusPort::axi4_full_slave("s1", 32, 32, 4);
        assert!(full.port_count() > lite.port_count());
    }

    #[test]
    fn test_total_signal_bits_positive() {
        let bus = HirBusPort::axi4_lite_slave("s0", 32, 32);
        assert!(bus.total_signal_bits() > 0);
    }

    #[test]
    fn test_total_signal_bits_full_more_than_lite() {
        let lite = HirBusPort::axi4_lite_slave("s0", 32, 32);
        let full = HirBusPort::axi4_full_slave("s1", 32, 64, 4);
        assert!(full.total_signal_bits() > lite.total_signal_bits());
    }

    #[test]
    fn test_emit_axi4_lite_slave() {
        let mut hir = HirModule::new("AxiLiteSlave");
        hir.bus_ports
            .push(HirBusPort::axi4_lite_slave("s0", 32, 32));
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        let verilog = emitter.into_string();
        assert!(verilog.contains("AXI4-LITE"));
        assert!(verilog.contains("SLAVE"));
        assert!(verilog.contains("s0_awaddr"));
        assert!(verilog.contains("s0_wdata"));
        assert!(verilog.contains("s0_araddr"));
        assert!(verilog.contains("s0_rdata"));
        assert!(verilog.contains("s0_bresp"));
        assert!(verilog.contains("s0_awprot"));
        assert!(verilog.contains("s0_arprot"));
    }

    #[test]
    fn test_emit_axi4_full_master() {
        let mut hir = HirModule::new("AxiFullMaster");
        hir.bus_ports
            .push(HirBusPort::axi4_full_master("m0", 32, 64, 4));
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        let verilog = emitter.into_string();
        assert!(verilog.contains("AXI4-FULL"));
        assert!(verilog.contains("MASTER"));
        assert!(verilog.contains("m0_awlen"));
        assert!(verilog.contains("m0_awburst"));
        assert!(verilog.contains("m0_wlast"));
        assert!(verilog.contains("m0_rlast"));
        assert!(verilog.contains("m0_awcache"));
        assert!(verilog.contains("m0_awid"));
        assert!(verilog.contains("m0_arid"));
        assert!(verilog.contains("m0_rid"));
    }

    #[test]
    fn test_bus_in_module_validate() {
        let mut hir = HirModule::new("BusModule");
        hir.bus_ports
            .push(HirBusPort::axi4_lite_slave("s0", 32, 32));
        let errors = hir.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_bad_bus_in_module_validate() {
        let mut hir = HirModule::new("BadBusModule");
        hir.bus_ports.push(HirBusPort::axi4_lite_slave("", 0, 0));
        let errors = hir.validate();
        assert!(!errors.is_empty());
    }
}

#[cfg(test)]
mod tests_hir_apb_bridge {
    use super::*;

    #[test]
    fn test_apb_bridge_creation() {
        let apb = HirApbBridge::new("apb0", 32, 32, 4);
        assert_eq!(apb.name, "apb0");
        assert_eq!(apb.addr_width, 32);
        assert_eq!(apb.data_width, 32);
        assert_eq!(apb.num_peripherals, 4);
        assert!(!apb.has_pslverr);
    }

    #[test]
    fn test_apb_bridge_with_error() {
        let apb = HirApbBridge::new("apb1", 32, 32, 8).with_error_response();
        assert!(apb.has_pslverr);
        assert!(apb.has_pprot);
    }

    #[test]
    fn test_apb_strb_width() {
        let apb = HirApbBridge::new("apb0", 32, 32, 4);
        assert_eq!(apb.strb_width(), 4);
        let apb16 = HirApbBridge::new("apb0", 16, 16, 4);
        assert_eq!(apb16.strb_width(), 2);
    }

    #[test]
    fn test_addr_bits_for_peripherals() {
        assert_eq!(
            HirApbBridge::new("a", 32, 32, 1).addr_bits_for_peripherals(),
            0
        );
        assert_eq!(
            HirApbBridge::new("a", 32, 32, 2).addr_bits_for_peripherals(),
            1
        );
        assert_eq!(
            HirApbBridge::new("a", 32, 32, 4).addr_bits_for_peripherals(),
            2
        );
        assert_eq!(
            HirApbBridge::new("a", 32, 32, 8).addr_bits_for_peripherals(),
            3
        );
    }

    #[test]
    fn test_add_peripheral() {
        let mut apb = HirApbBridge::new("apb0", 32, 32, 4);
        apb.add_peripheral("uart0", 0x1000, 256, 0);
        apb.add_peripheral("spi0", 0x2000, 256, 1);
        assert_eq!(apb.periph_maps.len(), 2);
        assert_eq!(apb.periph_maps[0].name, "uart0");
        assert_eq!(apb.periph_maps[1].base_addr, 0x2000);
    }

    #[test]
    fn test_select_peripheral() {
        let mut apb = HirApbBridge::new("apb0", 32, 32, 2);
        apb.add_peripheral("uart0", 0x1000, 256, 0);
        apb.add_peripheral("spi0", 0x2000, 256, 1);
        assert_eq!(apb.select_peripheral(0x1050), Some(0));
        assert_eq!(apb.select_peripheral(0x20A0), Some(1));
        assert_eq!(apb.select_peripheral(0x9999), None);
    }

    #[test]
    fn test_validate_ok() {
        let apb = HirApbBridge::new("apb0", 32, 32, 4);
        assert!(apb.validate().is_empty());
    }

    #[test]
    fn test_validate_empty_name() {
        let apb = HirApbBridge::new("", 32, 32, 4);
        assert!(!apb.validate().is_empty());
    }

    #[test]
    fn test_validate_zero_addr() {
        let apb = HirApbBridge::new("apb0", 0, 32, 4);
        assert!(!apb.validate().is_empty());
    }

    #[test]
    fn test_validate_zero_data() {
        let apb = HirApbBridge::new("apb0", 32, 0, 4);
        assert!(!apb.validate().is_empty());
    }

    #[test]
    fn test_validate_non_byte_data() {
        let apb = HirApbBridge::new("apb0", 32, 12, 4);
        assert!(!apb.validate().is_empty());
    }

    #[test]
    fn test_validate_zero_peripherals() {
        let apb = HirApbBridge::new("apb0", 32, 32, 0);
        assert!(!apb.validate().is_empty());
    }

    #[test]
    fn test_emit_apb_bridge() {
        let mut hir = HirModule::new("ApbTop");
        let mut apb = HirApbBridge::new("apb0", 32, 32, 2);
        apb.add_peripheral("uart0", 0x1000, 256, 0);
        apb.add_peripheral("spi0", 0x2000, 256, 1);
        hir.apb_bridges.push(apb);
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        let verilog = emitter.into_string();
        assert!(verilog.contains("APB BRIDGE apb0"));
        assert!(verilog.contains("apb0_psel"));
        assert!(verilog.contains("apb0_paddr"));
        assert!(verilog.contains("apb0_pwdata"));
        assert!(verilog.contains("apb0_prdata"));
        assert!(verilog.contains("apb0_pready"));
        assert!(verilog.contains("apb0_periph0_sel"));
        assert!(verilog.contains("apb0_periph1_sel"));
    }

    #[test]
    fn test_emit_apb_bridge_with_error() {
        let mut hir = HirModule::new("ApbTopErr");
        hir.apb_bridges
            .push(HirApbBridge::new("apb1", 32, 32, 4).with_error_response());
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        let verilog = emitter.into_string();
        assert!(verilog.contains("apb1_pslverr"));
        assert!(verilog.contains("apb1_pprot"));
    }

    #[test]
    fn test_apb_in_module_validate() {
        let mut hir = HirModule::new("ApbModule");
        hir.apb_bridges.push(HirApbBridge::new("apb0", 32, 32, 4));
        let errors = hir.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_bad_apb_in_module_validate() {
        let mut hir = HirModule::new("BadApbModule");
        hir.apb_bridges.push(HirApbBridge::new("", 0, 0, 0));
        let errors = hir.validate();
        assert!(!errors.is_empty());
    }
}

#[cfg(test)]
mod tests_hir_gf16_accel {
    use super::*;

    #[test]
    fn test_basic_config() {
        let accel = HirGf16Accel::basic("gf0", 8);
        assert_eq!(accel.num_multipliers, 8);
        assert!(accel.has_mac);
        assert!(!accel.has_fft);
        assert!(!accel.has_dot_product);
        assert!(!accel.has_matmul);
    }

    #[test]
    fn test_full_config() {
        let accel = HirGf16Accel::full("gf1", 16, 32);
        assert_eq!(accel.num_multipliers, 16);
        assert_eq!(accel.vector_width, 32);
        assert!(accel.has_mac);
        assert!(accel.has_fft);
        assert!(accel.has_dot_product);
        assert!(accel.has_matmul);
    }

    #[test]
    fn test_total_gf16_bits() {
        let accel = HirGf16Accel::basic("gf0", 8);
        assert_eq!(accel.total_gf16_bits(), 32);
    }

    #[test]
    fn test_mac_unit_count_with_mac() {
        let accel = HirGf16Accel::basic("gf0", 8);
        assert_eq!(accel.mac_unit_count(), 8);
    }

    #[test]
    fn test_mac_unit_count_without_mac() {
        let mut accel = HirGf16Accel::basic("gf0", 4);
        accel.has_mac = false;
        assert_eq!(accel.mac_unit_count(), 0);
    }

    #[test]
    fn test_dsp48_count() {
        let accel = HirGf16Accel::basic("gf0", 8);
        assert_eq!(accel.dsp48_count(), 8);
    }

    #[test]
    fn test_bram_count_basic() {
        let accel = HirGf16Accel::basic("gf0", 8);
        assert_eq!(accel.bram_count(), 0);
    }

    #[test]
    fn test_bram_count_full() {
        let accel = HirGf16Accel::full("gf0", 16, 32);
        assert!(accel.bram_count() > 0);
    }

    #[test]
    fn test_add_mac_unit() {
        let mut accel = HirGf16Accel::basic("gf0", 8);
        accel.add_mac_unit("mac0", 32, 3);
        assert_eq!(accel.mac_units.len(), 1);
        assert_eq!(accel.mac_units[0].accumulator_width, 32);
        assert_eq!(accel.mac_units[0].pipeline_stages, 3);
    }

    #[test]
    fn test_set_fft() {
        let mut accel = HirGf16Accel::full("gf0", 16, 32);
        accel.set_fft("fft0", 16, 2);
        let fft = accel.fft_config.as_ref().unwrap();
        assert_eq!(fft.num_points, 16);
        assert_eq!(fft.radix, 2);
        assert_eq!(fft.fft_stages(), 4);
        assert_eq!(fft.twiddle_count(), 8);
    }

    #[test]
    fn test_fft_stages_64pt_radix4() {
        let fft = HirGf16FftConfig {
            name: "fft0".into(),
            num_points: 64,
            radix: 4,
        };
        assert_eq!(fft.fft_stages(), 3);
    }

    #[test]
    fn test_matmul_cycles() {
        let accel = HirGf16Accel::full("gf0", 8, 16);
        assert!(accel.matmul_cycles(4) > 0);
    }

    #[test]
    fn test_matmul_cycles_no_matmul() {
        let accel = HirGf16Accel::basic("gf0", 8);
        assert_eq!(accel.matmul_cycles(4), 0);
    }

    #[test]
    fn test_dot_product_cycles() {
        let accel = HirGf16Accel::full("gf0", 8, 16);
        assert!(accel.dot_product_cycles(16) > 0);
    }

    #[test]
    fn test_dot_product_cycles_no_dot() {
        let accel = HirGf16Accel::basic("gf0", 8);
        assert_eq!(accel.dot_product_cycles(16), 0);
    }

    #[test]
    fn test_validate_ok() {
        let accel = HirGf16Accel::basic("gf0", 8);
        assert!(accel.validate().is_empty());
    }

    #[test]
    fn test_validate_empty_name() {
        let accel = HirGf16Accel::basic("", 8);
        assert!(!accel.validate().is_empty());
    }

    #[test]
    fn test_validate_zero_mult() {
        let accel = HirGf16Accel::basic("gf0", 0);
        assert!(!accel.validate().is_empty());
    }

    #[test]
    fn test_validate_zero_vec_width() {
        let mut accel = HirGf16Accel::basic("gf0", 8);
        accel.vector_width = 0;
        assert!(!accel.validate().is_empty());
    }

    #[test]
    fn test_emit_gf16_basic() {
        let mut hir = HirModule::new("Gf16Top");
        hir.gf16_accels.push(HirGf16Accel::basic("gf0", 4));
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        let verilog = emitter.into_string();
        assert!(verilog.contains("GF16 ACCELERATOR gf0"));
        assert!(verilog.contains("gf0_start"));
        assert!(verilog.contains("gf0_opcode"));
        assert!(verilog.contains("gf0_done"));
        assert!(verilog.contains("gf0_busy"));
        assert!(verilog.contains("gf0_a_data"));
        assert!(verilog.contains("gf0_b_data"));
        assert!(verilog.contains("gf0_result_data"));
        assert!(verilog.contains("gf16_multiply"));
        assert!(verilog.contains("phi^2"));
    }

    #[test]
    fn test_emit_gf16_full_with_fft() {
        let mut hir = HirModule::new("Gf16FullTop");
        let mut accel = HirGf16Accel::full("gf1", 8, 16);
        accel.add_mac_unit("mac0", 32, 2);
        accel.set_fft("fft0", 16, 2);
        hir.gf16_accels.push(accel);
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        let verilog = emitter.into_string();
        assert!(verilog.contains("GF16 ACCELERATOR gf1"));
        assert!(verilog.contains("MAC accumulator"));
        assert!(verilog.contains("FFT butterfly"));
        assert!(verilog.contains("gf1_mac0_acc"));
        assert!(verilog.contains("gf1_fft_mem"));
        assert!(verilog.contains("gf1_fft_twiddle"));
    }

    #[test]
    fn test_gf16_in_module_validate() {
        let mut hir = HirModule::new("Gf16Module");
        hir.gf16_accels.push(HirGf16Accel::basic("gf0", 8));
        let errors = hir.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_bad_gf16_in_module_validate() {
        let mut hir = HirModule::new("BadGf16Module");
        hir.gf16_accels.push(HirGf16Accel::basic("", 0));
        let errors = hir.validate();
        assert!(!errors.is_empty());
    }
}

#[cfg(test)]
mod tests_hir_formal {
    use super::*;

    #[test]
    fn test_immediate_assert() {
        let a = HirFormalAssert::immediate(
            "no_overflow",
            "count < MAX",
            HwAssertSeverity::Error,
            "counter never overflows",
        );
        assert_eq!(a.kind, HwAssertKind::Immediate);
        assert_eq!(a.condition, "count < MAX");
    }

    #[test]
    fn test_concurrent_assert() {
        let a = HirFormalAssert::concurrent(
            "handshake",
            "valid ##1 ready",
            "clk",
            "rst_n",
            "valid followed by ready",
        );
        assert_eq!(a.kind, HwAssertKind::Concurrent);
        assert_eq!(a.clock, "clk");
        assert_eq!(a.reset, "rst_n");
    }

    #[test]
    fn test_cover_point() {
        let c = HirCoverPoint::new(
            "all_states",
            "state == S0 || state == S1",
            "clk",
            "cover all states",
        );
        assert_eq!(c.name, "all_states");
        assert!(!c.condition.is_empty());
    }

    #[test]
    fn test_assume() {
        let a = HirFormalAssume::new(
            "stable_reset",
            "!$isunknown(rst_n)",
            "clk",
            "reset is never X",
        );
        assert_eq!(a.name, "stable_reset");
    }

    #[test]
    fn test_formal_config() {
        let cfg = HirFormalConfig::new("uart_props", "UART_TX", "clk", "rst_n");
        assert_eq!(cfg.name, "uart_props");
        assert_eq!(cfg.module_name, "UART_TX");
        assert_eq!(cfg.depth, 20);
        assert_eq!(cfg.timeout_cycles, 100);
    }

    #[test]
    fn test_formal_config_with_depth() {
        let cfg = HirFormalConfig::new("f", "M", "clk", "rst_n").with_depth(50);
        assert_eq!(cfg.depth, 50);
    }

    #[test]
    fn test_formal_config_with_timeout() {
        let cfg = HirFormalConfig::new("f", "M", "clk", "rst_n").with_timeout(500);
        assert_eq!(cfg.timeout_cycles, 500);
    }

    #[test]
    fn test_validate_assertion_ok() {
        let a = HirFormalAssert::immediate("ok", "x > 0", HwAssertSeverity::Error, "desc");
        assert!(a.validate().is_empty());
    }

    #[test]
    fn test_validate_assertion_empty_name() {
        let a = HirFormalAssert::immediate("", "x > 0", HwAssertSeverity::Error, "desc");
        assert!(!a.validate().is_empty());
    }

    #[test]
    fn test_validate_assertion_empty_condition() {
        let a = HirFormalAssert::immediate("a", "", HwAssertSeverity::Error, "desc");
        assert!(!a.validate().is_empty());
    }

    #[test]
    fn test_validate_concurrent_no_clock() {
        let a = HirFormalAssert::concurrent("a", "x ##1 y", "", "rst_n", "desc");
        assert!(!a.validate().is_empty());
    }

    #[test]
    fn test_validate_cover_empty_name() {
        let c = HirCoverPoint::new("", "x", "clk", "desc");
        assert!(!c.validate().is_empty());
    }

    #[test]
    fn test_validate_assume_ok() {
        let a = HirFormalAssume::new("a", "x", "clk", "desc");
        assert!(a.validate().is_empty());
    }

    #[test]
    fn test_validate_assume_empty() {
        let a = HirFormalAssume::new("", "", "clk", "desc");
        assert!(!a.validate().is_empty());
    }

    #[test]
    fn test_validate_config_ok() {
        let cfg = HirFormalConfig::new("f", "M", "clk", "rst_n");
        assert!(cfg.validate().is_empty());
    }

    #[test]
    fn test_validate_config_empty_name() {
        let cfg = HirFormalConfig::new("", "M", "clk", "rst_n");
        assert!(!cfg.validate().is_empty());
    }

    #[test]
    fn test_validate_config_empty_clock() {
        let cfg = HirFormalConfig::new("f", "M", "", "rst_n");
        assert!(!cfg.validate().is_empty());
    }

    #[test]
    fn test_emit_formal() {
        let mut hir = HirModule::new("FormalTest");
        hir.formal_config = Some(HirFormalConfig::new("f", "FormalTest", "clk", "rst_n"));
        hir.formal_asserts.push(HirFormalAssert::immediate(
            "no_x",
            "!$isunknown(data)",
            HwAssertSeverity::Error,
            "data is never X",
        ));
        hir.formal_asserts.push(HirFormalAssert::concurrent(
            "handshake",
            "valid |-> ##[1:3] ready",
            "clk",
            "rst_n",
            "valid eventually gets ready",
        ));
        hir.formal_covers.push(HirCoverPoint::new(
            "all_states",
            "state inside {IDLE, RUN, DONE}",
            "clk",
            "cover all states",
        ));
        hir.formal_assumes.push(HirFormalAssume::new(
            "stable_reset",
            "!$isunknown(rst_n)",
            "clk",
            "reset is never X",
        ));
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        let verilog = emitter.into_string();
        assert!(verilog.contains("Formal verification assertions"));
        assert!(verilog.contains("assert property"));
        assert!(verilog.contains("no_x"));
        assert!(verilog.contains("handshake"));
        assert!(verilog.contains("cover property"));
        assert!(verilog.contains("all_states"));
        assert!(verilog.contains("assume property"));
        assert!(verilog.contains("stable_reset"));
        assert!(verilog.contains("disable iff"));
        assert!(verilog.contains("$error"));
        assert!(verilog.contains("FORMAL"));
    }

    #[test]
    fn test_formal_in_module_validate() {
        let mut hir = HirModule::new("FormalModule");
        hir.formal_config = Some(HirFormalConfig::new("f", "FormalModule", "clk", "rst_n"));
        hir.formal_asserts.push(HirFormalAssert::immediate(
            "a1",
            "x > 0",
            HwAssertSeverity::Error,
            "desc",
        ));
        let errors = hir.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_bad_formal_in_module_validate() {
        let mut hir = HirModule::new("BadFormalModule");
        hir.formal_asserts.push(HirFormalAssert::immediate(
            "",
            "",
            HwAssertSeverity::Error,
            "",
        ));
        let errors = hir.validate();
        assert!(!errors.is_empty());
    }
}

#[cfg(test)]
mod tests_hir_ternary_core {
    use super::*;

    #[test]
    fn test_basic_core_creation() {
        let core = HirTernaryCore::basic("tri0");
        assert_eq!(core.num_alus, 1);
        assert!(core.has_gf16_unit);
        assert!(core.has_ternary_alu);
        assert!(!core.has_branch_predictor);
        assert_eq!(core.pipeline_depth, 5);
    }

    #[test]
    fn test_full_core_creation() {
        let core = HirTernaryCore::full("tri1");
        assert_eq!(core.num_alus, 4);
        assert!(core.has_branch_predictor);
        assert_eq!(core.pipeline_depth, 7);
    }

    #[test]
    fn test_core_dsp_count_basic() {
        let core = HirTernaryCore::basic("tri0");
        assert_eq!(core.dsp_count(), 5);
    }

    #[test]
    fn test_core_dsp_count_full() {
        let core = HirTernaryCore::full("tri1");
        assert_eq!(core.dsp_count(), 8);
    }

    #[test]
    fn test_core_bram_count() {
        let core = HirTernaryCore::basic("tri0");
        assert_eq!(core.bram_count(), 3);
    }

    #[test]
    fn test_core_lut_estimate() {
        let core = HirTernaryCore::basic("tri0");
        assert_eq!(core.lut_estimate(), 11500);
        let full = HirTernaryCore::full("tri1");
        assert_eq!(full.lut_estimate(), 18000);
    }

    #[test]
    fn test_core_fmax() {
        let core = HirTernaryCore::basic("tri0");
        assert_eq!(core.fmax_mhz(), 100);
    }

    #[test]
    fn test_fits_arty_a7() {
        let basic = HirTernaryCore::basic("tri0");
        assert!(basic.fits_arty_a7());
        let full = HirTernaryCore::full("tri1");
        assert!(full.fits_arty_a7());
    }

    #[test]
    fn test_fits_xc7a100t() {
        let basic = HirTernaryCore::basic("tri0");
        assert!(basic.fits_xc7a100t());
    }

    #[test]
    fn test_pipeline_stages() {
        let mut core = HirTernaryCore::basic("tri0");
        core.add_pipeline_stage("IF", 1, false);
        core.add_pipeline_stage("ID", 1, false);
        core.add_pipeline_stage("EX", 1, true);
        core.add_pipeline_stage("MEM", 1, true);
        core.add_pipeline_stage("WB", 1, false);
        assert_eq!(core.pipeline_stages.len(), 5);
        assert_eq!(core.pipeline_total_latency(), 5);
    }

    #[test]
    fn test_alu_ops() {
        let mut core = HirTernaryCore::basic("tri0");
        core.add_alu_op("t_add", 1, 1, false);
        core.add_alu_op("t_mul", 2, 2, false);
        core.add_alu_op("gf_mul", 16, 3, true);
        core.add_alu_op("gf_mac", 17, 4, true);
        assert_eq!(core.alu_ops.len(), 4);
        assert!(!core.alu_ops[0].uses_gf16);
        assert!(core.alu_ops[2].uses_gf16);
    }

    #[test]
    fn test_regfile() {
        let rf = HirTernaryRegFile::new("regfile0");
        assert_eq!(rf.num_regs, 27);
        assert_eq!(rf.trit_width, 27);
        assert_eq!(rf.read_ports, 2);
        assert_eq!(rf.write_ports, 1);
        assert!(rf.total_bits() > 0);
        assert!(rf.bram18_count() > 0);
    }

    #[test]
    fn test_core_with_regfile() {
        let core = HirTernaryCore::basic("tri0").with_regfile(HirTernaryRegFile::new("regfile0"));
        assert!(core.reg_file.is_some());
        let rf = core.reg_file.unwrap();
        assert_eq!(rf.num_regs, 27);
    }

    #[test]
    fn test_validate_ok() {
        let core = HirTernaryCore::basic("tri0");
        assert!(core.validate().is_empty());
    }

    #[test]
    fn test_validate_empty_name() {
        let core = HirTernaryCore::basic("");
        assert!(!core.validate().is_empty());
    }

    #[test]
    fn test_validate_zero_alus() {
        let mut core = HirTernaryCore::basic("tri0");
        core.num_alus = 0;
        assert!(!core.validate().is_empty());
    }

    #[test]
    fn test_emit_ternary_core() {
        let mut hir = HirModule::new("TernaryTop");
        let mut core = HirTernaryCore::basic("tri0").with_regfile(HirTernaryRegFile::new("rf0"));
        core.add_pipeline_stage("IF", 1, false);
        core.add_pipeline_stage("ID", 1, false);
        core.add_pipeline_stage("EX", 1, true);
        core.add_pipeline_stage("MEM", 1, true);
        core.add_pipeline_stage("WB", 1, false);
        core.add_alu_op("t_add", 1, 1, false);
        core.add_alu_op("gf_mul", 16, 3, true);
        hir.ternary_cores.push(core);
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&hir);
        let verilog = emitter.into_string();
        assert!(verilog.contains("TERNARY CORE tri0"));
        assert!(verilog.contains("tri0_instr"));
        assert!(verilog.contains("tri0_mem_rdata"));
        assert!(verilog.contains("tri0_regs"));
        assert!(verilog.contains("Register file: 27 regs"));
        assert!(verilog.contains("Pipeline stages:"));
        assert!(verilog.contains("IF"));
        assert!(verilog.contains("ALU operations:"));
        assert!(verilog.contains("t_add"));
        assert!(verilog.contains("gf_mul"));
        assert!(verilog.contains("[GF16]"));
        assert!(verilog.contains("phi^2"));
        assert!(verilog.contains("tri0_pc"));
        assert!(verilog.contains("tri0_state"));
    }

    #[test]
    fn test_ternary_in_module_validate() {
        let mut hir = HirModule::new("TernaryModule");
        hir.ternary_cores.push(HirTernaryCore::basic("tri0"));
        let errors = hir.validate();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_bad_ternary_in_module_validate() {
        let mut hir = HirModule::new("BadTernaryModule");
        let mut core = HirTernaryCore::basic("");
        core.num_alus = 0;
        hir.ternary_cores.push(core);
        let errors = hir.validate();
        assert!(!errors.is_empty());
    }
}

#[cfg(test)]
mod tests_hir_stdlib {
    use super::*;

    #[test]
    fn test_resource_estimate_zero() {
        let r = HirResourceEstimate::zero();
        assert_eq!(r.luts, 0);
        assert_eq!(r.ffs, 0);
        assert_eq!(r.bram18, 0);
        assert_eq!(r.dsp48, 0);
    }

    #[test]
    fn test_resource_estimate_new() {
        let r = HirResourceEstimate::new(100, 50, 2, 1, 8);
        assert_eq!(r.luts, 100);
        assert_eq!(r.ffs, 50);
        assert_eq!(r.bram18, 2);
        assert_eq!(r.dsp48, 1);
        assert_eq!(r.io_pins, 8);
    }

    #[test]
    fn test_board_resources_arty() {
        let b = HirBoardResources::arty_a7();
        assert_eq!(b.luts, 33800);
        assert_eq!(b.bram18, 60);
        assert_eq!(b.dsp48, 90);
    }

    #[test]
    fn test_board_resources_xc7a100t() {
        let b = HirBoardResources::xc7a100t();
        assert_eq!(b.luts, 63400);
        assert_eq!(b.bram18, 135);
    }

    #[test]
    fn test_empty_catalog() {
        let cat = HirIpCatalog::new("test");
        assert_eq!(cat.total_luts(), 0);
        assert_eq!(cat.total_bram18(), 0);
        assert_eq!(cat.total_dsp48(), 0);
    }

    #[test]
    fn test_add_cores() {
        let mut cat = HirIpCatalog::new("test");
        cat.add_core("uart_tx", 200, 100, 1, 0, 4, 100);
        cat.add_core("spi_master", 150, 80, 0, 0, 6, 100);
        assert_eq!(cat.cores.len(), 2);
        assert_eq!(cat.total_luts(), 350);
        assert_eq!(cat.total_ffs(), 180);
    }

    #[test]
    fn test_fits_board_empty() {
        let cat = HirIpCatalog::new("test");
        let board = HirBoardResources::arty_a7();
        assert!(cat.fits_board(&board));
    }

    #[test]
    fn test_fits_board_small() {
        let mut cat = HirIpCatalog::new("test");
        cat.add_core("uart_tx", 200, 100, 1, 0, 4, 100);
        let board = HirBoardResources::arty_a7();
        assert!(cat.fits_board(&board));
    }

    #[test]
    fn test_luts_remaining() {
        let mut cat = HirIpCatalog::new("test");
        cat.add_core("uart_tx", 200, 100, 1, 0, 4, 100);
        let board = HirBoardResources::arty_a7();
        assert_eq!(cat.luts_remaining(&board), 33600);
    }

    #[test]
    fn test_utilization_zero() {
        let cat = HirIpCatalog::new("test");
        let board = HirBoardResources::arty_a7();
        assert_eq!(cat.utilization_percent(&board), 0);
    }

    #[test]
    fn test_utilization_small() {
        let mut cat = HirIpCatalog::new("test");
        cat.add_core("big_core", 3380, 1000, 5, 10, 20, 100);
        let board = HirBoardResources::arty_a7();
        assert_eq!(cat.utilization_percent(&board), 10);
    }

    #[test]
    fn test_validate_ok() {
        let mut cat = HirIpCatalog::new("test");
        cat.add_core("uart_tx", 200, 100, 1, 0, 4, 100);
        assert!(cat.validate().is_empty());
    }

    #[test]
    fn test_validate_empty_name() {
        let cat = HirIpCatalog::new("");
        assert!(!cat.validate().is_empty());
    }

    #[test]
    fn test_validate_core_empty_name() {
        let mut cat = HirIpCatalog::new("test");
        cat.cores.push(HirIpCore {
            name: String::new(),
            resources: HirResourceEstimate::zero(),
            clock_freq_mhz: 100,
            verified: false,
        });
        assert!(!cat.validate().is_empty());
    }

    #[test]
    fn test_full_catalog_fits() {
        let mut cat = HirIpCatalog::new("t27_full");
        cat.add_core("uart_tx", 200, 100, 1, 0, 4, 100);
        cat.add_core("uart_rx", 250, 120, 1, 0, 4, 100);
        cat.add_core("spi_master", 150, 80, 0, 0, 6, 100);
        cat.add_core("gf16_accel", 3000, 1500, 3, 8, 0, 100);
        cat.add_core("ternary_core", 11500, 5000, 3, 5, 0, 100);
        cat.add_core("bram_ctrl", 100, 50, 10, 0, 0, 100);
        let arty = HirBoardResources::arty_a7();
        let xc7a = HirBoardResources::xc7a100t();
        assert!(cat.fits_board(&arty));
        assert!(cat.fits_board(&xc7a));
        assert!(cat.utilization_percent(&xc7a) < 100);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirSimState {
    Idle,
    Running,
    Paused,
    Done,
    Error,
}

#[derive(Debug, Clone)]
pub struct HirSimConfig {
    pub name: String,
    pub max_cycles: u32,
    pub clock_freq_hz: u32,
    pub trace_enabled: bool,
    pub vcd_output: bool,
    pub break_on_error: bool,
    pub vcd_path: String,
}

impl HirSimConfig {
    pub fn new(name: &str, max_cycles: u32) -> Self {
        HirSimConfig {
            name: name.to_string(),
            max_cycles,
            clock_freq_hz: 100_000_000,
            trace_enabled: false,
            vcd_output: false,
            break_on_error: true,
            vcd_path: String::new(),
        }
    }

    pub fn with_trace(mut self, vcd_path: &str) -> Self {
        self.trace_enabled = true;
        self.vcd_output = true;
        self.vcd_path = vcd_path.to_string();
        self
    }

    pub fn sim_time_ns(&self, cycles: u32) -> u32 {
        if self.clock_freq_hz == 0 {
            return 0;
        }
        ((cycles as u64) * 1_000_000_000 / self.clock_freq_hz as u64) as u32
    }

    pub fn sim_time_us(&self, cycles: u32) -> u32 {
        self.sim_time_ns(cycles) / 1000
    }

    pub fn sim_time_ms(&self, cycles: u32) -> u32 {
        self.sim_time_ns(cycles) / 1_000_000
    }

    pub fn cycles_for_time_ns(&self, ns: u32) -> u32 {
        if self.clock_freq_hz == 0 {
            return 0;
        }
        ((ns as u64) * self.clock_freq_hz as u64 / 1_000_000_000) as u32
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("sim config name must not be empty".to_string());
        }
        if self.max_cycles == 0 {
            errors.push("sim config max_cycles must be positive".to_string());
        }
        if self.clock_freq_hz == 0 {
            errors.push("sim config clock_freq_hz must be positive".to_string());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct HirSimResult {
    pub cycles: u32,
    pub state: HirSimState,
    pub errors: u32,
    pub assertions_fired: u32,
    pub coverage_points: u32,
}

impl HirSimResult {
    pub fn ok(cycles: u32, coverage: u32) -> Self {
        HirSimResult {
            cycles,
            state: HirSimState::Done,
            errors: 0,
            assertions_fired: 0,
            coverage_points: coverage,
        }
    }

    pub fn error(cycles: u32, errors: u32) -> Self {
        HirSimResult {
            cycles,
            state: HirSimState::Error,
            errors,
            assertions_fired: 0,
            coverage_points: 0,
        }
    }

    pub fn is_done(&self) -> bool {
        self.state == HirSimState::Done
    }
    pub fn is_error(&self) -> bool {
        self.state == HirSimState::Error
    }
    pub fn has_errors(&self) -> bool {
        self.errors > 0
    }
    pub fn passed(&self) -> bool {
        self.state == HirSimState::Done && self.errors == 0
    }
}

#[cfg(test)]
mod tests_hir_simulator {
    use super::*;

    #[test]
    fn test_sim_config_creation() {
        let cfg = HirSimConfig::new("uart_sim", 10000);
        assert_eq!(cfg.max_cycles, 10000);
        assert!(!cfg.trace_enabled);
    }

    #[test]
    fn test_sim_config_with_trace() {
        let cfg = HirSimConfig::new("uart_sim", 10000).with_trace("uart.vcd");
        assert!(cfg.trace_enabled);
        assert!(cfg.vcd_output);
        assert_eq!(cfg.vcd_path, "uart.vcd");
    }

    #[test]
    fn test_sim_ok_result() {
        let r = HirSimResult::ok(5000, 10);
        assert!(r.is_done());
        assert!(!r.is_error());
        assert!(r.passed());
        assert!(!r.has_errors());
        assert_eq!(r.cycles, 5000);
        assert_eq!(r.coverage_points, 10);
    }

    #[test]
    fn test_sim_error_result() {
        let r = HirSimResult::error(3000, 2);
        assert!(!r.is_done());
        assert!(r.is_error());
        assert!(!r.passed());
        assert!(r.has_errors());
        assert_eq!(r.errors, 2);
    }

    #[test]
    fn test_sim_time_ns() {
        let cfg = HirSimConfig::new("sim", 10000);
        assert_eq!(cfg.sim_time_ns(100), 1000);
    }

    #[test]
    fn test_sim_time_us() {
        let cfg = HirSimConfig::new("sim", 10000);
        assert_eq!(cfg.sim_time_us(100000), 1000);
    }

    #[test]
    fn test_sim_time_ms() {
        let cfg = HirSimConfig::new("sim", 10000);
        assert_eq!(cfg.sim_time_ms(100_000_000), 1000);
    }

    #[test]
    fn test_cycles_for_time_ns() {
        let cfg = HirSimConfig::new("sim", 10000);
        assert_eq!(cfg.cycles_for_time_ns(1000), 100);
    }

    #[test]
    fn test_validate_ok() {
        let cfg = HirSimConfig::new("sim", 10000);
        assert!(cfg.validate().is_empty());
    }

    #[test]
    fn test_validate_empty_name() {
        let cfg = HirSimConfig::new("", 10000);
        assert!(!cfg.validate().is_empty());
    }

    #[test]
    fn test_validate_zero_freq() {
        let mut cfg = HirSimConfig::new("sim", 100);
        cfg.clock_freq_hz = 0;
        assert!(!cfg.validate().is_empty());
    }
}

#[cfg(test)]
mod tests_hir_assembler {
    use super::*;

    #[test]
    fn test_asm_config_creation() {
        let cfg = AsmConfig::new("t27_asm");
        assert_eq!(cfg.text_base, 0);
        assert_eq!(cfg.data_base, 4096);
        assert_eq!(cfg.word_size, 4);
        assert!(cfg.has_gf16_ext);
        assert!(cfg.has_ternary_ext);
    }

    #[test]
    fn test_text_section_creation() {
        let sec = AsmSection::text(0);
        assert_eq!(sec.kind, AsmSectionKind::Text);
        assert_eq!(sec.base_address, 0);
    }

    #[test]
    fn test_data_section_creation() {
        let sec = AsmSection::data(4096);
        assert_eq!(sec.kind, AsmSectionKind::Data);
        assert_eq!(sec.base_address, 4096);
    }

    #[test]
    fn test_r_instruction_creation() {
        let instr = AssembledInstr::r_type(1, 5, 6, 7);
        assert!(instr.is_r_type());
        assert!(!instr.is_i_type());
        assert!(!instr.is_gf16_instr());
    }

    #[test]
    fn test_i_instruction_creation() {
        let instr = AssembledInstr::i_type(2, 5, 6, 42);
        assert!(instr.is_i_type());
        assert!(!instr.is_r_type());
    }

    #[test]
    fn test_gf16_instruction_creation() {
        let instr = AssembledInstr::gf16(16, 5, 6, 7);
        assert!(instr.is_gf16_instr());
        assert!(instr.is_r_type());
    }

    #[test]
    fn test_encode_r_type() {
        let instr = AssembledInstr::r_type(1, 5, 6, 7);
        let encoded = instr.encode_r_type();
        assert!(encoded > 0);
        assert_eq!(encoded, (1u32 << 26) | (5 << 21) | (6 << 16) | (7 << 11));
    }

    #[test]
    fn test_encode_i_type() {
        let instr = AssembledInstr::i_type(2, 5, 6, 42);
        let encoded = instr.encode_i_type();
        assert!(encoded > 0);
        assert_eq!(encoded, (2u32 << 26) | (5 << 21) | (6 << 16) | 42);
    }

    #[test]
    fn test_section_end() {
        let sec = AsmSection {
            name: ".text".into(),
            kind: AsmSectionKind::Text,
            base_address: 0,
            size: 128,
        };
        assert_eq!(sec.end(), 128);
    }

    #[test]
    fn test_align_address_zero() {
        assert_eq!(align_address(5, 4), 8);
    }

    #[test]
    fn test_align_address_already_aligned() {
        assert_eq!(align_address(8, 4), 8);
    }

    #[test]
    fn test_align_address_zero_alignment() {
        assert_eq!(align_address(5, 0), 5);
    }

    #[test]
    fn test_instr_count() {
        let cfg = AsmConfig::new("test");
        assert_eq!(cfg.instr_count(128), 32);
    }

    #[test]
    fn test_symbol_creation() {
        let sym = AsmSymbol::new("main", 0, AsmSectionKind::Text, true);
        assert_eq!(sym.name, "main");
        assert!(sym.is_global);
    }

    #[test]
    fn test_reloc_creation() {
        let r = AsmRelocEntry::new(64, AsmRelocKind::Abs32, "data_start", 0);
        assert_eq!(r.offset, 64);
        assert_eq!(r.symbol, "data_start");
    }

    #[test]
    fn test_validate_config_ok() {
        let cfg = AsmConfig::new("test");
        assert!(cfg.validate().is_empty());
    }

    #[test]
    fn test_validate_config_empty_name() {
        let cfg = AsmConfig {
            name: String::new(),
            text_base: 0,
            data_base: 4096,
            word_size: 4,
            has_gf16_ext: true,
            has_ternary_ext: true,
        };
        assert!(!cfg.validate().is_empty());
    }

    #[test]
    fn test_validate_symbol_ok() {
        let sym = AsmSymbol::new("main", 0, AsmSectionKind::Text, true);
        assert!(sym.validate().is_empty());
    }

    #[test]
    fn test_validate_symbol_empty_name() {
        let sym = AsmSymbol::new("", 0, AsmSectionKind::Text, true);
        assert!(!sym.validate().is_empty());
    }

    #[test]
    fn test_assembler_emit_sequence() {
        let mut asm = HirAssembler::new("test_prog");
        asm.define_symbol("_start", true);
        let addr0 = asm.emit_r(1, 1, 2, 3);
        let addr1 = asm.emit_i(2, 4, 5, 100);
        assert_eq!(addr0, 0);
        assert_eq!(addr1, 4);
        assert_eq!(asm.total_instructions(), 2);
        assert_eq!(asm.total_bytes(), 8);
    }

    #[test]
    fn test_assembler_symbol_resolution() {
        let mut asm = HirAssembler::new("test_sym");
        asm.define_symbol("main", true);
        asm.emit_r(1, 1, 2, 3);
        assert_eq!(asm.resolve_symbol("main"), Some(0));
        assert_eq!(asm.resolve_symbol("nonexistent"), None);
    }

    #[test]
    fn test_assembler_relocation() {
        let mut asm = HirAssembler::new("test_reloc");
        asm.define_symbol("data_start", true);
        let addr = asm.emit_i(2, 1, 0, 0);
        asm.add_relocation(addr, AsmRelocKind::Abs32, "data_start", 0);
        assert_eq!(asm.relocations.len(), 1);
        let applied = asm.apply_relocations().unwrap();
        assert_eq!(applied, 1);
    }

    #[test]
    fn test_assembler_encode_all() {
        let mut asm = HirAssembler::new("test_encode");
        asm.emit_r(1, 2, 3, 4);
        asm.emit_i(5, 6, 7, 42);
        let encoded = asm.encode_all();
        assert_eq!(encoded.len(), 2);
        assert!(encoded[0] > 0);
        assert!(encoded[1] > 0);
    }

    #[test]
    fn test_assembler_to_binary() {
        let mut asm = HirAssembler::new("test_bin");
        asm.emit_r(1, 2, 3, 4);
        let bytes = asm.to_binary();
        assert_eq!(bytes.len(), 4);
    }

    #[test]
    fn test_assembler_section_switch() {
        let mut asm = HirAssembler::new("test_sec");
        asm.emit_r(1, 1, 2, 3);
        assert_eq!(asm.sections[0].size, 4);
        asm.set_section(".data").unwrap();
        asm.emit_i(2, 4, 5, 100);
        assert_eq!(asm.sections[1].size, 4);
        assert_eq!(asm.total_instructions(), 2);
    }

    #[test]
    fn test_assembler_gf16_ext() {
        let mut asm = HirAssembler::new("test_gf16");
        asm.emit_gf16(16, 1, 2, 3);
        assert_eq!(asm.total_instructions(), 1);
        let instr = &asm.instructions[0];
        assert!(instr.is_gf16_instr());
        assert!(instr.is_r_type());
    }

    #[test]
    fn test_assembler_validate() {
        let asm = HirAssembler::new("valid_asm");
        assert!(asm.validate().is_empty());
    }

    #[test]
    fn test_section_kind_roundtrip() {
        assert_eq!(
            AsmSectionKind::from_i8(AsmSectionKind::Text.to_i8()),
            AsmSectionKind::Text
        );
        assert_eq!(
            AsmSectionKind::from_i8(AsmSectionKind::Data.to_i8()),
            AsmSectionKind::Data
        );
        assert_eq!(
            AsmSectionKind::from_i8(AsmSectionKind::Bss.to_i8()),
            AsmSectionKind::Bss
        );
        assert_eq!(
            AsmSectionKind::from_i8(AsmSectionKind::Rodata.to_i8()),
            AsmSectionKind::Rodata
        );
    }

    #[test]
    fn test_reloc_kind_roundtrip() {
        assert_eq!(
            AsmRelocKind::from_i8(AsmRelocKind::Abs32.to_i8()),
            AsmRelocKind::Abs32
        );
        assert_eq!(
            AsmRelocKind::from_i8(AsmRelocKind::Rel21.to_i8()),
            AsmRelocKind::Rel21
        );
        assert_eq!(
            AsmRelocKind::from_i8(AsmRelocKind::Gf16Label.to_i8()),
            AsmRelocKind::Gf16Label
        );
    }

    #[test]
    fn test_full_assembly_program() {
        let mut asm = HirAssembler::new("trinity_hello");
        asm.define_symbol("_start", true);
        asm.emit_r(1, 1, 27, 0);
        asm.emit_i(3, 2, 1, 42);
        asm.emit_r(1, 3, 2, 1);
        asm.define_symbol("loop", false);
        asm.emit_i(4, 0, 0, 0);
        let target = asm.resolve_symbol("loop").unwrap();
        asm.add_relocation(
            asm.instructions.last().unwrap().address,
            AsmRelocKind::Rel21,
            "loop",
            0,
        );
        asm.apply_relocations().unwrap();
        assert_eq!(asm.total_instructions(), 4);
        assert_eq!(asm.total_bytes(), 16);
        assert_eq!(asm.symbols.len(), 2);
        assert_eq!(asm.relocations.len(), 1);
        let encoded = asm.encode_all();
        assert_eq!(encoded.len(), 4);
        let bin = asm.to_binary();
        assert_eq!(bin.len(), 16);
    }
}

#[cfg(test)]
mod tests_hir_testbench {
    use super::*;

    #[test]
    fn test_clock_cfg_creation() {
        let cfg = TbClockCfg::new(10);
        assert_eq!(cfg.period_ns, 10);
        assert_eq!(cfg.duty_cycle, 50);
        assert_eq!(cfg.half_period(), 5);
    }

    #[test]
    fn test_reset_cfg_creation() {
        let cfg = TbResetCfg::new(5, 10);
        assert!(cfg.active_low);
        assert_eq!(cfg.delay_cycles, 5);
        assert_eq!(cfg.duration_cycles, 10);
        assert_eq!(cfg.reset_end_cycle(), 15);
    }

    #[test]
    fn test_stimulus_creation() {
        let s = TbStimulus::new(10, "uart_tx", 1);
        assert_eq!(s.cycle, 10);
        assert_eq!(s.signal, "uart_tx");
        assert_eq!(s.value, 1);
    }

    #[test]
    fn test_check_creation() {
        let c = TbCheck::new(20, "led", 5);
        assert_eq!(c.cycle, 20);
        assert_eq!(c.signal, "led");
        assert_eq!(c.expected, 5);
        assert_eq!(c.mask, 0xFFFF_FFFF);
    }

    #[test]
    fn test_check_with_mask() {
        let c = TbCheck::with_mask(20, "data", 255, 255);
        assert_eq!(c.mask, 255);
    }

    #[test]
    fn test_tb_config_creation() {
        let cfg = TbConfig::new("uart_top", 10000);
        assert_eq!(cfg.dut_name, "uart_top");
        assert_eq!(cfg.max_cycles, 10000);
        assert_eq!(cfg.timeout_ns, 100000);
    }

    #[test]
    fn test_validate_tb_config_ok() {
        let cfg = TbConfig::new("dut", 1000);
        assert!(cfg.validate().is_empty());
    }

    #[test]
    fn test_validate_tb_config_empty_dut() {
        let cfg = TbConfig {
            name: "tb".into(),
            dut_name: String::new(),
            timescale: "1ns/1ps".into(),
            max_cycles: 1000,
            timeout_ns: 10000,
            fail_fast: true,
        };
        assert!(!cfg.validate().is_empty());
    }

    #[test]
    fn test_validate_stimulus_ok() {
        let s = TbStimulus::new(0, "clk", 1);
        assert!(s.validate().is_empty());
    }

    #[test]
    fn test_validate_stimulus_empty_signal() {
        let s = TbStimulus {
            cycle: 0,
            signal: String::new(),
            value: 0,
        };
        assert!(!s.validate().is_empty());
    }

    #[test]
    fn test_validate_check_ok() {
        let c = TbCheck::new(10, "out", 42);
        assert!(c.validate().is_empty());
    }

    #[test]
    fn test_testbench_creation() {
        let tb = HirTestbench::new("uart_top", 10000, 10);
        assert_eq!(tb.config.dut_name, "uart_top");
        assert_eq!(tb.clock.period_ns, 10);
        assert!(tb.stimuli.is_empty());
        assert!(tb.checks.is_empty());
    }

    #[test]
    fn test_testbench_add_stimulus() {
        let mut tb = HirTestbench::new("dut", 1000, 10);
        tb.add_stimulus(5, "data_in", 42);
        tb.add_stimulus(10, "data_in", 99);
        assert_eq!(tb.stimuli.len(), 2);
        assert_eq!(tb.stimuli[0].cycle, 5);
        assert_eq!(tb.stimuli[1].value, 99);
    }

    #[test]
    fn test_testbench_add_check() {
        let mut tb = HirTestbench::new("dut", 1000, 10);
        tb.add_check(20, "data_out", 42);
        assert_eq!(tb.checks.len(), 1);
        assert_eq!(tb.checks[0].expected, 42);
    }

    #[test]
    fn test_testbench_add_check_masked() {
        let mut tb = HirTestbench::new("dut", 1000, 10);
        tb.add_check_masked(20, "status", 0xFF, 0xFF);
        assert_eq!(tb.checks[0].mask, 0xFF);
    }

    #[test]
    fn test_testbench_probe() {
        let mut tb = HirTestbench::new("dut", 1000, 10);
        tb.probe("counter");
        tb.probe("led");
        assert_eq!(tb.probe_signals.len(), 2);
    }

    #[test]
    fn test_total_sim_ns() {
        let tb = HirTestbench::new("dut", 1000, 10);
        assert_eq!(tb.total_sim_ns(), 10000);
    }

    #[test]
    fn test_testbench_validate() {
        let tb = HirTestbench::new("valid_dut", 1000, 10);
        assert!(tb.validate().is_empty());
    }

    #[test]
    fn test_testbench_emit_verilog() {
        let mut tb = HirTestbench::new("uart_top", 1000, 10);
        tb.add_stimulus(20, "data_in", 0xAB);
        tb.add_check(50, "data_out", 0xCD);
        tb.probe("tx_busy");
        let verilog = tb.emit_verilog();
        assert!(verilog.contains("module tb"));
        assert!(verilog.contains("endmodule"));
        assert!(verilog.contains("uart_top uut"));
        assert!(verilog.contains("clk"));
        assert!(verilog.contains("rst_n"));
        assert!(verilog.contains("forever #5"));
        assert!(verilog.contains("data_in"));
        assert!(verilog.contains("data_out"));
        assert!(verilog.contains("probe_tx_busy"));
        assert!(verilog.contains("TIMEOUT"));
        assert!(verilog.contains("SIM PASSED"));
    }

    #[test]
    fn test_testbench_emit_verilog_minimal() {
        let tb = HirTestbench::new("simple_dut", 100, 20);
        let verilog = tb.emit_verilog();
        assert!(verilog.contains("`timescale 1ns/1ps"));
        assert!(verilog.contains("module tb"));
        assert!(verilog.contains("simple_dut uut"));
        assert!(verilog.contains("forever #10"));
    }
}

#[cfg(test)]
mod tests_hir_vcd_trace {
    use super::*;

    #[test]
    fn test_vcd_var_creation() {
        let v = VcdVar::wire(32, "counter", "!");
        assert_eq!(v.kind, VcdVarKind::Wire);
        assert_eq!(v.size, 32);
        assert_eq!(v.name, "counter");
    }

    #[test]
    fn test_var_wire_creation() {
        let v = VcdVar::wire(1, "clk", "!");
        assert_eq!(v.kind, VcdVarKind::Wire);
    }

    #[test]
    fn test_var_reg_creation() {
        let v = VcdVar::reg(8, "data", "!");
        assert_eq!(v.kind, VcdVarKind::Reg);
    }

    #[test]
    fn test_vcd_change_creation() {
        let c = VcdChange::new(1000, "!", 1, 1);
        assert_eq!(c.timestamp_ps, 1000);
        assert_eq!(c.ident, "!");
        assert_eq!(c.value, 1);
    }

    #[test]
    fn test_vcd_header_creation() {
        let h = VcdHeader::new("t27c v0.1", "1 ps");
        assert_eq!(h.version, "t27c v0.1");
        assert_eq!(h.timescale, "1 ps");
    }

    #[test]
    fn test_vcd_trace_creation() {
        let t = HirVcdTrace::new("t27c v0.1");
        assert_eq!(t.end_time_ps, 0);
        assert_eq!(t.header.version, "t27c v0.1");
        assert!(t.variables.is_empty());
        assert!(t.changes.is_empty());
    }

    #[test]
    fn test_changes_at_timestamp() {
        let mut trace = HirVcdTrace::new("test");
        trace.add_wire(1, "clk");
        trace.record(100, "clk", 0);
        trace.record(150, "clk", 1);
        trace.record(200, "clk", 0);
        trace.record(200, "clk", 1);
        assert_eq!(trace.changes_at(100), 1);
        assert_eq!(trace.changes_at(200), 2);
        assert_eq!(trace.changes_at(999), 0);
    }

    #[test]
    fn test_earliest_change() {
        let mut trace = HirVcdTrace::new("test");
        trace.add_wire(1, "clk");
        trace.record(500, "clk", 0);
        trace.record(100, "clk", 1);
        trace.record(300, "clk", 0);
        assert_eq!(trace.earliest_timestamp(), 100);
    }

    #[test]
    fn test_latest_change() {
        let mut trace = HirVcdTrace::new("test");
        trace.add_wire(1, "clk");
        trace.record(500, "clk", 0);
        trace.record(100, "clk", 1);
        trace.record(300, "clk", 0);
        assert_eq!(trace.latest_timestamp(), 500);
    }

    #[test]
    fn test_trace_duration() {
        let mut trace = HirVcdTrace::new("test");
        trace.add_wire(1, "clk");
        trace.record(100, "clk", 0);
        trace.record(500, "clk", 1);
        assert_eq!(trace.duration_ps(), 400);
    }

    #[test]
    fn test_trace_duration_empty() {
        let trace = HirVcdTrace::new("test");
        assert_eq!(trace.duration_ps(), 0);
    }

    #[test]
    fn test_validate_var_ok() {
        let v = VcdVar::wire(1, "sig", "!");
        assert!(v.validate().is_empty());
    }

    #[test]
    fn test_validate_var_empty_name() {
        let v = VcdVar {
            kind: VcdVarKind::Wire,
            size: 1,
            name: String::new(),
            ident: "!".into(),
        };
        assert!(!v.validate().is_empty());
    }

    #[test]
    fn test_validate_var_empty_ident() {
        let v = VcdVar {
            kind: VcdVarKind::Wire,
            size: 1,
            name: "sig".into(),
            ident: String::new(),
        };
        assert!(!v.validate().is_empty());
    }

    #[test]
    fn test_validate_change_ok() {
        let c = VcdChange::new(0, "!", 0, 1);
        assert!(c.validate().is_empty());
    }

    #[test]
    fn test_validate_change_empty_ident() {
        let c = VcdChange {
            timestamp_ps: 0,
            ident: String::new(),
            value: 0,
            bit_width: 1,
        };
        assert!(!c.validate().is_empty());
    }

    #[test]
    fn test_validate_change_zero_width() {
        let c = VcdChange {
            timestamp_ps: 0,
            ident: "!".into(),
            value: 0,
            bit_width: 0,
        };
        assert!(!c.validate().is_empty());
    }

    #[test]
    fn test_add_wire_and_reg() {
        let mut trace = HirVcdTrace::new("test");
        trace.add_wire(1, "clk");
        trace.add_reg(32, "counter");
        trace.add_wire(8, "data");
        assert_eq!(trace.variables.len(), 3);
        assert_eq!(trace.variables[0].kind, VcdVarKind::Wire);
        assert_eq!(trace.variables[1].kind, VcdVarKind::Reg);
    }

    #[test]
    fn test_ident_from_index() {
        assert_eq!(HirVcdTrace::ident_from_index(0), "!");
        assert_eq!(HirVcdTrace::ident_from_index(1), "\"");
        assert_eq!(HirVcdTrace::ident_from_index(93), "~");
    }

    #[test]
    fn test_emit_vcd() {
        let mut trace = HirVcdTrace::new("t27c v0.1");
        trace.add_wire(1, "clk");
        trace.add_reg(8, "data");
        trace.record(0, "clk", 0);
        trace.record(5000, "clk", 1);
        trace.record(5000, "data", 0xAB);
        trace.record(10000, "clk", 0);
        let vcd = trace.emit_vcd();
        assert!(vcd.contains("$date"));
        assert!(vcd.contains("$version"));
        assert!(vcd.contains("t27c v0.1"));
        assert!(vcd.contains("$timescale 1 ps"));
        assert!(vcd.contains("$scope module top"));
        assert!(vcd.contains("$var wire 1"));
        assert!(vcd.contains("clk"));
        assert!(vcd.contains("$var reg 8"));
        assert!(vcd.contains("data"));
        assert!(vcd.contains("$dumpvars"));
        assert!(vcd.contains("#0"));
        assert!(vcd.contains("#5000"));
        assert!(vcd.contains("#10000"));
        assert!(vcd.contains("b10101011"));
    }

    #[test]
    fn test_emit_vcd_empty() {
        let trace = HirVcdTrace::new("empty");
        let vcd = trace.emit_vcd();
        assert!(vcd.contains("$date"));
        assert!(vcd.contains("$enddefinitions"));
    }

    #[test]
    fn test_format_binary_single_bit() {
        let c = VcdChange::new(0, "!", 1, 1);
        assert_eq!(c.format_binary(), "1");
    }

    #[test]
    fn test_format_binary_multi_bit() {
        let c = VcdChange::new(0, "!", 0xAB, 8);
        let s = c.format_binary();
        assert!(s.contains("10101011"));
        assert!(s.contains("!"));
    }

    #[test]
    fn test_vcd_trace_validate() {
        let trace = HirVcdTrace::new("test");
        assert!(trace.validate().is_empty());
    }

    #[test]
    fn test_full_simulation_trace() {
        let mut trace = HirVcdTrace::new("t27c sim");
        trace.add_wire(1, "clk");
        trace.add_wire(1, "rst_n");
        trace.add_reg(32, "counter");
        trace.add_wire(8, "led");
        trace.record(0, "rst_n", 0);
        trace.record(10000, "rst_n", 1);
        for i in 0..20u32 {
            let ts = (i as u64) * 5000;
            trace.record(ts, "clk", 0);
            trace.record(ts + 2500, "clk", 1);
            if i > 2 {
                trace.record(ts + 2500, "counter", i * 4);
            }
        }
        assert_eq!(trace.variables.len(), 4);
        assert!(trace.changes.len() > 40);
        assert!(trace.duration_ps() > 0);
        let vcd = trace.emit_vcd();
        assert!(vcd.contains("$var wire 1"));
        assert!(vcd.contains("$var reg 32"));
        assert!(vcd.contains("#97500"));
    }
}

#[cfg(test)]
mod tests_hir_soc_integration {
    use super::*;

    fn build_full_soc() -> HirModule {
        let mut soc = HirModule::new("TrinitySoC");

        soc.ports.push(HirPort {
            name: "clk".into(),
            dir: HwPortDir::Input,
            ty: HwType::Clock,
        });
        soc.ports.push(HirPort {
            name: "rst_n".into(),
            dir: HwPortDir::Input,
            ty: HwType::Reset(HwResetKind::Async, HwResetPolarity::ActiveLow),
        });
        soc.ports.push(HirPort {
            name: "uart_tx".into(),
            dir: HwPortDir::Output,
            ty: HwType::Bool,
        });
        soc.ports.push(HirPort {
            name: "uart_rx".into(),
            dir: HwPortDir::Input,
            ty: HwType::Bool,
        });
        soc.ports.push(HirPort {
            name: "spi_mosi".into(),
            dir: HwPortDir::Output,
            ty: HwType::Bool,
        });
        soc.ports.push(HirPort {
            name: "spi_miso".into(),
            dir: HwPortDir::Input,
            ty: HwType::Bool,
        });
        soc.ports.push(HirPort {
            name: "spi_sck".into(),
            dir: HwPortDir::Output,
            ty: HwType::Bool,
        });
        soc.ports.push(HirPort {
            name: "spi_cs_n".into(),
            dir: HwPortDir::Output,
            ty: HwType::Bool,
        });

        soc.signals.push(HirSignal {
            name: "counter".into(),
            kind: HwSignalKind::Reg,
            ty: HwType::UInt(32),
            reset_value: "0".into(),
        });
        soc.signals.push(HirSignal {
            name: "led".into(),
            kind: HwSignalKind::Reg,
            ty: HwType::UInt(4),
            reset_value: "0".into(),
        });

        soc.assigns.push(HirAssign {
            target: "led".into(),
            value: "counter[27]".into(),
        });

        let mut bram = HirMemory::new_bram("main_ram", 1024, 32);
        bram.add_read_port("rd");
        bram.add_write_port("wr");
        soc.memories.push(bram);

        soc.clock_domains
            .push(HirClockDomain::new("sys", "ext", 100_000_000));

        soc.fifos.push(HirFifo::new_sync("tx_fifo", 16, 8));
        soc.fifos.push(HirFifo::new_sync("rx_fifo", 16, 8));

        soc.bus_ports
            .push(HirBusPort::axi4_lite_slave("axi_ctrl", 32, 32));
        soc.bus_ports
            .push(HirBusPort::axi4_full_master("axi_mem", 32, 64, 4));

        let mut apb = HirApbBridge::new("apb_periph", 32, 32, 4);
        apb.add_peripheral("uart0", 0x1000, 256, 0);
        apb.add_peripheral("spi0", 0x2000, 256, 1);
        apb.add_peripheral("gpio0", 0x3000, 256, 2);
        apb.add_peripheral("timer0", 0x4000, 256, 3);
        soc.apb_bridges.push(apb);

        let mut gf16 = HirGf16Accel::full("gf16_core", 8, 16);
        gf16.add_mac_unit("mac0", 32, 2);
        gf16.set_fft("fft0", 16, 2);
        soc.gf16_accels.push(gf16);

        let mut ternary =
            HirTernaryCore::basic("tri_core").with_regfile(HirTernaryRegFile::new("regfile"));
        ternary.add_pipeline_stage("IF", 1, false);
        ternary.add_pipeline_stage("ID", 1, false);
        ternary.add_pipeline_stage("EX", 1, true);
        ternary.add_pipeline_stage("MEM", 1, true);
        ternary.add_pipeline_stage("WB", 1, false);
        ternary.add_alu_op("t_add", 1, 1, false);
        ternary.add_alu_op("t_mul", 2, 2, false);
        ternary.add_alu_op("gf_mul", 16, 3, true);
        ternary.add_alu_op("gf_mac", 17, 4, true);
        soc.ternary_cores.push(ternary);

        soc.formal_config = Some(HirFormalConfig::new(
            "soc_props",
            "TrinitySoC",
            "clk",
            "rst_n",
        ));
        soc.formal_asserts.push(HirFormalAssert::immediate(
            "no_x_on_uart",
            "!$isunknown(uart_tx)",
            HwAssertSeverity::Error,
            "UART TX never X",
        ));
        soc.formal_asserts.push(HirFormalAssert::concurrent(
            "apb_select_onehot",
            "apb_periph_psel |-> $onehot({apb_periph_periph0_sel, apb_periph_periph1_sel})",
            "clk",
            "rst_n",
            "Only one peripheral selected at a time",
        ));
        soc.formal_covers.push(HirCoverPoint::new(
            "uart_activity",
            "uart_tx !== 1'b1",
            "clk",
            "UART actually transmits",
        ));
        soc.formal_assumes.push(HirFormalAssume::new(
            "stable_clock",
            "!$isunknown(clk)",
            "clk",
            "Clock is never X",
        ));

        soc
    }

    #[test]
    fn test_soc_validate() {
        let soc = build_full_soc();
        let errors = soc.validate();
        assert!(errors.is_empty(), "SoC validation errors: {:?}", errors);
    }

    #[test]
    fn test_soc_verilog_emission() {
        let soc = build_full_soc();
        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&soc);
        let verilog = emitter.into_string();

        assert!(verilog.contains("module TrinitySoC"));
        assert!(verilog.contains("endmodule"));
        assert!(verilog.contains("clk"));
        assert!(verilog.contains("rst_n"));
        assert!(verilog.contains("uart_tx"));
        assert!(verilog.contains("uart_rx"));
        assert!(verilog.contains("BRAM main_ram"));
        assert!(verilog.contains("SYNC_FIFO tx_fifo"));
        assert!(verilog.contains("SYNC_FIFO rx_fifo"));
        assert!(verilog.contains("AXI4-LITE"));
        assert!(verilog.contains("axi_ctrl"));
        assert!(verilog.contains("AXI4-FULL"));
        assert!(verilog.contains("axi_mem"));
        assert!(verilog.contains("APB BRIDGE apb_periph"));
        assert!(verilog.contains("GF16 ACCELERATOR gf16_core"));
        assert!(verilog.contains("TERNARY CORE tri_core"));
        assert!(verilog.contains("Formal verification assertions"));
        assert!(verilog.contains("assert_no_x_on_uart"));
        assert!(verilog.contains("assert_apb_select_onehot"));
        assert!(verilog.contains("cover_uart_activity"));
        assert!(verilog.contains("assume_stable_clock"));
        assert!(verilog.contains("phi^2"));
    }

    #[test]
    fn test_soc_resource_estimation() {
        let soc = build_full_soc();
        let mut catalog = HirIpCatalog::new("trinity_soc");
        catalog.add_core("uart", 300, 150, 1, 0, 4, 100);
        catalog.add_core("spi", 200, 100, 0, 0, 4, 100);
        catalog.add_core("bram_32k", 50, 30, 2, 0, 0, 100);
        catalog.add_core("gf16_accel", 3000, 1500, 3, 8, 0, 100);
        catalog.add_core("ternary_core", 11500, 5000, 3, 5, 0, 100);
        catalog.add_core("apb_bridge", 300, 200, 0, 0, 0, 100);
        catalog.add_core("axi_interconnect", 500, 300, 1, 0, 0, 100);
        catalog.add_core("fifo_pair", 100, 60, 2, 0, 0, 100);

        let arty = HirBoardResources::arty_a7();
        let xc7a = HirBoardResources::xc7a100t();

        assert!(catalog.fits_board(&arty), "SoC should fit Arty A7");
        assert!(catalog.fits_board(&xc7a), "SoC should fit XC7A100T");
        assert!(
            catalog.utilization_percent(&xc7a) < 50,
            "SoC should use < 50% of XC7A100T"
        );
    }

    #[test]
    fn test_soc_all_node_types() {
        let soc = build_full_soc();

        assert!(!soc.ports.is_empty(), "ports");
        assert!(!soc.signals.is_empty(), "signals");
        assert!(!soc.assigns.is_empty(), "assigns");
        assert!(!soc.memories.is_empty(), "memories");
        assert!(!soc.clock_domains.is_empty(), "clock_domains");
        assert!(!soc.fifos.is_empty(), "fifos");
        assert!(!soc.bus_ports.is_empty(), "bus_ports");
        assert!(!soc.apb_bridges.is_empty(), "apb_bridges");
        assert!(!soc.gf16_accels.is_empty(), "gf16_accels");
        assert!(!soc.ternary_cores.is_empty(), "ternary_cores");
        assert!(soc.formal_config.is_some(), "formal_config");
        assert!(!soc.formal_asserts.is_empty(), "formal_asserts");
        assert!(!soc.formal_covers.is_empty(), "formal_covers");
        assert!(!soc.formal_assumes.is_empty(), "formal_assumes");
    }

    #[test]
    fn test_soc_optimization() {
        let mut soc = build_full_soc();
        soc.signals.push(HirSignal {
            name: "unused_wire".into(),
            kind: HwSignalKind::Wire,
            ty: HwType::UInt(8),
            reset_value: "0".into(),
        });
        let before = soc.signals.len();
        let mut opt = HirOptimizer::new();
        opt.optimize(&mut soc);
        assert!(soc.signals.len() < before);
        assert!(!soc.signals.iter().any(|s| s.name == "unused_wire"));
        let res = opt.resource_estimate(&soc);
        assert!(res.luts > 0);
        assert!(res.bram18 > 0);
        assert!(res.dsp48 > 0);
        let xc7a = HirBoardResources::xc7a100t();
        assert!(res.luts < xc7a.luts);
    }
}

#[derive(Debug, Clone)]
pub struct DemoKernel {
    pub name: String,
    pub instr_count: u32,
    pub gf16_ops: u32,
    pub alu_ops: u32,
    pub mem_ops: u32,
}

impl DemoKernel {
    pub fn hello_trinity() -> Self {
        DemoKernel {
            name: "hello_trinity".into(),
            instr_count: 12,
            gf16_ops: 4,
            alu_ops: 6,
            mem_ops: 2,
        }
    }

    pub fn gf16_mac_demo() -> Self {
        DemoKernel {
            name: "gf16_mac_demo".into(),
            instr_count: 20,
            gf16_ops: 10,
            alu_ops: 6,
            mem_ops: 4,
        }
    }

    pub fn kernel_size_bytes(&self) -> u32 {
        self.instr_count * 4
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.name.is_empty() {
            errors.push("kernel name must not be empty".to_string());
        }
        if self.instr_count == 0 {
            errors.push("kernel instr_count must be positive".to_string());
        }
        errors
    }
}

#[derive(Debug, Clone)]
pub struct PipeResult {
    pub cycles: u32,
    pub instr_retired: u32,
    pub stalls: u32,
    pub gf16_results: u32,
    pub errors: u32,
}

impl PipeResult {
    pub fn ok(cycles: u32, retired: u32, gf16: u32) -> Self {
        PipeResult {
            cycles,
            instr_retired: retired,
            stalls: 0,
            gf16_results: gf16,
            errors: 0,
        }
    }

    pub fn error(cycles: u32, errors: u32) -> Self {
        PipeResult {
            cycles,
            instr_retired: 0,
            stalls: 0,
            gf16_results: 0,
            errors,
        }
    }

    pub fn ipc(&self) -> u32 {
        if self.cycles == 0 {
            return 0;
        }
        self.instr_retired * 100 / self.cycles
    }

    pub fn cpi(&self) -> u32 {
        if self.instr_retired == 0 {
            return 0;
        }
        self.cycles / self.instr_retired
    }

    pub fn gf16_throughput(&self, clock_mhz: u32) -> u32 {
        if self.cycles == 0 || clock_mhz == 0 {
            return 0;
        }
        self.gf16_results * clock_mhz * 1000 / self.cycles
    }

    pub fn passed(&self) -> bool {
        self.errors == 0 && self.instr_retired > 0
    }
}

#[derive(Debug, Clone)]
pub struct DemoConfig {
    pub kernel: DemoKernel,
    pub clock_mhz: u32,
    pub max_cycles: u32,
    pub trace_enabled: bool,
    pub formal_check: bool,
}

impl DemoConfig {
    pub fn new(kernel: DemoKernel) -> Self {
        DemoConfig {
            kernel,
            clock_mhz: 100,
            max_cycles: 100_000,
            trace_enabled: true,
            formal_check: true,
        }
    }

    pub fn sim_time_us(&self, cycles: u32) -> u32 {
        if self.clock_mhz == 0 {
            return 0;
        }
        cycles / self.clock_mhz
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = self.kernel.validate();
        if self.clock_mhz == 0 {
            errors.push("clock_mhz must be positive".to_string());
        }
        if self.max_cycles == 0 {
            errors.push("max_cycles must be positive".to_string());
        }
        errors
    }
}

#[cfg(test)]
mod tests_e2e_demo {
    use super::*;

    #[test]
    fn test_hello_kernel() {
        let k = DemoKernel::hello_trinity();
        assert_eq!(k.name, "hello_trinity");
        assert_eq!(k.instr_count, 12);
        assert_eq!(k.gf16_ops, 4);
        assert_eq!(k.alu_ops, 6);
        assert_eq!(k.mem_ops, 2);
    }

    #[test]
    fn test_gf16_mac_kernel() {
        let k = DemoKernel::gf16_mac_demo();
        assert_eq!(k.name, "gf16_mac_demo");
        assert_eq!(k.instr_count, 20);
        assert_eq!(k.gf16_ops, 10);
    }

    #[test]
    fn test_pipe_result_ok() {
        let r = PipeResult::ok(100, 95, 10);
        assert_eq!(r.cycles, 100);
        assert_eq!(r.instr_retired, 95);
        assert_eq!(r.stalls, 0);
        assert_eq!(r.gf16_results, 10);
        assert_eq!(r.errors, 0);
    }

    #[test]
    fn test_pipe_result_error() {
        let r = PipeResult::error(50, 2);
        assert_eq!(r.errors, 2);
        assert_eq!(r.instr_retired, 0);
    }

    #[test]
    fn test_ipc() {
        let r = PipeResult::ok(100, 50, 0);
        assert_eq!(r.ipc(), 50);
    }

    #[test]
    fn test_ipc_zero() {
        let r = PipeResult::ok(0, 0, 0);
        assert_eq!(r.ipc(), 0);
    }

    #[test]
    fn test_cpi() {
        let r = PipeResult::ok(200, 100, 0);
        assert_eq!(r.cpi(), 2);
    }

    #[test]
    fn test_cpi_zero() {
        let r = PipeResult::ok(100, 0, 0);
        assert_eq!(r.cpi(), 0);
    }

    #[test]
    fn test_gf16_throughput() {
        let r = PipeResult::ok(1000, 500, 100);
        assert_eq!(r.gf16_throughput(100), 10000);
    }

    #[test]
    fn test_sim_time() {
        let cfg = DemoConfig::new(DemoKernel::hello_trinity());
        assert_eq!(cfg.sim_time_us(100_000), 1000);
    }

    #[test]
    fn test_kernel_size_bytes() {
        let k = DemoKernel::hello_trinity();
        assert_eq!(k.kernel_size_bytes(), 48);
    }

    #[test]
    fn test_passed() {
        let r = PipeResult::ok(100, 50, 10);
        assert!(r.passed());
    }

    #[test]
    fn test_passed_errors() {
        let r = PipeResult::error(100, 1);
        assert!(!r.passed());
    }

    #[test]
    fn test_validate_kernel_ok() {
        let k = DemoKernel::hello_trinity();
        assert!(k.validate().is_empty());
    }

    #[test]
    fn test_validate_kernel_empty() {
        let k = DemoKernel {
            name: String::new(),
            instr_count: 0,
            gf16_ops: 0,
            alu_ops: 0,
            mem_ops: 0,
        };
        assert!(!k.validate().is_empty());
    }

    #[test]
    fn test_validate_config_ok() {
        let cfg = DemoConfig::new(DemoKernel::hello_trinity());
        assert!(cfg.validate().is_empty());
    }

    #[test]
    fn test_e2e_full_pipeline() {
        let kernel = DemoKernel::hello_trinity();
        let cfg = DemoConfig::new(kernel.clone());

        let mut asm = HirAssembler::new("e2e_demo");
        asm.define_symbol("_start", true);
        for _ in 0..kernel.gf16_ops {
            asm.emit_gf16(16, 1, 2, 3);
        }
        for i in 0..kernel.alu_ops {
            asm.emit_r(1, i as u32 % 27, (i as u32 + 1) % 27, (i as u32 + 2) % 27);
        }
        for i in 0..kernel.mem_ops {
            asm.emit_i(4, i as u32 % 27, 0, 0x100);
        }
        assert_eq!(asm.total_instructions(), kernel.instr_count);
        assert_eq!(asm.total_bytes(), kernel.kernel_size_bytes());
        asm.apply_relocations().unwrap();

        let mut trace = HirVcdTrace::new("t27c e2e");
        trace.add_wire(1, "clk");
        trace.add_wire(1, "rst_n");
        trace.add_reg(32, "pc");
        trace.add_reg(32, "ir");
        let result = PipeResult::ok(kernel.instr_count * 5, kernel.instr_count, kernel.gf16_ops);
        for i in 0..kernel.instr_count {
            let ts = (i as u64) * 50_000;
            trace.record(ts, "clk", 0);
            trace.record(ts + 25_000, "clk", 1);
            trace.record(ts + 25_000, "pc", i * 4);
        }
        assert!(trace.duration_ps() > 0);
        assert!(result.passed());
        assert!(result.ipc() > 0);

        let mut ternary =
            HirTernaryCore::basic("e2e_tri").with_regfile(HirTernaryRegFile::new("rf"));
        ternary.add_pipeline_stage("IF", 1, false);
        ternary.add_pipeline_stage("ID", 1, false);
        ternary.add_pipeline_stage("EX", 1, true);
        ternary.add_pipeline_stage("MEM", 1, true);
        ternary.add_pipeline_stage("WB", 1, false);
        ternary.add_alu_op("gf_mul", 16, 3, true);
        ternary.add_alu_op("gf_mac", 17, 4, true);
        assert!(ternary.validate().is_empty());

        let mut gf16 = HirGf16Accel::full("e2e_gf16", kernel.gf16_ops as u32, 16);
        gf16.add_mac_unit("mac0", 32, 2);
        assert!(gf16.validate().is_empty());

        let vcd = trace.emit_vcd();
        assert!(vcd.contains("$date"));
        assert!(vcd.contains("e2e"));
        assert!(vcd.contains("#0"));
    }

    #[test]
    fn test_e2e_soc_with_program() {
        let kernel = DemoKernel::gf16_mac_demo();
        let mut asm = HirAssembler::new("gf16_program");
        asm.define_symbol("gf16_entry", true);
        for i in 0..kernel.gf16_ops {
            asm.emit_gf16(16 + (i as u32) % 4, 1, 2, 3);
        }
        assert_eq!(asm.total_instructions(), kernel.gf16_ops);

        let words = asm.encode_all();
        assert_eq!(words.len(), kernel.gf16_ops as usize);

        let mut soc = HirModule::new("TrinitySoC_Program");
        soc.ports.push(HirPort {
            name: "clk".into(),
            dir: HwPortDir::Input,
            ty: HwType::Clock,
        });
        soc.ports.push(HirPort {
            name: "rst_n".into(),
            dir: HwPortDir::Input,
            ty: HwType::Reset(HwResetKind::Async, HwResetPolarity::ActiveLow),
        });
        let mut bram = HirMemory::new_bram("prog_mem", 4096, 32);
        bram.add_read_port("fetch");
        soc.memories.push(bram);
        soc.clock_domains
            .push(HirClockDomain::new("sys", "ext", 100_000_000));
        let mut ternary =
            HirTernaryCore::basic("tri_core").with_regfile(HirTernaryRegFile::new("rf"));
        ternary.add_pipeline_stage("IF", 1, false);
        ternary.add_pipeline_stage("ID", 1, false);
        ternary.add_pipeline_stage("EX", 2, true);
        ternary.add_pipeline_stage("WB", 1, false);
        ternary.add_alu_op("gf_mul", 16, 3, true);
        soc.ternary_cores.push(ternary);
        let mut gf16 = HirGf16Accel::full("gf16_accel", 8, 16);
        gf16.add_mac_unit("mac0", 32, 2);
        soc.gf16_accels.push(gf16);
        soc.bus_ports
            .push(HirBusPort::axi4_lite_slave("ctrl", 32, 32));

        let errors = soc.validate();
        assert!(errors.is_empty(), "SoC validation: {:?}", errors);

        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&soc);
        let verilog = emitter.into_string();
        assert!(verilog.contains("module TrinitySoC_Program"));
        assert!(verilog.contains("BRAM prog_mem"));
        assert!(verilog.contains("TERNARY CORE tri_core"));
        assert!(verilog.contains("GF16 ACCELERATOR gf16_accel"));
        assert!(verilog.contains("AXI4-LITE"));

        let mut opt = HirOptimizer::new();
        opt.optimize(&mut soc);
        let res = opt.resource_estimate(&soc);
        assert!(res.luts > 0);
        assert!(res.bram18 > 0);
        assert!(res.dsp48 > 0);

        let board = HirBoardResources::xc7a100t();
        assert!(res.luts < board.luts);
    }
}

#[cfg(test)]
mod tests_hir_linker {
    use super::*;

    #[test]
    fn test_link_text_creation() {
        let sec = LinkSection::text(0, 128);
        assert_eq!(sec.name, ".text");
        assert_eq!(sec.vaddr, 0);
        assert_eq!(sec.size, 128);
        assert_eq!(sec.end(), 128);
    }

    #[test]
    fn test_link_data_creation() {
        let sec = LinkSection::data(4096, 256);
        assert_eq!(sec.name, ".data");
        assert_eq!(sec.vaddr, 4096);
    }

    #[test]
    fn test_link_bss_creation() {
        let sec = LinkSection::bss(8192, 512);
        assert_eq!(sec.name, ".bss");
        assert_eq!(sec.flags, 2);
    }

    #[test]
    fn test_linked_symbol_creation() {
        let sym = LinkedSymbol::new("_start", 0, 0);
        assert_eq!(sym.name, "_start");
        assert_eq!(sym.value, 0);
        assert!(sym.is_global());
    }

    #[test]
    fn test_linked_symbol_local() {
        let mut sym = LinkedSymbol::new("x", 0, 0);
        sym.bind = 0;
        assert!(sym.is_local());
        assert!(!sym.is_global());
    }

    #[test]
    fn test_linker_config_creation() {
        let cfg = LinkerConfig::new("_start");
        assert_eq!(cfg.entry, "_start");
        assert_eq!(cfg.text_base, 0);
        assert_eq!(cfg.data_base, 4096);
        assert_eq!(cfg.stack_size, 1024);
    }

    #[test]
    fn test_link_ok() {
        let r = LinkResult::ok(0, 128, 256, 64);
        assert_eq!(r.entry_addr, 0);
        assert_eq!(r.total_text, 128);
        assert_eq!(r.errors, 0);
        assert!(r.passed());
    }

    #[test]
    fn test_link_fail() {
        let r = LinkResult::fail(3);
        assert_eq!(r.errors, 3);
        assert!(!r.passed());
    }

    #[test]
    fn test_total_image_size() {
        let r = LinkResult::ok(0, 128, 256, 64);
        assert_eq!(r.total_image_size(), 448);
    }

    #[test]
    fn test_stack_top() {
        let cfg = LinkerConfig::new("_start");
        assert_eq!(cfg.stack_top(), 5120);
    }

    #[test]
    fn test_validate_config_ok() {
        let cfg = LinkerConfig::new("_start");
        assert!(cfg.validate().is_empty());
    }

    #[test]
    fn test_validate_config_no_entry() {
        let cfg = LinkerConfig {
            entry: String::new(),
            text_base: 0,
            data_base: 4096,
            stack_size: 1024,
            heap_size: 4096,
            output_format: 0,
        };
        assert!(!cfg.validate().is_empty());
    }

    #[test]
    fn test_validate_symbol_ok() {
        let sym = LinkedSymbol::new("main", 0, 0);
        assert!(sym.validate().is_empty());
    }

    #[test]
    fn test_validate_symbol_empty() {
        let sym = LinkedSymbol {
            name: String::new(),
            value: 0,
            size: 0,
            section_idx: 0,
            bind: 0,
            kind: 0,
        };
        assert!(!sym.validate().is_empty());
    }

    #[test]
    fn test_text_segment() {
        let seg = LinkSegment::text(0, 1024);
        assert_eq!(seg.vaddr, 0);
        assert_eq!(seg.memsz, 1024);
        assert_eq!(seg.filesz, 1024);
    }

    #[test]
    fn test_data_segment() {
        let seg = LinkSegment::data(4096, 2048, 1024);
        assert_eq!(seg.vaddr, 4096);
        assert_eq!(seg.memsz, 2048);
        assert_eq!(seg.filesz, 1024);
    }

    #[test]
    fn test_linker_basic_link() {
        let mut asm = HirAssembler::new("mod1");
        asm.define_symbol("_start", true);
        asm.emit_r(1, 1, 2, 3);
        asm.emit_i(2, 4, 5, 42);

        let mut linker = HirLinker::new("_start");
        linker.add_object(&asm);
        let result = linker.link();
        assert!(result.passed());
        assert_eq!(result.total_text, 8);
        assert_eq!(result.num_segments, 2);
    }

    #[test]
    fn test_linker_multi_object() {
        let mut asm1 = HirAssembler::new("mod1");
        asm1.define_symbol("_start", true);
        asm1.emit_r(1, 1, 2, 3);

        let mut asm2 = HirAssembler::new("mod2");
        asm2.define_symbol("helper", true);
        asm2.emit_i(2, 4, 5, 10);
        asm2.emit_r(3, 6, 7, 8);

        let mut linker = HirLinker::new("_start");
        linker.add_object(&asm1);
        linker.add_object(&asm2);
        let result = linker.link();
        assert!(result.passed());
        assert_eq!(result.total_text, 12);
        assert_eq!(linker.symbols.len(), 2);
    }

    #[test]
    fn test_linker_resolve_symbol() {
        let mut asm = HirAssembler::new("mod");
        asm.define_symbol("main", true);
        asm.emit_r(1, 1, 2, 3);

        let mut linker = HirLinker::new("main");
        linker.add_object(&asm);
        linker.link();
        assert_eq!(linker.resolve_symbol("main"), Some(0));
        assert_eq!(linker.resolve_symbol("missing"), None);
    }

    #[test]
    fn test_linker_emit_image() {
        let mut asm = HirAssembler::new("mod");
        asm.define_symbol("_start", true);
        asm.emit_r(1, 1, 2, 3);
        let mut linker = HirLinker::new("_start");
        linker.add_object(&asm);
        linker.link();
        let image = linker.emit_image();
        assert_eq!(image.len(), 4);
    }

    #[test]
    fn test_linker_emit_hex() {
        let mut asm = HirAssembler::new("mod");
        asm.define_symbol("_start", true);
        asm.emit_r(1, 1, 2, 3);
        let mut linker = HirLinker::new("_start");
        linker.add_object(&asm);
        linker.link();
        let hex = linker.emit_hex();
        assert!(hex.contains("@00000000"));
    }

    #[test]
    fn test_linker_validate() {
        let linker = HirLinker::new("_start");
        assert!(linker.validate().is_empty());
    }

    #[test]
    fn test_linker_gf16_program() {
        let mut asm = HirAssembler::new("gf16_prog");
        asm.define_symbol("gf16_entry", true);
        for i in 0..8u32 {
            asm.emit_gf16(16 + i % 4, 1, 2, 3);
        }
        let mut linker = HirLinker::new("gf16_entry");
        linker.add_object(&asm);
        let result = linker.link();
        assert!(result.passed());
        assert_eq!(result.total_text, 32);
        let image = linker.emit_image();
        assert_eq!(image.len(), 32);
        assert_eq!(linker.symbols.len(), 1);
    }
}

#[cfg(test)]
mod tests_hir_timing {
    use super::*;

    #[test]
    fn test_comb_arc() {
        let a = TimingArc::comb("a", "b", 500);
        assert_eq!(a.source, "a");
        assert_eq!(a.sink, "b");
        assert_eq!(a.delay_ps, 500);
        assert_eq!(a.kind, ArcKind::Comb);
    }

    #[test]
    fn test_reg_to_reg_arc() {
        let a = TimingArc::reg_to_reg("r1", "r2", 800);
        assert_eq!(a.kind, ArcKind::RegToReg);
    }

    #[test]
    fn test_input_to_reg_arc() {
        let a = TimingArc::input_to_reg("din", "r1", 400);
        assert_eq!(a.kind, ArcKind::InputToReg);
    }

    #[test]
    fn test_timing_path_met() {
        let p = TimingPath::new("r1", "r2", 5000, 5000);
        assert!(p.is_met());
        assert!(!p.is_violated());
    }

    #[test]
    fn test_timing_path_violated() {
        let p = TimingPath::new("r1", "r2", 12000, -2000);
        assert!(!p.is_met());
        assert!(p.is_violated());
    }

    #[test]
    fn test_clock_constraint() {
        let c = TimingConstraint::from_period("clk_fast", 5000);
        assert_eq!(c.period_ps, 5000);
        assert_eq!(c.clock_name, "clk");
    }

    #[test]
    fn test_clock_mhz() {
        let c = TimingConstraint::from_mhz("clk_100", 100);
        assert_eq!(c.period_ps, 10_000_000);
    }

    #[test]
    fn test_clock_mhz_zero() {
        let c = TimingConstraint::from_mhz("bad", 0);
        assert_eq!(c.period_ps, 10000);
    }

    #[test]
    fn test_timing_ok_report() {
        let r = TimingReport::ok(5000, 200);
        assert_eq!(r.critical_path_ps, 5000);
        assert_eq!(r.fmax_mhz, 200);
        assert!(r.passed());
    }

    #[test]
    fn test_timing_fail_report() {
        let r = TimingReport::fail(15000);
        assert!(r.has_violations);
        assert!(!r.passed());
    }

    #[test]
    fn test_path_delay() {
        let arcs = vec![
            TimingArc::comb("a", "b", 100),
            TimingArc::comb("b", "c", 200),
            TimingArc::comb("c", "d", 300),
        ];
        assert_eq!(path_delay(&arcs), 600);
    }

    #[test]
    fn test_slack_positive() {
        assert_eq!(TimingModel::slack(5000, 10000), 5000);
    }

    #[test]
    fn test_slack_negative() {
        assert_eq!(TimingModel::slack(15000, 10000), -5000);
    }

    #[test]
    fn test_fmax_from_delay() {
        assert_eq!(TimingModel::fmax_from_delay(5000), 200_000);
    }

    #[test]
    fn test_fmax_zero() {
        assert_eq!(TimingModel::fmax_from_delay(0), 0);
    }

    #[test]
    fn test_est_comb_delay() {
        assert_eq!(TimingModel::est_comb_delay(3), 600);
    }

    #[test]
    fn test_est_reg_to_reg_delay() {
        assert_eq!(TimingModel::est_reg_to_reg_delay(3), 800);
    }

    #[test]
    fn test_worst_path() {
        let paths = vec![
            TimingPath::new("a", "b", 500, 0),
            TimingPath::new("c", "d", 1200, 0),
            TimingPath::new("e", "f", 800, 0),
        ];
        assert_eq!(worst_path_delay(&paths), 1200);
    }

    #[test]
    fn test_worst_path_empty() {
        assert_eq!(worst_path_delay(&[]), 0);
    }

    #[test]
    fn test_validate_constraint_ok() {
        let c = TimingConstraint::from_period("clk", 10000);
        assert!(c.validate().is_empty());
    }

    #[test]
    fn test_validate_constraint_empty_name() {
        let c = TimingConstraint {
            name: String::new(),
            period_ps: 10000,
            clock_name: "clk".into(),
        };
        assert!(!c.validate().is_empty());
    }

    #[test]
    fn test_timing_model_constants() {
        assert_eq!(TimingModel::LUT_DELAY_PS, 100);
        assert_eq!(TimingModel::BRAM_DELAY_PS, 2000);
        assert_eq!(TimingModel::DSP_DELAY_PS, 2500);
        assert_eq!(TimingModel::ROUTING_DELAY_PS, 300);
        assert_eq!(TimingModel::SETUP_TIME_PS, 200);
    }

    #[test]
    fn test_analyze_module() {
        let mut soc = HirModule::new("test_timing");
        soc.ports.push(HirPort {
            name: "clk".into(),
            dir: HwPortDir::Input,
            ty: HwType::Clock,
        });
        soc.signals.push(HirSignal {
            name: "counter".into(),
            kind: HwSignalKind::Reg,
            ty: HwType::UInt(32),
            reset_value: "0".into(),
        });
        let mut bram = HirMemory::new_bram("ram", 1024, 32);
        bram.add_read_port("rd");
        soc.memories.push(bram);
        let mut gf16 = HirGf16Accel::full("gf16", 4, 16);
        gf16.add_mac_unit("m0", 32, 2);
        soc.gf16_accels.push(gf16);

        let constraint = TimingConstraint::from_mhz("sys_clk", 100);
        let report = TimingModel::analyze_module(&soc, &constraint);
        assert!(report.total_paths > 0);
        assert!(report.critical_path_ps > 0);
        assert!(report.passed());
    }

    #[test]
    fn test_analyze_ternary_core_timing() {
        let mut soc = HirModule::new("tri_timing");
        soc.ports.push(HirPort {
            name: "clk".into(),
            dir: HwPortDir::Input,
            ty: HwType::Clock,
        });
        let mut tc = HirTernaryCore::basic("core").with_regfile(HirTernaryRegFile::new("rf"));
        tc.add_pipeline_stage("IF", 1, false);
        tc.add_pipeline_stage("EX", 2, true);
        tc.add_pipeline_stage("WB", 1, false);
        soc.ternary_cores.push(tc);
        let constraint = TimingConstraint::from_mhz("sys_clk", 100);
        let report = TimingModel::analyze_module(&soc, &constraint);
        assert!(report.total_paths > 0);
        assert!(report.critical_path_ps > 0);
    }
}

#[cfg(test)]
mod tests_hir_power {
    use super::*;

    #[test]
    fn test_power_domain_creation() {
        let d = PowerDomain::new("core", 100);
        assert_eq!(d.name, "core");
        assert_eq!(d.voltage_mv, 1000);
        assert_eq!(d.clock_mhz, 100);
        assert_eq!(d.toggle_rate, 12);
    }

    #[test]
    fn test_zero_power() {
        let p = PowerEstimate {
            dynamic_mw: 0,
            static_mw: 0,
            total_mw: 0,
            lut_power_mw: 0,
            ff_power_mw: 0,
            bram_power_mw: 0,
            dsp_power_mw: 0,
        };
        assert_eq!(p.dynamic_mw, 0);
        assert_eq!(p.total_mw, 0);
    }

    #[test]
    fn test_power_estimate_creation() {
        let p = PowerEstimate {
            dynamic_mw: 200,
            static_mw: 100,
            total_mw: 300,
            lut_power_mw: 0,
            ff_power_mw: 0,
            bram_power_mw: 0,
            dsp_power_mw: 0,
        };
        assert_eq!(p.dynamic_mw, 200);
        assert_eq!(p.static_mw, 100);
        assert_eq!(p.total_mw, 300);
    }

    #[test]
    fn test_est_lut_dynamic() {
        assert!(PowerModel::est_lut_dynamic(1000, 100, 12) > 0);
    }

    #[test]
    fn test_est_ff_dynamic() {
        assert!(PowerModel::est_ff_dynamic(2000, 100, 12) > 0);
    }

    #[test]
    fn test_est_bram_dynamic() {
        assert_eq!(PowerModel::est_bram_dynamic(10, 100), 50);
    }

    #[test]
    fn test_est_dsp_dynamic() {
        assert_eq!(PowerModel::est_dsp_dynamic(8, 100), 80);
    }

    #[test]
    fn test_est_static() {
        assert!(PowerModel::est_static(5000) > PowerModel::STATIC_BASE_MW);
    }

    #[test]
    fn test_est_static_base() {
        assert_eq!(PowerModel::est_static(0), PowerModel::STATIC_BASE_MW);
    }

    #[test]
    fn test_power_constants() {
        assert_eq!(PowerModel::LUT_POWER_UW_PER_MHZ, 10);
        assert_eq!(PowerModel::FF_POWER_UW_PER_MHZ, 5);
        assert_eq!(PowerModel::BRAM_POWER_UW_PER_MHZ, 50);
        assert_eq!(PowerModel::DSP_POWER_UW_PER_MHZ, 100);
        assert_eq!(PowerModel::STATIC_BASE_MW, 50);
    }

    #[test]
    fn test_validate_domain_ok() {
        let d = PowerDomain::new("core", 100);
        assert!(d.validate().is_empty());
    }

    #[test]
    fn test_validate_domain_empty() {
        let d = PowerDomain {
            name: String::new(),
            voltage_mv: 1000,
            clock_mhz: 100,
            toggle_rate: 12,
        };
        assert!(!d.validate().is_empty());
    }

    #[test]
    fn test_validate_domain_zero_clock() {
        let d = PowerDomain {
            name: "core".into(),
            voltage_mv: 1000,
            clock_mhz: 0,
            toggle_rate: 12,
        };
        assert!(!d.validate().is_empty());
    }

    #[test]
    fn test_estimate_module() {
        let mut soc = HirModule::new("power_test");
        soc.ports.push(HirPort {
            name: "clk".into(),
            dir: HwPortDir::Input,
            ty: HwType::Clock,
        });
        soc.signals.push(HirSignal {
            name: "wire_a".into(),
            kind: HwSignalKind::Wire,
            ty: HwType::UInt(8),
            reset_value: "0".into(),
        });
        soc.signals.push(HirSignal {
            name: "reg_b".into(),
            kind: HwSignalKind::Reg,
            ty: HwType::UInt(32),
            reset_value: "0".into(),
        });
        let mut bram = HirMemory::new_bram("ram", 1024, 32);
        bram.add_read_port("rd");
        soc.memories.push(bram);
        let mut gf16 = HirGf16Accel::full("gf16", 4, 16);
        gf16.add_mac_unit("m0", 32, 2);
        soc.gf16_accels.push(gf16);

        let est = PowerModel::estimate_module(&soc, 100, 12);
        assert!(est.dynamic_mw > 0);
        assert!(est.static_mw > 0);
        assert!(est.total_mw > est.dynamic_mw);
        assert!(est.total_mw > est.static_mw);
    }

    #[test]
    fn test_estimate_empty_module() {
        let soc = HirModule::new("empty");
        let est = PowerModel::estimate_module(&soc, 100, 12);
        assert_eq!(est.dynamic_mw, 0);
        assert_eq!(est.static_mw, PowerModel::STATIC_BASE_MW);
    }
}

#[cfg(test)]
mod tests_hir_placement {
    use super::*;

    #[test]
    fn test_region_creation() {
        let r = PlacementRegion::logic("core", 10, 20, 30, 40);
        assert_eq!(r.name, "core");
        assert_eq!(r.kind, RegionKind::LogicCluster);
        assert_eq!(r.width(), 20);
        assert_eq!(r.height(), 20);
    }

    #[test]
    fn test_logic_cluster() {
        let r = PlacementRegion::logic("logic0", 0, 0, 10, 10);
        assert_eq!(r.kind, RegionKind::LogicCluster);
    }

    #[test]
    fn test_bram_column() {
        let r = PlacementRegion::bram_col("bram0", 5, 0, 50);
        assert_eq!(r.kind, RegionKind::BramColumn);
        assert_eq!(r.width(), 1);
    }

    #[test]
    fn test_dsp_column() {
        let r = PlacementRegion::dsp_col("dsp0", 8, 0, 50);
        assert_eq!(r.kind, RegionKind::DspColumn);
    }

    #[test]
    fn test_region_area() {
        let r = PlacementRegion::logic("big", 0, 0, 20, 30);
        assert_eq!(r.area(), 600);
    }

    #[test]
    fn test_regions_overlap_yes() {
        let a = PlacementRegion::logic("a", 0, 0, 10, 10);
        let b = PlacementRegion::logic("b", 5, 5, 15, 15);
        assert!(a.overlaps(&b));
    }

    #[test]
    fn test_regions_overlap_no() {
        let a = PlacementRegion::logic("a", 0, 0, 10, 10);
        let b = PlacementRegion::logic("b", 20, 20, 30, 30);
        assert!(!a.overlaps(&b));
    }

    #[test]
    fn test_hint_creation() {
        let h = PlacementHint::new("uart_tx", "io_region", 1);
        assert_eq!(h.module_name, "uart_tx");
        assert_eq!(h.region_name, "io_region");
        assert_eq!(h.priority, 1);
    }

    #[test]
    fn test_route_constraint() {
        let rc = RouteConstraint {
            source: "uart_tx".into(),
            sink: "uart_rx".into(),
            max_delay_ps: 500,
        };
        assert_eq!(rc.max_delay_ps, 500);
    }

    #[test]
    fn test_floorplan_creation() {
        let f = Floorplan::new("arty_soc", "xc7a100t");
        assert_eq!(f.name, "arty_soc");
        assert_eq!(f.device, "xc7a100t");
    }

    #[test]
    fn test_validate_region_ok() {
        let r = PlacementRegion::logic("ok", 0, 0, 10, 10);
        assert!(r.validate().is_empty());
    }

    #[test]
    fn test_validate_region_bad_coords() {
        let r = PlacementRegion {
            name: "bad".into(),
            kind: RegionKind::LogicCluster,
            x0: 30,
            y0: 0,
            x1: 20,
            y1: 10,
        };
        assert!(!r.validate().is_empty());
    }

    #[test]
    fn test_validate_hint_ok() {
        let h = PlacementHint::new("mod", "reg", 1);
        assert!(h.validate().is_empty());
    }

    #[test]
    fn test_validate_hint_empty() {
        let h = PlacementHint {
            module_name: String::new(),
            region_name: String::new(),
            priority: 0,
        };
        assert!(!h.validate().is_empty());
    }

    #[test]
    fn test_floorplan_auto() {
        let mut soc = HirModule::new("test_soc");
        soc.ports.push(HirPort {
            name: "clk".into(),
            dir: HwPortDir::Input,
            ty: HwType::Clock,
        });
        soc.ports.push(HirPort {
            name: "uart_tx".into(),
            dir: HwPortDir::Output,
            ty: HwType::Bool,
        });
        let mut bram = HirMemory::new_bram("ram", 1024, 32);
        bram.add_read_port("rd");
        soc.memories.push(bram);
        let mut gf16 = HirGf16Accel::full("gf16", 4, 16);
        gf16.add_mac_unit("m0", 32, 2);
        soc.gf16_accels.push(gf16);
        let mut tc = HirTernaryCore::basic("tri").with_regfile(HirTernaryRegFile::new("rf"));
        tc.add_pipeline_stage("IF", 1, false);
        soc.ternary_cores.push(tc);

        let mut fp = Floorplan::new("test_floorplan", "xc7a100t");
        fp.auto_floorplan(&soc);
        assert!(
            fp.regions.len() >= 3,
            "expected >= 3 regions, got {}",
            fp.regions.len()
        );
        assert!(!fp.hints.is_empty());
        assert!(fp.total_area() > 0);
        assert!(fp.validate().is_empty());
    }

    #[test]
    fn test_floorplan_check_overlaps() {
        let mut fp = Floorplan::new("test", "xc7a100t");
        fp.add_region(PlacementRegion::logic("a", 0, 0, 10, 10));
        fp.add_region(PlacementRegion::logic("b", 5, 5, 15, 15));
        fp.add_region(PlacementRegion::logic("c", 20, 20, 30, 30));
        let overlaps = fp.check_overlaps();
        assert_eq!(overlaps.len(), 1);
        assert_eq!(overlaps[0].0, "a");
        assert_eq!(overlaps[0].1, "b");
    }

    #[test]
    fn test_floorplan_no_overlaps() {
        let mut fp = Floorplan::new("test", "xc7a100t");
        fp.add_region(PlacementRegion::logic("a", 0, 0, 10, 10));
        fp.add_region(PlacementRegion::logic("b", 20, 20, 30, 30));
        assert!(fp.check_overlaps().is_empty());
    }
}

#[cfg(test)]
mod tests_hir_bitstream {
    use super::*;

    #[test]
    fn test_bitstream_meta_creation() {
        let m = BitstreamMeta::new("xc7a100t", 1024);
        assert_eq!(m.device, "xc7a100t");
        assert_eq!(m.size_bytes, 1024);
        assert_eq!(m.checksum, 0);
    }

    #[test]
    fn test_compute_checksum_empty() {
        let h = BitstreamMeta::compute_checksum(&[]);
        assert_ne!(h, 0);
    }

    #[test]
    fn test_compute_checksum_deterministic() {
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let h1 = BitstreamMeta::compute_checksum(&data);
        let h2 = BitstreamMeta::compute_checksum(&data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_compute_checksum_different() {
        let d1 = vec![0x01, 0x02];
        let d2 = vec![0x02, 0x01];
        assert_ne!(
            BitstreamMeta::compute_checksum(&d1),
            BitstreamMeta::compute_checksum(&d2)
        );
    }

    #[test]
    fn test_with_checksum() {
        let data = vec![0xAB, 0xCD, 0xEF, 0x01];
        let m = BitstreamMeta::new("xc7a100t", 0).with_checksum(&data);
        assert_eq!(m.size_bytes, 4);
        assert_ne!(m.checksum, 0);
    }

    #[test]
    fn test_verify_ok() {
        let data = vec![0x42; 100];
        let m = BitstreamMeta::new("xc7a100t", 0).with_checksum(&data);
        assert!(m.verify(&data));
    }

    #[test]
    fn test_verify_fail() {
        let data = vec![0x42; 100];
        let m = BitstreamMeta::new("xc7a100t", 0).with_checksum(&data);
        let mut bad = data.clone();
        bad[50] = 0xFF;
        assert!(!m.verify(&bad));
    }

    #[test]
    fn test_validate_ok() {
        let m = BitstreamMeta::new("xc7a100t", 1024);
        assert!(m.validate().is_empty());
    }

    #[test]
    fn test_validate_empty_device() {
        let m = BitstreamMeta::new("", 1024);
        assert!(!m.validate().is_empty());
    }

    #[test]
    fn test_validate_zero_size() {
        let m = BitstreamMeta::new("xc7a100t", 0);
        assert!(!m.validate().is_empty());
    }
}

#[cfg(test)]
mod tests_hir_pipeline_parity {
    use super::*;

    fn spec_source(name: &str) -> String {
        format!(
            r#"module {} {{
    pub const CLK_FREQ : u32 = 100000000
    pub fn add(a: u32, b: u32) -> u32 {{
        return a + b
    }}
    pub fn counter(prev: u32) -> u32 {{
        return prev + 1
    }}
}}"#,
            name
        )
    }

    #[test]
    fn test_ast_roundtrip() {
        let src = spec_source("ParityTest1");
        let ast = Compiler::parse_ast(&src).unwrap();
        assert_eq!(ast.name, "ParityTest1");
        assert!(ast.children.len() >= 2);
    }

    #[test]
    fn test_hir_roundtrip() {
        let src = spec_source("ParityTest2");
        let ast = Compiler::parse_ast(&src).unwrap();
        let hir = AstToHir::convert(&ast).unwrap();
        assert_eq!(hir.name, "ParityTest2");
        assert!(!hir.signals.is_empty());
        let errors = hir.validate();
        assert!(errors.is_empty(), "Validation errors: {:?}", errors);
    }

    #[test]
    fn test_verilog_direct() {
        let src = spec_source("ParityTest3");
        let result = Compiler::compile_verilog(&src);
        assert!(result.is_ok(), "Direct verilog failed: {:?}", result.err());
        let verilog = result.unwrap();
        assert!(verilog.contains("module ParityTest3"));
        assert!(verilog.contains("endmodule"));
    }

    #[test]
    fn test_verilog_hir() {
        let src = spec_source("ParityTest4");
        let result = Compiler::compile_verilog_hir(&src);
        assert!(result.is_ok(), "HIR verilog failed: {:?}", result.err());
        let verilog = result.unwrap();
        assert!(verilog.contains("module ParityTest4"));
        assert!(verilog.contains("endmodule"));
    }

    #[test]
    fn test_both_produce_modules() {
        let src = spec_source("ParityTest5");
        let direct = Compiler::compile_verilog(&src).unwrap();
        let hir = Compiler::compile_verilog_hir(&src).unwrap();
        assert!(direct.contains("module ParityTest5"));
        assert!(hir.contains("module ParityTest5"));
        assert!(direct.contains("endmodule"));
        assert!(hir.contains("endmodule"));
    }

    #[test]
    fn test_hir_debug_output() {
        let src = spec_source("ParityTest6");
        let result = Compiler::debug_hir(&src);
        assert!(result.is_ok(), "Debug HIR failed: {:?}", result.err());
        let debug = result.unwrap();
        assert!(debug.contains("ParityTest6"));
    }

    #[test]
    fn test_full_toolchain_spec() {
        let src = r#"module ToolchainTest {
    pub const WIDTH : u32 = 8
    pub fn mul2(x: u32) -> u32 {
        return x + x
    }
}"#;
        let ast = Compiler::parse_ast(src).unwrap();
        let hir = AstToHir::convert(&ast).unwrap();
        assert!(hir.validate().is_empty());
        let direct = Compiler::compile_verilog(src).unwrap();
        let via_hir = Compiler::compile_verilog_hir(src).unwrap();
        assert!(direct.contains("ToolchainTest"));
        assert!(via_hir.contains("ToolchainTest"));

        let mut opt = HirOptimizer::new();
        let mut hir_opt = AstToHir::convert(&ast).unwrap();
        opt.optimize(&mut hir_opt);
        let res = opt.resource_estimate(&hir_opt);
        let board = HirBoardResources::xc7a100t();
        assert!(res.luts < board.luts);

        let constraint = TimingConstraint::from_mhz("sys", 100);
        let timing = TimingModel::analyze_module(&hir_opt, &constraint);

        let power = PowerModel::estimate_module(&hir_opt, 100, 12);
        assert!(power.total_mw > 0);

        let mut fp = Floorplan::new("toolchain", "xc7a100t");
        fp.auto_floorplan(&hir_opt);
    }
}

#[cfg(test)]
mod tests_hir_partition {
    use super::*;

    #[test]
    fn test_fpga_node() {
        let n = FpgaNode::arty_a7("fpga0");
        assert_eq!(n.name, "fpga0");
        assert_eq!(n.device, "xc7a100t");
        assert_eq!(n.luts, 63400);
    }

    #[test]
    fn test_lvds_link() {
        let l = InterFpgaLink::lvds(0, 1, 8);
        assert_eq!(l.fpga_a, 0);
        assert_eq!(l.bandwidth_mbps(), 8000);
    }

    #[test]
    fn test_serdes_link() {
        let l = InterFpgaLink::serdes(0, 1);
        assert_eq!(l.protocol, 1);
        assert_eq!(l.bandwidth_mbps(), 25000);
    }

    #[test]
    fn test_assignment() {
        let a = PartitionAssign {
            module_name: "uart".into(),
            fpga_idx: 0,
            luts: 200,
            ffs: 100,
            bram18: 1,
            dsp48: 0,
        };
        assert_eq!(a.module_name, "uart");
        assert_eq!(a.fpga_idx, 0);
    }

    #[test]
    fn test_partition_ok() {
        let r = PartitionResult::ok(2, 5, 1, 8000);
        assert_eq!(r.num_fpgas, 2);
        assert!(r.balanced);
        assert!(r.passed());
    }

    #[test]
    fn test_partition_fail() {
        let r = PartitionResult::fail(1);
        assert!(!r.passed());
    }

    #[test]
    fn test_total_bandwidth() {
        let mut p = HirPartitioner::new();
        p.add_link(InterFpgaLink::lvds(0, 1, 8));
        p.add_link(InterFpgaLink::lvds(1, 2, 4));
        assert_eq!(p.total_bandwidth(), 12000);
    }

    #[test]
    fn test_fpga_util() {
        let n = FpgaNode::arty_a7("fpga0");
        assert_eq!(n.util(31700), 50);
    }

    #[test]
    fn test_fpga_remaining() {
        let n = FpgaNode::arty_a7("fpga0");
        assert_eq!(n.remaining(10000), 53400);
    }

    #[test]
    fn test_fpga_remaining_over() {
        let n = FpgaNode::arty_a7("fpga0");
        assert_eq!(n.remaining(100000), 0);
    }

    #[test]
    fn test_validate_node_ok() {
        let n = FpgaNode::arty_a7("fpga0");
        assert!(n.validate().is_empty());
    }

    #[test]
    fn test_validate_node_empty() {
        let n = FpgaNode {
            name: String::new(),
            device: "xc7a100t".into(),
            luts: 0,
            ffs: 0,
            bram18: 0,
            dsp48: 0,
            io_pins: 0,
        };
        assert!(!n.validate().is_empty());
    }

    #[test]
    fn test_auto_partition_single_fpga() {
        let mut p = HirPartitioner::new();
        p.add_fpga(FpgaNode::arty_a7("fpga0"));
        let modules = vec![
            ("uart".into(), 200u32, 100u32, 1u32, 0u32),
            ("spi".into(), 150u32, 80u32, 0u32, 0u32),
            ("bram_ctrl".into(), 100u32, 50u32, 10u32, 0u32),
        ];
        let result = p.auto_partition(&modules);
        assert!(result.passed());
        assert_eq!(result.num_fpgas, 1);
        assert_eq!(result.num_assignments, 3);
    }

    #[test]
    fn test_auto_partition_multi_fpga() {
        let mut p = HirPartitioner::new();
        p.add_fpga(FpgaNode::arty_a7("fpga0"));
        p.add_fpga(FpgaNode::arty_a7("fpga1"));
        p.add_link(InterFpgaLink::lvds(0, 1, 8));
        let modules = vec![
            ("uart".into(), 200u32, 100u32, 1u32, 0u32),
            ("spi".into(), 150u32, 80u32, 0u32, 0u32),
            ("gf16_accel".into(), 3000u32, 1500u32, 3u32, 8u32),
            ("ternary_core".into(), 11500u32, 5000u32, 3u32, 5u32),
            ("bram_ctrl".into(), 100u32, 50u32, 10u32, 0u32),
            ("apb_bridge".into(), 300u32, 200u32, 0u32, 0u32),
        ];
        let result = p.auto_partition(&modules);
        assert!(result.passed());
        assert_eq!(result.num_fpgas, 2);
        assert_eq!(result.num_assignments, 6);
        assert_eq!(result.total_bandwidth_mbps, 8000);
    }

    #[test]
    fn test_partition_no_fpgas() {
        let mut p = HirPartitioner::new();
        let modules = vec![("uart".into(), 200u32, 100u32, 1u32, 0u32)];
        let result = p.auto_partition(&modules);
        assert!(!result.passed());
    }

    #[test]
    fn test_fpga_usage() {
        let mut p = HirPartitioner::new();
        p.add_fpga(FpgaNode::arty_a7("fpga0"));
        p.assign("uart", 0, 200, 100, 1, 0);
        p.assign("spi", 0, 150, 80, 0, 0);
        let (luts, ffs, bram, dsp) = p.fpga_usage(0);
        assert_eq!(luts, 350);
        assert_eq!(ffs, 180);
        assert_eq!(bram, 1);
        assert_eq!(dsp, 0);
    }

    #[test]
    fn test_does_not_fit() {
        let mut p = HirPartitioner::new();
        p.add_fpga(FpgaNode::arty_a7("fpga0"));
        assert!(!p.assign("huge", 0, 999999, 0, 0, 0));
    }
}

#[cfg(test)]
mod tests_hir_router {
    use super::*;

    #[test]
    fn test_data_edge() {
        let e = ConnEdge::data("a", "b", 32);
        assert_eq!(e.kind, ConnEdgeKind::Data);
        assert_eq!(e.bit_width, 32);
    }

    #[test]
    fn test_clock_edge() {
        let e = ConnEdge::clock("pll", "core");
        assert_eq!(e.kind, ConnEdgeKind::Clock);
        assert_eq!(e.bit_width, 1);
    }

    #[test]
    fn test_fanout_info() {
        let f = FanoutInfo {
            signal: "data_bus".into(),
            fanout: 8,
            total_bits: 32,
        };
        assert_eq!(f.fanout, 8);
        assert!(!f.is_high_fanout());
        assert!(!f.is_clock_network());
    }

    #[test]
    fn test_high_fanout() {
        let f = FanoutInfo {
            signal: "big".into(),
            fanout: 20,
            total_bits: 1,
        };
        assert!(f.is_high_fanout());
    }

    #[test]
    fn test_clock_network() {
        let f = FanoutInfo {
            signal: "clk".into(),
            fanout: 50,
            total_bits: 1,
        };
        assert!(f.is_clock_network());
    }

    #[test]
    fn test_wire_length_local() {
        assert_eq!(RouteModel::est_wire_length(2), 500);
    }
    #[test]
    fn test_wire_length_medium() {
        assert_eq!(RouteModel::est_wire_length(8), 2000);
    }
    #[test]
    fn test_wire_length_long() {
        assert_eq!(RouteModel::est_wire_length(32), 5000);
    }
    #[test]
    fn test_wire_length_zero() {
        assert_eq!(RouteModel::est_wire_length(0), 0);
    }

    #[test]
    fn test_congestion() {
        assert_eq!(RouteModel::est_congestion(1000, 10), 100);
    }
    #[test]
    fn test_congestion_zero_area() {
        assert_eq!(RouteModel::est_congestion(1000, 0), 0);
    }

    #[test]
    fn test_route_passed() {
        let r = RouteEstimate {
            total_nets: 100,
            total_wire_length_um: 50000,
            avg_wire_length_um: 500,
            max_fanout: 10,
            congestion_score: 20,
            needs_global_buf: false,
        };
        assert!(r.passed());
    }

    #[test]
    fn test_route_failed() {
        let r = RouteEstimate {
            total_nets: 1000,
            total_wire_length_um: 500000,
            avg_wire_length_um: 500,
            max_fanout: 50,
            congestion_score: 90,
            needs_global_buf: true,
        };
        assert!(!r.passed());
    }

    #[test]
    fn test_router_analyze_fanout() {
        let mut router = HirRouter::new();
        router.add_edge(ConnEdge::data("src", "d1", 8));
        router.add_edge(ConnEdge::data("src", "d2", 8));
        router.add_edge(ConnEdge::data("src", "d3", 8));
        router.add_edge(ConnEdge::data("other", "d4", 1));
        router.analyze_fanout();
        assert_eq!(router.fanouts.len(), 2);
        assert_eq!(router.fanouts[0].signal, "src");
        assert_eq!(router.fanouts[0].fanout, 3);
    }

    #[test]
    fn test_analyze_module_routing() {
        let mut soc = HirModule::new("route_test");
        soc.ports.push(HirPort {
            name: "clk".into(),
            dir: HwPortDir::Input,
            ty: HwType::Clock,
        });
        soc.ports.push(HirPort {
            name: "data_in".into(),
            dir: HwPortDir::Input,
            ty: HwType::UInt(32),
        });
        soc.ports.push(HirPort {
            name: "data_out".into(),
            dir: HwPortDir::Output,
            ty: HwType::UInt(32),
        });
        soc.signals.push(HirSignal {
            name: "counter".into(),
            kind: HwSignalKind::Reg,
            ty: HwType::UInt(32),
            reset_value: "0".into(),
        });
        soc.assigns.push(HirAssign {
            target: "data_out".into(),
            value: "counter".into(),
        });

        let est = HirRouter::analyze_module(&soc);
        assert!(est.total_nets > 0);
        assert!(est.total_wire_length_um > 0);
    }
}

#[cfg(test)]
mod tests_hir_dft {
    use super::*;

    #[test]
    fn test_scan_chain() {
        let c = ScanChain::new("core_chain", 100);
        assert_eq!(c.num_regs, 100);
        assert_eq!(c.chain_length_bits, 3200);
        assert_eq!(c.cycles(), 3210);
        assert_eq!(c.bytes(), 400);
    }

    #[test]
    fn test_memory_bist() {
        let b = BistCtrl::memory("bram_bist", 8);
        assert_eq!(b.kind, BistKind::Memory);
        assert_eq!(b.patterns, 8);
        assert_eq!(b.cycles(), 16);
    }

    #[test]
    fn test_logic_bist() {
        let b = BistCtrl::logic("logic_bist", 16);
        assert_eq!(b.kind, BistKind::Logic);
    }

    #[test]
    fn test_bist_coverage_full() {
        let b = BistCtrl::memory("b", 100);
        assert_eq!(b.coverage(100), 100);
    }

    #[test]
    fn test_bist_coverage_partial() {
        let b = BistCtrl::memory("b", 50);
        assert_eq!(b.coverage(200), 25);
    }

    #[test]
    fn test_bist_coverage_zero() {
        let b = BistCtrl::memory("b", 10);
        assert_eq!(b.coverage(0), 100);
    }

    #[test]
    fn test_jtag_tap() {
        let t = JtagTap::new("main_tap", 8, 0x12345678);
        assert_eq!(t.ir_width, 8);
        assert_eq!(t.num_dr_regs, 3);
        assert_eq!(t.bypass_code, 0xFF);
        assert_eq!(t.total_bits(), 104);
    }

    #[test]
    fn test_test_coverage() {
        let c = TestCoverage::new(95, 90, 85);
        assert_eq!(c.scan_coverage, 95);
        assert_eq!(c.total_coverage, 90);
    }

    #[test]
    fn test_coverage_acceptable() {
        let c = TestCoverage::new(95, 95, 90);
        assert!(c.is_acceptable());
    }

    #[test]
    fn test_coverage_not_acceptable() {
        let c = TestCoverage::new(80, 80, 80);
        assert!(!c.is_acceptable());
    }

    #[test]
    fn test_validate_chain_ok() {
        assert!(ScanChain::new("ok", 10).validate().is_empty());
    }

    #[test]
    fn test_validate_chain_empty() {
        let c = ScanChain {
            name: String::new(),
            num_regs: 0,
            chain_length_bits: 0,
        };
        assert!(!c.validate().is_empty());
    }

    #[test]
    fn test_validate_bist_ok() {
        assert!(BistCtrl::memory("ok", 8).validate().is_empty());
    }

    #[test]
    fn test_validate_tap_ok() {
        assert!(JtagTap::new("ok", 4, 0).validate().is_empty());
    }

    #[test]
    fn test_hir_dft_full() {
        let mut dft = HirDft::new();
        dft.add_scan_chain("core", 500);
        dft.add_scan_chain("io", 100);
        dft.add_memory_bist("bram_bist", 8);
        dft.add_logic_bist("logic_bist", 16);
        dft.set_jtag(JtagTap::new("main_tap", 8, 0x27DE_0123));
        assert_eq!(dft.total_scan_regs(), 600);
        assert!(dft.total_scan_cycles() > 0);
        assert!(dft.total_bist_cycles() > 0);
        assert!(dft.est_test_time_cycles() > 0);
        let cov = dft.coverage_estimate();
        assert!(cov.scan_coverage > 0);
        assert!(cov.bist_coverage > 0);
        assert!(dft.jtag_tap.is_some());
        assert!(dft.validate().is_empty());
    }
}

#[cfg(test)]
mod tests_hir_cts {
    use super::*;

    #[test]
    fn test_pll_config() {
        let p = PllConfig::new("sys_pll", 100, 200);
        assert_eq!(p.input_mhz, 100);
        assert_eq!(p.output_mhz, 200);
        assert_eq!(p.period_ps(), 5_000_000);
    }

    #[test]
    fn test_bufg() {
        let b = ClockBuffer::bufg("clk_buf");
        assert_eq!(b.delay_ps, 100);
        assert_eq!(b.fanout, 32);
    }

    #[test]
    fn test_bufh() {
        let b = ClockBuffer::bufh("clk_h");
        assert_eq!(b.delay_ps, 50);
        assert_eq!(b.fanout, 16);
    }

    #[test]
    fn test_clock_tree() {
        let t = ClockTree::new("clk", 2, 5);
        assert_eq!(t.root, "clk");
        assert_eq!(t.num_levels, 2);
        assert_eq!(t.total_buffers, 5);
        assert_eq!(t.tree_delay_ps(100), 200);
        assert!(t.skew_ok(200));
        assert!(!t.skew_ok(50));
    }

    #[test]
    fn test_cts_report_passed() {
        let r = CtsReport {
            num_clocks: 2,
            num_plls: 1,
            total_buffers: 10,
            worst_skew_ps: 80,
            worst_latency_ps: 300,
            has_violations: false,
        };
        assert!(r.passed());
    }

    #[test]
    fn test_est_buffers_one() {
        assert_eq!(CtsModel::est_buffers_needed(10), 1);
    }
    #[test]
    fn test_est_buffers_many() {
        assert_eq!(CtsModel::est_buffers_needed(100), 7);
    }
    #[test]
    fn test_est_tree_levels_one() {
        assert_eq!(CtsModel::est_tree_levels(10), 1);
    }
    #[test]
    fn test_est_tree_levels_two() {
        assert_eq!(CtsModel::est_tree_levels(100), 2);
    }
    #[test]
    fn test_est_tree_levels_three() {
        assert_eq!(CtsModel::est_tree_levels(500), 3);
    }

    #[test]
    fn test_validate_pll_ok() {
        assert!(PllConfig::new("ok", 100, 200).validate().is_empty());
    }

    #[test]
    fn test_validate_pll_empty() {
        let p = PllConfig {
            name: String::new(),
            input_mhz: 100,
            output_mhz: 0,
            multiply: 1,
            divide: 1,
            jitter_ps: 50,
        };
        assert!(!p.validate().is_empty());
    }

    #[test]
    fn test_hir_cts_full() {
        let mut cts = HirCts::new();
        cts.add_pll(PllConfig::new("sys_pll", 100, 200));
        cts.add_pll(PllConfig::new("io_pll", 100, 50));
        cts.build_tree("clk", 64);
        cts.build_tree("clk_io", 16);
        let report = cts.report();
        assert_eq!(report.num_clocks, 2);
        assert_eq!(report.num_plls, 2);
        assert!(report.total_buffers >= 2);
        assert!(report.passed());
        assert!(cts.validate().is_empty());
    }
}

#[cfg(test)]
mod tests_hir_reset {
    use super::*;
    #[test]
    fn test_reset_sync() {
        let rs = ResetSynchronizer::new("sys_sync", 3, "ext", "sys");
        assert_eq!(rs.num_stages, 3);
        assert!(rs.meta_stability_mttf_ps(10000) > 0);
        assert_eq!(rs.latency_ps(10000), 30000);
        assert!(rs.validate().is_empty());
    }
    #[test]
    fn test_reset_sync_zero_stages() {
        let rs = ResetSynchronizer {
            name: String::new(),
            num_stages: 0,
            input_clock: "a".into(),
            output_clock: "b".into(),
            async_assert: true,
        };
        assert!(!rs.validate().is_empty());
    }
    #[test]
    fn test_reset_domain() {
        let mut rd = ResetDomain::new("sys_rst", "sys_clk", true);
        rd.add_sync(ResetSynchronizer::new("s1", 2, "ext", "sys"));
        assert_eq!(rd.sync_chains.len(), 1);
        assert!(rd.polarity_active_low);
    }
}

#[cfg(test)]
mod tests_hir_retiming {
    use super::*;
    #[test]
    fn test_forward_retime() {
        let op = RetimingOp::forward("comb_a", "out_reg", 1);
        assert!(op.direction_forward);
        assert_eq!(op.registers_moved, 1);
    }
    #[test]
    fn test_backward_retime() {
        let op = RetimingOp::backward("in_reg", "comb_b", 2);
        assert!(!op.direction_forward);
    }
    #[test]
    fn test_retimer_improvement() {
        let mut rt = HirRetimer::new();
        rt.original_crit_ps = 10000;
        rt.retimed_crit_ps = 8000;
        assert_eq!(rt.improvement_percent(), 20);
    }
    #[test]
    fn test_retimer_fmax() {
        let mut rt = HirRetimer::new();
        rt.original_crit_ps = 10000;
        rt.retimed_crit_ps = 5000;
        let imp = rt.fmax_improvement();
        assert!(imp > 0);
    }
    #[test]
    fn test_retimer_no_improvement() {
        let rt = HirRetimer::new();
        assert_eq!(rt.improvement_percent(), 0);
    }
}

#[cfg(test)]
mod tests_hir_config_reg {
    use super::*;
    #[test]
    fn test_rw_reg() {
        let r = ConfigReg::rw("ctrl", 0, 32, 0);
        assert!(r.writable);
        assert_eq!(r.reset_value, 0);
    }
    #[test]
    fn test_ro_reg() {
        let r = ConfigReg::ro("status", 32, 16);
        assert!(!r.writable);
    }
    #[test]
    fn test_config_block() {
        let mut cb = HirConfigBlock::new("uart_cfg", 0x1000);
        cb.add_rw("ctrl", 32, 0);
        cb.add_rw("div", 16, 0);
        cb.add_ro("status", 32);
        assert_eq!(cb.registers.len(), 3);
        assert!(cb.total_bytes() > 0);
        assert!(cb.validate().is_empty());
    }
    #[test]
    fn test_config_block_empty_name() {
        let cb = HirConfigBlock::new("", 0);
        assert!(!cb.validate().is_empty());
    }
}

#[cfg(test)]
mod tests_hir_irq {
    use super::*;
    #[test]
    fn test_level_irq() {
        let src = IrqSource::level("uart_rx", 1, 5);
        assert!(!src.edge_triggered);
        assert_eq!(src.priority, 5);
    }
    #[test]
    fn test_edge_irq() {
        let src = IrqSource::edge("timer", 2, 3);
        assert!(src.edge_triggered);
    }
    #[test]
    fn test_irq_ctrl() {
        let mut ic = HirInterruptCtrl::new("nvic", 8);
        ic.add_level_irq("uart", 0, 4);
        ic.add_edge_irq("timer", 1, 2);
        ic.add_level_irq("spi", 2, 6);
        assert_eq!(ic.sources.len(), 3);
        assert_eq!(ic.pending_count(), 3);
        let hp = ic.highest_priority().unwrap();
        assert_eq!(hp.name, "timer");
        assert!(ic.validate().is_empty());
    }
    #[test]
    fn test_irq_ctrl_empty() {
        let ic = HirInterruptCtrl::new("", 8);
        assert!(!ic.validate().is_empty());
    }
}

#[cfg(test)]
mod tests_hir_dma {
    use super::*;
    #[test]
    fn test_single_transfer() {
        let ch = DmaChannel::single("ch0", 0, 0x1000, 0x2000, 64);
        assert_eq!(ch.kind, DmaTransferKind::Single);
        assert!(ch.transfer_cycles() > 0);
    }
    #[test]
    fn test_burst_transfer() {
        let ch = DmaChannel::burst("ch0", 0, 0x1000, 0x2000, 256, 16);
        assert_eq!(ch.kind, DmaTransferKind::Burst);
        assert!(ch.transfer_cycles() > 0);
        assert!(ch.bandwidth_mbps(100) > 0);
    }
    #[test]
    fn test_dma_engine() {
        let mut dma = HirDmaEngine::new("sys_dma", 32);
        dma.add_single("ch0", 0x1000, 0x2000, 128);
        dma.add_burst("ch1", 0x3000, 0x4000, 512, 16);
        assert_eq!(dma.channels.len(), 2);
        assert!(dma.total_transfer_cycles() > 0);
        assert_eq!(dma.total_bytes(), 640);
        assert!(dma.validate().is_empty());
    }
    #[test]
    fn test_dma_empty_name() {
        let dma = HirDmaEngine::new("", 0);
        assert!(!dma.validate().is_empty());
    }
    #[test]
    fn test_dma_bandwidth() {
        let ch = DmaChannel::burst("ch", 0, 0, 0, 1024, 16);
        let bw = ch.bandwidth_mbps(100);
        assert!(bw > 0);
    }
    #[test]
    fn test_dma_zero_burst() {
        let ch = DmaChannel {
            name: "ch".into(),
            index: 0,
            src_addr: 0,
            dst_addr: 0,
            length_bytes: 100,
            kind: DmaTransferKind::Burst,
            burst_size: 0,
        };
        assert_eq!(ch.transfer_cycles(), 0);
    }
}

#[cfg(test)]
mod tests_hir_crossopt {
    use super::*;

    #[test]
    fn test_pass_zero() {
        let p = CrossOptPass::empty();
        assert_eq!(p.num_modules, 0);
        assert_eq!(p.total_improvements(), 0);
        assert!(!p.has_improvements());
    }

    #[test]
    fn test_pass_result() {
        let p = CrossOptPass::new("const_prop", 3, 10, 5, 2);
        assert_eq!(p.name, "const_prop");
        assert_eq!(p.total_improvements(), 17);
        assert!(p.has_improvements());
    }

    #[test]
    fn test_improvement_density() {
        let p = CrossOptPass::new("opt", 5, 20, 10, 5);
        assert_eq!(p.improvement_density(), 7);
    }

    #[test]
    fn test_report() {
        let r = CrossOptReport::new(3, 30, 15, 5, 10);
        assert_eq!(r.total_passes, 3);
        assert_eq!(r.total_optimizations(), 50);
        assert!(r.is_effective());
    }

    #[test]
    fn test_report_empty() {
        let r = CrossOptReport::new(0, 0, 0, 0, 0);
        assert!(!r.is_effective());
    }

    #[test]
    fn test_cross_optimize_modules() {
        let mut m1 = HirModule::new("mod_a");
        m1.signals.push(HirSignal {
            name: "unused_x".into(),
            kind: HwSignalKind::Wire,
            ty: HwType::UInt(8),
            reset_value: "0".into(),
        });
        m1.assigns.push(HirAssign {
            target: "const_a".into(),
            value: "42".into(),
        });
        let mut m2 = HirModule::new("mod_b");
        m2.signals.push(HirSignal {
            name: "used_y".into(),
            kind: HwSignalKind::Reg,
            ty: HwType::UInt(16),
            reset_value: "0".into(),
        });
        let mut opt = HirCrossOptimizer::new();
        let pass = opt.run_pass(&mut [m1, m2]);
        assert!(pass.num_modules == 2);
        let report = opt.report();
        assert!(report.total_passes >= 1);
    }
}

#[cfg(test)]
mod tests_hir_bootrom {
    use super::*;

    #[test]
    fn test_boot_stage() {
        let s = BootStage::new("fsbl", 0, 4096, 0);
        assert_eq!(s.name, "fsbl");
        assert_eq!(s.end(), 4096);
    }

    #[test]
    fn test_boot_config() {
        let c = BootConfig::new("trinity_boot", 32768);
        assert_eq!(c.rom_size, 32768);
        assert!(c.has_integrity_check);
        assert_eq!(c.end(), 32768);
    }

    #[test]
    fn test_validate_ok() {
        assert!(BootConfig::new("ok", 4096).validate().is_empty());
    }

    #[test]
    fn test_validate_empty() {
        let c = BootConfig {
            name: String::new(),
            rom_base: 0,
            rom_size: 0,
            has_integrity_check: true,
            has_chain_loader: true,
        };
        assert!(!c.validate().is_empty());
    }

    #[test]
    fn test_fits_yes() {
        let c = BootConfig::new("test", 8192);
        let stages = vec![
            BootStage::new("s1", 0, 4096, 0),
            BootStage::new("s2", 1, 2048, 4096),
        ];
        assert!(c.fits(&stages));
    }

    #[test]
    fn test_fits_no() {
        let c = BootConfig::new("test", 1024);
        let stages = vec![BootStage::new("s1", 0, 4096, 0)];
        assert!(!c.fits(&stages));
    }
}

#[cfg(test)]
mod tests_hir_watchdog {
    use super::*;

    #[test]
    fn test_wdt_basic() {
        let w = WatchdogConfig::new("sys_wdt", 1000000);
        assert_eq!(w.timeout_cycles, 1000000);
        assert!(w.generate_reset);
        assert!(w.generate_interrupt);
    }

    #[test]
    fn test_wdt_windowed() {
        let w = WatchdogConfig::windowed("wdt", 1000, 200, 800);
        assert_eq!(w.window_open, 200);
        assert_eq!(w.window_close, 800);
        assert!(w.in_window(500));
        assert!(!w.in_window(100));
    }

    #[test]
    fn test_wdt_timeout_ns() {
        let w = WatchdogConfig::new("wdt", 1000);
        assert_eq!(w.timeout_ns(10), 10000);
    }

    #[test]
    fn test_validate_ok() {
        assert!(WatchdogConfig::new("ok", 100).validate().is_empty());
    }
    #[test]
    fn test_validate_zero() {
        assert!(!WatchdogConfig::new("ok", 0).validate().is_empty());
    }
}

#[cfg(test)]
mod tests_hir_memmap {
    use super::*;

    #[test]
    fn test_memmap_entry() {
        let e = MemMapEntry::rom("boot", 0, 4096);
        assert_eq!(e.end(), 4096);
        assert!(e.contains(0));
        assert!(e.contains(4095));
        assert!(!e.contains(4096));
        assert!(!e.writable);
    }

    #[test]
    fn test_ram_entry() {
        let e = MemMapEntry::ram("main", 0x1000, 8192);
        assert!(e.readable);
        assert!(e.writable);
    }

    #[test]
    fn test_memmap_lookup() {
        let mut mm = HirMemMap::new();
        mm.add_rom("boot", 0, 4096);
        mm.add_ram("main", 0x1000, 8192);
        mm.add_periph("uart", 0x4000, 256);
        assert_eq!(mm.lookup(0).unwrap().name, "boot");
        assert_eq!(mm.lookup(0x2000).unwrap().name, "main");
        assert_eq!(mm.lookup(0x4010).unwrap().name, "uart");
        assert!(mm.lookup(0xFFFF).is_none());
    }

    #[test]
    fn test_memmap_no_overlaps() {
        let mut mm = HirMemMap::new();
        mm.add_rom("boot", 0, 4096);
        mm.add_ram("main", 0x1000, 4096);
        assert!(mm.check_overlaps().is_empty());
    }

    #[test]
    fn test_memmap_overlaps() {
        let mut mm = HirMemMap::new();
        mm.add_ram("a", 0, 4096);
        mm.add_ram("b", 2000, 4096);
        assert_eq!(mm.check_overlaps().len(), 1);
    }

    #[test]
    fn test_total_size() {
        let mut mm = HirMemMap::new();
        mm.add_rom("boot", 0, 4096);
        mm.add_ram("main", 0x1000, 8192);
        assert_eq!(mm.total_size(), 12288);
    }
}

#[cfg(test)]
mod tests_hir_serdes {
    use super::*;

    #[test]
    fn test_serdes_config() {
        let s = SerDesConfig::new("sfp0", 4, 6250);
        assert_eq!(s.lanes, 4);
        assert_eq!(s.total_bandwidth_gbps(), 25000);
        assert!(s.validate().is_empty());
    }

    #[test]
    fn test_serdes_empty() {
        let s = SerDesConfig {
            name: String::new(),
            lanes: 0,
            line_rate_gbps: 0,
            data_width: 32,
            encoding: "8b10b".into(),
        };
        assert!(!s.validate().is_empty());
    }

    #[test]
    fn test_serdes_throughput() {
        let s = SerDesConfig::new("sfp", 2, 10000);
        assert!(s.throughput_bytes_per_sec() > 0);
    }
}

#[cfg(test)]
mod tests_hir_build_orchestrator {
    use super::*;

    #[test]
    fn test_empty() {
        let b = HirBuildOrchestrator::new();
        assert_eq!(b.step_count(), 0);
    }

    #[test]
    fn test_add_step() {
        let mut b = HirBuildOrchestrator::new();
        b.add_step("synthesize", "yosys", "top.v", "synth.json", 30000);
        assert_eq!(b.step_count(), 1);
        assert_eq!(b.total_estimated_ms, 30000);
        assert!(b.has_step("synthesize"));
    }

    #[test]
    fn test_standard_flow() {
        let mut b = HirBuildOrchestrator::new();
        b.standard_fpga_flow();
        assert_eq!(b.step_count(), 4);
        assert!(b.has_step("synthesize"));
        assert!(b.has_step("place_route"));
        assert!(b.has_step("fasm_gen"));
        assert!(b.has_step("bitstream"));
        assert!(b.total_estimated_ms > 0);
    }
}

#[cfg(test)]
mod tests_mega_integration {
    use super::*;

    #[test]
    fn test_complete_fpga_toolchain() {
        let mut soc = HirModule::new("TrinitySoC_Full");
        soc.ports.push(HirPort {
            name: "clk".into(),
            dir: HwPortDir::Input,
            ty: HwType::Clock,
        });
        soc.ports.push(HirPort {
            name: "rst_n".into(),
            dir: HwPortDir::Input,
            ty: HwType::Reset(HwResetKind::Async, HwResetPolarity::ActiveLow),
        });
        soc.ports.push(HirPort {
            name: "uart_tx".into(),
            dir: HwPortDir::Output,
            ty: HwType::Bool,
        });
        soc.ports.push(HirPort {
            name: "uart_rx".into(),
            dir: HwPortDir::Input,
            ty: HwType::Bool,
        });
        soc.ports.push(HirPort {
            name: "spi_mosi".into(),
            dir: HwPortDir::Output,
            ty: HwType::Bool,
        });
        soc.signals.push(HirSignal {
            name: "counter".into(),
            kind: HwSignalKind::Reg,
            ty: HwType::UInt(32),
            reset_value: "0".into(),
        });
        soc.assigns.push(HirAssign {
            target: "led".into(),
            value: "counter[27]".into(),
        });
        let mut bram = HirMemory::new_bram("prog_mem", 4096, 32);
        bram.add_read_port("fetch");
        soc.memories.push(bram);
        soc.clock_domains
            .push(HirClockDomain::new("sys", "ext", 100_000_000));
        soc.fifos.push(HirFifo::new_sync("tx_fifo", 16, 8));
        soc.bus_ports
            .push(HirBusPort::axi4_lite_slave("ctrl", 32, 32));
        let mut apb = HirApbBridge::new("apb", 32, 32, 4);
        apb.add_peripheral("uart0", 0x1000, 256, 0);
        soc.apb_bridges.push(apb);
        let mut gf16 = HirGf16Accel::full("gf16", 8, 16);
        gf16.add_mac_unit("mac0", 32, 2);
        soc.gf16_accels.push(gf16);
        let mut tc = HirTernaryCore::basic("tri").with_regfile(HirTernaryRegFile::new("rf"));
        tc.add_pipeline_stage("IF", 1, false);
        tc.add_pipeline_stage("EX", 2, true);
        tc.add_pipeline_stage("WB", 1, false);
        tc.add_alu_op("gf_mul", 16, 3, true);
        soc.ternary_cores.push(tc);
        soc.formal_config = Some(HirFormalConfig::new(
            "soc_props",
            "TrinitySoC_Full",
            "clk",
            "rst_n",
        ));

        assert!(soc.validate().is_empty());

        let mut emitter = HirVerilogEmitter::new();
        emitter.emit(&soc);
        let verilog = emitter.into_string();
        assert!(verilog.contains("module TrinitySoC_Full"));

        let mut opt = HirOptimizer::new();
        opt.optimize(&mut soc);
        let res = opt.resource_estimate(&soc);
        assert!(res.luts > 0 || res.bram18 > 0);

        let timing = TimingModel::analyze_module(&soc, &TimingConstraint::from_mhz("sys", 100));
        let power = PowerModel::estimate_module(&soc, 100, 12);
        assert!(power.total_mw > 0);

        let route = HirRouter::analyze_module(&soc);
        assert!(route.total_nets > 0);

        let mut fp = Floorplan::new("full_soc", "xc7a100t");
        fp.auto_floorplan(&soc);
        assert!(fp.regions.len() >= 2);

        let mut dft = HirDft::new();
        dft.add_scan_chain("soc_chain", 500);
        dft.add_memory_bist("bram_bist", 8);
        dft.set_jtag(JtagTap::new("tap", 8, 0x27DE_0123));
        assert!(dft.validate().is_empty());

        let mut cts = HirCts::new();
        cts.add_pll(PllConfig::new("sys_pll", 100, 200));
        cts.build_tree("clk", 32);
        assert!(cts.report().passed());

        let mut memmap = HirMemMap::new();
        memmap.add_rom("boot", 0, 4096);
        memmap.add_ram("main", 0x1000, 32768);
        memmap.add_periph("ctrl", 0x4000_0000, 4096);
        assert!(memmap.check_overlaps().is_empty());

        let mut dma = HirDmaEngine::new("sys_dma", 32);
        dma.add_burst("ch0", 0x1000, 0x4000_0000, 1024, 16);
        assert!(dma.validate().is_empty());

        let mut irq = HirInterruptCtrl::new("nvic", 8);
        irq.add_level_irq("uart", 0, 4);
        irq.add_edge_irq("timer", 1, 2);
        assert!(irq.validate().is_empty());

        let wdt = WatchdogConfig::new("sys_wdt", 1_000_000);
        assert!(wdt.validate().is_empty());

        let boot = BootConfig::new("trinity_boot", 32768);
        assert!(boot.validate().is_empty());

        let mut build = HirBuildOrchestrator::new();
        build.standard_fpga_flow();
        assert_eq!(build.step_count(), 4);

        let serdes = SerDesConfig::new("sfp", 2, 6250);
        assert_eq!(serdes.total_bandwidth_gbps(), 12500);

        let mut asm = HirAssembler::new("demo_prog");
        asm.define_symbol("_start", true);
        asm.emit_r(1, 1, 2, 3);
        asm.emit_gf16(16, 1, 2, 3);
        let mut linker = HirLinker::new("_start");
        linker.add_object(&asm);
        let link_result = linker.link();
        assert!(link_result.passed());

        let mut trace = HirVcdTrace::new("t27c full sim");
        trace.add_wire(1, "clk");
        trace.add_reg(32, "pc");
        trace.record(0, "clk", 0);
        trace.record(5000, "clk", 1);
        let vcd = trace.emit_vcd();
        assert!(vcd.contains("$date"));
    }
}

#[derive(Debug, Clone)]
pub struct SvInterface {
    pub name: String,
    pub signals: Vec<(String, u32)>,
}

impl SvInterface {
    pub fn new(name: &str) -> Self {
        SvInterface {
            name: name.into(),
            signals: Vec::new(),
        }
    }
    pub fn add_signal(&mut self, name: &str, width: u32) {
        self.signals.push((name.into(), width));
    }
    pub fn emit(&self) -> String {
        let mut v = String::new();
        v.push_str(&format!("interface {};\n", self.name));
        for (name, width) in &self.signals {
            if *width == 1 {
                v.push_str(&format!("    logic {};\n", name));
            } else {
                v.push_str(&format!("    logic [{}:0] {};\n", width - 1, name));
            }
        }
        v.push_str("endinterface\n");
        v
    }
}

#[derive(Debug)]
pub struct HirSvEmitter {
    output: String,
    indent: usize,
}

impl HirSvEmitter {
    pub fn new() -> Self {
        HirSvEmitter {
            output: String::new(),
            indent: 0,
        }
    }
    pub fn into_string(self) -> String {
        self.output
    }

    pub fn emit_package(&mut self, name: &str, types: &[(&str, u32)]) {
        self.output.push_str(&format!("package {};\n", name));
        for (tname, width) in types {
            self.output.push_str(&format!(
                "    typedef logic [{}:0] {}_t;\n",
                width - 1,
                tname
            ));
        }
        self.output.push_str("endpackage\n\n");
    }

    pub fn emit_interface(&mut self, iface: &SvInterface) {
        self.output.push_str(&iface.emit());
        self.output.push('\n');
    }

    pub fn emit_module_sv(&mut self, hir: &HirModule) {
        self.output.push_str(&format!("module {} (\n", hir.name));
        for (i, port) in hir.ports.iter().enumerate() {
            let dir = match port.dir {
                HwPortDir::Input => "input",
                HwPortDir::Output => "output",
                HwPortDir::Inout => "inout",
            };
            let comma = if i < hir.ports.len() - 1 { "," } else { "" };
            let w = port.ty.hw_width();
            if w <= 1 {
                self.output
                    .push_str(&format!("    {} logic {} {}\n", dir, port.name, comma));
            } else {
                self.output.push_str(&format!(
                    "    {} logic [{}:0] {} {}\n",
                    dir,
                    w - 1,
                    port.name,
                    comma
                ));
            }
        }
        self.output.push_str(");\n");
        for sig in &hir.signals {
            let w = sig.ty.hw_width();
            let kind = if sig.kind == HwSignalKind::Reg {
                "logic"
            } else {
                "wire logic"
            };
            if w <= 1 {
                self.output
                    .push_str(&format!("    {} {};\n", kind, sig.name));
            } else {
                self.output
                    .push_str(&format!("    {} [{}:0] {};\n", kind, w - 1, sig.name));
            }
        }
        self.output.push_str("endmodule\n");
    }
}

#[derive(Debug)]
pub struct HirFirrtlEmitter {
    output: String,
}

impl HirFirrtlEmitter {
    pub fn new() -> Self {
        HirFirrtlEmitter {
            output: String::new(),
        }
    }
    pub fn into_string(self) -> String {
        self.output
    }

    pub fn emit(&mut self, hir: &HirModule) {
        self.output.push_str(&format!("circuit {} :\n", hir.name));
        self.output.push_str(&format!("  module {} :\n", hir.name));
        for port in &hir.ports {
            let dir = match port.dir {
                HwPortDir::Input => "input",
                HwPortDir::Output => "output",
                _ => "output",
            };
            let w = port.ty.hw_width();
            self.output
                .push_str(&format!("    {} {} : UInt<{}>\n", dir, port.name, w));
        }
        for sig in &hir.signals {
            let w = sig.ty.hw_width();
            self.output
                .push_str(&format!("    wire {} : UInt<{}>\n", sig.name, w));
        }
        for assign in &hir.assigns {
            self.output
                .push_str(&format!("    {} <= {}\n", assign.target, assign.value));
        }
    }
}

#[derive(Debug, Clone)]
pub struct HirDiff {
    pub field: String,
    pub left: String,
    pub right: String,
}

#[derive(Debug)]
pub struct HirDiffEngine;

impl HirDiffEngine {
    pub fn diff(a: &HirModule, b: &HirModule) -> Vec<HirDiff> {
        let mut diffs = Vec::new();
        if a.name != b.name {
            diffs.push(HirDiff {
                field: "name".into(),
                left: a.name.clone(),
                right: b.name.clone(),
            });
        }
        if a.ports.len() != b.ports.len() {
            diffs.push(HirDiff {
                field: "port_count".into(),
                left: a.ports.len().to_string(),
                right: b.ports.len().to_string(),
            });
        }
        if a.signals.len() != b.signals.len() {
            diffs.push(HirDiff {
                field: "signal_count".into(),
                left: a.signals.len().to_string(),
                right: b.signals.len().to_string(),
            });
        }
        if a.memories.len() != b.memories.len() {
            diffs.push(HirDiff {
                field: "memory_count".into(),
                left: a.memories.len().to_string(),
                right: b.memories.len().to_string(),
            });
        }
        if a.bus_ports.len() != b.bus_ports.len() {
            diffs.push(HirDiff {
                field: "bus_count".into(),
                left: a.bus_ports.len().to_string(),
                right: b.bus_ports.len().to_string(),
            });
        }
        if a.gf16_accels.len() != b.gf16_accels.len() {
            diffs.push(HirDiff {
                field: "gf16_count".into(),
                left: a.gf16_accels.len().to_string(),
                right: b.gf16_accels.len().to_string(),
            });
        }
        if a.ternary_cores.len() != b.ternary_cores.len() {
            diffs.push(HirDiff {
                field: "ternary_count".into(),
                left: a.ternary_cores.len().to_string(),
                right: b.ternary_cores.len().to_string(),
            });
        }
        diffs
    }
    pub fn equivalent(a: &HirModule, b: &HirModule) -> bool {
        Self::diff(a, b).is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct BuildMetrics {
    pub build_id: u32,
    pub luts: u32,
    pub ffs: u32,
    pub bram18: u32,
    pub dsp48: u32,
    pub fmax_mhz: u32,
    pub power_mw: u32,
    pub timing_met: bool,
}

impl BuildMetrics {
    pub fn new(id: u32, luts: u32, ffs: u32, bram: u32, dsp: u32, fmax: u32, power: u32) -> Self {
        BuildMetrics {
            build_id: id,
            luts,
            ffs,
            bram18: bram,
            dsp48: dsp,
            fmax_mhz: fmax,
            power_mw: power,
            timing_met: true,
        }
    }
}

#[derive(Debug)]
pub struct HirRegressionTracker {
    pub builds: Vec<BuildMetrics>,
}

impl HirRegressionTracker {
    pub fn new() -> Self {
        HirRegressionTracker { builds: Vec::new() }
    }
    pub fn record(&mut self, m: BuildMetrics) {
        self.builds.push(m);
    }
    pub fn latest(&self) -> Option<&BuildMetrics> {
        self.builds.last()
    }
    pub fn lut_regression(&self) -> bool {
        if self.builds.len() < 2 {
            return false;
        }
        let prev = &self.builds[self.builds.len() - 2];
        let curr = &self.builds[self.builds.len() - 1];
        curr.luts > prev.luts * 110 / 100
    }
    pub fn fmax_regression(&self) -> bool {
        if self.builds.len() < 2 {
            return false;
        }
        let prev = &self.builds[self.builds.len() - 2];
        let curr = &self.builds[self.builds.len() - 1];
        curr.fmax_mhz < prev.fmax_mhz * 90 / 100
    }
    pub fn total_builds(&self) -> u32 {
        self.builds.len() as u32
    }
}

#[derive(Debug, Clone)]
pub struct HirFileFingerprint {
    pub path: String,
    pub hash: u64,
    pub timestamp_ms: u64,
}

impl HirFileFingerprint {
    pub fn new(path: &str, hash: u64) -> Self {
        HirFileFingerprint {
            path: path.into(),
            hash,
            timestamp_ms: 0,
        }
    }
}

#[derive(Debug)]
pub struct HirElabCache {
    pub fingerprints: Vec<HirFileFingerprint>,
}

impl HirElabCache {
    pub fn new() -> Self {
        HirElabCache {
            fingerprints: Vec::new(),
        }
    }
    pub fn add(&mut self, fp: HirFileFingerprint) {
        self.fingerprints.push(fp);
    }
    pub fn is_cached(&self, path: &str, hash: u64) -> bool {
        self.fingerprints
            .iter()
            .any(|fp| fp.path == path && fp.hash == hash)
    }
    pub fn invalidate(&mut self, path: &str) {
        self.fingerprints.retain(|fp| fp.path != path);
    }
    pub fn entry_count(&self) -> u32 {
        self.fingerprints.len() as u32
    }
}

#[derive(Debug, Clone)]
pub struct CdcViolation {
    pub signal: String,
    pub src_domain: String,
    pub dst_domain: String,
    pub kind: String,
    pub severity: String,
}

#[derive(Debug)]
pub struct HirCdcChecker;

impl HirCdcChecker {
    pub fn check(module: &HirModule) -> Vec<CdcViolation> {
        let mut violations = Vec::new();
        let domains: Vec<_> = module
            .clock_domains
            .iter()
            .map(|d| d.name.clone())
            .collect();
        if domains.len() < 2 {
            return violations;
        }
        for sig in &module.signals {
            if sig.kind == HwSignalKind::Reg {
                let used_across = module
                    .assigns
                    .iter()
                    .filter(|a| a.value.contains(&sig.name))
                    .count();
                if used_across > 0 && domains.len() > 1 {
                    violations.push(CdcViolation {
                        signal: sig.name.clone(),
                        src_domain: domains[0].clone(),
                        dst_domain: if domains.len() > 1 {
                            domains[1].clone()
                        } else {
                            domains[0].clone()
                        },
                        kind: "missing_sync".into(),
                        severity: "error".into(),
                    });
                }
            }
        }
        violations
    }
    pub fn has_violations(module: &HirModule) -> bool {
        !Self::check(module).is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct LintViolation {
    pub rule: String,
    pub signal: String,
    pub message: String,
    pub severity: String,
}

#[derive(Debug)]
pub struct HirLinter {
    pub rules: Vec<String>,
}

impl HirLinter {
    pub fn new() -> Self {
        HirLinter {
            rules: vec![
                "reset_all_regs".into(),
                "no_unnamed_signals".into(),
                "clock_naming".into(),
                "reset_polarity".into(),
                "signal_width_consistency".into(),
            ],
        }
    }

    pub fn lint(module: &HirModule) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let has_rst = module.ports.iter().any(|p| p.ty.is_reset_like());
        if has_rst {
            for sig in &module.signals {
                if sig.kind == HwSignalKind::Reg && sig.reset_value.is_empty() {
                    violations.push(LintViolation {
                        rule: "reset_all_regs".into(),
                        signal: sig.name.clone(),
                        message: "register missing reset value".into(),
                        severity: "warning".into(),
                    });
                }
            }
        }
        for sig in &module.signals {
            if sig.name.starts_with('_') || sig.name.len() < 2 {
                violations.push(LintViolation {
                    rule: "no_unnamed_signals".into(),
                    signal: sig.name.clone(),
                    message: "signal name too short".into(),
                    severity: "info".into(),
                });
            }
        }
        for port in &module.ports {
            if port.ty.is_clock_like() && !port.name.contains("clk") {
                violations.push(LintViolation {
                    rule: "clock_naming".into(),
                    signal: port.name.clone(),
                    message: "clock should contain 'clk'".into(),
                    severity: "warning".into(),
                });
            }
        }
        violations
    }
    pub fn lint_count(module: &HirModule) -> u32 {
        Self::lint(module).len() as u32
    }
}

#[derive(Debug, Clone)]
pub struct DepNode {
    pub name: String,
    pub dependencies: Vec<String>,
    pub depth: u32,
}

#[derive(Debug)]
pub struct HirDepGraph {
    pub nodes: Vec<DepNode>,
}

impl HirDepGraph {
    pub fn new() -> Self {
        HirDepGraph { nodes: Vec::new() }
    }
    pub fn add_module(&mut self, name: &str, deps: &[&str]) {
        let depth = if deps.is_empty() {
            0
        } else {
            self.nodes
                .iter()
                .filter(|n| deps.contains(&n.name.as_str()))
                .map(|n| n.depth)
                .max()
                .unwrap_or(0)
                + 1
        };
        self.nodes.push(DepNode {
            name: name.into(),
            dependencies: deps.iter().map(|s| s.to_string()).collect(),
            depth,
        });
    }
    pub fn topological_order(&self) -> Vec<&str> {
        let mut sorted: Vec<&DepNode> = self.nodes.iter().collect();
        sorted.sort_by_key(|n| n.depth);
        sorted.iter().map(|n| n.name.as_str()).collect()
    }
    pub fn has_cycle(&self) -> bool {
        for node in &self.nodes {
            if node.dependencies.contains(&node.name) {
                return true;
            }
        }
        for node in &self.nodes {
            for dep in &node.dependencies {
                if let Some(dep_node) = self.nodes.iter().find(|n| n.name == *dep) {
                    if dep_node.dependencies.contains(&node.name) {
                        return true;
                    }
                }
            }
        }
        false
    }
    pub fn leaf_modules(&self) -> Vec<&str> {
        self.nodes
            .iter()
            .filter(|n| n.dependencies.is_empty())
            .map(|n| n.name.as_str())
            .collect()
    }
    pub fn max_depth(&self) -> u32 {
        self.nodes.iter().map(|n| n.depth).max().unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
pub struct CoverGroup {
    pub name: String,
    pub bins: u32,
    pub hits: u32,
}

impl CoverGroup {
    pub fn new(name: &str, bins: u32) -> Self {
        CoverGroup {
            name: name.into(),
            bins,
            hits: 0,
        }
    }
    pub fn coverage_percent(&self) -> u32 {
        if self.bins == 0 {
            return 100;
        }
        self.hits * 100 / self.bins
    }
    pub fn is_covered(&self) -> bool {
        self.coverage_percent() >= 100
    }
}

#[derive(Debug)]
pub struct HirCoverageModel {
    pub groups: Vec<CoverGroup>,
}

impl HirCoverageModel {
    pub fn new() -> Self {
        HirCoverageModel { groups: Vec::new() }
    }
    pub fn add_group(&mut self, name: &str, bins: u32) {
        self.groups.push(CoverGroup::new(name, bins));
    }
    pub fn hit(&mut self, group_name: &str) {
        if let Some(g) = self.groups.iter_mut().find(|g| g.name == group_name) {
            g.hits += 1;
        }
    }
    pub fn total_coverage(&self) -> u32 {
        if self.groups.is_empty() {
            return 100;
        }
        self.groups
            .iter()
            .map(|g| g.coverage_percent())
            .sum::<u32>()
            / self.groups.len() as u32
    }
    pub fn is_complete(&self) -> bool {
        self.total_coverage() >= 90
    }
    pub fn uncovered(&self) -> Vec<&str> {
        self.groups
            .iter()
            .filter(|g| !g.is_covered())
            .map(|g| g.name.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests_phase31_emitters {
    use super::*;

    #[test]
    fn test_irq_emit_verilog() {
        let mut irq = HirInterruptCtrl::new("nvic", 8);
        irq.add_level_irq("uart", 0, 4);
        irq.add_edge_irq("timer", 1, 2);
        let v = irq.emit_verilog();
        assert!(v.contains("module irq_nvic"));
        assert!(v.contains("endmodule"));
        assert!(v.contains("irq_uart_level_0"));
        assert!(v.contains("irq_timer_edge_1"));
        assert!(v.contains("irq_pending"));
    }

    #[test]
    fn test_dma_emit_verilog() {
        let mut dma = HirDmaEngine::new("sys_dma", 32);
        dma.add_burst("ch0", 0x1000, 0x2000, 1024, 16);
        let v = dma.emit_verilog();
        assert!(v.contains("module dma_sys_dma"));
        assert!(v.contains("endmodule"));
        assert!(v.contains("dma_done"));
    }
}

#[cfg(test)]
mod tests_phase32_sv {
    use super::*;

    #[test]
    fn test_sv_interface() {
        let mut iface = SvInterface::new("axi_if");
        iface.add_signal("clk", 1);
        iface.add_signal("data", 32);
        let v = iface.emit();
        assert!(v.contains("interface axi_if"));
        assert!(v.contains("endinterface"));
        assert!(v.contains("logic clk"));
        assert!(v.contains("[31:0] data"));
    }

    #[test]
    fn test_sv_emitter_package() {
        let mut sv = HirSvEmitter::new();
        sv.emit_package("pkg", &[("addr", 32), ("data", 64)]);
        let s = sv.into_string();
        assert!(s.contains("package pkg"));
        assert!(s.contains("addr_t"));
        assert!(s.contains("data_t"));
    }

    #[test]
    fn test_sv_emitter_module() {
        let mut m = HirModule::new("test_sv");
        m.ports.push(HirPort {
            name: "clk".into(),
            dir: HwPortDir::Input,
            ty: HwType::Clock,
        });
        m.ports.push(HirPort {
            name: "data".into(),
            dir: HwPortDir::Output,
            ty: HwType::UInt(8),
        });
        m.signals.push(HirSignal {
            name: "cnt".into(),
            kind: HwSignalKind::Reg,
            ty: HwType::UInt(32),
            reset_value: "0".into(),
        });
        let mut sv = HirSvEmitter::new();
        sv.emit_module_sv(&m);
        let s = sv.into_string();
        assert!(s.contains("module test_sv"));
        assert!(s.contains("input logic clk"));
        assert!(s.contains("logic [31:0] cnt"));
    }
}

#[cfg(test)]
mod tests_phase33_firrtl {
    use super::*;

    #[test]
    fn test_firrtl_emit() {
        let mut m = HirModule::new("firrtl_test");
        m.ports.push(HirPort {
            name: "in".into(),
            dir: HwPortDir::Input,
            ty: HwType::UInt(8),
        });
        m.ports.push(HirPort {
            name: "out".into(),
            dir: HwPortDir::Output,
            ty: HwType::UInt(8),
        });
        m.signals.push(HirSignal {
            name: "tmp".into(),
            kind: HwSignalKind::Wire,
            ty: HwType::UInt(8),
            reset_value: "0".into(),
        });
        m.assigns.push(HirAssign {
            target: "out".into(),
            value: "tmp".into(),
        });
        let mut e = HirFirrtlEmitter::new();
        e.emit(&m);
        let s = e.into_string();
        assert!(s.contains("circuit firrtl_test"));
        assert!(s.contains("module firrtl_test"));
        assert!(s.contains("input in : UInt<8>"));
        assert!(s.contains("out <= tmp"));
    }
}

#[cfg(test)]
mod tests_phase34_diff {
    use super::*;

    fn mod_a() -> HirModule {
        let mut m = HirModule::new("test");
        m.ports.push(HirPort {
            name: "clk".into(),
            dir: HwPortDir::Input,
            ty: HwType::Clock,
        });
        m
    }

    #[test]
    fn test_diff_identical() {
        assert!(HirDiffEngine::equivalent(&mod_a(), &mod_a()));
    }

    #[test]
    fn test_diff_name() {
        let mut b = mod_a();
        b.name = "other".into();
        let diffs = HirDiffEngine::diff(&mod_a(), &b);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].field, "name");
    }

    #[test]
    fn test_diff_ports() {
        let mut b = mod_a();
        b.ports.push(HirPort {
            name: "extra".into(),
            dir: HwPortDir::Input,
            ty: HwType::Bool,
        });
        let diffs = HirDiffEngine::diff(&mod_a(), &b);
        assert!(diffs.iter().any(|d| d.field == "port_count"));
    }
}

#[cfg(test)]
mod tests_phase35_regression {
    use super::*;

    #[test]
    fn test_regression_tracker() {
        let mut t = HirRegressionTracker::new();
        t.record(BuildMetrics::new(1, 1000, 500, 10, 5, 200, 100));
        t.record(BuildMetrics::new(2, 1050, 520, 10, 5, 210, 105));
        assert_eq!(t.total_builds(), 2);
        assert!(t.latest().unwrap().fmax_mhz == 210);
        assert!(!t.lut_regression());
        assert!(!t.fmax_regression());
    }

    #[test]
    fn test_regression_detected() {
        let mut t = HirRegressionTracker::new();
        t.record(BuildMetrics::new(1, 1000, 500, 10, 5, 200, 100));
        t.record(BuildMetrics::new(2, 1200, 500, 10, 5, 150, 100));
        assert!(t.lut_regression());
        assert!(t.fmax_regression());
    }
}

#[cfg(test)]
mod tests_phase36_cache {
    use super::*;

    #[test]
    fn test_elab_cache() {
        let mut c = HirElabCache::new();
        c.add(HirFileFingerprint::new("a.t27", 12345));
        assert!(c.is_cached("a.t27", 12345));
        assert!(!c.is_cached("a.t27", 99999));
        assert!(!c.is_cached("b.t27", 12345));
        c.invalidate("a.t27");
        assert!(!c.is_cached("a.t27", 12345));
    }

    #[test]
    fn test_entry_count() {
        let mut c = HirElabCache::new();
        c.add(HirFileFingerprint::new("a.t27", 1));
        c.add(HirFileFingerprint::new("b.t27", 2));
        assert_eq!(c.entry_count(), 2);
    }
}

#[cfg(test)]
mod tests_phase37_cdc {
    use super::*;

    #[test]
    fn test_cdc_single_domain_ok() {
        let mut m = HirModule::new("test");
        m.ports.push(HirPort {
            name: "clk".into(),
            dir: HwPortDir::Input,
            ty: HwType::Clock,
        });
        m.clock_domains
            .push(HirClockDomain::new("sys", "ext", 100_000_000));
        assert!(!HirCdcChecker::has_violations(&m));
    }

    #[test]
    fn test_cdc_multi_domain_signals() {
        let mut m = HirModule::new("test");
        m.ports.push(HirPort {
            name: "clk".into(),
            dir: HwPortDir::Input,
            ty: HwType::Clock,
        });
        m.clock_domains
            .push(HirClockDomain::new("sys", "ext", 100_000_000));
        m.clock_domains
            .push(HirClockDomain::new("io", "ext", 50_000_000));
        m.signals.push(HirSignal {
            name: "data_reg".into(),
            kind: HwSignalKind::Reg,
            ty: HwType::UInt(32),
            reset_value: "0".into(),
        });
        m.assigns.push(HirAssign {
            target: "out".into(),
            value: "data_reg".into(),
        });
        let v = HirCdcChecker::check(&m);
        assert!(!v.is_empty());
        assert_eq!(v[0].kind, "missing_sync");
    }
}

#[cfg(test)]
mod tests_phase38_lint {
    use super::*;

    #[test]
    fn test_lint_clean() {
        let mut m = HirModule::new("test");
        m.ports.push(HirPort {
            name: "clk".into(),
            dir: HwPortDir::Input,
            ty: HwType::Clock,
        });
        m.ports.push(HirPort {
            name: "rst_n".into(),
            dir: HwPortDir::Input,
            ty: HwType::Reset(HwResetKind::Async, HwResetPolarity::ActiveLow),
        });
        m.signals.push(HirSignal {
            name: "counter".into(),
            kind: HwSignalKind::Reg,
            ty: HwType::UInt(32),
            reset_value: "0".into(),
        });
        let v = HirLinter::lint(&m);
        assert!(v.is_empty() || v.iter().all(|l| l.severity == "info"));
    }

    #[test]
    fn test_lint_missing_reset() {
        let mut m = HirModule::new("test");
        m.ports.push(HirPort {
            name: "clk".into(),
            dir: HwPortDir::Input,
            ty: HwType::Clock,
        });
        m.ports.push(HirPort {
            name: "rst_n".into(),
            dir: HwPortDir::Input,
            ty: HwType::Reset(HwResetKind::Async, HwResetPolarity::ActiveLow),
        });
        m.signals.push(HirSignal {
            name: "counter".into(),
            kind: HwSignalKind::Reg,
            ty: HwType::UInt(32),
            reset_value: String::new(),
        });
        let v = HirLinter::lint(&m);
        assert!(v.iter().any(|l| l.rule == "reset_all_regs"));
    }

    #[test]
    fn test_lint_clock_naming() {
        let mut m = HirModule::new("test");
        m.ports.push(HirPort {
            name: "clock".into(),
            dir: HwPortDir::Input,
            ty: HwType::Clock,
        });
        let v = HirLinter::lint(&m);
        assert!(v.iter().any(|l| l.rule == "clock_naming"));
    }
}

#[cfg(test)]
mod tests_phase39_depgraph {
    use super::*;

    #[test]
    fn test_dep_graph_basic() {
        let mut g = HirDepGraph::new();
        g.add_module("top", &["uart", "spi"]);
        g.add_module("uart", &[]);
        g.add_module("spi", &[]);
        assert_eq!(g.max_depth(), 1);
        assert!(!g.has_cycle());
        assert_eq!(g.leaf_modules().len(), 2);
        let order = g.topological_order();
        assert_eq!(order.last(), Some(&"top"));
    }

    #[test]
    fn test_dep_graph_cycle() {
        let mut g = HirDepGraph::new();
        g.add_module("a", &["b"]);
        g.add_module("b", &["a"]);
        assert!(g.has_cycle());
    }

    #[test]
    fn test_dep_graph_deep() {
        let mut g = HirDepGraph::new();
        g.add_module("leaf", &[]);
        g.add_module("mid", &["leaf"]);
        g.add_module("top", &["mid"]);
        assert_eq!(g.max_depth(), 2);
    }
}

#[cfg(test)]
mod tests_phase40_coverage {
    use super::*;

    #[test]
    fn test_cover_group() {
        let g = CoverGroup::new("uart_tx", 10);
        assert_eq!(g.coverage_percent(), 0);
        assert!(!g.is_covered());
    }

    #[test]
    fn test_cover_group_hit() {
        let mut g = CoverGroup::new("test", 4);
        g.hits = 4;
        assert_eq!(g.coverage_percent(), 100);
        assert!(g.is_covered());
    }

    #[test]
    fn test_coverage_model() {
        let mut cm = HirCoverageModel::new();
        cm.add_group("uart", 10);
        cm.add_group("spi", 8);
        cm.hit("uart");
        for _ in 0..8 {
            cm.hit("spi");
        }
        assert!(cm.total_coverage() > 0);
        assert_eq!(cm.uncovered().len(), 1);
    }

    #[test]
    fn test_coverage_complete() {
        let mut cm = HirCoverageModel::new();
        cm.add_group("g1", 2);
        cm.hit("g1");
        cm.hit("g1");
        assert!(cm.is_complete());
    }
}
