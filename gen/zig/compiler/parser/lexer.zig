// Auto-generated from compiler/parser/lexer.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/parser/lexer.t27
// phi^2 + phi^-2 = 3 | TRINITY

const std = @import("std");

// ============================================================================
// Configuration
// ============================================================================

pub const MAX_IDENTIFIER_LEN: usize = 64;
pub const MAX_NUMBER_LEN: usize = 20;
pub const MAX_TOKENS: usize = 10000;

// ============================================================================
// Token Types
// ============================================================================

pub const TokenType = enum(u8) {
    eof = 0,
    newline = 1,
    dot = 2,
    colon = 3,
    semicolon = 4,
    comma = 5,
    hash = 6,
    lparen = 7,
    rparen = 8,
    lbracket = 9,
    rbracket = 10,
    plus = 11,
    minus = 12,
    star = 13,
    slash = 14,
    percent = 15,
    and_ = 16,
    or_ = 17,
    xor = 18,
    tilde = 19,
    lt = 20,
    gt = 21,
    eq = 22,
    excl = 23,
    // Keywords
    use = 24,
    const_ = 25,
    data = 26,
    code = 27,
    dword = 28,
    dspace = 29,
    dtrit = 30,
    // TDD sections
    test_ = 31,
    invariant = 32,
    bench = 33,
    verify = 34,
    expected = 35,
    setup = 36,
    rationale = 37,
    measure = 38,
    target = 39,
    // Spec-style
    spec = 40,
    rule = 41,
    given = 42,
    when_ = 43,
    then_ = 44,
    assert_ = 45,
    and_kw = 46,
    expect = 47,
    // Literals
    integer = 48,
    float_ = 49,
    string = 50,
    identifier = 51,
    reg = 52,
    label = 53,
    // Opcodes
    mov = 60,
    jz = 61,
    jnz = 62,
    jmp = 63,
    halt = 78,
};

// ============================================================================
// Token
// ============================================================================

pub const Token = struct {
    kind: TokenType,
    start: usize,
    end: usize,
    line: u32,
    column: u32,
};

// ============================================================================
// Lexer
// ============================================================================

pub const Lexer = struct {
    source: []const u8,
    pos: usize,
    line: u32,
    column: u32,

    pub fn init(source: []const u8) Lexer {
        return Lexer{
            .source = source,
            .pos = 0,
            .line = 1,
            .column = 1,
        };
    }

    pub fn is_ident_start(c: u8) bool {
        return (c >= 'a' and c <= 'z') or (c >= 'A' and c <= 'Z') or c == '_';
    }

    pub fn is_ident_continue(c: u8) bool {
        return is_ident_start(c) or (c >= '0' and c <= '9') or c == '-';
    }

    pub fn is_digit(c: u8) bool {
        return c >= '0' and c <= '9';
    }

    pub fn is_whitespace(c: u8) bool {
        return c == ' ' or c == '\t' or c == '\r';
    }

    pub fn advance(self: *Lexer) void {
        if (self.pos < self.source.len) {
            if (self.source[self.pos] == '\n') {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.pos += 1;
        }
    }

    pub fn peek(self: *const Lexer) ?u8 {
        if (self.pos < self.source.len) {
            return self.source[self.pos];
        }
        return null;
    }

    pub fn next_token(self: *Lexer) Token {
        // Skip whitespace (not newlines)
        while (self.pos < self.source.len and is_whitespace(self.source[self.pos])) {
            self.advance();
        }

        if (self.pos >= self.source.len) {
            return Token{ .kind = .eof, .start = self.pos, .end = self.pos, .line = self.line, .column = self.column };
        }

        const start = self.pos;
        const c = self.source[self.pos];
        const line = self.line;
        const col = self.column;

        // Single char tokens
        const single: ?TokenType = switch (c) {
            '\n' => .newline,
            '.' => .dot,
            ':' => .colon,
            ';' => .semicolon,
            ',' => .comma,
            '#' => .hash,
            '(' => .lparen,
            ')' => .rparen,
            '[' => .lbracket,
            ']' => .rbracket,
            '+' => .plus,
            '-' => .minus,
            '*' => .star,
            '/' => .slash,
            '%' => .percent,
            '&' => .and_,
            '|' => .or_,
            '^' => .xor,
            '~' => .tilde,
            '<' => .lt,
            '>' => .gt,
            '=' => .eq,
            '!' => .excl,
            else => null,
        };

        if (single) |kind| {
            self.advance();
            return Token{ .kind = kind, .start = start, .end = self.pos, .line = line, .column = col };
        }

        // Identifier or keyword
        if (is_ident_start(c)) {
            while (self.pos < self.source.len and is_ident_continue(self.source[self.pos])) {
                self.advance();
            }
            return Token{ .kind = .identifier, .start = start, .end = self.pos, .line = line, .column = col };
        }

        // Number
        if (is_digit(c)) {
            while (self.pos < self.source.len and is_digit(self.source[self.pos])) {
                self.advance();
            }
            return Token{ .kind = .integer, .start = start, .end = self.pos, .line = line, .column = col };
        }

        // String
        if (c == '"') {
            self.advance();
            while (self.pos < self.source.len and self.source[self.pos] != '"') {
                self.advance();
            }
            if (self.pos < self.source.len) self.advance();
            return Token{ .kind = .string, .start = start, .end = self.pos, .line = line, .column = col };
        }

        // Unknown -- advance past it
        self.advance();
        return Token{ .kind = .eof, .start = start, .end = self.pos, .line = line, .column = col };
    }
};

// ============================================================================
// Tests (from spec TDD-Inside-Spec)
// ============================================================================

test "is_ident_start" {
    try std.testing.expect(Lexer.is_ident_start('a'));
    try std.testing.expect(Lexer.is_ident_start('Z'));
    try std.testing.expect(Lexer.is_ident_start('_'));
    try std.testing.expect(!Lexer.is_ident_start('0'));
    try std.testing.expect(!Lexer.is_ident_start('+'));
}

test "is_ident_continue" {
    try std.testing.expect(Lexer.is_ident_continue('a'));
    try std.testing.expect(Lexer.is_ident_continue('0'));
    try std.testing.expect(Lexer.is_ident_continue('-'));
    try std.testing.expect(!Lexer.is_ident_continue('+'));
}

test "is_digit" {
    try std.testing.expect(Lexer.is_digit('0'));
    try std.testing.expect(Lexer.is_digit('9'));
    try std.testing.expect(!Lexer.is_digit('a'));
}

test "lexer_init" {
    const lexer = Lexer.init("test");
    try std.testing.expect(lexer.pos == 0);
    try std.testing.expect(lexer.line == 1);
    try std.testing.expect(lexer.column == 1);
}

test "lexer_tokenize_simple" {
    var lexer = Lexer.init("hello");
    const tok = lexer.next_token();
    try std.testing.expect(tok.kind == .identifier);
    try std.testing.expect(tok.start == 0);
    try std.testing.expect(tok.end == 5);
}

test "lexer_tokenize_number" {
    var lexer = Lexer.init("42");
    const tok = lexer.next_token();
    try std.testing.expect(tok.kind == .integer);
}

test "lexer_tokenize_punctuation" {
    var lexer = Lexer.init("+");
    const tok = lexer.next_token();
    try std.testing.expect(tok.kind == .plus);
}

test "lexer_eof" {
    var lexer = Lexer.init("");
    const tok = lexer.next_token();
    try std.testing.expect(tok.kind == .eof);
}

test "lexer_tracks_line" {
    var lexer = Lexer.init("a\nb");
    _ = lexer.next_token(); // 'a'
    _ = lexer.next_token(); // '\n'
    const tok = lexer.next_token(); // 'b'
    try std.testing.expect(tok.line == 2);
}
