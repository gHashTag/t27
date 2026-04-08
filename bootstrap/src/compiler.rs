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

            return Token {
                kind: TokenKind::Number,
                lexeme: number,
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

        self.advance(); // consume 'var'

        // Name
        if self.current.kind == TokenKind::Ident {
            decl.name = self.current.lexeme.clone();
            self.advance();
        }

        // Skip everything to semicolon (type annotation, = value, etc.)
        self.skip_to_semicolon()?;
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
        while self.current.kind == TokenKind::KwOr {
            self.advance(); // consume 'or'
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
        while self.current.kind == TokenKind::KwAnd {
            self.advance(); // consume 'and'
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
        while self.current.kind == TokenKind::Pipe {
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
        while self.current.kind == TokenKind::Amp {
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

    /// Parse postfix expressions: field access (.field), deref (.*), indexing ([i]), call (f(args))
    fn parse_expr_postfix(&mut self) -> Result<Node, String> {
        let mut expr = self.parse_expr_primary()?;

        loop {
            if self.current.kind == TokenKind::Dot {
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
                    decl.name, decl.name
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

        // Section: Bench → placeholder comments
        if !benches.is_empty() {
            self.write_indent();
            self.write_line("// -------------------------------------------------------");
            self.write_indent();
            self.write_line("// Benchmark placeholders");
            self.write_indent();
            self.write_line("// -------------------------------------------------------");
            for b in &benches {
                self.write_indent();
                self.write_line(&format!("// bench: {}", b.name));
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
    }

    fn gen_verilog_invariant(&mut self, node: &Node) {
        self.write_indent();
        self.write_line(&format!("// invariant: {}", node.name));
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
                let kw = if node.extra_mutable {
                    "reg"
                } else {
                    "// const"
                };
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
                // Verilog doesn't have field access — flatten to name_field
                if !node.children.is_empty() {
                    self.gen_verilog_expr(&node.children[0]);
                    self.write("_");
                } else {
                    // Just the field name
                }
                self.write(&node.name);
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
            && stmt.children.len() == 1
            && stmt.children[0].kind == NodeKind::ExprLiteral
            && is_literal(&stmt.children[0])
        {
            consts.push((stmt.name.clone(), stmt.children[0].value.clone()));
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
    for child in &mut node.children {
        replace_ident_with_literal(child, name, val, stats);
    }
}

fn propagate_ident(node: &mut Node, from: &str, to: &str) {
    if node.kind == NodeKind::ExprIdentifier && node.name == *from {
        node.name = to.to_string();
    }
    for child in &mut node.children {
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
