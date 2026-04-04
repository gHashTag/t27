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
    pub extra_return_type: String,
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
            extra_return_type: String::new(),
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
            extra_return_type: String::new(),
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
    KwModule, KwIf, KwElse, KwFor, KwSwitch, KwReturn, KwVar, KwUsing, KwVoid,
    KwTrue, KwFalse, KwUse,

    // Literals
    Ident, Number, String,

    // Operators
    Plus, Minus, Star, Slash, Percent, Amp, Pipe, Caret, Tilde,
    Lt, Gt, Lte, Gte, Eq, Neq,

    // Delimiters
    Colon, Comma, Equals, LParen, RParen, LBrace, RBrace,
    LBracket, RBracket, Dot, Bang,

    // Multi-char
    Arrow, FatArrow, Power,

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
            }
        }

        while self.current.kind != TokenKind::Eof {
            // [BUG 5 FIX] Skip use/using statements
            if self.current.kind == TokenKind::KwUse || self.current.kind == TokenKind::KwUsing {
                while self.current.kind != TokenKind::Semicolon && self.current.kind != TokenKind::Eof {
                    self.advance();
                }
                if self.current.kind == TokenKind::Semicolon {
                    self.advance();
                }
                continue;
            }

            match self.parse_top_level_decl() {
                Ok(decl) => module.children.push(decl),
                Err(e) => return Err(e),
            }
        }

        Ok(module)
    }

    fn parse_top_level_decl(&mut self) -> Result<Node, String> {
        let is_pub = self.current.kind == TokenKind::KwPub;

        if is_pub {
            self.advance(); // consume pub
        }

        match self.current.kind {
            TokenKind::KwConst => self.parse_const_decl(is_pub),
            TokenKind::KwFn => self.parse_fn_decl(is_pub),
            TokenKind::KwEnum => self.parse_enum_decl(is_pub),
            TokenKind::KwStruct => self.parse_struct_decl(is_pub),
            TokenKind::KwTest => self.parse_test_block(),
            TokenKind::KwInvariant => self.parse_invariant_block(),
            TokenKind::KwBench => self.parse_bench_block(),
            _ => {
                // Skip unknown tokens to be resilient
                let tok = format!("{:?}", self.current.kind);
                self.advance();
                Err(format!("Unexpected top-level token: {}", tok))
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
                self.skip_brace_body()?;
                self.expect(TokenKind::RBrace)?;
            } else if self.current.kind == TokenKind::LBracket {
                // pub const TernaryWord = [WORD_BYTES]u8; or [_]u8{...} ** N
                // Skip everything to semicolon
                while self.current.kind != TokenKind::Semicolon && self.current.kind != TokenKind::Eof {
                    self.advance();
                }
                if self.current.kind == TokenKind::Semicolon {
                    self.advance();
                }
                return Ok(decl);
            } else if self.current.kind == TokenKind::Minus {
                // [BUG 10 FIX] Negative number: -1
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
                // Type alias: pub const PackedTrit = u8;
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
            } else if self.current.kind == TokenKind::Tilde {
                // Bitwise NOT expression like ~(TRIT_MASK << bit_pos)
                // Skip to semicolon
                while self.current.kind != TokenKind::Semicolon && self.current.kind != TokenKind::Eof {
                    self.advance();
                }
                if self.current.kind == TokenKind::Semicolon {
                    self.advance();
                }
                return Ok(decl);
            } else {
                // Unknown RHS — skip to semicolon
                while self.current.kind != TokenKind::Semicolon && self.current.kind != TokenKind::Eof {
                    // If we hit a brace, skip its contents
                    if self.current.kind == TokenKind::LBrace {
                        self.advance();
                        self.skip_brace_body()?;
                        if self.current.kind == TokenKind::RBrace {
                            self.advance();
                        }
                    } else {
                        self.advance();
                    }
                }
                if self.current.kind == TokenKind::Semicolon {
                    self.advance();
                }
                return Ok(decl);
            }
        }

        // Consume trailing semicolon
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

    fn parse_fn_decl(&mut self, is_pub: bool) -> Result<Node, String> {
        let mut decl = Node::new(NodeKind::FnDecl);
        decl.extra_pub = is_pub;

        self.advance(); // consume 'fn'

        if self.current.kind == TokenKind::Ident {
            decl.name = self.current.lexeme.clone();
            self.advance();
        }

        // Skip parameter list with parens
        self.expect(TokenKind::LParen)?;
        // Skip params — just balance parens
        let mut paren_depth = 1;
        while paren_depth > 0 && self.current.kind != TokenKind::Eof {
            if self.current.kind == TokenKind::LParen {
                paren_depth += 1;
            } else if self.current.kind == TokenKind::RParen {
                paren_depth -= 1;
                if paren_depth == 0 {
                    break;
                }
            }
            self.advance();
        }
        self.expect(TokenKind::RParen)?;

        // Return type (identifier, or []T / [N]T slice/array types before {)
        if self.current.kind == TokenKind::Ident {
            decl.extra_return_type = self.current.lexeme.clone();
            self.advance();
        } else if self.current.kind == TokenKind::LBracket {
            // []Trit or [N]Type return type
            let mut rt = String::from("[");
            self.advance(); // consume [
            while self.current.kind != TokenKind::RBracket && self.current.kind != TokenKind::Eof {
                rt.push_str(&self.current.lexeme);
                self.advance();
            }
            rt.push(']');
            if self.current.kind == TokenKind::RBracket {
                self.advance();
            }
            if self.current.kind == TokenKind::Ident {
                rt.push_str(&self.current.lexeme);
                self.advance();
            }
            decl.extra_return_type = rt;
        }

        // [BUG 7 FIX] Body: brace-skip
        self.expect(TokenKind::LBrace)?;
        self.skip_brace_body()?;
        self.expect(TokenKind::RBrace)?;
        Ok(decl)
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
        self.skip_brace_body()?;
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

        // [BUG 7 FIX] Brace-skip body
        self.expect(TokenKind::LBrace)?;
        self.skip_brace_body()?;
        self.expect(TokenKind::RBrace)?;
        Ok(block)
    }

    fn parse_invariant_block(&mut self) -> Result<Node, String> {
        let mut block = Node::new(NodeKind::InvariantBlock);

        self.advance(); // consume 'invariant'
        block.name = self.parse_block_name();

        // [BUG 7 FIX] Brace-skip body
        self.expect(TokenKind::LBrace)?;
        self.skip_brace_body()?;
        self.expect(TokenKind::RBrace)?;
        Ok(block)
    }

    fn parse_bench_block(&mut self) -> Result<Node, String> {
        let mut block = Node::new(NodeKind::BenchBlock);

        self.advance(); // consume 'bench'
        block.name = self.parse_block_name();

        // [BUG 7 FIX] Brace-skip body
        self.expect(TokenKind::LBrace)?;
        self.skip_brace_body()?;
        self.expect(TokenKind::RBrace)?;
        Ok(block)
    }

    #[allow(dead_code)]
    fn parse_statement(&mut self) -> Result<Node, String> {
        if self.check_peek(TokenKind::KwReturn) {
            self.parse_return_stmt()
        } else {
            let expr = self.parse_expression()?;
            if self.check_peek(TokenKind::Semicolon) {
                self.advance();
            }
            Ok(expr)
        }
    }

    #[allow(dead_code)]
    fn parse_return_stmt(&mut self) -> Result<Node, String> {
        let mut stmt = Node::new(NodeKind::ExprReturn);
        self.expect(TokenKind::KwReturn)?;

        let expr = self.parse_expression()?;
        stmt.children.push(expr);
        Ok(stmt)
    }

    #[allow(dead_code)]
    fn parse_expression(&mut self) -> Result<Node, String> {
        self.parse_or()
    }

    #[allow(dead_code)]
    fn parse_or(&mut self) -> Result<Node, String> {
        let mut left = self.parse_and()?;

        while self.check_peek(TokenKind::Pipe) {
            self.advance();
            let op = self.current.lexeme.clone();
            let right = self.parse_and()?;
            left = Node {
                kind: NodeKind::ExprBinary,
                extra_op: op,
                children: vec![left, right],
                ..Default::default()
            };
        }

        Ok(left)
    }

    #[allow(dead_code)]
    fn parse_and(&mut self) -> Result<Node, String> {
        let mut left = self.parse_comparison()?;

        while self.check_peek(TokenKind::Amp) {
            self.advance();
            let op = self.current.lexeme.clone();
            let right = self.parse_comparison()?;
            left = Node {
                kind: NodeKind::ExprBinary,
                extra_op: op,
                children: vec![left, right],
                ..Default::default()
            };
        }

        Ok(left)
    }

    #[allow(dead_code)]
    fn parse_comparison(&mut self) -> Result<Node, String> {
        let mut left = self.parse_switch()?;

        let ops = [TokenKind::Lt, TokenKind::Gt, TokenKind::Lte, TokenKind::Gte,
                     TokenKind::Eq, TokenKind::Neq];

        while ops.contains(&self.peek.kind) {
            self.advance();
            let op = self.current.lexeme.clone();
            let right = self.parse_switch()?;
            left = Node {
                kind: NodeKind::ExprBinary,
                extra_op: op,
                children: vec![left, right],
                ..Default::default()
            };
        }

        Ok(left)
    }

    #[allow(dead_code)]
    fn parse_switch(&mut self) -> Result<Node, String> {
        if !self.check_peek(TokenKind::KwSwitch) && !self.check_peek(TokenKind::KwIf) {
            return self.parse_term();
        }

        let mut switch_node = Node::new(NodeKind::ExprSwitch);

        if self.check_peek(TokenKind::KwSwitch) {
            self.advance();
        }

        let value = self.parse_term()?;
        switch_node.children.push(value);

        self.expect(TokenKind::LBrace)?;

        while !self.check(TokenKind::RBrace) && self.peek.kind != TokenKind::Eof {
            let mut case_node = Node::new(NodeKind::ConstDecl);

            if self.check_peek(TokenKind::Dot) {
                self.advance();
                if self.check_peek(TokenKind::Ident) {
                    self.advance();
                    case_node.name = self.current.lexeme.clone();
                }
            } else if self.check_peek(TokenKind::KwElse) {
                self.advance();
                case_node.name = "else".to_string();
            } else if self.check_peek(TokenKind::Ident) || self.check_peek(TokenKind::Number) {
                self.advance();
                case_node.name = self.current.lexeme.clone();
            } else {
                break;
            }

            if self.check_peek(TokenKind::FatArrow) {
                self.advance();
                if let Ok(expr) = self.parse_expression() {
                    case_node.children.push(expr);
                }
            }

            if self.check_peek(TokenKind::Comma) {
                self.advance();
            }

            switch_node.children.push(case_node);
        }

        self.expect(TokenKind::RBrace)?;
        Ok(switch_node)
    }

    #[allow(dead_code)]
    fn parse_term(&mut self) -> Result<Node, String> {
        let mut left = self.parse_factor()?;

        while self.check_peek(TokenKind::Star) || self.check_peek(TokenKind::Slash) || self.check_peek(TokenKind::Percent) {
            self.advance();
            let op = self.current.lexeme.clone();
            let right = self.parse_factor()?;
            left = Node {
                kind: NodeKind::ExprBinary,
                extra_op: op,
                children: vec![left, right],
                ..Default::default()
            };
        }

        Ok(left)
    }

    #[allow(dead_code)]
    fn parse_factor(&mut self) -> Result<Node, String> {
        let mut left = self.parse_unary()?;

        while self.check_peek(TokenKind::Plus) || self.check_peek(TokenKind::Minus) {
            self.advance();
            let op = self.current.lexeme.clone();
            let right = self.parse_unary()?;
            left = Node {
                kind: NodeKind::ExprBinary,
                extra_op: op,
                children: vec![left, right],
                ..Default::default()
            };
        }

        Ok(left)
    }

    #[allow(dead_code)]
    fn parse_unary(&mut self) -> Result<Node, String> {
        self.parse_primary()
    }

    #[allow(dead_code)]
    fn parse_primary(&mut self) -> Result<Node, String> {
        if self.check_peek(TokenKind::Number) {
            self.advance();
            return Ok(Node {
                kind: NodeKind::ExprLiteral,
                value: self.current.lexeme.clone(),
                ..Default::default()
            });
        }

        if self.check_peek(TokenKind::KwTrue) || self.check_peek(TokenKind::KwFalse) {
            self.advance();
            return Ok(Node {
                kind: NodeKind::ExprLiteral,
                value: self.current.lexeme.clone(),
                ..Default::default()
            });
        }

        if self.check(TokenKind::Dot) {
            self.advance();
            if self.check_peek(TokenKind::Ident) {
                self.advance();
                return Ok(Node {
                    kind: NodeKind::ExprEnumValue,
                    name: self.current.lexeme.clone(),
                    ..Default::default()
                });
            }
        }

        if self.check_peek(TokenKind::Ident) {
            self.advance();
            let name = self.current.lexeme.clone();

            if self.check_peek(TokenKind::LParen) {
                self.advance();
                while !self.check(TokenKind::RParen) && self.peek.kind != TokenKind::Eof {
                    self.advance();
                }
                self.expect(TokenKind::RParen)?;

                return Ok(Node {
                    kind: NodeKind::ExprCall,
                    name,
                    ..Default::default()
                });
            }

            return Ok(Node {
                kind: NodeKind::ExprIdentifier,
                name,
                ..Default::default()
            });
        }

        if self.check_peek(TokenKind::LParen) {
            self.advance();
            let expr = self.parse_expression()?;
            self.expect(TokenKind::RParen)?;
            return Ok(expr);
        }

        Err(format!("Unexpected token: {:?}", self.peek.kind))
    }

    #[allow(dead_code)]
    fn parse_literal(&mut self) -> Result<String, String> {
        let value = self.current.lexeme.clone();
        self.advance();
        Ok(value)
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
        self.write_line("// Generated from t27 AST");
        self.write_line("// DO NOT EDIT - generated code");
        self.write_line("");

        for decl in &ast.children {
            self.gen_decl(decl);
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

        self.write(&format!("const {} ", node.name));

        if !node.extra_type.is_empty() {
            self.write(&format!(": {} ", node.extra_type));
        }

        if !node.children.is_empty() {
            self.write("= ");
            self.gen_expr(&node.children[0]);
        }

        self.write_line(";");
    }

    fn gen_enum_decl(&mut self, node: &Node) {
        if node.extra_pub {
            self.write("pub ");
        }

        self.write(&format!("const {} ", node.name));
        self.write("= enum");

        if !node.extra_type.is_empty() {
            self.write(&format!("({})", node.extra_type));
        }

        self.write_line(" {");
        self.indent();

        for (i, value_node) in node.children.iter().enumerate() {
            self.write(&format!("{} = ", value_node.name));
            if !value_node.children.is_empty() {
                self.gen_expr(&value_node.children[0]);
            }

            if i < node.children.len() - 1 {
                self.write(",");
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

        self.write(")");

        if !node.extra_return_type.is_empty() {
            self.write(&format!(" {}", node.extra_return_type));
        }

        self.write_line(" {");

        self.indent();

        for stmt in &node.children {
            self.gen_stmt(stmt);
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
            NodeKind::ExprReturn => self.gen_return_stmt(node),
            _ => { self.gen_expr(node); self.write_line(";"); }
        }
    }

    fn gen_return_stmt(&mut self, node: &Node) {
        self.write("return ");
        if !node.children.is_empty() {
            self.gen_expr(&node.children[0]);
        }
        self.write_line(";");
    }

    fn gen_expr(&mut self, node: &Node) {
        match node.kind {
            NodeKind::ExprLiteral => self.gen_literal(node),
            NodeKind::ExprIdentifier => self.gen_identifier(node),
            NodeKind::ExprEnumValue => self.gen_enum_value(node),
            NodeKind::ExprCall => self.gen_call(node),
            NodeKind::ExprSwitch => self.gen_switch(node),
            NodeKind::ExprBinary => self.gen_binary(node),
            _ => {}
        }
    }

    fn gen_literal(&mut self, node: &Node) {
        self.write(&node.value);
    }

    fn gen_identifier(&mut self, node: &Node) {
        self.write(&node.name);
    }

    fn gen_enum_value(&mut self, node: &Node) {
        self.write(".");
        self.write(&node.name);
    }

    fn gen_call(&mut self, node: &Node) {
        self.write(&node.name);
        self.write("(");
        self.write(")");
    }

    fn gen_switch(&mut self, node: &Node) {
        self.write("switch (");

        if !node.children.is_empty() {
            self.gen_expr(&node.children[0]);
        }

        self.write(") {");
        self.write_line("");

        self.indent();

        for case_node in &node.children[1..] {
            if case_node.kind == NodeKind::ConstDecl {
                if !case_node.name.is_empty() && case_node.name != "else" {
                    self.write(&case_node.name);
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
        self.write_line("}");
    }

    fn gen_binary(&mut self, node: &Node) {
        self.gen_expr(&node.children[0]);
        self.write(&format!(" {} ", node.extra_op));
        self.gen_expr(&node.children[1]);
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
