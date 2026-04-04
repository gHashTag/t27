/* Auto-generated from compiler/parser/lexer.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/parser/lexer.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef T27_LEXER_H
#define T27_LEXER_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define T27_MAX_IDENTIFIER_LEN 64
#define T27_MAX_NUMBER_LEN     20
#define T27_MAX_TOKENS         10000

typedef enum {
    LTOK_EOF = 0, LTOK_NEWLINE = 1, LTOK_DOT = 2, LTOK_COLON = 3,
    LTOK_SEMICOLON = 4, LTOK_COMMA = 5, LTOK_HASH = 6,
    LTOK_LPAREN = 7, LTOK_RPAREN = 8, LTOK_LBRACKET = 9,
    LTOK_RBRACKET = 10, LTOK_PLUS = 11, LTOK_MINUS = 12,
    LTOK_STAR = 13, LTOK_SLASH = 14, LTOK_PERCENT = 15,
    LTOK_AND = 16, LTOK_OR = 17, LTOK_XOR = 18, LTOK_TILDE = 19,
    LTOK_LT = 20, LTOK_GT = 21, LTOK_EQ = 22, LTOK_EXCL = 23,
    LTOK_USE = 24, LTOK_CONST = 25, LTOK_DATA = 26, LTOK_CODE = 27,
    LTOK_TEST = 31, LTOK_INVARIANT = 32, LTOK_BENCH = 33,
    LTOK_INTEGER = 48, LTOK_FLOAT = 49, LTOK_STRING = 50,
    LTOK_IDENTIFIER = 51, LTOK_REG = 52, LTOK_LABEL = 53,
    LTOK_MOV = 60, LTOK_HALT = 78
} T27LexTokenType;

typedef struct {
    T27LexTokenType kind;
    size_t          start;
    size_t          end;
    uint32_t        line;
    uint32_t        column;
} T27LexToken;

typedef struct {
    const char *source;
    size_t      source_len;
    size_t      pos;
    uint32_t    line;
    uint32_t    column;
} T27Lexer;

T27Lexer    t27_lexer_init(const char *source, size_t len);
T27LexToken t27_lexer_next(T27Lexer *lex);
bool        t27_lex_is_ident_start(char c);
bool        t27_lex_is_ident_continue(char c);
bool        t27_lex_is_digit(char c);

#ifdef __cplusplus
}
#endif

#endif /* T27_LEXER_H */
