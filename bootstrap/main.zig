// Bootstrap t27 Compiler - Minimal implementation
// This is a self-contained single-file compiler for t27 language

const std = @import("std");

// ============================================================================
// Token Type
// ============================================================================

const TokenType = enum {
    kw_pub,
    kw_const,
    kw_fn,
    kw_enum,
    kw_struct,
    kw_test,
    kw_invariant,
    kw_bench,
    kw_module,
    kw_if,
    kw_else,
    kw_for,
    kw_return,
    kw_var,
    kw_using,
    kw_underscore,
    kw_void,
    kw_true,
    kw_false,

    identifier,
    number,
    string,

    colon,
    semicolon,
    comma,
    equals,
    lparen,
    rparen,
    lbrace,
    rbrace,
    lbracket,
    rbracket,
    arrow,
    dot,

    eof,
    unknown,
};

// ============================================================================
// Token
// ============================================================================

const Token = struct {
    type: TokenType,
    lexeme: []const u8,
    line: usize,
    column: usize,
};

// ============================================================================
// Lexer
// ============================================================================

const Lexer = struct {
    source: []const u8,
    pos: usize,
    line: usize = 1,
    column: usize = 1,

    fn init(source: []const u8) Lexer {
        return .{
            .source = source,
            .pos = 0,
        };
    }

    fn peek(self: *Lexer) u8 {
        if (self.pos >= self.source.len) return 0;
        return self.source[self.pos];
    }

    fn advance(self: *Lexer) u8 {
        if (self.pos >= self.source.len) return 0;
        const ch = self.source[self.pos];
        self.pos += 1;
        if (ch == '\n') {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        return ch;
    }

    fn skipWhitespace(self: *Lexer) void {
        while (self.pos < self.source.len) {
            const ch = self.peek();
            if (ch != ' ' and ch != '\t' and ch != '\r' and ch != '\n') break;
            _ = self.advance();
        }
    }

    fn skipLineComment(self: *Lexer) void {
        while (self.pos < self.source.len) {
            const ch = self.peek();
            _ = self.advance();
            if (ch == '\n') break;
        }
    }

    fn skipSemicolonComment(self: *Lexer) void {
        while (self.pos < self.source.len) {
            const ch = self.peek();
            _ = self.advance();
            if (ch == '\n') break;
        }
    }

    fn nextToken(self: *Lexer) Token {
        self.skipWhitespace();

        if (self.pos >= self.source.len) {
            return .{
                .type = .eof,
                .lexeme = "",
                .line = self.line,
                .column = self.column,
            };
        }

        const ch = self.peek();

        // Line comment (//)
        if (ch == '/' and self.pos + 1 < self.source.len and self.source[self.pos + 1] == '/') {
            self.advance();
            self.advance();
            self.skipLineComment();
            return self.nextToken();
        }

        // Semicolon comment (;)
        if (ch == ';') {
            self.skipSemicolonComment();
            return self.nextToken();
        }

        // Identifiers and keywords
        if (isAlpha(ch) or ch == '_') {
            const start = self.pos;
            while (self.pos < self.source.len and (isAlphaNumeric(self.peek()) or self.peek() == '_')) {
                _ = self.advance();
            }
            const lexeme = self.source[start..self.pos];
            const token_type = getKeywordType(lexeme);

            return .{
                .type = token_type,
                .lexeme = lexeme,
                .line = self.line,
                .column = self.column - lexeme.len,
            };
        }

        // Numbers
        if (isDigit(ch) or (ch == '-' and self.pos + 1 < self.source.len and isDigit(self.source[self.pos + 1]))) {
            const start = self.pos;
            if (ch == '-') _ = self.advance();
            while (self.pos < self.source.len and isDigit(self.peek())) {
                _ = self.advance();
            }
            // Hex prefix 0x
            if (self.pos < self.source.len and self.peek() == 'x') {
                _ = self.advance();
                while (self.pos < self.source.len and isHexDigit(self.peek())) {
                    _ = self.advance();
                }
            }
            const lexeme = self.source[start..self.pos];
            return .{
                .type = .number,
                .lexeme = lexeme,
                .line = self.line,
                .column = self.column - lexeme.len,
            };
        }

        // Strings (for comments/docs, not actual string literals in minimal parser)
        if (ch == '"') {
            const start = self.pos;
            _ = self.advance();
            while (self.pos < self.source.len and self.peek() != '"') {
                if (self.peek() == '\\') {
                    _ = self.advance(); // Skip escape
                }
                _ = self.advance();
            }
            if (self.pos < self.source.len) _ = self.advance(); // Closing quote
            const lexeme = self.source[start..self.pos];
            return .{
                .type = .string,
                .lexeme = lexeme,
                .line = self.line,
                .column = self.column - lexeme.len,
            };
        }

        // Multi-char operators
        if (self.pos + 1 < self.source.len) {
            const two_chars = self.source[self.pos..self.pos+2];
            if (std.mem.eql(u8, two_chars, "->")) {
                _ = self.advance();
                _ = self.advance();
                return .{
                    .type = .arrow,
                    .lexeme = two_chars,
                    .line = self.line,
                    .column = self.column - 2,
                };
            }
        }

        // Single char tokens
        const token_type = switch (ch) {
            ':' => .colon,
            ';' => .semicolon,
            ',' => .comma,
            '=' => .equals,
            '(' => .lparen,
            ')' => .rparen,
            '{' => .lbrace,
            '}' => .rbrace,
            '[' => .lbracket,
            ']' => .rbracket,
            '.' => .dot,
            else => .unknown,
        };

        _ = self.advance();
        return .{
            .type = token_type,
            .lexeme = self.source[self.pos-1..self.pos],
            .line = self.line,
            .column = self.column - 1,
        };
    }
};

fn isAlpha(ch: u8) bool {
    return (ch >= 'a' and ch <= 'z') or (ch >= 'A' and ch <= 'Z');
}

fn isDigit(ch: u8) bool {
    return ch >= '0' and ch <= '9';
}

fn isAlphaNumeric(ch: u8) bool {
    return isAlpha(ch) or isDigit(ch);
}

fn isHexDigit(ch: u8) bool {
    return isDigit(ch) or (ch >= 'a' and ch <= 'f') or (ch >= 'A' and ch <= 'F');
}

fn getKeywordType(lexeme: []const u8) TokenType {
    if (std.mem.eql(u8, lexeme, "pub")) return .kw_pub;
    if (std.mem.eql(u8, lexeme, "const")) return .kw_const;
    if (std.mem.eql(u8, lexeme, "fn")) return .kw_fn;
    if (std.mem.eql(u8, lexeme, "enum")) return .kw_enum;
    if (std.mem.eql(u8, lexeme, "struct")) return .kw_struct;
    if (std.mem.eql(u8, lexeme, "test")) return .kw_test;
    if (std.mem.eql(u8, lexeme, "invariant")) return .kw_invariant;
    if (std.mem.eql(u8, lexeme, "bench")) return .kw_bench;
    if (std.mem.eql(u8, lexeme, "module")) return .kw_module;
    if (std.mem.eql(u8, lexeme, "if")) return .kw_if;
    if (std.mem.eql(u8, lexeme, "else")) return .kw_else;
    if (std.mem.eql(u8, lexeme, "for")) return .kw_for;
    if (std.mem.eql(u8, lexeme, "return")) return .kw_return;
    if (std.mem.eql(u8, lexeme, "var")) return .kw_var;
    if (std.mem.eql(u8, lexeme, "using")) return .kw_using;
    if (std.mem.eql(u8, lexeme, "void")) return .kw_void;
    if (std.mem.eql(u8, lexeme, "true")) return .kw_true;
    if (std.mem.eql(u8, lexeme, "false")) return .kw_false;
    if (std.mem.eql(u8, lexeme, "_")) return .kw_underscore;
    return .identifier;
}

// ============================================================================
// AST Node Types
// ============================================================================

const NodeType = enum {
    program,
    module_decl,
    const_decl,
    fn_decl,
    enum_decl,
    struct_decl,
    test_block,
    invariant_block,
    bench_block,
    param,
    field,
    enum_field,
    expr_literal,
    expr_identifier,
    expr_binary,
    expr_call,
    expr_field_access,
    expr_switch,
    expr_if,
    expr_for,
    expr_return,
    expr_var_decl,
    expr_array_type,
    expr_block,
};

// ============================================================================
// AST Node
// ============================================================================

const Node = struct {
    type: NodeType,
    name: []const u8 = "",
    value: []const u8 = "",
    children: std.ArrayList(Node),
    extra: std.ArrayList(NodeKeyValue),

    const NodeKeyValue = struct {
        key: []const u8,
        value: []const u8,
    };

    fn init(allocator: std.mem.Allocator, node_type: NodeType) Node {
        return .{
            .type = node_type,
            .children = std.ArrayList(Node).init(allocator),
            .extra = std.ArrayList(NodeKeyValue).init(allocator),
        };
    }
};

// ============================================================================
// Parser
// ============================================================================

const Parser = struct {
    lexer: Lexer,
    current: Token,
    peek: Token,
    allocator: std.mem.Allocator,
    arena: std.heap.ArenaAllocator,

    fn init(allocator: std.mem.Allocator, source: []const u8) Parser {
        var lexer = Lexer.init(source);
        const first = lexer.nextToken();
        const second = lexer.nextToken();

        return .{
            .lexer = lexer,
            .current = first,
            .peek = second,
            .allocator = allocator,
            .arena = std.heap.ArenaAllocator.init(allocator),
        };
    }

    fn deinit(self: *Parser) void {
        self.arena.deinit();
    }

    fn next(self: *Parser) void {
        self.current = self.peek;
        self.peek = self.lexer.nextToken();
    }

    fn expect(self: *Parser, token_type: TokenType) !void {
        if (self.current.type != token_type) {
            return error.ExpectedToken;
        }
        self.next();
    }

    fn parse(self: *Parser) !Node {
        const node = Node.init(self.allocator, .program);
        errdefer node.children.deinit();
        errdefer node.extra.deinit();

        while (self.current.type != .eof) {
            const decl = try self.parseTopLevelDecl();
            try node.children.append(decl);
        }

        return node;
    }

    fn parseTopLevelDecl(self: *Parser) !Node {
        // pub const NAME: TYPE = VALUE;
        if (self.current.type == .kw_pub) {
            self.next();
            if (self.current.type == .kw_const) {
                return self.parseConstDecl(true);
            } else if (self.current.type == .kw_fn) {
                return self.parseFnDecl(true);
            } else if (self.current.type == .kw_struct) {
                return self.parseStructDecl(true);
            } else if (self.current.type == .kw_enum) {
                return self.parseEnumDecl(true);
            }
            return error.UnexpectedToken;
        }

        // module NAME;
        if (self.current.type == .kw_module) {
            return self.parseModuleDecl();
        }

        // const NAME: TYPE = VALUE;
        if (self.current.type == .kw_const) {
            return self.parseConstDecl(false);
        }

        // fn name(...) TYPE { ... }
        if (self.current.type == .kw_fn) {
            return self.parseFnDecl(false);
        }

        // struct Name { ... }
        if (self.current.type == .kw_struct) {
            return self.parseStructDecl(false);
        }

        // test "name" { ... }
        if (self.current.type == .kw_test) {
            return self.parseTestBlock();
        }

        // invariant name { ... }
        if (self.current.type == .kw_invariant) {
            return self.parseInvariantBlock();
        }

        // bench "name" { ... }
        if (self.current.type == .kw_bench) {
            return self.parseBenchBlock();
        }

        return error.UnexpectedToken;
    }

    fn parseModuleDecl(self: *Parser) !Node {
        const node = Node.init(self.allocator, .module_decl);
        errdefer node.children.deinit();
        errdefer node.extra.deinit();

        try self.expect(.kw_module);

        if (self.current.type == .identifier) {
            node.name = self.arena.allocator.dupe(u8, self.current.lexeme) catch "";
            self.next();
        }

        try self.expect(.semicolon);
        return node;
    }

    fn parseConstDecl(self: *Parser, is_pub: bool) !Node {
        const node = Node.init(self.allocator, .const_decl);
        errdefer node.children.deinit();
        errdefer node.extra.deinit();

        if (is_pub) {
            try self.expect(.kw_pub);
        }

        try self.expect(.kw_const);

        if (self.current.type == .identifier) {
            node.name = self.arena.allocator.dupe(u8, self.current.lexeme) catch "";
            self.next();
        }

        try self.expect(.colon);

        if (self.current.type == .identifier or self.current.type == .kw_underscore) {
            try node.extra.append(.{
                .key = "type",
                .value = self.arena.allocator.dupe(u8, self.current.lexeme) catch "",
            });
            self.next();
        }

        if (self.current.type == .equals) {
            self.next();
            const init = try self.parseExpression();
            try node.children.append(init);
        }

        try self.expect(.semicolon);
        return node;
    }

    fn parseFnDecl(self: *Parser, is_pub: bool) !Node {
        const node = Node.init(self.allocator, .fn_decl);
        errdefer node.children.deinit();
        errdefer node.extra.deinit();

        if (is_pub) {
            try self.expect(.kw_pub);
        }

        try self.expect(.kw_fn);

        if (self.current.type == .identifier) {
            node.name = self.arena.allocator.dupe(u8, self.current.lexeme) catch "";
            self.next();
        }

        try self.expect(.lparen);

        // Parameters
        while (self.current.type != .rparen) {
            const param = try self.parseParam();
            try node.children.append(param);

            if (self.current.type == .comma) {
                self.next();
            }
        }

        try self.expect(.rparen);

        // Return type (optional)
        if (self.current.type == .arrow) {
            self.next();
            if (self.current.type == .identifier or self.current.type == .kw_void) {
                try node.extra.append(.{
                    .key = "return_type",
                    .value = self.arena.allocator.dupe(u8, self.current.lexeme) catch "",
                });
                self.next();
            }
        }

        // Body
        const body = try self.parseBlock();
        try node.children.append(body);

        return node;
    }

    fn parseParam(self: *Parser) !Node {
        const node = Node.init(self.allocator, .param);
        errdefer node.children.deinit();
        errdefer node.extra.deinit();

        if (self.current.type == .identifier) {
            node.name = self.arena.allocator.dupe(u8, self.current.lexeme) catch "";
            self.next();
        }

        try self.expect(.colon);

        if (self.current.type == .identifier or self.current.type == .kw_underscore) {
            try node.extra.append(.{
                .key = "type",
                .value = self.arena.allocator.dupe(u8, self.current.lexeme) catch "",
            });
            self.next();
        }

        return node;
    }

    fn parseStructDecl(self: *Parser, is_pub: bool) !Node {
        const node = Node.init(self.allocator, .struct_decl);
        errdefer node.children.deinit();
        errdefer node.extra.deinit();

        if (is_pub) {
            try self.expect(.kw_pub);
        }

        try self.expect(.kw_struct);

        if (self.current.type == .identifier) {
            node.name = self.arena.allocator.dupe(u8, self.current.lexeme) catch "";
            self.next();
        }

        try self.expect(.lbrace);

        // Fields
        while (self.current.type != .rbrace and self.current.type != .eof) {
            const field = try self.parseField();
            try node.children.append(field);
        }

        try self.expect(.rbrace);
        return node;
    }

    fn parseField(self: *Parser) !Node {
        const node = Node.init(self.allocator, .field);
        errdefer node.children.deinit();
        errdefer node.extra.deinit();

        if (self.current.type == .identifier) {
            node.name = self.arena.allocator.dupe(u8, self.current.lexeme) catch "";
            self.next();
        }

        try self.expect(.colon);

        if (self.current.type == .identifier or self.current.type == .kw_underscore) {
            try node.extra.append(.{
                .key = "type",
                .value = self.arena.allocator.dupe(u8, self.current.lexeme) catch "",
            });
            self.next();
        }

        try self.expect(.semicolon);
        return node;
    }

    fn parseEnumDecl(self: *Parser, is_pub: bool) !Node {
        const node = Node.init(self.allocator, .enum_decl);
        errdefer node.children.deinit();
        errdefer node.extra.deinit();

        if (is_pub) {
            try self.expect(.kw_pub);
        }

        try self.expect(.kw_const); // In t27: pub const Name = enum(...)

        if (self.current.type == .identifier) {
            node.name = self.arena.allocator.dupe(u8, self.current.lexeme) catch "";
            self.next();
        }

        try self.expect(.equals);

        try self.expect(.kw_enum);

        try self.expect(.lparen);

        // Enum backing type
        if (self.current.type == .identifier) {
            try node.extra.append(.{
                .key = "backing_type",
                .value = self.arena.allocator.dupe(u8, self.current.lexeme) catch "",
            });
            self.next();
        }

        try self.expect(.rparen);

        try self.expect(.lbrace);

        // Enum fields
        while (self.current.type != .rbrace and self.current.type != .eof) {
            const field = try self.parseEnumField();
            try node.children.append(field);

            if (self.current.type == .comma) {
                self.next();
            }
        }

        try self.expect(.rbrace);
        return node;
    }

    fn parseEnumField(self: *Parser) !Node {
        const node = Node.init(self.allocator, .enum_field);
        errdefer node.children.deinit();
        errdefer node.extra.deinit();

        if (self.current.type == .identifier) {
            node.name = self.arena.allocator.dupe(u8, self.current.lexeme) catch "";
            self.next();
        }

        if (self.current.type == .equals) {
            self.next();
            if (self.current.type == .number or self.current.type == .identifier) {
                try node.extra.append(.{
                    .key = "value",
                    .value = self.arena.allocator.dupe(u8, self.current.lexeme) catch "",
                });
                self.next();
            }
        }

        return node;
    }

    fn parseTestBlock(self: *Parser) !Node {
        const node = Node.init(self.allocator, .test_block);
        errdefer node.children.deinit();
        errdefer node.extra.deinit();

        try self.expect(.kw_test);

        if (self.current.type == .string) {
            node.name = self.arena.allocator.dupe(u8, self.current.lexeme) catch "";
            self.next();
        }

        const body = try self.parseBlock();
        try node.children.append(body);

        return node;
    }

    fn parseInvariantBlock(self: *Parser) !Node {
        const node = Node.init(self.allocator, .invariant_block);
        errdefer node.children.deinit();
        errdefer node.extra.deinit();

        try self.expect(.kw_invariant);

        if (self.current.type == .identifier) {
            node.name = self.arena.allocator.dupe(u8, self.current.lexeme) catch "";
            self.next();
        }

        const body = try self.parseBlock();
        try node.children.append(body);

        return node;
    }

    fn parseBenchBlock(self: *Parser) !Node {
        const node = Node.init(self.allocator, .bench_block);
        errdefer node.children.deinit();
        errdefer node.extra.deinit();

        try self.expect(.kw_bench);

        if (self.current.type == .string) {
            node.name = self.arena.allocator.dupe(u8, self.current.lexeme) catch "";
            self.next();
        }

        const body = try self.parseBlock();
        try node.children.append(body);

        return node;
    }

    fn parseBlock(self: *Parser) !Node {
        const node = Node.init(self.allocator, .expr_block);
        errdefer node.children.deinit();
        errdefer node.extra.deinit();

        try self.expect(.lbrace);

        while (self.current.type != .rbrace and self.current.type != .eof) {
            const stmt = try self.parseStatement();
            try node.children.append(stmt);
        }

        try self.expect(.rbrace);
        return node;
    }

    fn parseStatement(self: *Parser) !Node {
        // var NAME: TYPE = init;
        if (self.current.type == .kw_var) {
            return self.parseVarDecl();
        }

        // return EXPR;
        if (self.current.type == .kw_return) {
            const node = Node.init(self.allocator, .expr_return);
            errdefer node.children.deinit();
            errdefer node.extra.deinit();

            self.next();
            const expr = try self.parseExpression();
            try node.children.append(expr);

            try self.expect(.semicolon);
            return node;
        }

        // if EXPR { ... } else { ... }
        if (self.current.type == .kw_if) {
            return self.parseIf();
        }

        // for ( ... ) { ... }
        if (self.current.type == .kw_for) {
            return self.parseFor();
        }

        // EXPR;
        const expr = try self.parseExpression();
        try self.expect(.semicolon);
        return expr;
    }

    fn parseVarDecl(self: *Parser) !Node {
        const node = Node.init(self.allocator, .expr_var_decl);
        errdefer node.children.deinit();
        errdefer node.extra.deinit();

        try self.expect(.kw_var);

        if (self.current.type == .identifier) {
            node.name = self.arena.allocator.dupe(u8, self.current.lexeme) catch "";
            self.next();
        }

        try self.expect(.colon);

        if (self.current.type == .identifier or self.current.type == .kw_underscore) {
            try node.extra.append(.{
                .key = "type",
                .value = self.arena.allocator.dupe(u8, self.current.lexeme) catch "",
            });
            self.next();
        }

        if (self.current.type == .equals) {
            self.next();
            const init = try self.parseExpression();
            try node.children.append(init);
        }

        try self.expect(.semicolon);
        return node;
    }

    fn parseIf(self: *Parser) !Node {
        const node = Node.init(self.allocator, .expr_if);
        errdefer node.children.deinit();
        errdefer node.extra.deinit();

        try self.expect(.kw_if);

        try self.expect(.lparen);
        const cond = try self.parseExpression();
        try node.children.append(cond);
        try self.expect(.rparen);

        const then_block = try self.parseBlock();
        try node.children.append(then_block);

        if (self.current.type == .kw_else) {
            self.next();
            const else_block = try self.parseBlock();
            try node.children.append(else_block);
        }

        return node;
    }

    fn parseFor(self: *Parser) !Node {
        const node = Node.init(self.allocator, .expr_for);
        errdefer node.children.deinit();
        errdefer node.extra.deinit();

        try self.expect(.kw_for);

        try self.expect(.lparen);
        const range = try self.parseExpression();
        try node.children.append(range);
        try self.expect(.rparen);

        const body = try self.parseBlock();
        try node.children.append(body);

        return node;
    }

    fn parseExpression(self: *Parser) !Node {
        return self.parseAssignment();
    }

    fn parseAssignment(self: *Parser) !Node {
        // For now, just pass through to expression
        return self.parseOr();
    }

    fn parseOr(self: *Parser) !Node {
        var left = try self.parseAnd();

        while (self.current.type == .identifier) {
            const op = self.current.lexeme;
            self.next();
            const right = try self.parseAnd();

            const node = Node.init(self.allocator, .expr_binary);
            errdefer node.children.deinit();
            errdefer node.extra.deinit();

            try node.children.append(left);
            try node.extra.append(.{
                .key = "operator",
                .value = self.arena.allocator.dupe(u8, op) catch "",
            });
            try node.children.append(right);

            left = node;
        }

        return left;
    }

    fn parseAnd(self: *Parser) !Node {
        var left = try self.parseComparison();

        while (self.current.type == .identifier) {
            const op = self.current.lexeme;
            self.next();
            const right = try self.parseComparison();

            const node = Node.init(self.allocator, .expr_binary);
            errdefer node.children.deinit();
            errdefer node.extra.deinit();

            try node.children.append(left);
            try node.extra.append(.{
                .key = "operator",
                .value = self.arena.allocator.dupe(u8, op) catch "",
            });
            try node.children.append(right);

            left = node;
        }

        return left;
    }

    fn parseComparison(self: *Parser) !Node {
        var left = try self.parseSwitch();

        while (self.current.type == .identifier) {
            const op = self.current.lexeme;
            self.next();
            const right = try self.parseSwitch();

            const node = Node.init(self.allocator, .expr_binary);
            errdefer node.children.deinit();
            errdefer node.extra.deinit();

            try node.children.append(left);
            try node.extra.append(.{
                .key = "operator",
                .value = self.arena.allocator.dupe(u8, op) catch "",
            });
            try node.children.append(right);

            left = node;
        }

        return left;
    }

    fn parseSwitch(self: *Parser) !Node {
        if (self.current.type != .kw_if) {
            return try self.parseTerm();
        }

        const node = Node.init(self.allocator, .expr_switch);
        errdefer node.children.deinit();
        errdefer node.extra.deinit();

        try self.expect(.kw_if);

        const value = try self.parseTerm();
        try node.children.append(value);

        try self.expect(.lbrace);

        while (self.current.type != .rbrace and self.current.type != .eof) {
            const case_node = Node.init(self.allocator, .expr_block);
            errdefer case_node.children.deinit();
            errdefer case_node.extra.deinit();

            // case label like .neg, .zero, .pos
            if (self.current.type == .dot) {
                self.next();
                if (self.current.type == .identifier) {
                    case_node.name = self.arena.allocator.dupe(u8, self.current.lexeme) catch "";
                    self.next();
                }

                // => for arrow
                if (self.current.type == .arrow) {
                    self.next();
                }

                const case_expr = try self.parseExpression();
                try case_node.children.append(case_expr);

                if (self.current.type == .comma) {
                    self.next();
                }

                try node.children.append(case_node);
            } else {
                break;
            }
        }

        try self.expect(.rbrace);
        return node;
    }

    fn parseTerm(self: *Parser) !Node {
        var left = try self.parseFactor();

        while (self.current.type == .identifier) {
            const op = self.current.lexeme;
            self.next();
            const right = try self.parseFactor();

            const node = Node.init(self.allocator, .expr_binary);
            errdefer node.children.deinit();
            errdefer node.extra.deinit();

            try node.children.append(left);
            try node.extra.append(.{
                .key = "operator",
                .value = self.arena.allocator.dupe(u8, op) catch "",
            });
            try node.children.append(right);

            left = node;
        }

        return left;
    }

    fn parseFactor(self: *Parser) !Node {
        var left = try self.parseUnary();

        while (self.current.type == .identifier) {
            const op = self.current.lexeme;
            self.next();
            const right = try self.parseUnary();

            const node = Node.init(self.allocator, .expr_binary);
            errdefer node.children.deinit();
            errdefer node.extra.deinit();

            try node.children.append(left);
            try node.extra.append(.{
                .key = "operator",
                .value = self.arena.allocator.dupe(u8, op) catch "",
            });
            try node.children.append(right);

            left = node;
        }

        return left;
    }

    fn parseUnary(self: *Parser) !Node {
        if (self.current.type == .identifier and std.mem.eql(u8, self.current.lexeme, "!")) {
            const node = Node.init(self.allocator, .expr_binary);
            errdefer node.children.deinit();
            errdefer node.extra.deinit();

            try node.extra.append(.{
                .key = "operator",
                .value = "!",
            });
            self.next();
            const operand = try self.parseUnary();
            try node.children.append(operand);

            return node;
        }

        return try parsePrimary();
    }

    fn parsePrimary(self: *Parser) !Node {
        // Literal
        if (self.current.type == .number) {
            const node = Node.init(self.allocator, .expr_literal);
            node.value = self.arena.allocator.dupe(u8, self.current.lexeme) catch "";
            try node.extra.append(.{
                .key = "kind",
                .value = "number",
            });
            self.next();
            return node;
        }

        // Boolean literal
        if (self.current.type == .kw_true or self.current.type == .kw_false) {
            const node = Node.init(self.allocator, .expr_literal);
            node.value = self.arena.allocator.dupe(u8, self.current.lexeme) catch "";
            try node.extra.append(.{
                .key = "kind",
                .value = "boolean",
            });
            self.next();
            return node;
        }

        // String literal
        if (self.current.type == .string) {
            const node = Node.init(self.allocator, .expr_literal);
            node.value = self.arena.allocator.dupe(u8, self.current.lexeme) catch "";
            try node.extra.append(.{
                .key = "kind",
                .value = "string",
            });
            self.next();
            return node;
        }

        // Array type [N]TYPE
        if (self.current.type == .lbracket) {
            const node = Node.init(self.allocator, .expr_array_type);
            errdefer node.children.deinit();
            errdefer node.extra.deinit();

            self.next();
            if (self.current.type == .number or self.current.type == .identifier) {
                try node.extra.append(.{
                    .key = "size",
                    .value = self.arena.allocator.dupe(u8, self.current.lexeme) catch "",
                });
                self.next();
            }

            if (self.current.type == .rbracket) {
                self.next();
            }

            if (self.current.type == .identifier or self.current.type == .kw_underscore) {
                try node.extra.append(.{
                    .key = "type",
                    .value = self.arena.allocator.dupe(u8, self.current.lexeme) catch "",
                });
                self.next();
            }

            return node;
        }

        // Identifier or function call
        if (self.current.type == .identifier or self.current.type == .kw_underscore) {
            const name = self.arena.allocator.dupe(u8, self.current.lexeme) catch "";
            self.next();

            // Function call
            if (self.current.type == .lparen) {
                const node = Node.init(self.allocator, .expr_call);
                errdefer node.children.deinit();
                errdefer node.extra.deinit();

                node.name = name;
                self.next();

                while (self.current.type != .rparen and self.current.type != .eof) {
                    const arg = try self.parseExpression();
                    try node.children.append(arg);

                    if (self.current.type == .comma) {
                        self.next();
                    }
                }

                try self.expect(.rparen);
                return node;
            }

            // Field access
            if (self.current.type == .dot) {
                const node = Node.init(self.allocator, .expr_field_access);
                errdefer node.children.deinit();
                errdefer node.extra.deinit();

                node.name = name;
                self.next();

                if (self.current.type == .identifier) {
                    try node.extra.append(.{
                        .key = "field",
                        .value = self.arena.allocator.dupe(u8, self.current.lexeme) catch "",
                    });
                    self.next();
                }

                return node;
            }

            // Simple identifier
            const node = Node.init(self.allocator, .expr_identifier);
            node.name = name;
            return node;
        }

        // Parenthesized expression
        if (self.current.type == .lparen) {
            self.next();
            const expr = try self.parseExpression();
            try self.expect(.rparen);
            return expr;
        }

        return error.UnexpectedToken;
    }
};

// ============================================================================
// JSON Generator
// ============================================================================

fn generateJSON(allocator: std.mem.Allocator, node: Node) ![]u8 {
    var buffer = std.ArrayList(u8).init(allocator);

    try writeNode(&buffer, node, 0);

    return buffer.toOwnedSlice();
}

fn writeNode(buffer: *std.ArrayList(u8), node: Node, indent: usize) !void {
    const indent_str = "                                        ";

    try buffer.writer().writeAll("{");

    // node_type
    try buffer.writer().writeAll("\"node_type\": \"");
    try buffer.writer().writeAll(@tagName(NodeType, node.type));
    try buffer.writer().writeByte('"');

    if (node.name.len > 0) {
        try buffer.writer().writeAll(", \"name\": \"");
        try buffer.writer().writeAll(std.zig.fmtEscapes(node.name));
        try buffer.writer().writeByte('"');
    }

    if (node.value.len > 0) {
        try buffer.writer().writeAll(", \"value\": \"");
        try buffer.writer().writeAll(std.zig.fmtEscapes(node.value));
        try buffer.writer().writeByte('"');
    }

    if (node.extra.items.len > 0) {
        try buffer.writer().writeAll(", \"extra\": {");
        var first = true;
        for (node.extra.items) |kv| {
            if (!first) try buffer.writer().writeAll(", ");
            try buffer.writer().print("\"{s}\": \"{s}\"", .{kv.key, std.zig.fmtEscapes(kv.value)});
            first = false;
        }
        try buffer.writer().writeByte('}');
    }

    if (node.children.items.len > 0) {
        try buffer.writer().writeAll(", \"children\": [\n");

        for (node.children.items, 0..) |child, i| {
            try buffer.writer().writeAll(indent_str[0..@min(indent + 2, indent_str.len)]);
            try writeNode(buffer, child, indent + 2);
            if (i < node.children.items.len - 1) {
                try buffer.writer().writeByte(',');
            }
            try buffer.writer().writeByte('\n');
        }

        try buffer.writer().writeAll(indent_str[0..indent]);
        try buffer.writer().writeAll("]");
    }

    try buffer.writer().writeByte('}');
}

// ============================================================================
// Main
// ============================================================================

pub fn main() !void {
    const gpa = std.heap.GeneralPurposeAllocator(.{});
    defer _ = gpa.deinit();

    const args = try std.process.argsAlloc(gpa);
    defer std.process.argsFree(gpa, args);

    if (args.len < 2) {
        std.io.getStdErr().writeAll("Usage: bootstrap/main.zig <file.t27>\n");
        std.process.exit(1);
    }

    const file_path = args[1];
    const source = try std.fs.cwd().readFileAlloc(gpa, file_path);
    defer gpa.free(source);

    var parser = Parser.init(gpa, source);
    defer parser.deinit();

    const ast = try parser.parse();

    const json = try generateJSON(gpa, ast);
    defer gpa.free(json);

    std.io.getStdOut().writeAll(json);
    std.io.getStdOut().writeByte('\n');
}
