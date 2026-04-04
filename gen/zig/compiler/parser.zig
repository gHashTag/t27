// Auto-generated from specs/compiler/parser.t27
// DO NOT EDIT -- regenerate with: tri gen specs/compiler/parser.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");
const mem = std.mem;
const Allocator = std.mem.Allocator;
const ArrayList = std.ArrayList;

// ============================================================================
// Token Types
// ============================================================================

pub const TokenKind = enum(u8) {
    eof,
    ident,
    literal,
    // Keywords
    pub_,
    const_,
    extern_,
    struct_,
    enum_,
    fn_,
    return_,
    if_,
    else_,
    using,
    test_,
    bench,
    invariant,
    module_,
    type_,
    // Operators
    plus,
    minus,
    star,
    slash,
    percent,
    eq,
    ne,
    lt,
    le,
    gt,
    ge,
    and_,
    or_,
    xor,
    not_,
    assign,
    arrow,
    // Delimiters
    l_paren,
    r_paren,
    l_brace,
    r_brace,
    l_bracket,
    r_bracket,
    comma,
    colon,
    dot,
    semicolon,
    question,
};

pub const Token = struct {
    kind: TokenKind,
    text: []const u8,
    line: u32,
    col: u32,
};

// ============================================================================
// AST Node Types
// ============================================================================

pub const AstNodeKind = enum {
    module,
    const_decl,
    enum_decl,
    struct_decl,
    fn_decl,
    invariant_decl,
    test_decl,
    bench_decl,
};

pub const BinaryOp = enum {
    add,
    sub,
    mul,
    div,
    mod,
    op_eq,
    op_ne,
    op_lt,
    op_le,
    op_gt,
    op_ge,
    op_and,
    op_or,
    op_xor,
    op_assign,
    op_arrow,
};

pub const EnumValueNode = struct {
    name: []const u8,
    value: []const u8,
};

pub const FieldNode = struct {
    name: []const u8,
    type_ref: []const u8,
};

pub const ParamNode = struct {
    name: []const u8,
    type_ref: []const u8,
};

pub const ConstDeclNode = struct {
    is_pub: bool,
    name: []const u8,
    type_ref: []const u8,
    value: []const u8,
};

pub const EnumDeclNode = struct {
    name: []const u8,
    backing: []const u8,
    values: []const EnumValueNode,
};

pub const StructDeclNode = struct {
    name: []const u8,
    fields: []const FieldNode,
};

pub const FnDeclNode = struct {
    name: []const u8,
    params: []const ParamNode,
    return_type: []const u8,
};

pub const InvariantDeclNode = struct {
    name: []const u8,
};

pub const TestDeclNode = struct {
    name: []const u8,
};

pub const BenchDeclNode = struct {
    name: []const u8,
};

pub const ModuleNode = struct {
    name: []const u8,
    decls: []const AstNode,
};

pub const AstNode = union(AstNodeKind) {
    module: ModuleNode,
    const_decl: ConstDeclNode,
    enum_decl: EnumDeclNode,
    struct_decl: StructDeclNode,
    fn_decl: FnDeclNode,
    invariant_decl: InvariantDeclNode,
    test_decl: TestDeclNode,
    bench_decl: BenchDeclNode,
};

// ============================================================================
// Character Classification
// ============================================================================

/// Returns true if c can start an identifier
pub fn isIdentStart(c: u8) bool {
    return (c >= 'a' and c <= 'z') or (c >= 'A' and c <= 'Z') or c == '_';
}

/// Returns true if c can continue an identifier
pub fn isIdentContinue(c: u8) bool {
    return isIdentStart(c) or (c >= '0' and c <= '9') or c == '-';
}

/// Returns true if c is a decimal digit
pub fn isDigit(c: u8) bool {
    return c >= '0' and c <= '9';
}

/// Returns true if c is whitespace
pub fn isWhitespace(c: u8) bool {
    return c == ' ' or c == '\t' or c == '\n' or c == '\r';
}

// ============================================================================
// Keyword Lookup
// ============================================================================

/// Maps identifier text to keyword token kind, or .ident if not a keyword
pub fn keywordKind(text: []const u8) TokenKind {
    const keywords = .{
        .{ "pub", TokenKind.pub_ },
        .{ "const", TokenKind.const_ },
        .{ "extern", TokenKind.extern_ },
        .{ "struct", TokenKind.struct_ },
        .{ "enum", TokenKind.enum_ },
        .{ "fn", TokenKind.fn_ },
        .{ "return", TokenKind.return_ },
        .{ "if", TokenKind.if_ },
        .{ "else", TokenKind.else_ },
        .{ "using", TokenKind.using },
        .{ "test", TokenKind.test_ },
        .{ "bench", TokenKind.bench },
        .{ "invariant", TokenKind.invariant },
        .{ "module", TokenKind.module_ },
        .{ "type", TokenKind.type_ },
    };
    inline for (keywords) |kw| {
        if (mem.eql(u8, text, kw[0])) return kw[1];
    }
    return .ident;
}

/// Maps single character to token kind
pub fn singleCharTokenKind(c: u8) TokenKind {
    return switch (c) {
        '+' => .plus,
        '*' => .star,
        '/' => .slash,
        '%' => .percent,
        '=' => .assign,
        '<' => .lt,
        '>' => .gt,
        '!' => .not_,
        '&' => .and_,
        '|' => .or_,
        '^' => .xor,
        '(' => .l_paren,
        ')' => .r_paren,
        '{' => .l_brace,
        '}' => .r_brace,
        '[' => .l_bracket,
        ']' => .r_bracket,
        ',' => .comma,
        ':' => .colon,
        '.' => .dot,
        ';' => .semicolon,
        '?' => .question,
        else => .eof,
    };
}

// ============================================================================
// Tokenizer
// ============================================================================

/// Scans source text and returns list of tokens.
/// Handles whitespace, // and ; comments, identifiers/keywords,
/// numbers (including negative), string literals, multi-char operators,
/// and single-char tokens. Appends EOF at end.
pub fn tokenize(allocator: Allocator, source: []const u8) ![]Token {
    var tokens = ArrayList(Token).init(allocator);
    defer tokens.deinit();

    var pos: usize = 0;
    var line: u32 = 1;
    var col: u32 = 1;

    while (pos < source.len) {
        const c = source[pos];

        // Skip whitespace
        if (isWhitespace(c)) {
            if (c == '\n') {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
            pos += 1;
            continue;
        }

        // Skip ; comments (to end of line)
        if (c == ';') {
            while (pos < source.len and source[pos] != '\n') {
                pos += 1;
            }
            continue;
        }

        // Skip // comments (to end of line)
        if (c == '/' and pos + 1 < source.len and source[pos + 1] == '/') {
            while (pos < source.len and source[pos] != '\n') {
                pos += 1;
            }
            continue;
        }

        // Identifiers and keywords
        if (isIdentStart(c)) {
            const start = pos;
            const start_col = col;
            while (pos < source.len and isIdentContinue(source[pos])) {
                pos += 1;
                col += 1;
            }
            const text = source[start..pos];
            const kind = keywordKind(text);
            try tokens.append(Token{
                .kind = kind,
                .text = text,
                .line = line,
                .col = start_col,
            });
            continue;
        }

        // Numbers (including negative: -digit)
        if (isDigit(c)) {
            const start = pos;
            const start_col = col;
            while (pos < source.len and isDigit(source[pos])) {
                pos += 1;
                col += 1;
            }
            try tokens.append(Token{
                .kind = .literal,
                .text = source[start..pos],
                .line = line,
                .col = start_col,
            });
            continue;
        }

        // String literals
        if (c == '"') {
            pos += 1; // skip opening quote
            col += 1;
            const start = pos;
            const start_col = col;
            while (pos < source.len and source[pos] != '"') {
                pos += 1;
                col += 1;
            }
            const text = source[start..pos];
            if (pos < source.len) {
                pos += 1; // skip closing quote
                col += 1;
            }
            try tokens.append(Token{
                .kind = .literal,
                .text = text,
                .line = line,
                .col = start_col,
            });
            continue;
        }

        // Multi-char operators
        if (c == '-') {
            const start_col = col;
            if (pos + 1 < source.len and source[pos + 1] == '>') {
                // Arrow ->
                try tokens.append(Token{
                    .kind = .arrow,
                    .text = source[pos .. pos + 2],
                    .line = line,
                    .col = start_col,
                });
                pos += 2;
                col += 2;
                continue;
            }
            if (pos + 1 < source.len and isDigit(source[pos + 1])) {
                // Negative number
                const start = pos;
                pos += 1; // skip '-'
                col += 1;
                while (pos < source.len and isDigit(source[pos])) {
                    pos += 1;
                    col += 1;
                }
                try tokens.append(Token{
                    .kind = .literal,
                    .text = source[start..pos],
                    .line = line,
                    .col = start_col,
                });
                continue;
            }
            // Bare minus
            try tokens.append(Token{
                .kind = .minus,
                .text = source[pos .. pos + 1],
                .line = line,
                .col = start_col,
            });
            pos += 1;
            col += 1;
            continue;
        }

        if (c == '=' and pos + 1 < source.len and source[pos + 1] == '=') {
            try tokens.append(Token{
                .kind = .eq,
                .text = source[pos .. pos + 2],
                .line = line,
                .col = col,
            });
            pos += 2;
            col += 2;
            continue;
        }
        if (c == '!' and pos + 1 < source.len and source[pos + 1] == '=') {
            try tokens.append(Token{
                .kind = .ne,
                .text = source[pos .. pos + 2],
                .line = line,
                .col = col,
            });
            pos += 2;
            col += 2;
            continue;
        }
        if (c == '<' and pos + 1 < source.len and source[pos + 1] == '=') {
            try tokens.append(Token{
                .kind = .le,
                .text = source[pos .. pos + 2],
                .line = line,
                .col = col,
            });
            pos += 2;
            col += 2;
            continue;
        }
        if (c == '>' and pos + 1 < source.len and source[pos + 1] == '=') {
            try tokens.append(Token{
                .kind = .ge,
                .text = source[pos .. pos + 2],
                .line = line,
                .col = col,
            });
            pos += 2;
            col += 2;
            continue;
        }

        // Single-char tokens
        const kind = singleCharTokenKind(c);
        if (kind != .eof) {
            try tokens.append(Token{
                .kind = kind,
                .text = source[pos .. pos + 1],
                .line = line,
                .col = col,
            });
            pos += 1;
            col += 1;
            continue;
        }

        // Unknown character: skip
        pos += 1;
        col += 1;
    }

    // Append EOF
    try tokens.append(Token{
        .kind = .eof,
        .text = "",
        .line = line,
        .col = col,
    });

    return tokens.toOwnedSlice();
}

// ============================================================================
// Parser State
// ============================================================================

pub const ParseState = struct {
    tokens: []const Token,
    pos: usize,
    allocator: Allocator,
};

/// Returns current token without consuming
pub fn peek(state: *ParseState) Token {
    if (state.pos < state.tokens.len) {
        return state.tokens[state.pos];
    }
    return Token{ .kind = .eof, .text = "", .line = 0, .col = 0 };
}

/// Consumes and returns current token
pub fn next(state: *ParseState) Token {
    const tok = peek(state);
    if (state.pos < state.tokens.len) {
        state.pos += 1;
    }
    return tok;
}

/// Consumes token if it matches kind; returns true if consumed
pub fn eat(state: *ParseState, kind: TokenKind) bool {
    if (peek(state).kind == kind) {
        _ = next(state);
        return true;
    }
    return false;
}

// ============================================================================
// Parser Functions
// ============================================================================

/// Main entry point: parses .t27 source and returns AST module node
pub fn parse(allocator: Allocator, source: []const u8) !ModuleNode {
    const tokens = try tokenize(allocator, source);
    var state = ParseState{
        .tokens = tokens,
        .pos = 0,
        .allocator = allocator,
    };
    return parseModule(&state);
}

/// Parses module declaration and all top-level declarations
pub fn parseModule(state: *ParseState) !ModuleNode {
    var name: []const u8 = "";

    // Optional: module NAME;
    if (eat(state, .module_)) {
        const name_tok = next(state);
        name = name_tok.text;
        _ = eat(state, .semicolon);
    }

    var decls = ArrayList(AstNode).init(state.allocator);
    defer decls.deinit();

    while (peek(state).kind != .eof) {
        if (try parseDecl(state)) |decl| {
            try decls.append(decl);
        }
    }

    return ModuleNode{
        .name = name,
        .decls = try decls.toOwnedSlice(),
    };
}

/// Parses a single top-level declaration
pub fn parseDecl(state: *ParseState) !?AstNode {
    const is_pub = eat(state, .pub_);

    if (eat(state, .const_)) {
        return try parseConstDecl(state, is_pub);
    } else if (eat(state, .enum_)) {
        return try parseEnumDecl(state);
    } else if (eat(state, .struct_)) {
        return try parseStructDecl(state);
    } else if (eat(state, .fn_)) {
        return try parseFnDecl(state);
    } else if (eat(state, .invariant)) {
        return try parseInvariantDecl(state);
    } else if (eat(state, .test_)) {
        return try parseTestDecl(state);
    } else if (eat(state, .bench)) {
        return try parseBenchDecl(state);
    }

    // Skip unknown token
    _ = next(state);
    return null;
}

/// Parses: const NAME : TYPE = VALUE;
pub fn parseConstDecl(state: *ParseState, is_pub: bool) !AstNode {
    const name_tok = next(state);
    _ = eat(state, .colon);

    // Parse type reference
    var type_parts = ArrayList(u8).init(state.allocator);
    defer type_parts.deinit();
    while (peek(state).kind == .ident or peek(state).kind == .l_bracket or
        peek(state).kind == .r_bracket or peek(state).kind == .star or
        peek(state).kind == .question)
    {
        const tok = next(state);
        try type_parts.appendSlice(tok.text);
    }

    _ = eat(state, .assign);

    // Parse value (capture until semicolon)
    var value_parts = ArrayList(u8).init(state.allocator);
    defer value_parts.deinit();
    var first_val = true;
    while (peek(state).kind != .semicolon and peek(state).kind != .eof) {
        if (!first_val) {
            try value_parts.append(' ');
        }
        const tok = next(state);
        try value_parts.appendSlice(tok.text);
        first_val = false;
    }
    _ = eat(state, .semicolon);

    return AstNode{
        .const_decl = ConstDeclNode{
            .is_pub = is_pub,
            .name = name_tok.text,
            .type_ref = try type_parts.toOwnedSlice(),
            .value = try value_parts.toOwnedSlice(),
        },
    };
}

/// Parses: enum NAME (BACKING) { VALUES }
pub fn parseEnumDecl(state: *ParseState) !AstNode {
    // enum may have name or just (backing)
    var name: []const u8 = "";
    if (peek(state).kind == .ident) {
        const name_tok = next(state);
        name = name_tok.text;
    }
    _ = eat(state, .l_paren);
    const backing_tok = next(state);
    _ = eat(state, .r_paren);
    _ = eat(state, .l_brace);

    var values = ArrayList(EnumValueNode).init(state.allocator);
    defer values.deinit();

    while (peek(state).kind != .r_brace and peek(state).kind != .eof) {
        const val_name_tok = next(state);
        _ = eat(state, .assign);

        var val_parts = ArrayList(u8).init(state.allocator);
        defer val_parts.deinit();
        var first_v = true;
        while (peek(state).kind != .comma and peek(state).kind != .r_brace and
            peek(state).kind != .eof)
        {
            if (!first_v) {
                try val_parts.append(' ');
            }
            const tok = next(state);
            try val_parts.appendSlice(tok.text);
            first_v = false;
        }

        try values.append(EnumValueNode{
            .name = val_name_tok.text,
            .value = try val_parts.toOwnedSlice(),
        });
        _ = eat(state, .comma);
    }
    _ = eat(state, .r_brace);

    return AstNode{
        .enum_decl = EnumDeclNode{
            .name = name,
            .backing = backing_tok.text,
            .values = try values.toOwnedSlice(),
        },
    };
}

/// Parses: struct NAME { FIELDS }
pub fn parseStructDecl(state: *ParseState) !AstNode {
    const name_tok = next(state);
    _ = eat(state, .l_brace);

    var fields = ArrayList(FieldNode).init(state.allocator);
    defer fields.deinit();

    while (peek(state).kind != .r_brace and peek(state).kind != .eof) {
        const field_name_tok = next(state);
        _ = eat(state, .colon);

        var type_parts = ArrayList(u8).init(state.allocator);
        defer type_parts.deinit();
        while (peek(state).kind == .ident or peek(state).kind == .l_bracket or
            peek(state).kind == .r_bracket or peek(state).kind == .star or
            peek(state).kind == .question)
        {
            const tok = next(state);
            try type_parts.appendSlice(tok.text);
        }

        try fields.append(FieldNode{
            .name = field_name_tok.text,
            .type_ref = try type_parts.toOwnedSlice(),
        });
        _ = eat(state, .comma);
    }
    _ = eat(state, .r_brace);

    return AstNode{
        .struct_decl = StructDeclNode{
            .name = name_tok.text,
            .fields = try fields.toOwnedSlice(),
        },
    };
}

/// Parses: fn NAME (PARAMS) [-> RETURN] { BODY }
pub fn parseFnDecl(state: *ParseState) !AstNode {
    const name_tok = next(state);
    _ = eat(state, .l_paren);

    var params = ArrayList(ParamNode).init(state.allocator);
    defer params.deinit();

    while (peek(state).kind != .r_paren and peek(state).kind != .eof) {
        const param_name_tok = next(state);
        _ = eat(state, .colon);

        var type_parts = ArrayList(u8).init(state.allocator);
        defer type_parts.deinit();
        while (peek(state).kind == .ident or peek(state).kind == .l_bracket or
            peek(state).kind == .r_bracket or peek(state).kind == .star or
            peek(state).kind == .question)
        {
            const tok = next(state);
            try type_parts.appendSlice(tok.text);
        }

        try params.append(ParamNode{
            .name = param_name_tok.text,
            .type_ref = try type_parts.toOwnedSlice(),
        });
        _ = eat(state, .comma);
    }
    _ = eat(state, .r_paren);

    // Return type
    var return_type: []const u8 = "void";
    if (eat(state, .arrow)) {
        const ret_tok = next(state);
        return_type = ret_tok.text;
    } else if (peek(state).kind == .ident) {
        // Allow return type without arrow (spec style)
        const ret_tok = next(state);
        return_type = ret_tok.text;
    }

    // Skip body
    if (eat(state, .l_brace)) {
        var brace_count: u32 = 1;
        while (brace_count > 0 and peek(state).kind != .eof) {
            const tok = next(state);
            if (tok.kind == .l_brace) brace_count += 1;
            if (tok.kind == .r_brace) brace_count -= 1;
        }
    }

    return AstNode{
        .fn_decl = FnDeclNode{
            .name = name_tok.text,
            .params = try params.toOwnedSlice(),
            .return_type = return_type,
        },
    };
}

/// Parses: invariant NAME { BODY }
pub fn parseInvariantDecl(state: *ParseState) !AstNode {
    const name_tok = next(state);

    // Skip body
    if (eat(state, .l_brace)) {
        var brace_count: u32 = 1;
        while (brace_count > 0 and peek(state).kind != .eof) {
            const tok = next(state);
            if (tok.kind == .l_brace) brace_count += 1;
            if (tok.kind == .r_brace) brace_count -= 1;
        }
    }

    return AstNode{
        .invariant_decl = InvariantDeclNode{ .name = name_tok.text },
    };
}

/// Parses: test "NAME" { BODY }
pub fn parseTestDecl(state: *ParseState) !AstNode {
    const name_tok = next(state);

    // Skip body
    if (eat(state, .l_brace)) {
        var brace_count: u32 = 1;
        while (brace_count > 0 and peek(state).kind != .eof) {
            const tok = next(state);
            if (tok.kind == .l_brace) brace_count += 1;
            if (tok.kind == .r_brace) brace_count -= 1;
        }
    }

    return AstNode{
        .test_decl = TestDeclNode{ .name = name_tok.text },
    };
}

/// Parses: bench "NAME" { BODY }
pub fn parseBenchDecl(state: *ParseState) !AstNode {
    const name_tok = next(state);

    // Skip body
    if (eat(state, .l_brace)) {
        var brace_count: u32 = 1;
        while (brace_count > 0 and peek(state).kind != .eof) {
            const tok = next(state);
            if (tok.kind == .l_brace) brace_count += 1;
            if (tok.kind == .r_brace) brace_count -= 1;
        }
    }

    return AstNode{
        .bench_decl = BenchDeclNode{ .name = name_tok.text },
    };
}

// ============================================================================
// JSON Output
// ============================================================================

/// Convert AST ModuleNode to JSON string
pub fn moduleToJson(allocator: Allocator, mod: ModuleNode) ![]u8 {
    var buf = ArrayList(u8).init(allocator);
    const writer = buf.writer();

    try writer.writeAll("{\"module\":\"");
    try writeJsonEscaped(writer, mod.name);
    try writer.writeAll("\",\"decls\":[");

    for (mod.decls, 0..) |decl, i| {
        if (i > 0) try writer.writeAll(",");
        try astNodeToJson(writer, decl);
    }

    try writer.writeAll("]}");
    return buf.toOwnedSlice();
}

/// Convert a single AstNode to JSON, writing to the given writer
fn astNodeToJson(writer: anytype, node: AstNode) !void {
    switch (node) {
        .module => |m| {
            try writer.writeAll("{\"kind\":\"module\",\"name\":\"");
            try writeJsonEscaped(writer, m.name);
            try writer.writeAll("\"}");
        },
        .const_decl => |c| {
            try writer.writeAll("{\"kind\":\"const\",\"pub\":");
            try writer.writeAll(if (c.is_pub) "true" else "false");
            try writer.writeAll(",\"name\":\"");
            try writeJsonEscaped(writer, c.name);
            try writer.writeAll("\",\"type\":\"");
            try writeJsonEscaped(writer, c.type_ref);
            try writer.writeAll("\",\"value\":\"");
            try writeJsonEscaped(writer, c.value);
            try writer.writeAll("\"}");
        },
        .enum_decl => |e| {
            try writer.writeAll("{\"kind\":\"enum\",\"name\":\"");
            try writeJsonEscaped(writer, e.name);
            try writer.writeAll("\",\"backing\":\"");
            try writeJsonEscaped(writer, e.backing);
            try writer.writeAll("\",\"values\":[");
            for (e.values, 0..) |v, i| {
                if (i > 0) try writer.writeAll(",");
                try writer.writeAll("{\"name\":\"");
                try writeJsonEscaped(writer, v.name);
                try writer.writeAll("\",\"value\":\"");
                try writeJsonEscaped(writer, v.value);
                try writer.writeAll("\"}");
            }
            try writer.writeAll("]}");
        },
        .struct_decl => |s| {
            try writer.writeAll("{\"kind\":\"struct\",\"name\":\"");
            try writeJsonEscaped(writer, s.name);
            try writer.writeAll("\",\"fields\":[");
            for (s.fields, 0..) |f, i| {
                if (i > 0) try writer.writeAll(",");
                try writer.writeAll("{\"name\":\"");
                try writeJsonEscaped(writer, f.name);
                try writer.writeAll("\",\"type\":\"");
                try writeJsonEscaped(writer, f.type_ref);
                try writer.writeAll("\"}");
            }
            try writer.writeAll("]}");
        },
        .fn_decl => |func| {
            try writer.writeAll("{\"kind\":\"fn\",\"name\":\"");
            try writeJsonEscaped(writer, func.name);
            try writer.writeAll("\",\"params\":[");
            for (func.params, 0..) |p, i| {
                if (i > 0) try writer.writeAll(",");
                try writer.writeAll("{\"name\":\"");
                try writeJsonEscaped(writer, p.name);
                try writer.writeAll("\",\"type\":\"");
                try writeJsonEscaped(writer, p.type_ref);
                try writer.writeAll("\"}");
            }
            try writer.writeAll("],\"return\":\"");
            try writeJsonEscaped(writer, func.return_type);
            try writer.writeAll("\"}");
        },
        .invariant_decl => |inv| {
            try writer.writeAll("{\"kind\":\"invariant\",\"name\":\"");
            try writeJsonEscaped(writer, inv.name);
            try writer.writeAll("\"}");
        },
        .test_decl => |t| {
            try writer.writeAll("{\"kind\":\"test\",\"name\":\"");
            try writeJsonEscaped(writer, t.name);
            try writer.writeAll("\"}");
        },
        .bench_decl => |b| {
            try writer.writeAll("{\"kind\":\"bench\",\"name\":\"");
            try writeJsonEscaped(writer, b.name);
            try writer.writeAll("\"}");
        },
    }
}

/// Write a JSON-escaped string (handles \, ", control chars)
fn writeJsonEscaped(writer: anytype, s: []const u8) !void {
    for (s) |c| {
        switch (c) {
            '"' => try writer.writeAll("\\\""),
            '\\' => try writer.writeAll("\\\\"),
            '\n' => try writer.writeAll("\\n"),
            '\r' => try writer.writeAll("\\r"),
            '\t' => try writer.writeAll("\\t"),
            else => try writer.writeByte(c),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

test "test_isIdentStart" {
    try std.testing.expect(isIdentStart('a'));
    try std.testing.expect(isIdentStart('Z'));
    try std.testing.expect(isIdentStart('_'));
    try std.testing.expect(!isIdentStart('0'));
    try std.testing.expect(!isIdentStart('+'));
}

test "test_isDigit" {
    try std.testing.expect(isDigit('0'));
    try std.testing.expect(isDigit('9'));
    try std.testing.expect(!isDigit('a'));
}

test "test_keywordKind" {
    try std.testing.expectEqual(TokenKind.pub_, keywordKind("pub"));
    try std.testing.expectEqual(TokenKind.const_, keywordKind("const"));
    try std.testing.expectEqual(TokenKind.fn_, keywordKind("fn"));
    try std.testing.expectEqual(TokenKind.ident, keywordKind("foo"));
}

test "test_singleCharTokenKind" {
    try std.testing.expectEqual(TokenKind.plus, singleCharTokenKind('+'));
    try std.testing.expectEqual(TokenKind.l_paren, singleCharTokenKind('('));
    try std.testing.expectEqual(TokenKind.semicolon, singleCharTokenKind(';'));
    try std.testing.expectEqual(TokenKind.eof, singleCharTokenKind('~'));
}

test "test_tokenize_identifiers" {
    const allocator = std.testing.allocator;
    const source = "module test_mod";
    const tokens = try tokenize(allocator, source);
    defer allocator.free(tokens);

    try std.testing.expectEqual(TokenKind.module_, tokens[0].kind);
    try std.testing.expectEqual(TokenKind.ident, tokens[1].kind);
    try std.testing.expect(mem.eql(u8, tokens[1].text, "test_mod"));
    try std.testing.expectEqual(TokenKind.eof, tokens[2].kind);
}

test "test_tokenize_keywords" {
    const allocator = std.testing.allocator;
    const source = "pub const extern struct enum fn return if else using test bench invariant module type";
    const tokens = try tokenize(allocator, source);
    defer allocator.free(tokens);

    try std.testing.expectEqual(TokenKind.pub_, tokens[0].kind);
    try std.testing.expectEqual(TokenKind.const_, tokens[1].kind);
    try std.testing.expectEqual(TokenKind.extern_, tokens[2].kind);
    try std.testing.expectEqual(TokenKind.struct_, tokens[3].kind);
    try std.testing.expectEqual(TokenKind.enum_, tokens[4].kind);
    try std.testing.expectEqual(TokenKind.fn_, tokens[5].kind);
}

test "test_tokenize_numbers" {
    const allocator = std.testing.allocator;
    const source = "0 1 -1 42 -99";
    const tokens = try tokenize(allocator, source);
    defer allocator.free(tokens);

    try std.testing.expectEqual(TokenKind.literal, tokens[0].kind);
    try std.testing.expect(mem.eql(u8, tokens[0].text, "0"));
    try std.testing.expectEqual(TokenKind.literal, tokens[1].kind);
    try std.testing.expect(mem.eql(u8, tokens[1].text, "1"));
    try std.testing.expectEqual(TokenKind.literal, tokens[2].kind);
    try std.testing.expect(mem.eql(u8, tokens[2].text, "-1"));
}

test "test_tokenize_string_literals" {
    const allocator = std.testing.allocator;
    const source =
        \\  "hello" "world"
    ;
    const tokens = try tokenize(allocator, source);
    defer allocator.free(tokens);

    try std.testing.expectEqual(TokenKind.literal, tokens[0].kind);
    try std.testing.expect(mem.eql(u8, tokens[0].text, "hello"));
    try std.testing.expectEqual(TokenKind.literal, tokens[1].kind);
    try std.testing.expect(mem.eql(u8, tokens[1].text, "world"));
}

test "test_tokenize_comments_semicolon" {
    const allocator = std.testing.allocator;
    const source = "; this is a comment\nx = 1";
    const tokens = try tokenize(allocator, source);
    defer allocator.free(tokens);

    // Comment should be skipped
    try std.testing.expectEqual(TokenKind.ident, tokens[0].kind);
    try std.testing.expect(mem.eql(u8, tokens[0].text, "x"));
}

test "test_tokenize_comments_double_slash" {
    const allocator = std.testing.allocator;
    const source = "// this is a comment\nx = 1";
    const tokens = try tokenize(allocator, source);
    defer allocator.free(tokens);

    // Comment should be skipped
    try std.testing.expectEqual(TokenKind.ident, tokens[0].kind);
    try std.testing.expect(mem.eql(u8, tokens[0].text, "x"));
}

test "test_tokenize_operators_arrow" {
    const allocator = std.testing.allocator;
    const source = "->";
    const tokens = try tokenize(allocator, source);
    defer allocator.free(tokens);

    try std.testing.expectEqual(TokenKind.arrow, tokens[0].kind);
    try std.testing.expect(mem.eql(u8, tokens[0].text, "->"));
}

test "test_tokenize_operators_multi_char" {
    const allocator = std.testing.allocator;
    const source = "== != <= >=";
    const tokens = try tokenize(allocator, source);
    defer allocator.free(tokens);

    try std.testing.expectEqual(TokenKind.eq, tokens[0].kind);
    try std.testing.expect(mem.eql(u8, tokens[0].text, "=="));
    try std.testing.expectEqual(TokenKind.ne, tokens[1].kind);
    try std.testing.expect(mem.eql(u8, tokens[1].text, "!="));
    try std.testing.expectEqual(TokenKind.le, tokens[2].kind);
    try std.testing.expect(mem.eql(u8, tokens[2].text, "<="));
    try std.testing.expectEqual(TokenKind.ge, tokens[3].kind);
    try std.testing.expect(mem.eql(u8, tokens[3].text, ">="));
}

test "test_parse_module_with_name" {
    const allocator = std.testing.allocator;
    const source = "module mymod;";
    const ast = try parse(allocator, source);
    defer allocator.free(ast.decls);

    try std.testing.expect(mem.eql(u8, ast.name, "mymod"));
}

test "test_parse_const_decl" {
    const allocator = std.testing.allocator;
    const source = "const x : i8 = 42;";
    const ast = try parse(allocator, source);
    defer allocator.free(ast.decls);

    try std.testing.expect(ast.decls.len == 1);
    try std.testing.expect(mem.eql(u8, ast.decls[0].const_decl.name, "x"));
    defer allocator.free(ast.decls[0].const_decl.type_ref);
    defer allocator.free(ast.decls[0].const_decl.value);
    try std.testing.expect(mem.eql(u8, ast.decls[0].const_decl.type_ref, "i8"));
}

test "test_parse_pub_const_decl" {
    const allocator = std.testing.allocator;
    const source = "pub const X : u8 = 1;";
    const ast = try parse(allocator, source);
    defer allocator.free(ast.decls);

    try std.testing.expect(ast.decls[0].const_decl.is_pub == true);
    try std.testing.expect(mem.eql(u8, ast.decls[0].const_decl.name, "X"));
    defer allocator.free(ast.decls[0].const_decl.type_ref);
    defer allocator.free(ast.decls[0].const_decl.value);
}

test "test_parse_struct_decl" {
    const allocator = std.testing.allocator;
    const source = "struct Point { x : i8, y : i8 }";
    const ast = try parse(allocator, source);
    defer allocator.free(ast.decls);

    try std.testing.expect(mem.eql(u8, ast.decls[0].struct_decl.name, "Point"));
    try std.testing.expect(ast.decls[0].struct_decl.fields.len == 2);
    defer {
        for (ast.decls[0].struct_decl.fields) |f| {
            allocator.free(f.type_ref);
        }
        allocator.free(ast.decls[0].struct_decl.fields);
    }
}

test "test_parse_fn_decl_void_return" {
    const allocator = std.testing.allocator;
    const source = "fn foo() { }";
    const ast = try parse(allocator, source);
    defer allocator.free(ast.decls);

    try std.testing.expect(mem.eql(u8, ast.decls[0].fn_decl.name, "foo"));
    try std.testing.expect(mem.eql(u8, ast.decls[0].fn_decl.return_type, "void"));
    defer allocator.free(ast.decls[0].fn_decl.params);
}

test "test_parse_invariant_decl" {
    const allocator = std.testing.allocator;
    const source = "invariant trit_value_range { }";
    const ast = try parse(allocator, source);
    defer allocator.free(ast.decls);

    try std.testing.expect(mem.eql(u8, ast.decls[0].invariant_decl.name, "trit_value_range"));
}

test "test_parse_test_decl" {
    const allocator = std.testing.allocator;
    const source = "test \"simple test\" { }";
    const ast = try parse(allocator, source);
    defer allocator.free(ast.decls);

    try std.testing.expect(mem.eql(u8, ast.decls[0].test_decl.name, "simple test"));
}

test "test_parse_bench_decl" {
    const allocator = std.testing.allocator;
    const source = "bench \"measure\" { }";
    const ast = try parse(allocator, source);
    defer allocator.free(ast.decls);

    try std.testing.expect(mem.eql(u8, ast.decls[0].bench_decl.name, "measure"));
}
