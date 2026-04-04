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
    // Statement nodes for fn bodies
    StmtLocal,    // const x = expr; or var x: T = expr;
    StmtAssign,   // x = expr; or x.field = expr;
    StmtIf,       // if (...) { ... } else if (...) { ... } else { ... }
    StmtWhile,    // while (cond) { ... }
    StmtFor,      // for (iter) |capture| { ... }
    StmtExpr,     // bare expression statement: func(a, b);
}

#[derive(Debug, Clone)]
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
    pub params: Vec<(String, String)>, // (name, type) pairs for FnDecl
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
            children: Vec::new(),
        }
    }

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
    KwPub, KwConst, KwFn, KwEnum, KwStruct, KwTest, KwInvariant, KwBench,
    KwModule, KwIf, KwElse, KwFor, KwWhile, KwSwitch, KwReturn, KwVar, KwUsing, KwVoid,
    KwTrue, KwFalse, KwUse, KwOr, KwAnd, KwTry,

    // Literals
    Ident, Number, String,

    // Operators
    Plus, Minus, Star, Slash, Percent, Amp, Pipe, Caret, Tilde,
    Lt, Gt, Lte, Gte, Eq, Neq,

    // Delimiters
    Colon, Comma, Equals, LParen, RParen, LBrace, RBrace,
    LBracket, RBracket, Dot, Bang,

    // Multi-char
    Arrow, FatArrow, Power, DotDot,
    ShiftLeft, ShiftRight, PlusEquals, PlusPercent,

    // Special
    Semicolon, Eof,
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
                if c.is_ascii_digit() || c == b'.' || c == b'x' || c == b'X' || c == b'b' || c == b'B' || c == b'_' {
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
        matches!(self.current.kind,
            TokenKind::KwPub | TokenKind::KwFn | TokenKind::KwEnum | TokenKind::KwStruct |
            TokenKind::KwTest | TokenKind::KwInvariant | TokenKind::KwBench |
            TokenKind::KwUse | TokenKind::KwUsing | TokenKind::KwModule |
            TokenKind::RBrace | TokenKind::Eof
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
                    if self.current.kind == TokenKind::Ident || self.current.kind == TokenKind::Number {
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
                if self.current.kind == TokenKind::Ident {
                    full_path.push_str(&self.current.lexeme);
                    self.advance();
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
                if self.current.kind == TokenKind::Semicolon {
                    self.advance();
                }
                // Extract the last segment as the import name
                let import_name = full_path.rsplit("::").next().unwrap_or(&full_path).to_string();
                let mut use_node = Node::new(NodeKind::UseDecl);
                use_node.name = import_name;   // e.g. "types"
                use_node.value = full_path;     // e.g. "base::types"
                module.children.push(use_node);
                continue;
            }

            match self.parse_top_level_decl() {
                Ok(decl) => module.children.push(decl),
                Err(e) => return Err(e),
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
                Err(format!("Unexpected top-level token: {} ('{}') at line {}:{}", tok, lexeme, line, col))
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
            return Err(format!("Expected identifier after 'const', got {:?}", self.current.kind));
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
                while self.current.kind != TokenKind::RBracket && self.current.kind != TokenKind::Eof {
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
                while self.current.kind != TokenKind::Semicolon && self.current.kind != TokenKind::Eof {
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
            } else if self.current.kind == TokenKind::KwTrue || self.current.kind == TokenKind::KwFalse {
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
            if self.current.kind != TokenKind::Semicolon {
                self.skip_to_semicolon()?;
                return Ok(decl);
            }
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

                let mut variant = Node::new(NodeKind::ExprLiteral);
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
                    // Collect type tokens until comma or closing brace
                    while self.current.kind != TokenKind::Comma
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

                if self.current.kind == TokenKind::Comma {
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

        // Handle slice/array prefix: []Type, [N]Type, []const Type
        while self.current.kind == TokenKind::LBracket {
            ty.push('[');
            self.advance(); // consume [
            while self.current.kind != TokenKind::RBracket && self.current.kind != TokenKind::Eof {
                ty.push_str(&self.current.lexeme);
                self.advance();
            }
            ty.push(']');
            if self.current.kind == TokenKind::RBracket {
                self.advance();
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

        // Main type identifier
        if self.current.kind == TokenKind::Ident {
            ty.push_str(&self.current.lexeme);
            self.advance();
        } else if self.current.kind == TokenKind::KwVoid {
            ty.push_str("void");
            self.advance();
        }

        ty
    }

    fn parse_fn_decl(&mut self, is_pub: bool) -> Result<Node, String> {
        let mut decl = Node::new(NodeKind::FnDecl);
        decl.extra_pub = is_pub;

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
            // Handle one or more bracket levels: []Type, [][]const u8, [N]Type
            let mut rt = String::new();
            while self.current.kind == TokenKind::LBracket {
                rt.push('[');
                self.advance(); // consume [
                while self.current.kind != TokenKind::RBracket && self.current.kind != TokenKind::Eof {
                    rt.push_str(&self.current.lexeme);
                    self.advance();
                }
                rt.push(']');
                if self.current.kind == TokenKind::RBracket {
                    self.advance();
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
                TokenKind::LBrace => { brace_depth += 1; self.advance(); }
                TokenKind::RBrace => { brace_depth -= 1; self.advance(); }
                _ => { self.advance(); }
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
                while self.current.kind != TokenKind::RBrace && self.current.kind != TokenKind::Eof {
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

        // Capture variables: |x| or |x, y|
        if self.current.kind == TokenKind::Pipe {
            self.advance(); // consume |
            while self.current.kind != TokenKind::Pipe && self.current.kind != TokenKind::Eof {
                if self.current.kind == TokenKind::Ident {
                    for_node.params.push((self.current.lexeme.clone(), String::new()));
                    self.advance();
                }
                if self.current.kind == TokenKind::Comma {
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
        while matches!(self.current.kind,
            TokenKind::Eq | TokenKind::Neq | TokenKind::Lt | TokenKind::Gt |
            TokenKind::Lte | TokenKind::Gte | TokenKind::DotDot
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
        while matches!(self.current.kind, TokenKind::ShiftLeft | TokenKind::ShiftRight) {
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
        while matches!(self.current.kind, TokenKind::Plus | TokenKind::Minus | TokenKind::PlusPercent) {
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
        while matches!(self.current.kind, TokenKind::Star | TokenKind::Slash | TokenKind::Percent | TokenKind::Power) {
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
        if matches!(self.current.kind, TokenKind::Minus | TokenKind::Bang | TokenKind::Tilde | TokenKind::Amp) {
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
                    Err(format!("Expected identifier after '.', got {:?}", self.current.kind))
                }
            }

            // Identifier, function call, @builtin call, or struct literal
            TokenKind::Ident => {
                let name = self.current.lexeme.clone();
                self.advance();

                // Check for struct literal: Name{ .field = expr, ... }
                if self.current.kind == TokenKind::LBrace {
                    return self.parse_struct_literal(name);
                }

                // Check for function call: name(args)
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
            TokenKind::KwIf => {
                self.parse_if_expr()
            }

            // switch expression: switch (val) { ... }
            TokenKind::KwSwitch => {
                self.parse_switch_expr()
            }

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
            TokenKind::LBracket => {
                self.parse_array_literal()
            }

            _ => {
                Err(format!("Unexpected token in expression: {:?} ('{}') at line {}:{}",
                    self.current.kind, self.current.lexeme, self.current.line, self.current.col))
            }
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
            // Expect .field = expr
            if self.current.kind == TokenKind::Dot {
                self.advance(); // consume .
                let field_name = if self.current.kind == TokenKind::Ident {
                    let n = self.current.lexeme.clone();
                    self.advance();
                    n
                } else {
                    String::new()
                };

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
            } else {
                break;
            }
        }
        self.expect(TokenKind::RBrace)?;
        Ok(lit)
    }

    /// Parse array literal: [_]Type{ values } or [N]Type{ values }
    /// Collected verbatim as a literal string since Zig passes through
    fn parse_array_literal(&mut self) -> Result<Node, String> {
        let mut text = String::from("[");
        self.advance(); // consume [

        // Collect everything up to ]
        while self.current.kind != TokenKind::RBracket && self.current.kind != TokenKind::Eof {
            text.push_str(&self.current.lexeme);
            self.advance();
        }
        text.push(']');
        self.expect(TokenKind::RBracket)?;

        // Collect the type name (e.g. Trit, u8)
        if self.current.kind == TokenKind::Ident {
            text.push_str(&self.current.lexeme);
            self.advance();
        }

        // Check for { ... } initializer — collect verbatim with proper spacing
        if self.current.kind == TokenKind::LBrace {
            text.push_str("{ ");
            self.advance(); // consume {
            let mut depth = 1;
            while depth > 0 && self.current.kind != TokenKind::Eof {
                if self.current.kind == TokenKind::LBrace {
                    depth += 1;
                } else if self.current.kind == TokenKind::RBrace {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                // Dot immediately before an identifier (enum value like .neg)
                // should not have a space between them
                if self.current.kind == TokenKind::Dot {
                    text.push('.');
                    self.advance();
                    continue;
                }
                // After comma, add space
                if self.current.kind == TokenKind::Comma {
                    text.push_str(", ");
                    self.advance();
                    continue;
                }
                text.push_str(&self.current.lexeme);
                self.advance();
            }
            text.push_str(" }");
            self.expect(TokenKind::RBrace)?;
        }

        Ok(Node {
            kind: NodeKind::ExprLiteral,
            value: text,
            ..Default::default()
        })
    }

    /// Parse if expression: if (cond) expr else expr
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
            } else if self.current.kind == TokenKind::Ident || self.current.kind == TokenKind::Number {
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
        self.skip_brace_body()?;
        self.expect(TokenKind::RBrace)?;
        Ok(decl)
    }

    fn parse_struct_decl(&mut self, is_pub: bool) -> Result<Node, String> {
        let mut decl = Node::new(NodeKind::StructDecl);
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
            self.advance(); // consume {
            self.skip_brace_body()?;
            self.expect(TokenKind::RBrace)?;
        } else {
            self.skip_to_next_top_level();
        }
        Ok(block)
    }

    fn parse_bench_block(&mut self) -> Result<Node, String> {
        let mut block = Node::new(NodeKind::BenchBlock);

        self.advance(); // consume 'bench'
        block.name = self.parse_block_name();

        if self.current.kind == TokenKind::LBrace {
            self.advance(); // consume {
            self.skip_brace_body()?;
            self.expect(TokenKind::RBrace)?;
        } else {
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
        self.write_line(&format!("// Generated from t27 spec: {} (module name)", module_name));
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
                self.write_line(&format!("const {} = @import(\"{}.zig\");", decl.name, decl.name));
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

        self.write(&format!("const {} = struct {{", node.name));
        self.write_line("");

        self.indent();

        for field in &node.children {
            self.write_indent();
            self.write(&format!("{}: ", field.name));
            if !field.extra_type.is_empty() {
                self.write(&field.extra_type);
            }
            self.write_line(",");
        }

        self.dedent();
        self.write_line("};");
    }

    fn gen_fn_decl(&mut self, node: &Node) {
        if node.extra_pub {
            self.write("pub ");
        }

        self.write(&format!("fn {}(", node.name));
        for (i, (pname, ptype)) in node.params.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.write(&format!("{}: {}", pname, ptype));
        }
        self.write(")");

        if !node.extra_return_type.is_empty() {
            self.write(&format!(" {}", node.extra_return_type));
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
        self.write(&format!("invariant(\"{}\")", node.name));
        self.write_line(" {");

        self.indent();

        for stmt in &node.children {
            self.gen_stmt(stmt);
        }

        self.dedent();
        self.write_line("}");
    }

    fn gen_bench_block(&mut self, node: &Node) {
        self.write(&format!("bench(\"{}\")", node.name));
        self.write_line(" {");

        self.indent();

        for stmt in &node.children {
            self.gen_stmt(stmt);
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
                            self.write(&format!(".{}", case_node.name));
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
// Compiler Interface
// ============================================================================

pub struct Compiler;

impl Compiler {
    pub fn compile(source: &str) -> Result<String, String> {
        // [BUG 1 FIX] Do NOT call lexer.tokenize() — let Parser use next_token() directly
        let lexer = Lexer::new(source);

        let mut parser = Parser::new(lexer);
        let ast = parser.parse()?;

        let mut codegen = Codegen::new();
        codegen.gen_zig(&ast);
        Ok(codegen.into_string())
    }

    pub fn parse_ast(source: &str) -> Result<Node, String> {
        // [BUG 1 FIX] Do NOT call lexer.tokenize() — let Parser use next_token() directly
        let lexer = Lexer::new(source);

        let mut parser = Parser::new(lexer);
        parser.parse()
    }
}
