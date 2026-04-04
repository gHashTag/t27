#!/usr/bin/env python3
"""
Bootstrap T27 Parser
Minimal implementation to read .t27 files and output JSON AST.
This is a temporary bootstrap layer - the canonical spec is in specs/compiler/parser.t27
"""

import sys
import re
import json
from enum import Enum, auto
from dataclasses import dataclass, field
from typing import List, Optional, Union


# ============================================================================
# Token Types (matches specs/compiler/parser.t27 TokenKind)
# ============================================================================

class TokenKind(Enum):
    # Special
    EOF = auto()
    IDENT = auto()
    LITERAL = auto()

    # Keywords
    PUB = auto()
    CONST = auto()
    EXTERN = auto()
    STRUCT = auto()
    ENUM = auto()
    FN = auto()
    RETURN = auto()
    IF = auto()
    ELSE = auto()
    USING = auto()
    TEST = auto()
    BENCH = auto()
    INVARIANT = auto()
    MODULE = auto()
    TYPE = auto()

    # Operators
    PLUS = auto()
    MINUS = auto()
    STAR = auto()
    SLASH = auto()
    PERCENT = auto()
    EQ = auto()
    NE = auto()
    LT = auto()
    LE = auto()
    GT = auto()
    GE = auto()
    AND = auto()
    OR = auto()
    XOR = auto()
    NOT = auto()
    ASSIGN = auto()
    ARROW = auto()

    # Delimiters
    LPAREN = auto()
    RPAREN = auto()
    LBRACE = auto()
    RBRACE = auto()
    LBRACKET = auto()
    RBRACKET = auto()
    COMMA = auto()
    COLON = auto()
    DOT = auto()
    SEMICOLON = auto()
    QUESTION = auto()


@dataclass
class Token:
    kind: TokenKind
    text: str
    line: int
    col: int


# ============================================================================
# AST Node Types (matches specs/compiler/parser.t27)
# ============================================================================

@dataclass
class ModuleNode:
    name: str
    decls: List['AstNode']


@dataclass
class ConstDeclNode:
    pub_: bool
    name: str
    type_ref: str
    value: str


@dataclass
class EnumValueNode:
    name: str
    value: str


@dataclass
class EnumDeclNode:
    name: str
    backing: str
    values: List[EnumValueNode]


@dataclass
class FieldNode:
    name: str
    type_ref: str


@dataclass
class StructDeclNode:
    name: str
    fields: List[FieldNode]


@dataclass
class ParamNode:
    name: str
    type_ref: str


@dataclass
class FnDeclNode:
    name: str
    params: List[ParamNode]
    return_type: str


@dataclass
class InvariantDeclNode:
    name: str


@dataclass
class TestDeclNode:
    name: str


@dataclass
class BenchDeclNode:
    name: str


# Union type for AST nodes
AstNode = Union[
    ModuleNode,
    ConstDeclNode,
    EnumDeclNode,
    StructDeclNode,
    FnDeclNode,
    InvariantDeclNode,
    TestDeclNode,
    BenchDeclNode,
]


# ============================================================================
# Tokenizer
# ============================================================================

class Tokenizer:
    def __init__(self, source: str):
        self.source = source
        self.pos = 0
        self.line = 1
        self.col = 1
        self.tokens: List[Token] = []

    def _peek(self, n: int = 0) -> Optional[str]:
        if self.pos + n < len(self.source):
            return self.source[self.pos + n]
        return None

    def _advance(self, count: int = 1) -> str:
        c = self._peek(0)
        for _ in range(count):
            if self.pos < len(self.source):
                if self.source[self.pos] == '\n':
                    self.line += 1
                    self.col = 1
                else:
                    self.col += 1
                self.pos += 1
        return c if c else ''

    def _skip_whitespace(self):
        while True:
            c = self._peek()
            if c in ' \t\r':
                self._advance()
            elif c == '\n':
                self._advance()
            else:
                break

    def _skip_line_comment(self):
        # Skip to end of line (; or // comments)
        while self._peek() and self._peek() != '\n':
            self._advance()
        if self._peek() == '\n':
            self._advance()

    def _read_identifier(self) -> str:
        start = self.pos
        while self._peek() and (self._peek().isalnum() or self._peek() == '_'):
            self._advance()
        return self.source[start:self.pos]

    def _read_number(self) -> str:
        start = self.pos
        # Handle negative numbers
        if self._peek() == '-':
            self._advance()
        while self._peek() and self._peek().isdigit():
            self._advance()
        return self.source[start:self.pos]

    def _read_string(self) -> str:
        self._advance()  # skip opening quote
        start = self.pos
        while self._peek() and self._peek() != '"':
            if self._peek() == '\\':
                self._advance()  # skip escape char
            self._advance()
        text = self.source[start:self.pos]
        self._advance()  # skip closing quote
        return text

    def tokenize(self) -> List[Token]:
        while self.pos < len(self.source):
            self._skip_whitespace()

            c = self._peek()
            if c is None:
                break

            # Comments
            if (c == '/' and self._peek(1) == '/') or c == ';':
                self._skip_line_comment()
                continue

            # Identifiers and keywords
            if c.isalpha() or c == '_':
                text = self._read_identifier()
                kind = self._keyword_kind(text)
                self.tokens.append(Token(kind, text, self.line, self.col - len(text)))
                continue

            # Numbers
            if c.isdigit() or (c == '-' and self._peek(1) and self._peek(1).isdigit()):
                text = self._read_number()
                self.tokens.append(Token(TokenKind.LITERAL, text, self.line, self.col - len(text)))
                continue

            # String literals
            if c == '"':
                text = self._read_string()
                self.tokens.append(Token(TokenKind.LITERAL, text, self.line, self.col - len(text) - 2))
                continue

            # Multi-char operators
            if c == '-' and self._peek(1) == '>':
                self._advance(2)
                self.tokens.append(Token(TokenKind.ARROW, '->', self.line, self.col - 2))
                continue

            if c in '=!<' and self._peek(1) == '=':
                op = c + '='
                kind_map = {'=': TokenKind.EQ, '!': TokenKind.NE, '<': TokenKind.LE, '>': TokenKind.GE}
                self._advance(2)
                self.tokens.append(Token(kind_map[c], op, self.line, self.col - 2))
                continue

            # Single-char tokens
            kind = self._single_char_kind(c)
            if kind != TokenKind.EOF:
                self._advance()
                self.tokens.append(Token(kind, c, self.line, self.col - 1))
                continue

            # Unknown - skip
            self._advance()

        # EOF token
        self.tokens.append(Token(TokenKind.EOF, '', self.line, self.col))
        return self.tokens

    def _keyword_kind(self, text: str) -> TokenKind:
        keywords = {
            'pub': TokenKind.PUB,
            'const': TokenKind.CONST,
            'extern': TokenKind.EXTERN,
            'struct': TokenKind.STRUCT,
            'enum': TokenKind.ENUM,
            'fn': TokenKind.FN,
            'return': TokenKind.RETURN,
            'if': TokenKind.IF,
            'else': TokenKind.ELSE,
            'using': TokenKind.USING,
            'test': TokenKind.TEST,
            'bench': TokenKind.BENCH,
            'invariant': TokenKind.INVARIANT,
            'module': TokenKind.MODULE,
            'type': TokenKind.TYPE,
        }
        return keywords.get(text, TokenKind.IDENT)

    def _single_char_kind(self, c: str) -> TokenKind:
        return {
            '+': TokenKind.PLUS,
            '-': TokenKind.MINUS,
            '*': TokenKind.STAR,
            '/': TokenKind.SLASH,
            '%': TokenKind.PERCENT,
            '=': TokenKind.ASSIGN,
            '<': TokenKind.LT,
            '>': TokenKind.GT,
            '!': TokenKind.NOT,
            '&': TokenKind.AND,
            '|': TokenKind.OR,
            '^': TokenKind.XOR,
            '(': TokenKind.LPAREN,
            ')': TokenKind.RPAREN,
            '{': TokenKind.LBRACE,
            '}': TokenKind.RBRACE,
            '[': TokenKind.LBRACKET,
            ']': TokenKind.RBRACKET,
            ',': TokenKind.COMMA,
            ':': TokenKind.COLON,
            '.': TokenKind.DOT,
            ';': TokenKind.SEMICOLON,
            '?': TokenKind.QUESTION,
        }.get(c, TokenKind.EOF)


# ============================================================================
# Parser
# ============================================================================

class Parser:
    def __init__(self, tokens: List[Token]):
        self.tokens = tokens
        self.pos = 0

    def _peek(self) -> Token:
        if self.pos < len(self.tokens):
            return self.tokens[self.pos]
        return Token(TokenKind.EOF, '', 0, 0)

    def _next(self) -> Token:
        tok = self._peek()
        if self.pos < len(self.tokens):
            self.pos += 1
        return tok

    def _eat(self, kind: TokenKind) -> bool:
        if self._peek().kind == kind:
            self._next()
            return True
        return False

    def _expect(self, kind: TokenKind) -> Optional[Token]:
        if self._eat(kind):
            return self.tokens[self.pos - 1] if self.pos > 0 else self.tokens[0]
        return None

    def parse_module(self) -> ModuleNode:
        name = ""

        # Optional: module NAME;
        if self._eat(TokenKind.MODULE):
            name_tok = self._expect(TokenKind.IDENT)
            if name_tok:
                name = name_tok.text
            self._eat(TokenKind.SEMICOLON)

        decls = []
        while self._peek().kind != TokenKind.EOF:
            decl = self._parse_decl()
            if decl:
                decls.append(decl)

        return ModuleNode(name=name, decls=decls)

    def _parse_decl(self) -> Optional[AstNode]:
        is_pub = self._eat(TokenKind.PUB)

        if self._eat(TokenKind.CONST):
            return self._parse_const_decl(is_pub)
        if self._eat(TokenKind.ENUM):
            return self._parse_enum_decl(is_pub)
        if self._eat(TokenKind.STRUCT):
            return self._parse_struct_decl(is_pub)
        if self._eat(TokenKind.FN):
            return self._parse_fn_decl(is_pub)
        if self._eat(TokenKind.INVARIANT):
            return self._parse_invariant_decl()
        if self._eat(TokenKind.TEST):
            return self._parse_test_decl()
        if self._eat(TokenKind.BENCH):
            return self._parse_bench_decl()

        # Skip unknown
        self._next()
        return None

    def _parse_const_decl(self, is_pub: bool) -> ConstDeclNode:
        name_tok = self._expect(TokenKind.IDENT)
        self._expect(TokenKind.COLON)

        # Parse type reference
        type_ref = self._parse_type_ref()

        self._expect(TokenKind.ASSIGN)

        # Parse value (simplified: capture until semicolon)
        value_parts = []
        while self._peek().kind != TokenKind.SEMICOLON and self._peek().kind != TokenKind.EOF:
            tok = self._next()
            value_parts.append(tok.text)

        self._expect(TokenKind.SEMICOLON)

        return ConstDeclNode(
            pub_=is_pub,
            name=name_tok.text if name_tok else "",
            type_ref=type_ref,
            value=' '.join(value_parts)
        )

    def _parse_type_ref(self) -> str:
        parts = []
        while self._peek().kind in [TokenKind.IDENT, TokenKind.LBRACKET, TokenKind.RBRACKET,
                                       TokenKind.STAR, TokenKind.QUESTION]:
            parts.append(self._next().text)
        return ''.join(parts)

    def _parse_enum_decl(self, is_pub: bool) -> EnumDeclNode:
        name_tok = self._expect(TokenKind.IDENT)
        self._expect(TokenKind.LPAREN)
        backing_tok = self._expect(TokenKind.IDENT)
        self._expect(TokenKind.RPAREN)
        self._expect(TokenKind.LBRACE)

        values = []
        while self._peek().kind != TokenKind.RBRACE:
            value_name_tok = self._expect(TokenKind.IDENT)
            self._expect(TokenKind.EQ)

            # Parse value
            value_parts = []
            while self._peek().kind != TokenKind.COMMA and self._peek().kind != TokenKind.RBRACE:
                tok = self._next()
                value_parts.append(tok.text)

            values.append(EnumValueNode(
                name=value_name_tok.text if value_name_tok else "",
                value=' '.join(value_parts)
            ))
            self._expect(TokenKind.COMMA)

        self._expect(TokenKind.RBRACE)

        return EnumDeclNode(
            name=name_tok.text if name_tok else "",
            backing=backing_tok.text if backing_tok else "",
            values=values
        )

    def _parse_struct_decl(self, is_pub: bool) -> StructDeclNode:
        name_tok = self._expect(TokenKind.IDENT)
        self._expect(TokenKind.LBRACE)

        fields = []
        while self._peek().kind != TokenKind.RBRACE:
            field_name_tok = self._expect(TokenKind.IDENT)
            self._expect(TokenKind.COLON)
            type_ref = self._parse_type_ref()
            self._expect(TokenKind.COMMA)

            fields.append(FieldNode(
                name=field_name_tok.text if field_name_tok else "",
                type_ref=type_ref
            ))

        self._expect(TokenKind.RBRACE)

        return StructDeclNode(
            name=name_tok.text if name_tok else "",
            fields=fields
        )

    def _parse_fn_decl(self, is_pub: bool) -> FnDeclNode:
        name_tok = self._expect(TokenKind.IDENT)
        self._expect(TokenKind.LPAREN)

        params = []
        while self._peek().kind != TokenKind.RPAREN:
            param_name_tok = self._expect(TokenKind.IDENT)
            self._expect(TokenKind.COLON)
            type_ref = self._parse_type_ref()
            self._expect(TokenKind.COMMA)

            params.append(ParamNode(
                name=param_name_tok.text if param_name_tok else "",
                type_ref=type_ref
            ))

        self._expect(TokenKind.RPAREN)

        # Return type
        return_type = "void"
        if self._eat(TokenKind.ARROW):
            return_tok = self._expect(TokenKind.IDENT)
            if return_tok:
                return_type = return_tok.text

        # Skip body
        self._skip_block()

        return FnDeclNode(
            name=name_tok.text if name_tok else "",
            params=params,
            return_type=return_type
        )

    def _skip_block(self):
        brace_count = 0
        while self._peek().kind != TokenKind.EOF:
            tok = self._next()
            if tok.kind == TokenKind.LBRACE:
                brace_count += 1
            elif tok.kind == TokenKind.RBRACE:
                brace_count -= 1
                if brace_count == 0:
                    break

    def _parse_invariant_decl(self) -> InvariantDeclNode:
        name_tok = self._expect(TokenKind.IDENT)
        self._skip_block()
        return InvariantDeclNode(name=name_tok.text if name_tok else "")

    def _parse_test_decl(self) -> TestDeclNode:
        name_tok = self._expect(TokenKind.LITERAL)
        self._skip_block()
        return TestDeclNode(name=name_tok.text if name_tok else "")

    def _parse_bench_decl(self) -> BenchDeclNode:
        name_tok = self._expect(TokenKind.LITERAL)
        self._skip_block()
        return BenchDeclNode(name=name_tok.text if name_tok else "")


# ============================================================================
# JSON Output
# ============================================================================

def ast_to_json(node: AstNode) -> dict:
    if isinstance(node, ModuleNode):
        return {
            "module": node.name,
            "decls": [ast_to_json(d) for d in node.decls]
        }
    elif isinstance(node, ConstDeclNode):
        return {
            "kind": "const",
            "pub": node.pub_,
            "name": node.name,
            "type": node.type_ref,
            "value": node.value
        }
    elif isinstance(node, EnumDeclNode):
        return {
            "kind": "enum",
            "name": node.name,
            "backing": node.backing,
            "values": [
                {"name": v.name, "value": v.value}
                for v in node.values
            ]
        }
    elif isinstance(node, StructDeclNode):
        return {
            "kind": "struct",
            "name": node.name,
            "fields": [
                {"name": f.name, "type": f.type_ref}
                for f in node.fields
            ]
        }
    elif isinstance(node, FnDeclNode):
        return {
            "kind": "fn",
            "name": node.name,
            "params": [
                {"name": p.name, "type": p.type_ref}
                for p in node.params
            ],
            "return": node.return_type
        }
    elif isinstance(node, InvariantDeclNode):
        return {
            "kind": "invariant",
            "name": node.name
        }
    elif isinstance(node, TestDeclNode):
        return {
            "kind": "test",
            "name": node.name
        }
    elif isinstance(node, BenchDeclNode):
        return {
            "kind": "bench",
            "name": node.name
        }
    else:
        return {}


def parse_file(filepath: str) -> dict:
    """Parse a .t27 file and return JSON AST."""
    with open(filepath, 'r') as f:
        source = f.read()

    tokenizer = Tokenizer(source)
    tokens = tokenizer.tokenize()

    parser = Parser(tokens)
    ast = parser.parse_module()

    return ast_to_json(ast)


def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <file.t27>", file=sys.stderr)
        sys.exit(1)

    filepath = sys.argv[1]
    try:
        result = parse_file(filepath)
        print(json.dumps(result, indent=2))
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
