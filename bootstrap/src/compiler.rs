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
    KwTrue, KwFalse,

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

    fn skip_whitespace(&mut self) {
        while self.pos < self.source.len() {
            let ch = self.peek();
            if ch != b' ' && ch != b'\t' && ch != b'\n' && ch != b'\r' {
                break;
            }
            self.advance();
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
            "void" => TokenKind::KwVoid,
            "true" => TokenKind::KwTrue,
            "false" => TokenKind::KwFalse,
            _ => TokenKind::Ident,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.pos >= self.source.len() {
            return Token {
                kind: TokenKind::Eof,
                lexeme: String::new(),
                line: self.line,
                col: self.col,
            };
        }

        let ch = self.peek();

        // Semicolon dual-use: comment or statement terminator
        if ch == b';' {
            // Check if at line start
            if self.col == 1 {
                let next_ch = self.peek_offset(1);
                if next_ch == b' ' && self.pos + 1 < self.source.len() {
                    // Comment prefix - skip to end of line
                    while self.pos < self.source.len() {
                        let c = self.peek();
                        if c == b'\n' {
                            self.line += 1;
                            self.col = 1;
                        } else {
                            self.col += 1;
                        }
                        self.pos += 1;
                    }
                    return self.next_token();
                }
            }
            // Statement terminator
            self.advance();
            return Token {
                kind: TokenKind::Semicolon,
                lexeme: String::from(";"),
                line: self.line - 1,
                col: self.col - 1,
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
                    line: self.line,
                    col: self.col - 2,
                };
            }

            if two == [b'=', b'>'] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::FatArrow,
                    lexeme: String::from("=>"),
                    line: self.line,
                    col: self.col - 2,
                };
            }

            if two == [b'*', b'*'] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::Power,
                    lexeme: String::from("**"),
                    line: self.line,
                    col: self.col - 2,
                };
            }

            if two == [b'<', b'='] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::Lte,
                    lexeme: String::from("<="),
                    line: self.line,
                    col: self.col - 2,
                };
            }

            if two == [b'>', b'='] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::Gte,
                    lexeme: String::from(">="),
                    line: self.line,
                    col: self.col - 2,
                };
            }

            if two == [b'=', b'='] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::Eq,
                    lexeme: String::from("=="),
                    line: self.line,
                    col: self.col - 2,
                };
            }

            if two == [b'!', b'='] {
                self.advance();
                self.advance();
                return Token {
                    kind: TokenKind::Neq,
                    lexeme: String::from("!="),
                    line: self.line,
                    col: self.col - 2,
                };
            }
        }

        // Identifier or keyword
        if ch.is_ascii_alphabetic() || ch == b'_' {
            let start_col = self.col;
            let start_line = self.line;
            let mut ident = String::new();

            while self.pos < self.source.len() {
                let c = self.peek();
                if c.is_ascii_alphanumeric() || c == b'_' {
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

        // Number literal
        if ch.is_ascii_digit() {
            let start_col = self.col;
            let start_line = self.line;
            let mut number = String::new();

            while self.pos < self.source.len() {
                let c = self.peek();
                if c.is_ascii_digit() || c == b'.' || c == b'x' || c == b'b' {
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
            _ => TokenKind::Ident,
        };

        self.advance();

        Token {
            kind,
            lexeme: String::from_utf8_lossy(&[ch]).to_string(),
            line: self.line - 1,
            col: self.col - 1,
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
    pub fn new(lexer: Lexer) -> Self {
        let mut p = Self {
            lexer,
            current: Token {
                kind: TokenKind::Eof,
                lexeme: String::new(),
                line: 0,
                col: 0,
            },
            peek: Token {
                kind: TokenKind::Eof,
                lexeme: String::new(),
                line: 0,
                col: 0,
            },
        };
        p.advance(); // Initialize: current = first token, peek = second token
        p.advance(); // Advance once more: current = second token, peek = third token
        p
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
                "Expected {:?}, got {:?} at line {}",
                kind, self.current.kind, self.current.line
            ));
        }
        self.advance();
        Ok(())
    }

    pub fn parse(&mut self) -> Result<Node, String> {
        let mut module = Node::new(NodeKind::Module);

        while self.peek.kind != TokenKind::Eof {
            if let Ok(decl) = self.parse_top_level_decl() {
                module.children.push(decl);
            } else {
                return Err(format!("Unexpected token: {:?}", self.peek.kind));
            }
        }

        Ok(module)
    }

    fn parse_top_level_decl(&mut self) -> Result<Node, String> {
        let is_pub = self.check_peek(TokenKind::KwPub);

        if is_pub {
            self.advance();
        }

        if self.check_peek(TokenKind::KwConst) {
            self.parse_const_decl()
        } else if self.check_peek(TokenKind::KwFn) {
            self.parse_fn_decl()
        } else if self.check_peek(TokenKind::KwEnum) {
            self.parse_enum_decl()
        } else if self.check_peek(TokenKind::KwStruct) {
            self.parse_struct_decl()
        } else if self.check_peek(TokenKind::KwTest) {
            self.parse_test_block()
        } else if self.check_peek(TokenKind::KwInvariant) {
            self.parse_invariant_block()
        } else if self.check_peek(TokenKind::KwBench) {
            self.parse_bench_block()
        } else {
            Err(format!("Unexpected token: {:?}", self.peek.kind))
        }
    }

    fn parse_const_decl(&mut self) -> Result<Node, String> {
        let mut decl = Node::new(NodeKind::ConstDecl);
        decl.extra_pub = self.check_peek(TokenKind::KwPub);

        if decl.extra_pub {
            self.advance(); // consume pub
        }

        self.advance(); // consume const

        if self.check(TokenKind::Ident) {
            self.advance();
            decl.name = self.current.lexeme.clone();

            if self.check_peek(TokenKind::Colon) {
                self.advance();
                if self.check_peek(TokenKind::Ident) {
                    self.advance();
                    decl.extra_type = self.current.lexeme.clone();
                }
            }

            if self.check_peek(TokenKind::Equals) {
                self.advance();
                if let Ok(expr) = self.parse_expression() {
                    decl.children.push(expr);
                }
            }

            self.expect(TokenKind::Semicolon)?;
            Ok(decl)
        } else {
            Err(format!("Expected identifier after 'const'"))
        }
    }

    fn parse_fn_decl(&mut self) -> Result<Node, String> {
        let mut decl = Node::new(NodeKind::FnDecl);
        decl.extra_pub = self.check_peek(TokenKind::KwPub);

        if decl.extra_pub {
            self.advance(); // consume pub
        }

        self.advance(); // consume fn

        if self.check_peek(TokenKind::Ident) {
            self.advance();
            decl.name = self.current.lexeme.clone();
        }

        self.expect(TokenKind::LParen)?;

        while !self.check(TokenKind::RParen) && self.peek.kind != TokenKind::Eof {
            self.advance();
        }

        self.expect(TokenKind::RParen)?;

        if self.check_peek(TokenKind::Ident) {
            self.advance();
            decl.extra_return_type = self.current.lexeme.clone();
        }

        self.expect(TokenKind::LBrace)?;

        while !self.check(TokenKind::RBrace) && self.peek.kind != TokenKind::Eof {
            if let Ok(stmt) = self.parse_statement() {
                decl.children.push(stmt);
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(decl)
    }

    fn parse_enum_decl(&mut self) -> Result<Node, String> {
        let mut decl = Node::new(NodeKind::EnumDecl);
        decl.extra_pub = self.check_peek(TokenKind::KwPub);

        if decl.extra_pub {
            self.advance(); // consume pub
        }

        if self.check(TokenKind::Ident) {
            self.advance();
            decl.name = self.current.lexeme.clone();
        }

        self.expect(TokenKind::KwEnum)?;

        if self.check(TokenKind::LParen) {
            self.advance();
            while !self.check(TokenKind::RParen) && self.peek.kind != TokenKind::Eof {
                self.advance();
            }
            self.expect(TokenKind::RParen)?;
        }

        self.expect(TokenKind::LBrace)?;

        while !self.check(TokenKind::RBrace) && self.peek.kind != TokenKind::Eof {
            if self.check_peek(TokenKind::Ident) {
                self.advance();
                let name = self.current.lexeme.clone();

                if self.check_peek(TokenKind::Equals) {
                    self.advance();
                    let value = self.parse_literal()?;
                    let mut value_node = Node::new(NodeKind::ExprLiteral);
                    value_node.name = name;
                    value_node.value = value;
                    decl.children.push(value_node);
                }

                if self.check_peek(TokenKind::Comma) {
                    self.advance();
                }
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(decl)
    }

    fn parse_struct_decl(&mut self) -> Result<Node, String> {
        let mut decl = Node::new(NodeKind::StructDecl);
        decl.extra_pub = self.check_peek(TokenKind::KwPub);

        if decl.extra_pub {
            self.advance(); // consume pub
        }

        self.advance(); // consume struct

        if self.check_peek(TokenKind::Ident) {
            self.advance();
            decl.name = self.current.lexeme.clone();
        }

        self.expect(TokenKind::LBrace)?;

        while !self.check(TokenKind::RBrace) && self.peek.kind != TokenKind::Eof {
            if self.check_peek(TokenKind::Ident) {
                self.advance();
                let field_name = self.current.lexeme.clone();

                if self.check_peek(TokenKind::Colon) {
                    self.advance();
                    if self.check_peek(TokenKind::Ident) {
                        self.advance();
                        let mut field = Node::new(NodeKind::ConstDecl);
                        field.name = field_name;
                        field.extra_type = self.current.lexeme.clone();
                        decl.children.push(field);
                    }
                }

                if self.check_peek(TokenKind::Comma) {
                    self.advance();
                }
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(decl)
    }

    fn parse_test_block(&mut self) -> Result<Node, String> {
        let mut block = Node::new(NodeKind::TestBlock);

        self.expect(TokenKind::KwTest)?;

        if self.check_peek(TokenKind::String) {
            self.advance();
            block.name = self.current.lexeme.clone();
        }

        self.expect(TokenKind::LBrace)?;

        while !self.check(TokenKind::RBrace) && self.peek.kind != TokenKind::Eof {
            if let Ok(stmt) = self.parse_statement() {
                block.children.push(stmt);
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(block)
    }

    fn parse_invariant_block(&mut self) -> Result<Node, String> {
        let mut block = Node::new(NodeKind::InvariantBlock);

        self.expect(TokenKind::KwInvariant)?;

        if self.check_peek(TokenKind::String) {
            self.advance();
            block.name = self.current.lexeme.clone();
        }

        self.expect(TokenKind::LBrace)?;

        while !self.check(TokenKind::RBrace) && self.peek.kind != TokenKind::Eof {
            if let Ok(stmt) = self.parse_statement() {
                block.children.push(stmt);
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(block)
    }

    fn parse_bench_block(&mut self) -> Result<Node, String> {
        let mut block = Node::new(NodeKind::BenchBlock);

        self.expect(TokenKind::KwBench)?;

        if self.check_peek(TokenKind::String) {
            self.advance();
            block.name = self.current.lexeme.clone();
        }

        self.expect(TokenKind::LBrace)?;

        while !self.check(TokenKind::RBrace) && self.peek.kind != TokenKind::Eof {
            if let Ok(stmt) = self.parse_statement() {
                block.children.push(stmt);
            }
        }

        self.expect(TokenKind::RBrace)?;
        Ok(block)
    }

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

    fn parse_return_stmt(&mut self) -> Result<Node, String> {
        let mut stmt = Node::new(NodeKind::ExprReturn);
        self.expect(TokenKind::KwReturn)?;

        let expr = self.parse_expression()?;
        stmt.children.push(expr);
        Ok(stmt)
    }

    fn parse_expression(&mut self) -> Result<Node, String> {
        self.parse_or()
    }

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

    fn parse_unary(&mut self) -> Result<Node, String> {
        self.parse_primary()
    }

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
        let mut lexer = Lexer::new(source);
        lexer.tokenize();

        let mut parser = Parser::new(lexer);
        let ast = parser.parse()?;

        let mut codegen = Codegen::new();
        codegen.gen_zig(&ast);
        Ok(codegen.into_string())
    }

    pub fn parse_ast(source: &str) -> Result<Node, String> {
        let mut lexer = Lexer::new(source);
        lexer.tokenize();

        let mut parser = Parser::new(lexer);
        parser.parse()
    }
}
