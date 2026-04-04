/* Auto-generated from compiler/parser/lexer.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/parser/lexer.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "lexer.h"
#include <string.h>

bool t27_lex_is_ident_start(char c) {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
}

bool t27_lex_is_ident_continue(char c) {
    return t27_lex_is_ident_start(c) || (c >= '0' && c <= '9') || c == '-';
}

bool t27_lex_is_digit(char c) {
    return c >= '0' && c <= '9';
}

static bool is_whitespace(char c) {
    return c == ' ' || c == '\t' || c == '\r';
}

T27Lexer t27_lexer_init(const char *source, size_t len) {
    T27Lexer lex;
    lex.source     = source;
    lex.source_len = len;
    lex.pos        = 0;
    lex.line       = 1;
    lex.column     = 1;
    return lex;
}

static void advance(T27Lexer *lex) {
    if (lex->pos < lex->source_len) {
        if (lex->source[lex->pos] == '\n') {
            lex->line++;
            lex->column = 1;
        } else {
            lex->column++;
        }
        lex->pos++;
    }
}

T27LexToken t27_lexer_next(T27Lexer *lex) {
    /* Skip whitespace */
    while (lex->pos < lex->source_len && is_whitespace(lex->source[lex->pos])) {
        advance(lex);
    }

    T27LexToken tok;
    tok.line   = lex->line;
    tok.column = lex->column;
    tok.start  = lex->pos;

    if (lex->pos >= lex->source_len) {
        tok.kind = LTOK_EOF;
        tok.end  = lex->pos;
        return tok;
    }

    char c = lex->source[lex->pos];

    /* Single char tokens */
    T27LexTokenType single = LTOK_EOF;
    switch (c) {
        case '\n': single = LTOK_NEWLINE; break;
        case '.':  single = LTOK_DOT;     break;
        case ':':  single = LTOK_COLON;   break;
        case ';':  single = LTOK_SEMICOLON; break;
        case ',':  single = LTOK_COMMA;   break;
        case '#':  single = LTOK_HASH;    break;
        case '(':  single = LTOK_LPAREN;  break;
        case ')':  single = LTOK_RPAREN;  break;
        case '[':  single = LTOK_LBRACKET; break;
        case ']':  single = LTOK_RBRACKET; break;
        case '+':  single = LTOK_PLUS;    break;
        case '-':  single = LTOK_MINUS;   break;
        case '*':  single = LTOK_STAR;    break;
        case '/':  single = LTOK_SLASH;   break;
        case '%':  single = LTOK_PERCENT; break;
        case '&':  single = LTOK_AND;     break;
        case '|':  single = LTOK_OR;      break;
        case '^':  single = LTOK_XOR;     break;
        case '~':  single = LTOK_TILDE;   break;
        case '<':  single = LTOK_LT;      break;
        case '>':  single = LTOK_GT;      break;
        case '=':  single = LTOK_EQ;      break;
        case '!':  single = LTOK_EXCL;    break;
        default:   single = LTOK_EOF;     break;
    }

    if (single != LTOK_EOF) {
        advance(lex);
        tok.kind = single;
        tok.end  = lex->pos;
        return tok;
    }

    /* Identifier */
    if (t27_lex_is_ident_start(c)) {
        while (lex->pos < lex->source_len && t27_lex_is_ident_continue(lex->source[lex->pos])) {
            advance(lex);
        }
        tok.kind = LTOK_IDENTIFIER;
        tok.end  = lex->pos;
        return tok;
    }

    /* Number */
    if (t27_lex_is_digit(c)) {
        while (lex->pos < lex->source_len && t27_lex_is_digit(lex->source[lex->pos])) {
            advance(lex);
        }
        tok.kind = LTOK_INTEGER;
        tok.end  = lex->pos;
        return tok;
    }

    /* String */
    if (c == '"') {
        advance(lex);
        while (lex->pos < lex->source_len && lex->source[lex->pos] != '"') {
            advance(lex);
        }
        if (lex->pos < lex->source_len) advance(lex);
        tok.kind = LTOK_STRING;
        tok.end  = lex->pos;
        return tok;
    }

    /* Unknown */
    advance(lex);
    tok.kind = LTOK_EOF;
    tok.end  = lex->pos;
    return tok;
}
