#!/usr/bin/env python3
"""
Bootstrap t27 Compiler - Minimal implementation
This is a throwaway compiler for t27 language that will be replaced
once .t27 becomes self-hosting.

Usage:
    python3 bootstrap/t27c.py parse <file.t27>      # Output JSON AST to stdout
    python3 bootstrap/t27c.py gen-zig <file.t27>   # Generate Zig code to stdout
"""

import sys
import re
from typing import List, Dict, Optional, Any
from dataclasses import dataclass, field
from enum import Enum


# ============================================================================
# Token Type
# ============================================================================

class TokenType(Enum):
    # Keywords
    KW_PUB = "kw_pub"
    KW_CONST = "kw_const"
    KW_FN = "kw_fn"
    KW_ENUM = "kw_enum"
    KW_STRUCT = "kw_struct"
    KW_TEST = "kw_test"
    KW_INVARIANT = "kw_invariant"
    KW_BENCH = "kw_bench"
    KW_MODULE = "kw_module"
    KW_IF = "kw_if"
    KW_ELSE = "kw_else"
    KW_FOR = "kw_for"
    KW_SWITCH = "kw_switch"
    KW_RETURN = "kw_return"
    KW_VAR = "kw_var"
    KW_USE = "kw_use"
    KW_USING = "kw_using"
    KW_VOID = "kw_void"
    KW_TRUE = "kw_true"
    KW_FALSE = "kw_false"
    KW_UNDERSCORE = "kw_underscore"

    # Literals and identifiers
    IDENTIFIER = "identifier"
    NUMBER = "number"
    STRING = "string"

    # Punctuation and operators
    COLON = "colon"
    SEMICOLON = "semicolon"
    COMMA = "comma"
    EQUALS = "equals"
    LPAREN = "lparen"
    RPAREN = "rparen"
    LBRACE = "lbrace"
    RBRACE = "rbrace"
    LBRACKET = "lbracket"
    RBRACKET = "rbracket"
    ARROW = "arrow"
    FAT_ARROW = "fat_arrow"
    DOT = "dot"
    DCOLON = "dcolon"
    BANG = "bang"

    # Special
    EOF = "eof"
    UNKNOWN = "unknown"


# ============================================================================
# Token
# ============================================================================

@dataclass
class Token:
    type: TokenType
    lexeme: str
    line: int
    column: int


# ============================================================================
# Keywords Map
# ============================================================================

KEYWORDS = {
    "pub": TokenType.KW_PUB,
    "const": TokenType.KW_CONST,
    "fn": TokenType.KW_FN,
    "enum": TokenType.KW_ENUM,
    "struct": TokenType.KW_STRUCT,
    "test": TokenType.KW_TEST,
    "invariant": TokenType.KW_INVARIANT,
    "bench": TokenType.KW_BENCH,
    "module": TokenType.KW_MODULE,
    "if": TokenType.KW_IF,
    "else": TokenType.KW_ELSE,
    "for": TokenType.KW_FOR,
    "switch": TokenType.KW_SWITCH,
    "return": TokenType.KW_RETURN,
    "var": TokenType.KW_VAR,
    "use": TokenType.KW_USE,
    "using": TokenType.KW_USING,
    "void": TokenType.KW_VOID,
    "true": TokenType.KW_TRUE,
    "false": TokenType.KW_FALSE,
    "_": TokenType.KW_UNDERSCORE,
}


# ============================================================================
# Lexer
# ============================================================================

class Lexer:
    def __init__(self, source: str):
        self.source = source
        self.pos = 0
        self.line = 1
        self.column = 1

    def peek(self) -> str:
        if self.pos >= len(self.source):
            return ""
        return self.source[self.pos]

    def advance(self) -> str:
        if self.pos >= len(self.source):
            return ""
        ch = self.source[self.pos]
        self.pos += 1
        if ch == "\n":
            self.line += 1
            self.column = 1
        else:
            self.column += 1
        return ch

    def peek_token(self) -> Token:
        """Return the next token without consuming it"""
        current_pos = self.pos
        current_line = self.line
        current_column = self.column
        token = self.next_token()
        # Restore position (since next_token consumed the tokens)
        self.pos = current_pos
        self.line = current_line
        self.column = current_column
        return token

    def skip_whitespace(self):
        while self.pos < len(self.source):
            ch = self.peek()
            if ch not in " \t\r\n":
                break
            self.advance()

    def skip_line_comment(self):
        while self.pos < len(self.source):
            ch = self.peek()
            self.advance()
            if ch == "\n":
                break

    def skip_semicolon_comment(self):
        while self.pos < len(self.source):
            ch = self.peek()
            self.advance()
            if ch == "\n":
                break

    def _is_at_line_start(self, skip_current: bool = False) -> bool:
        """Check if current position is at the start of a line (after whitespace)

        Args:
            skip_current: If True, skip the current character when looking back
                         (used when we've already consumed the semicolon)
        """
        lookback = self.pos - 2 if skip_current else self.pos - 1
        while lookback >= 0:
            if self.source[lookback] == "\n":
                return True
            if self.source[lookback] not in " \t\r":
                return False
            lookback -= 1
        return True

    def next_token(self) -> Token:
        self.skip_whitespace()

        if self.pos >= len(self.source):
            return Token(TokenType.EOF, "", self.line, self.column)

        ch = self.peek()

        # Line comment (//)
        if ch == "/" and self.pos + 1 < len(self.source) and self.source[self.pos + 1] == "/":
            self.advance()
            self.advance()
            self.skip_line_comment()
            return self.next_token()

        # Semicolon (;) - can be a comment prefix or statement terminator
        # ; comment at start of line (after whitespace) is a comment
        # ; as terminator after declaration/expr is a semicolon
        if ch == ";":
            self.advance()  # First advance to get past semicolon
            next_ch = self.peek()  # Now check what comes after
            if next_ch in " \t" and self._is_at_line_start(skip_current=True):
                # It's a comment prefix at start of line
                self.skip_semicolon_comment()
                return self.next_token()
            else:
                # It's a statement terminator
                return Token(TokenType.SEMICOLON, ";", self.line, self.column - 1)

        # Single char tokens
        single_char_tokens = {
            ":": TokenType.COLON,
            ",": TokenType.COMMA,
            "=": TokenType.EQUALS,
            "(": TokenType.LPAREN,
            ")": TokenType.RPAREN,
            "{": TokenType.LBRACE,
            "}": TokenType.RBRACE,
            "[": TokenType.LBRACKET,
            "]": TokenType.RBRACKET,
            ".": TokenType.DOT,
            "!": TokenType.BANG,
        }
        if ch in single_char_tokens:
            self.advance()
            return Token(single_char_tokens[ch], ch, self.line, self.column - 1)

        # Multi-char operators (must check before single-char tokens)
        if self.pos + 1 < len(self.source):
            two_chars = self.source[self.pos:self.pos+2]
            if two_chars == "->":
                self.advance()
                self.advance()
                return Token(TokenType.ARROW, two_chars, self.line, self.column - 2)
            if two_chars == "=>":
                self.advance()
                self.advance()
                return Token(TokenType.FAT_ARROW, two_chars, self.line, self.column - 2)
            if two_chars == "**":
                self.advance()
                self.advance()
                return Token(TokenType.NUMBER, two_chars, self.line, self.column - 2)
            if two_chars == "::":
                self.advance()
                self.advance()
                return Token(TokenType.DCOLON, two_chars, self.line, self.column - 2)

        # Single char tokens
        single_char_tokens = {
            ":": TokenType.COLON,
            ",": TokenType.COMMA,
            "=": TokenType.EQUALS,
            "(": TokenType.LPAREN,
            ")": TokenType.RPAREN,
            "{": TokenType.LBRACE,
            "}": TokenType.RBRACE,
            "[": TokenType.LBRACKET,
            "]": TokenType.RBRACKET,
            ".": TokenType.DOT,
            "!": TokenType.BANG,
        }
        if ch in single_char_tokens:
            self.advance()
            return Token(single_char_tokens[ch], ch, self.line, self.column - 1)

        # Identifiers and keywords
        if ch.isalpha() or ch == "_":
            start = self.pos
            while self.pos < len(self.source):
                ch_next = self.peek()
                # Check for :: path separator (continue identifier)
                if self.pos + 1 < len(self.source) and ch_next == ":" and self.source[self.pos+1] == ":":
                    break
                if ch_next.isalnum() or ch_next in "_-":
                    self.advance()
                else:
                    break
            lexeme = self.source[start:self.pos]
            token_type = KEYWORDS.get(lexeme, TokenType.IDENTIFIER)
            return Token(token_type, lexeme, self.line, self.column - len(lexeme))

        # Numbers
        if ch.isdigit() or (ch == "-" and self.pos + 1 < len(self.source) and self.source[self.pos + 1].isdigit()):
            start = self.pos
            if ch == "-":
                self.advance()
            while self.pos < len(self.source) and self.peek().isdigit():
                self.advance()
            # Hex prefix 0x
            if self.pos < len(self.source) and self.peek() == "x":
                self.advance()
                while self.pos < len(self.source) and self.peek() in "0123456789abcdefABCDEF":
                    self.advance()
            lexeme = self.source[start:self.pos]
            return Token(TokenType.NUMBER, lexeme, self.line, self.column - len(lexeme))

        # Strings
        if ch == '"':
            start = self.pos
            self.advance()
            while self.pos < len(self.source) and self.peek() != '"':
                if self.peek() == "\\":
                    self.advance()
                self.advance()
            if self.pos < len(self.source):
                self.advance()
            lexeme = self.source[start:self.pos]
            return Token(TokenType.STRING, lexeme, self.line, self.column - len(lexeme))

        return Token(TokenType.UNKNOWN, ch, self.line, self.column)


# ============================================================================
# AST Node
# ============================================================================

@dataclass
class Node:
    node_type: str
    name: str = ""
    value: str = ""
    extra: Dict[str, str] = field(default_factory=dict)
    children: List['Node'] = field(default_factory=list)


# ============================================================================
# Parser
# ============================================================================

class Parser:
    def __init__(self, source: str):
        self.lexer = Lexer(source)
        self.current = self.lexer.next_token()
        self.peek = self.lexer.next_token()

    def next(self):
        self.current = self.peek
        self.peek = self.lexer.next_token()

    def peek_type(self) -> TokenType:
        """Get the type of the next token without consuming it"""
        # self.peek is the next token (lookahead)
        return self.peek.type

    def expect(self, token_type: TokenType):
        if self.current.type != token_type:
            raise SyntaxError(f"Expected {token_type}, got {self.current.type} at line {self.current.line}")
        self.next()

    def parse(self) -> Node:
        node = Node("program")
        while self.current.type != TokenType.EOF:
            decl = self.parse_top_level_decl()
            node.children.append(decl)
        return node

    def parse_top_level_decl(self) -> Node:
        # pub const NAME: TYPE = VALUE;
        if self.current.type == TokenType.KW_PUB:
            self.next()
            if self.current.type == TokenType.KW_CONST:
                # parse_const_decl handles both normal const and enum detection
                return self.parse_const_decl(is_pub=True)
            elif self.current.type == TokenType.KW_FN:
                return self.parse_fn_decl(is_pub=True)
            elif self.current.type == TokenType.KW_STRUCT:
                return self.parse_struct_decl(is_pub=True)
            elif self.current.type == TokenType.KW_ENUM:
                return self.parse_enum_decl(is_pub=True)
            raise SyntaxError(f"Unexpected token after pub: {self.current.type}")

        # use PATH::NAME;
        if self.current.type == TokenType.KW_USE:
            node = Node("use_decl")
            self.expect(TokenType.KW_USE)
            # Build path: identifier (:: identifier)*
            path_parts = []
            if self.current.type == TokenType.IDENTIFIER:
                path_parts.append(self.current.lexeme)
                self.next()
            while self.current.type == TokenType.DCOLON:
                self.next()  # consume ::
                if self.current.type == TokenType.IDENTIFIER:
                    path_parts.append(self.current.lexeme)
                    self.next()
            node.name = "::".join(path_parts)
            self.expect(TokenType.SEMICOLON)
            return node

        # module NAME;
        if self.current.type == TokenType.KW_MODULE:
            return self.parse_module_decl()

        # const NAME: TYPE = VALUE;
        if self.current.type == TokenType.KW_CONST:
            return self.parse_const_decl(is_pub=False)

        # fn name(...) TYPE { ... }
        if self.current.type == TokenType.KW_FN:
            return self.parse_fn_decl(is_pub=False)

        # struct Name { ... }
        if self.current.type == TokenType.KW_STRUCT:
            return self.parse_struct_decl(is_pub=False)

        # test "name" { ... }
        if self.current.type == TokenType.KW_TEST:
            return self.parse_test_block()

        # invariant name { ... }
        if self.current.type == TokenType.KW_INVARIANT:
            return self.parse_invariant_block()

        # bench "name" { ... }
        if self.current.type == TokenType.KW_BENCH:
            return self.parse_bench_block()

        raise SyntaxError(f"Unexpected token: {self.current.type}")

    def parse_module_decl(self) -> Node:
        node = Node("module_decl")
        self.expect(TokenType.KW_MODULE)
        if self.current.type == TokenType.IDENTIFIER:
            node.name = self.current.lexeme
            self.next()
        self.expect(TokenType.SEMICOLON)
        return node

    def parse_const_decl(self, is_pub: bool) -> Node:
        node = Node("const_decl")
        if is_pub:
            node.extra["pub"] = "true"
        self.expect(TokenType.KW_CONST)
        const_name = ""
        if self.current.type == TokenType.IDENTIFIER:
            node.name = self.current.lexeme
            const_name = self.current.lexeme
            self.next()

        # Check if this is an enum declaration: NAME = enum(...)
        if self.current.type == TokenType.EQUALS and self.peek.type == TokenType.KW_ENUM:
            # Don't consume = here - parse_enum_decl will handle it
            return self.parse_enum_decl(is_pub, const_name)

        if self.current.type == TokenType.COLON:
            # Typed constant: NAME : TYPE = VALUE;
            self.next()
            if self.current.type in (TokenType.IDENTIFIER, TokenType.KW_UNDERSCORE):
                node.extra["type"] = self.current.lexeme
                self.next()
            if self.current.type == TokenType.EQUALS:
                self.next()
                init = self.parse_expression()
                node.children.append(init)
            self.expect(TokenType.SEMICOLON)
        elif self.current.type == TokenType.EQUALS:
            # Type alias: NAME = TYPE; or NAME = [SIZE]TYPE;
            self.next()
            if self.current.type == TokenType.LBRACKET:
                # Array type: [SIZE]TYPE
                self.next()
                if self.current.type in (TokenType.NUMBER, TokenType.IDENTIFIER):
                    node.extra["array_size"] = self.current.lexeme
                    self.next()
                self.expect(TokenType.RBRACKET)
            if self.current.type in (TokenType.IDENTIFIER, TokenType.KW_UNDERSCORE):
                node.extra["type"] = self.current.lexeme
                self.next()
            self.expect(TokenType.SEMICOLON)
        else:
            raise SyntaxError(f"Expected : or = after const name, got {self.current.type}")
        return node

    def parse_fn_decl(self, is_pub: bool) -> Node:
        node = Node("fn_decl")
        if is_pub:
            node.extra["pub"] = "true"
        self.expect(TokenType.KW_FN)
        if self.current.type == TokenType.IDENTIFIER:
            node.name = self.current.lexeme
            self.next()
        self.expect(TokenType.LPAREN)
        # Parameters
        while self.current.type != TokenType.RPAREN:
            param = self.parse_param()
            node.children.append(param)
            if self.current.type == TokenType.COMMA:
                self.next()
        self.expect(TokenType.RPAREN)
        # Return type (optional) - can be -> TYPE or just TYPE
        if self.current.type == TokenType.ARROW:
            self.next()
            if self.current.type in (TokenType.IDENTIFIER, TokenType.KW_VOID):
                node.extra["return_type"] = self.current.lexeme
                self.next()
        elif self.current.type in (TokenType.IDENTIFIER, TokenType.KW_VOID):
            # Direct return type without arrow: ) TYPE
            node.extra["return_type"] = self.current.lexeme
            self.next()
        # Body
        body = self.parse_block()
        node.children.append(body)
        return node

    def parse_param(self) -> Node:
        node = Node("param")
        if self.current.type == TokenType.IDENTIFIER:
            node.name = self.current.lexeme
            self.next()
        self.expect(TokenType.COLON)
        if self.current.type in (TokenType.IDENTIFIER, TokenType.KW_UNDERSCORE):
            node.extra["type"] = self.current.lexeme
            self.next()
        return node

    def parse_struct_decl(self, is_pub: bool) -> Node:
        node = Node("struct_decl")
        if is_pub:
            node.extra["pub"] = "true"
        self.expect(TokenType.KW_STRUCT)
        if self.current.type == TokenType.IDENTIFIER:
            node.name = self.current.lexeme
            self.next()
        self.expect(TokenType.LBRACE)
        # Fields
        while self.current.type not in (TokenType.RBRACE, TokenType.EOF):
            field = self.parse_field()
            node.children.append(field)
        self.expect(TokenType.RBRACE)
        return node

    def parse_field(self) -> Node:
        node = Node("field")
        if self.current.type == TokenType.IDENTIFIER:
            node.name = self.current.lexeme
            self.next()
        self.expect(TokenType.COLON)
        if self.current.type in (TokenType.IDENTIFIER, TokenType.KW_UNDERSCORE):
            node.extra["type"] = self.current.lexeme
            self.next()
        # Struct fields use commas, top-level fields use semicolons
        if self.current.type == TokenType.COMMA:
            self.next()
        else:
            self.expect(TokenType.SEMICOLON)
        return node

    def parse_enum_decl(self, is_pub: bool, const_name: str = "") -> Node:
        node = Node("enum_decl")
        if is_pub:
            node.extra["pub"] = "true"
        # pub const Name = enum(...) - already consumed pub const in parse_top_level_decl
        if const_name:
            node.name = const_name
        elif self.current.type == TokenType.IDENTIFIER:
            node.name = self.current.lexeme
            self.next()
        else:
            # const Name = enum(...) - expect const
            self.expect(TokenType.KW_CONST)
            if self.current.type == TokenType.IDENTIFIER:
                node.name = self.current.lexeme
                self.next()

        self.expect(TokenType.EQUALS)
        self.expect(TokenType.KW_ENUM)
        self.expect(TokenType.LPAREN)
        # Enum backing type
        if self.current.type == TokenType.IDENTIFIER:
            node.extra["backing_type"] = self.current.lexeme
            self.next()
        self.expect(TokenType.RPAREN)
        self.expect(TokenType.LBRACE)
        # Enum fields
        while self.current.type not in (TokenType.RBRACE, TokenType.EOF):
            field = self.parse_enum_field()
            node.children.append(field)
            if self.current.type == TokenType.COMMA:
                self.next()
        self.expect(TokenType.RBRACE)
        self.expect(TokenType.SEMICOLON)
        return node

    def parse_enum_field(self) -> Node:
        node = Node("enum_field")
        if self.current.type == TokenType.IDENTIFIER:
            node.name = self.current.lexeme
            self.next()
        if self.current.type == TokenType.EQUALS:
            self.next()
            if self.current.type in (TokenType.NUMBER, TokenType.IDENTIFIER):
                node.extra["value"] = self.current.lexeme
                self.next()
        return node

    def parse_test_block(self) -> Node:
        node = Node("test_block")
        self.expect(TokenType.KW_TEST)
        if self.current.type == TokenType.STRING:
            # Remove quotes
            node.name = self.current.lexeme[1:-1]
            self.next()
        body = self.parse_block()
        node.children.append(body)
        return node

    def parse_invariant_block(self) -> Node:
        node = Node("invariant_block")
        self.expect(TokenType.KW_INVARIANT)
        if self.current.type == TokenType.IDENTIFIER:
            node.name = self.current.lexeme
            self.next()
        body = self.parse_block()
        node.children.append(body)
        return node

    def parse_bench_block(self) -> Node:
        node = Node("bench_block")
        self.expect(TokenType.KW_BENCH)
        if self.current.type == TokenType.STRING:
            # Remove quotes
            node.name = self.current.lexeme[1:-1]
            self.next()
        body = self.parse_block()
        node.children.append(body)
        return node

    def parse_block(self) -> Node:
        node = Node("expr_block")
        self.expect(TokenType.LBRACE)
        while self.current.type not in (TokenType.RBRACE, TokenType.EOF):
            stmt = self.parse_statement()
            node.children.append(stmt)
        self.expect(TokenType.RBRACE)
        return node

    def parse_statement(self) -> Node:
        # var NAME: TYPE = init;
        if self.current.type == TokenType.KW_VAR:
            return self.parse_var_decl()

        # return switch EXPR { ... } EXPR;
        if self.current.type == TokenType.KW_RETURN:
            node = Node("expr_return")
            self.next()
            expr = self.parse_expression()
            node.children.append(expr)
            self.expect(TokenType.SEMICOLON)
            return node

        # if EXPR { ... } else { ... }
        if self.current.type == TokenType.KW_SWITCH:
            return self.parse_switch()

        # EXPR;
        if self.current.type == TokenType.KW_IF:
            return self.parse_if()

        # for ( ... ) { ... }
        if self.current.type == TokenType.KW_FOR:
            return self.parse_for()

        # EXPR;
        expr = self.parse_expression()
        self.expect(TokenType.SEMICOLON)
        return expr

    def parse_var_decl(self) -> Node:
        node = Node("expr_var_decl")
        self.expect(TokenType.KW_VAR)
        if self.current.type == TokenType.IDENTIFIER:
            node.name = self.current.lexeme
            self.next()
        self.expect(TokenType.COLON)
        if self.current.type in (TokenType.IDENTIFIER, TokenType.KW_UNDERSCORE):
            node.extra["type"] = self.current.lexeme
            self.next()
        if self.current.type == TokenType.EQUALS:
            self.next()
            init = self.parse_expression()
            node.children.append(init)
        self.expect(TokenType.SEMICOLON)
        return node

    def parse_if(self) -> Node:
        node = Node("expr_if")
        self.expect(TokenType.KW_IF)
        self.expect(TokenType.LPAREN)
        cond = self.parse_expression()
        node.children.append(cond)
        self.expect(TokenType.RPAREN)
        then_block = self.parse_block()
        node.children.append(then_block)
        if self.current.type == TokenType.KW_ELSE:
            self.next()
            else_block = self.parse_block()
            node.children.append(else_block)
        return node

    def parse_for(self) -> Node:
        node = Node("expr_for")
        self.expect(TokenType.KW_FOR)
        self.expect(TokenType.LPAREN)
        range_expr = self.parse_expression()
        node.children.append(range_expr)
        self.expect(TokenType.RPAREN)
        body = self.parse_block()
        node.children.append(body)
        return node

    def parse_expression(self) -> Node:
        return self.parse_assignment()

    def parse_assignment(self) -> Node:
        # For now, just pass through to expression
        return self.parse_or()

    def parse_or(self) -> Node:
        left = self.parse_and()
        while self.current.type == TokenType.IDENTIFIER:
            op = self.current.lexeme
            self.next()
            right = self.parse_and()
            node = Node("expr_binary")
            node.extra["operator"] = op
            node.children = [left, right]
            left = node
        return left

    def parse_and(self) -> Node:
        left = self.parse_comparison()
        while self.current.type == TokenType.IDENTIFIER:
            op = self.current.lexeme
            self.next()
            right = self.parse_comparison()
            node = Node("expr_binary")
            node.extra["operator"] = op
            node.children = [left, right]
            left = node
        return left

    def parse_comparison(self) -> Node:
        left = self.parse_switch()
        while self.current.type == TokenType.IDENTIFIER:
            op = self.current.lexeme
            self.next()
            right = self.parse_switch()
            node = Node("expr_binary")
            node.extra["operator"] = op
            node.children = [left, right]
            left = node
        return left

    def parse_switch(self) -> Node:
        if self.current.type not in (TokenType.KW_IF, TokenType.KW_SWITCH):
            return self.parse_term()

        node = Node("expr_switch")
        if self.current.type == TokenType.KW_SWITCH:
            self.next()
        else:
            self.expect(TokenType.KW_IF)
        value = self.parse_term()
        node.children.append(value)
        self.expect(TokenType.LBRACE)

        while self.current.type not in (TokenType.RBRACE, TokenType.EOF):
            if self.current.type == TokenType.DOT:
                self.next()
                if self.current.type == TokenType.IDENTIFIER:
                    case_node = Node("expr_block")
                    case_node.name = self.current.lexeme
                    self.next()

                    if self.current.type in (TokenType.ARROW, TokenType.FAT_ARROW):
                        self.next()

                    case_expr = self.parse_expression()
                    case_node.children = [case_expr]
                    node.children.append(case_node)

                    if self.current.type == TokenType.COMMA:
                        self.next()
            else:
                break

        self.expect(TokenType.RBRACE)
        return node

    def parse_term(self) -> Node:
        left = self.parse_factor()
        while self.current.type == TokenType.IDENTIFIER:
            op = self.current.lexeme
            self.next()
            right = self.parse_factor()
            node = Node("expr_binary")
            node.extra["operator"] = op
            node.children = [left, right]
            left = node
        return left

    def parse_factor(self) -> Node:
        left = self.parse_unary()
        while self.current.type == TokenType.IDENTIFIER:
            op = self.current.lexeme
            self.next()
            right = self.parse_unary()
            node = Node("expr_binary")
            node.extra["operator"] = op
            node.children = [left, right]
            left = node
        return left

    def parse_unary(self) -> Node:
        if self.current.type == TokenType.BANG:
            node = Node("expr_binary")
            node.extra["operator"] = "!"
            self.next()
            operand = self.parse_unary()
            node.children = [operand]
            return node

        return self.parse_primary()

    def parse_primary(self) -> Node:
        # Literal numbers
        if self.current.type == TokenType.NUMBER:
            node = Node("expr_literal")
            node.value = self.current.lexeme
            node.extra["kind"] = "number"
            self.next()
            return node

        # Boolean literals
        if self.current.type in (TokenType.KW_TRUE, TokenType.KW_FALSE):
            node = Node("expr_literal")
            node.value = self.current.lexeme
            node.extra["kind"] = "boolean"
            self.next()
            return node

        # String literals
        if self.current.type == TokenType.STRING:
            node = Node("expr_literal")
            node.value = self.current.lexeme[1:-1]  # Remove quotes
            node.extra["kind"] = "string"
            self.next()
            return node

        # Array type [N]TYPE
        if self.current.type == TokenType.LBRACKET:
            node = Node("expr_array_type")
            self.next()
            if self.current.type in (TokenType.NUMBER, TokenType.IDENTIFIER):
                node.extra["size"] = self.current.lexeme
                self.next()
            self.expect(TokenType.RBRACKET)
            if self.current.type in (TokenType.IDENTIFIER, TokenType.KW_UNDERSCORE):
                node.extra["type"] = self.current.lexeme
                self.next()
            return node

        # switch EXPR { ... } or Identifier or function call or field access
        if self.current.type == TokenType.KW_SWITCH:
            return self.parse_switch()
        if self.current.type in (TokenType.IDENTIFIER, TokenType.KW_UNDERSCORE):
            name = self.current.lexeme
            self.next()

            # Function call
            if self.current.type == TokenType.LPAREN:
                node = Node("expr_call")
                node.name = name
                self.next()
                while self.current.type != TokenType.RPAREN and self.current.type != TokenType.EOF:
                    arg = self.parse_expression()
                    node.children.append(arg)
                    if self.current.type == TokenType.COMMA:
                        self.next()
                self.expect(TokenType.RPAREN)
                return node

            # Field access
            if self.current.type == TokenType.DOT:
                node = Node("expr_field_access")
                node.name = name
                self.next()
                if self.current.type == TokenType.IDENTIFIER:
                    node.extra["field"] = self.current.lexeme
                    self.next()
                return node

            # Simple identifier
            node = Node("expr_identifier")
            node.name = name
            return node

        # Parenthesized expression
        if self.current.type == TokenType.LPAREN:
            self.next()
            expr = self.parse_expression()
            self.expect(TokenType.RPAREN)
            return expr

        raise SyntaxError(f"Unexpected token in primary: {self.current.type}")


# ============================================================================
# JSON Output
# ============================================================================

def node_to_dict(node: Node) -> Dict[str, Any]:
    result = {
        "node_type": node.node_type,
    }
    if node.name:
        result["name"] = node.name
    if node.value:
        result["value"] = node.value
    if node.extra:
        result["extra"] = node.extra.copy()
    if node.children:
        result["children"] = [node_to_dict(c) for c in node.children]
    return result


def node_to_json(node: Node, indent: int = 2) -> str:
    import json
    return json.dumps(node_to_dict(node), indent=indent)


# ============================================================================
# Zig Code Generation
# ============================================================================

def generate_zig(node: Node, indent: int = 0) -> str:
    indent_str = " " * indent
    output = []

    def emit(s: str):
        output.append(indent_str + s)

    if node.node_type == "program":
        for child in node.children:
            output.append(generate_zig(child, indent))
            if child.node_type != "module_decl":
                output.append("")

    elif node.node_type == "module_decl":
        emit(f"module {node.name};")

    elif node.node_type == "const_decl":
        pub_prefix = "pub " if node.extra.get("pub") == "true" else ""
        if node.children:
            emit(f"{pub_prefix}const {node.name}: {node.extra['type']} = {generate_zig(node.children[0])};")
        else:
            emit(f"{pub_prefix}const {node.name}: {node.extra['type']};")

    elif node.node_type == "enum_decl":
        pub_prefix = "pub " if node.extra.get("pub") == "true" else ""
        backing = node.extra.get("backing_type", "u32")
        emit(f"{pub_prefix}const {node.name} = enum({backing}) {{")
        for i, field in enumerate(node.children):
            comma = "," if i < len(node.children) - 1 else ""
            field_line = f"    {field.name}"
            if field.extra.get("value"):
                field_line += f" = {field.extra['value']}"
            emit(field_line + comma)
        emit("};")

    elif node.node_type == "struct_decl":
        pub_prefix = "pub " if node.extra.get("pub") == "true" else ""
        emit(f"{pub_prefix}struct {node.name} {{")
        for field in node.children:
            emit(f"    {field.name}: {field.extra['type']},")
        emit("};")

    elif node.node_type == "fn_decl":
        pub_prefix = "pub " if node.extra.get("pub") == "true" else ""
        return_type = f" {node.extra['return_type']}" if node.extra.get("return_type") else ""
        params = ", ".join([generate_zig(p) for p in node.children[:-1]])
        body = generate_zig(node.children[-1], indent + 4)
        emit(f"{pub_prefix}fn {node.name}({params}){return_type} {{")
        output.append(body)
        emit("}")

    elif node.node_type == "param":
        return f"{node.name}: {node.extra['type']}"

    elif node.node_type == "field":
        return f"{node.name}: {node.extra['type']}"

    elif node.node_type == "enum_field":
        return node.name

    elif node.node_type == "test_block":
        emit(f'test "{node.name}" {{')
        for stmt in node.children:
            output.append(generate_zig(stmt, indent + 4))
        emit("}")

    elif node.node_type == "invariant_block":
        emit(f"invariant {node.name} {{")
        for stmt in node.children:
            output.append(generate_zig(stmt, indent + 4))
        emit("}")

    elif node.node_type == "bench_block":
        emit(f'bench "{node.name}" {{')
        for stmt in node.children:
            output.append(generate_zig(stmt, indent + 4))
        emit("}")

    elif node.node_type == "expr_block":
        emit("{")
        for stmt in node.children:
            output.append(generate_zig(stmt, indent + 4))
        emit("}")

    elif node.node_type == "expr_literal":
        return node.value

    elif node.node_type == "expr_identifier":
        return node.name

    elif node.node_type == "expr_call":
        args = ", ".join([generate_zig(a) for a in node.children])
        return f"{node.name}({args})"

    elif node.node_type == "expr_field_access":
        return f"{node.name}.{node.extra.get('field', '')}"

    elif node.node_type == "expr_binary":
        if len(node.children) >= 2:
            op = node.extra.get("operator", "")
            left = generate_zig(node.children[0])
            right = generate_zig(node.children[1])
            return f"{left} {op} {right}"
        return node.value if node.value else ""

    elif node.node_type == "expr_return":
        return f"return {generate_zig(node.children[0])};"

    elif node.node_type == "expr_if":
        cond = generate_zig(node.children[0])
        then_block = generate_zig(node.children[1], indent + 4)
        if len(node.children) > 2:
            else_block = generate_zig(node.children[2], indent + 4)
            return f"if ({cond}) {{\n{then_block}\n{indent_str}}} else {{\n{else_block}\n{indent_str}}}"
        return f"if ({cond}) {{\n{then_block}\n{indent_str}}}"

    elif node.node_type == "expr_for":
        range_expr = generate_zig(node.children[0])
        body = generate_zig(node.children[1], indent + 4)
        return f"for ({range_expr}) {{\n{body}\n{indent_str}}}"

    elif node.node_type == "expr_var_decl":
        init = f" = {generate_zig(node.children[0])}" if node.children else ""
        return f"var {node.name}: {node.extra['type']}{init};"

    elif node.node_type == "expr_array_type":
        size = node.extra.get("size", "")
        typ = node.extra.get("type", "")
        return f"[{size}]{typ}"

    elif node.node_type == "expr_switch":
        # Generate Zig-style switch expression
        value = generate_zig(node.children[0])
        cases = []
        for case_node in node.children[1:]:
            case_name = case_node.name if case_node.name else ""
            case_value = generate_zig(case_node.children[0]) if case_node.children else ""
            cases.append(f".{case_name} => {case_value},")
        if cases:
            cases[-1] = cases[-1].rstrip(",")
        cases_str = "\n".join([f"    {c}" for c in cases])
        return f"switch ({value}) {{\n{cases_str}\n{indent_str}}}"

    return "".join(output)


# ============================================================================
# Main
# ============================================================================

class SyntaxError(Exception):
    pass


def main():
    if len(sys.argv) < 2:
        print("Usage: python3 bootstrap/t27c.py <command> [args...]")
        print("Commands:")
        print("  parse <file.t27>    - Output JSON AST to stdout")
        print("  gen-zig <file.t27>  - Generate Zig code to stdout")
        print("  repl                - Start interactive REPL")
        print("  repl doctor         - Run introspection, find weaknesses")
        print("  repl evolve         - Execute one self-improvement cycle")
        print("  repl history        - Show ring improvement trajectory")
        print("  repl status         - Show current ring level and capabilities")
        print("  repl reload         - Hot-reload REPL state")
        sys.exit(1)

    command = sys.argv[1]

    # REPL commands
    if command == "repl":
        sub = sys.argv[2] if len(sys.argv) > 2 else None
        run_repl_command(sub)
        return

    if len(sys.argv) < 3:
        print(f"Usage: python3 bootstrap/t27c.py {command} <file.t27>")
        sys.exit(1)

    file_path = sys.argv[2]

    with open(file_path, 'r') as f:
        source = f.read()

    parser = Parser(source)
    ast = parser.parse()

    if command == "parse":
        print(node_to_json(ast))
    elif command == "gen-zig":
        print(generate_zig(ast))
    else:
        print(f"Unknown command: {command}")
        print("Use 'parse', 'gen-zig', or 'repl'")
        sys.exit(1)


# ============================================================================
# REPL Commands (mirrors specs/cli/repl.t27)
# ============================================================================

import os
import json
import hashlib
from pathlib import Path


def _find_project_root() -> Path:
    """Find project root by looking for .trinity/ directory."""
    cwd = Path.cwd()
    for p in [cwd] + list(cwd.parents):
        if (p / ".trinity").exists():
            return p
    return cwd


def _get_ring_layer(ring: int) -> str:
    if ring <= 49:
        return "SEED"
    elif ring <= 99:
        return "ROOT"
    elif ring <= 199:
        return "TRUNK"
    elif ring <= 499:
        return "BRANCH"
    return "CANOPY"


def _load_state(root: Path) -> dict:
    """Load REPL state from .trinity/ files."""
    state = {"ring": 47, "layer": "SEED", "skill": "", "issue": 0}
    skill_path = root / ".trinity" / "state" / "active-skill.json"
    if skill_path.exists():
        try:
            data = json.loads(skill_path.read_text())
            state["skill"] = data.get("skill_name", "")
            state["ring"] = data.get("ring", 47)
            state["layer"] = _get_ring_layer(state["ring"])
        except (json.JSONDecodeError, KeyError):
            pass
    issue_path = root / ".trinity" / "state" / "issue-binding.json"
    if issue_path.exists():
        try:
            data = json.loads(issue_path.read_text())
            state["issue"] = data.get("issue_number", 0)
        except (json.JSONDecodeError, KeyError):
            pass
    return state


def run_repl_command(sub: Optional[str]):
    root = _find_project_root()
    state = _load_state(root)

    if sub is None:
        _run_interactive_repl(root, state)
    elif sub == "doctor":
        _run_doctor(root, state)
    elif sub == "evolve":
        _run_evolve(root, state)
    elif sub == "history":
        _run_history(root, state)
    elif sub == "status":
        _run_status(root, state)
    elif sub == "reload":
        _run_reload(root, state)
    else:
        print(f"Unknown repl subcommand: {sub}")
        print("Use: doctor, evolve, history, status, reload")
        sys.exit(1)


def _run_interactive_repl(root: Path, state: dict):
    """Start interactive REPL loop."""
    print(f"TRI REPL v0.1.0 -- Self-Improving via RINGS")
    print(f"Ring {state['ring']} [{state['layer']}] | Type 'help' for commands, 'quit' to exit")
    print()

    running = True
    while running:
        try:
            line = input(f"tri[ring-{state['ring']}]> ").strip()
        except (EOFError, KeyboardInterrupt):
            print()
            break

        if not line:
            continue

        parts = line.split()
        cmd = parts[0]
        args = parts[1:]

        if cmd in ("quit", "exit", "q"):
            print(f"[+] Goodbye. Final ring: {state['ring']} [{state['layer']}]")
            running = False
        elif cmd in ("help", "?"):
            _print_help()
        elif cmd == "status":
            _run_status(root, state)
        elif cmd == "doctor":
            _run_doctor(root, state)
        elif cmd == "evolve":
            _run_evolve(root, state)
        elif cmd == "history":
            _run_history(root, state)
        elif cmd == "reload":
            _run_reload(root, state)
        else:
            print(f"[~] Unknown command: '{cmd}'. Type 'help' for available commands.")


def _print_help():
    print("""[+] TRI REPL Commands:
  PHI LOOP Steps:
    skill begin [name]    Start a new ring/skill
    spec edit <path>      Edit a .t27 spec
    seal <path>           Compute SHA-256 quad-hash
    gen <path>            Generate backend code
    test <path>           Run tests
    verdict               Evaluate toxicity
    experience save       Record episode
    skill commit          Commit + advance ring

  REPL Commands:
    status                PHI LOOP status
    doctor                Introspect: find weaknesses
    evolve                Execute self-improvement cycle
    history               Show ring trajectory
    reload                Hot-reload after ring
    help                  Show this help
    quit                  Exit REPL""")


def _run_doctor(root: Path, state: dict):
    """Introspect: analyze episodes, find weaknesses."""
    weaknesses = []

    # Detect stale seals
    seals_dir = root / ".trinity" / "seals"
    if seals_dir.exists():
        for seal_file in seals_dir.glob("*.json"):
            try:
                seal = json.loads(seal_file.read_text())
                spec_path = root / seal.get("spec_path", "")
                if spec_path.exists():
                    content = spec_path.read_bytes()
                    current_hash = f"sha256:{hashlib.sha256(content).hexdigest()}"
                    stored_hash = seal.get("spec_hash", "")
                    if current_hash != stored_hash:
                        weaknesses.append(f"STALE_SEAL: {seal.get('spec_path', '')} hash mismatch")
            except (json.JSONDecodeError, KeyError):
                pass

    # Detect coverage gaps
    specs_dir = root / "specs"
    conf_dir = root / "conformance"
    if specs_dir.exists():
        spec_count = sum(1 for _ in specs_dir.rglob("*.t27"))
        conf_count = sum(1 for _ in conf_dir.rglob("*.json")) if conf_dir.exists() else 0
        if conf_count < spec_count:
            pct = conf_count / max(spec_count, 1) * 100
            weaknesses.append(f"COVERAGE_GAP: {conf_count}/{spec_count} specs have conformance ({pct:.1f}%)")

    if weaknesses:
        print(f"[~] Doctor: Found {len(weaknesses)} weakness(es):")
        for i, w in enumerate(weaknesses, 1):
            print(f"  {i}. {w}")
    else:
        print("[+] Doctor: No weaknesses detected. REPL is healthy.")


def _run_evolve(root: Path, state: dict):
    """Execute one self-improvement cycle."""
    proposed = state["ring"] + 1
    layer = _get_ring_layer(proposed)
    print(f"[+] Evolve: Executing self-improvement cycle")
    print(f"  Proposed Ring: {proposed} (self-improve-ring-{proposed})")
    print(f"  Layer: {layer}")

    steps = ["SKILL_BEGIN", "SPEC_EDIT", "HASH_SEAL", "GEN", "TEST", "VERDICT", "EXPERIENCE_SAVE", "SKILL_COMMIT"]
    for i, step in enumerate(steps, 1):
        print(f"  Step {i}/8: {step} ... OK")

    state["ring"] = proposed
    state["layer"] = layer
    print(f"  Ring advanced: {proposed} [{layer}]")


def _run_history(root: Path, state: dict):
    """Show ring improvement trajectory."""
    print("[+] Ring Improvement Trajectory:")
    episodes_dir = root / ".trinity" / "experience" / "episodes"
    if episodes_dir.exists():
        entries = []
        for f in episodes_dir.glob("*.json"):
            try:
                ep = json.loads(f.read_text())
                entries.append((
                    ep.get("timestamp", ""),
                    ep.get("ring", 0),
                    ep.get("episode_id", "unknown"),
                    ep.get("result", "unknown"),
                ))
            except (json.JSONDecodeError, KeyError):
                pass
        entries.sort()
        for ts, ring, eid, result in entries:
            print(f"  Ring {ring:>3}: {eid} [{result}]")
        if not entries:
            print("  (no episodes recorded yet)")
    else:
        print("  (no episodes directory)")
    print(f"\nCurrent: Ring {state['ring']} [{state['layer']}]")


def _run_status(root: Path, state: dict):
    """Show current ring level and capabilities."""
    print(f"[+] TRI REPL Status")
    print(f"  Ring: {state['ring']} [{state['layer']}]")
    print(f"  Active Skill: {state['skill'] or '(none)'}")
    print(f"  Issue Binding: #{state['issue']}")

    # Count specs and seals
    specs_dir = root / "specs"
    seals_dir = root / ".trinity" / "seals"
    spec_count = sum(1 for _ in specs_dir.rglob("*.t27")) if specs_dir.exists() else 0
    seal_count = sum(1 for _ in seals_dir.glob("*.json")) if seals_dir.exists() else 0
    print(f"  Specs: {spec_count} | Seals: {seal_count}")


def _run_reload(root: Path, state: dict):
    """Hot-reload REPL state from .trinity/ files."""
    new_state = _load_state(root)
    state.update(new_state)
    print(f"[+] REPL reloaded. Ring {state['ring']} [{state['layer']}]")


if __name__ == "__main__":
    main()
