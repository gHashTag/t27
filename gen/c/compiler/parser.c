/* Auto-generated from specs/compiler/parser.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/compiler/parser.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "parser.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <assert.h>

/* ========================================================================== */
/* Internal Constants                                                          */
/* ========================================================================== */

#define INITIAL_TOKEN_CAPACITY   64
#define INITIAL_DECL_CAPACITY    16
#define INITIAL_FIELD_CAPACITY   16
#define INITIAL_VALUE_CAPACITY   16
#define INITIAL_JSON_CAPACITY   256
#define TYPE_BUF_SIZE           128
#define VALUE_BUF_SIZE          256

/* ========================================================================== */
/* Character Classification                                                    */
/* ========================================================================== */

bool t27_is_ident_start(char c) {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
}

bool t27_is_ident_continue(char c) {
    return t27_is_ident_start(c) || (c >= '0' && c <= '9') || c == '-';
}

bool t27_is_digit(char c) {
    return c >= '0' && c <= '9';
}

bool t27_is_whitespace(char c) {
    return c == ' ' || c == '\t' || c == '\n' || c == '\r';
}

/* ========================================================================== */
/* Keyword Lookup                                                              */
/* ========================================================================== */

T27TokenKind t27_keyword_kind(const char *text, size_t len) {
    /* Table of keywords mapped to token kinds */
    static const struct {
        const char  *word;
        size_t       len;
        T27TokenKind kind;
    } keywords[] = {
        { "pub",       3,  TOK_PUB       },
        { "const",     5,  TOK_CONST     },
        { "extern",    6,  TOK_EXTERN    },
        { "struct",    6,  TOK_STRUCT    },
        { "enum",      4,  TOK_ENUM      },
        { "fn",        2,  TOK_FN        },
        { "return",    6,  TOK_RETURN    },
        { "if",        2,  TOK_IF        },
        { "else",      4,  TOK_ELSE      },
        { "using",     5,  TOK_USING     },
        { "test",      4,  TOK_TEST      },
        { "bench",     5,  TOK_BENCH     },
        { "invariant", 9,  TOK_INVARIANT },
        { "module",    6,  TOK_MODULE    },
        { "type",      4,  TOK_TYPE      },
    };
    size_t i;
    for (i = 0; i < sizeof(keywords) / sizeof(keywords[0]); i++) {
        if (len == keywords[i].len && memcmp(text, keywords[i].word, len) == 0) {
            return keywords[i].kind;
        }
    }
    return TOK_IDENT;
}

T27TokenKind t27_single_char_token_kind(char c) {
    switch (c) {
        case '+': return TOK_PLUS;
        case '*': return TOK_STAR;
        case '/': return TOK_SLASH;
        case '%': return TOK_PERCENT;
        case '=': return TOK_ASSIGN;
        case '<': return TOK_LT;
        case '>': return TOK_GT;
        case '!': return TOK_NOT;
        case '&': return TOK_AND;
        case '|': return TOK_OR;
        case '^': return TOK_XOR;
        case '(': return TOK_L_PAREN;
        case ')': return TOK_R_PAREN;
        case '{': return TOK_L_BRACE;
        case '}': return TOK_R_BRACE;
        case '[': return TOK_L_BRACKET;
        case ']': return TOK_R_BRACKET;
        case ',': return TOK_COMMA;
        case ':': return TOK_COLON;
        case '.': return TOK_DOT;
        case ';': return TOK_SEMICOLON;
        case '?': return TOK_QUESTION;
        default:  return TOK_EOF;
    }
}

/* ========================================================================== */
/* Token List                                                                  */
/* ========================================================================== */

T27TokenList t27_token_list_create(void) {
    T27TokenList list;
    list.tokens = (T27Token *)malloc(INITIAL_TOKEN_CAPACITY * sizeof(T27Token));
    list.count = 0;
    list.capacity = INITIAL_TOKEN_CAPACITY;
    return list;
}

void t27_token_list_destroy(T27TokenList *list) {
    if (list && list->tokens) {
        free(list->tokens);
        list->tokens = NULL;
        list->count = 0;
        list->capacity = 0;
    }
}

bool t27_token_list_append(T27TokenList *list, T27Token token) {
    if (list->count >= list->capacity) {
        size_t new_cap = list->capacity * 2;
        T27Token *new_buf = (T27Token *)realloc(list->tokens, new_cap * sizeof(T27Token));
        if (!new_buf) return false;
        list->tokens = new_buf;
        list->capacity = new_cap;
    }
    list->tokens[list->count++] = token;
    return true;
}

/* ========================================================================== */
/* Tokenizer                                                                   */
/* ========================================================================== */

T27TokenList t27_tokenize(const char *source, size_t source_len) {
    T27TokenList list = t27_token_list_create();
    size_t   pos  = 0;
    uint32_t line = 1;
    uint32_t col  = 1;

    while (pos < source_len) {
        char c = source[pos];

        /* Skip whitespace */
        if (t27_is_whitespace(c)) {
            if (c == '\n') { line++; col = 1; }
            else { col++; }
            pos++;
            continue;
        }

        /* Skip ; comments */
        if (c == ';') {
            while (pos < source_len && source[pos] != '\n') pos++;
            continue;
        }

        /* Skip // comments */
        if (c == '/' && pos + 1 < source_len && source[pos + 1] == '/') {
            while (pos < source_len && source[pos] != '\n') pos++;
            continue;
        }

        /* Identifiers and keywords */
        if (t27_is_ident_start(c)) {
            size_t start = pos;
            uint32_t start_col = col;
            while (pos < source_len && t27_is_ident_continue(source[pos])) {
                pos++; col++;
            }
            T27Token tok;
            tok.text     = source + start;
            tok.text_len = pos - start;
            tok.kind     = t27_keyword_kind(tok.text, tok.text_len);
            tok.line     = line;
            tok.col      = start_col;
            t27_token_list_append(&list, tok);
            continue;
        }

        /* Numbers */
        if (t27_is_digit(c)) {
            size_t start = pos;
            uint32_t start_col = col;
            while (pos < source_len && t27_is_digit(source[pos])) {
                pos++; col++;
            }
            T27Token tok;
            tok.kind     = TOK_LITERAL;
            tok.text     = source + start;
            tok.text_len = pos - start;
            tok.line     = line;
            tok.col      = start_col;
            t27_token_list_append(&list, tok);
            continue;
        }

        /* String literals */
        if (c == '"') {
            pos++; col++; /* skip opening quote */
            size_t start = pos;
            uint32_t start_col = col;
            while (pos < source_len && source[pos] != '"') {
                pos++; col++;
            }
            T27Token tok;
            tok.kind     = TOK_LITERAL;
            tok.text     = source + start;
            tok.text_len = pos - start;
            tok.line     = line;
            tok.col      = start_col;
            if (pos < source_len) { pos++; col++; } /* skip closing quote */
            t27_token_list_append(&list, tok);
            continue;
        }

        /* Multi-char operators starting with '-' */
        if (c == '-') {
            uint32_t start_col = col;
            if (pos + 1 < source_len && source[pos + 1] == '>') {
                /* Arrow -> */
                T27Token tok;
                tok.kind     = TOK_ARROW;
                tok.text     = source + pos;
                tok.text_len = 2;
                tok.line     = line;
                tok.col      = start_col;
                t27_token_list_append(&list, tok);
                pos += 2; col += 2;
                continue;
            }
            if (pos + 1 < source_len && t27_is_digit(source[pos + 1])) {
                /* Negative number */
                size_t start = pos;
                pos++; col++; /* skip '-' */
                while (pos < source_len && t27_is_digit(source[pos])) {
                    pos++; col++;
                }
                T27Token tok;
                tok.kind     = TOK_LITERAL;
                tok.text     = source + start;
                tok.text_len = pos - start;
                tok.line     = line;
                tok.col      = start_col;
                t27_token_list_append(&list, tok);
                continue;
            }
            /* Bare minus */
            {
                T27Token tok;
                tok.kind     = TOK_MINUS;
                tok.text     = source + pos;
                tok.text_len = 1;
                tok.line     = line;
                tok.col      = start_col;
                t27_token_list_append(&list, tok);
                pos++; col++;
                continue;
            }
        }

        /* == */
        if (c == '=' && pos + 1 < source_len && source[pos + 1] == '=') {
            T27Token tok;
            tok.kind = TOK_EQ; tok.text = source + pos; tok.text_len = 2;
            tok.line = line; tok.col = col;
            t27_token_list_append(&list, tok);
            pos += 2; col += 2;
            continue;
        }
        /* != */
        if (c == '!' && pos + 1 < source_len && source[pos + 1] == '=') {
            T27Token tok;
            tok.kind = TOK_NE; tok.text = source + pos; tok.text_len = 2;
            tok.line = line; tok.col = col;
            t27_token_list_append(&list, tok);
            pos += 2; col += 2;
            continue;
        }
        /* <= */
        if (c == '<' && pos + 1 < source_len && source[pos + 1] == '=') {
            T27Token tok;
            tok.kind = TOK_LE; tok.text = source + pos; tok.text_len = 2;
            tok.line = line; tok.col = col;
            t27_token_list_append(&list, tok);
            pos += 2; col += 2;
            continue;
        }
        /* >= */
        if (c == '>' && pos + 1 < source_len && source[pos + 1] == '=') {
            T27Token tok;
            tok.kind = TOK_GE; tok.text = source + pos; tok.text_len = 2;
            tok.line = line; tok.col = col;
            t27_token_list_append(&list, tok);
            pos += 2; col += 2;
            continue;
        }

        /* Single-char tokens */
        {
            T27TokenKind kind = t27_single_char_token_kind(c);
            if (kind != TOK_EOF) {
                T27Token tok;
                tok.kind     = kind;
                tok.text     = source + pos;
                tok.text_len = 1;
                tok.line     = line;
                tok.col      = col;
                t27_token_list_append(&list, tok);
                pos++; col++;
                continue;
            }
        }

        /* Unknown: skip */
        pos++; col++;
    }

    /* Append EOF */
    {
        T27Token tok;
        tok.kind     = TOK_EOF;
        tok.text     = "";
        tok.text_len = 0;
        tok.line     = line;
        tok.col      = col;
        t27_token_list_append(&list, tok);
    }

    return list;
}

/* ========================================================================== */
/* Parser Helpers                                                              */
/* ========================================================================== */

T27Token t27_peek(T27ParseState *state) {
    if (state->pos < state->count) {
        return state->tokens[state->pos];
    }
    T27Token eof;
    eof.kind = TOK_EOF; eof.text = ""; eof.text_len = 0;
    eof.line = 0; eof.col = 0;
    return eof;
}

T27Token t27_next(T27ParseState *state) {
    T27Token tok = t27_peek(state);
    if (state->pos < state->count) {
        state->pos++;
    }
    return tok;
}

bool t27_eat(T27ParseState *state, T27TokenKind kind) {
    if (t27_peek(state).kind == kind) {
        t27_next(state);
        return true;
    }
    return false;
}

/* ========================================================================== */
/* Internal: dynamic string buffer                                             */
/* ========================================================================== */

typedef struct {
    char  *data;
    size_t len;
    size_t cap;
} T27StrBuf;

static T27StrBuf t27_strbuf_create(void) {
    T27StrBuf sb;
    sb.data = (char *)malloc(64);
    sb.len  = 0;
    sb.cap  = 64;
    if (sb.data) sb.data[0] = '\0';
    return sb;
}

static void t27_strbuf_append(T27StrBuf *sb, const char *text, size_t len) {
    if (!sb->data || !text) return;
    while (sb->len + len + 1 > sb->cap) {
        sb->cap *= 2;
        char *new_data = (char *)realloc(sb->data, sb->cap);
        if (!new_data) return;
        sb->data = new_data;
    }
    memcpy(sb->data + sb->len, text, len);
    sb->len += len;
    sb->data[sb->len] = '\0';
}

static void t27_strbuf_append_char(T27StrBuf *sb, char c) {
    t27_strbuf_append(sb, &c, 1);
}

static char *t27_strbuf_take(T27StrBuf *sb) {
    char *result = sb->data;
    sb->data = NULL;
    sb->len = 0;
    sb->cap = 0;
    return result;
}

static void t27_strbuf_destroy(T27StrBuf *sb) {
    if (sb->data) free(sb->data);
    sb->data = NULL;
    sb->len = 0;
    sb->cap = 0;
}

/* ========================================================================== */
/* Internal: duplicate token text as null-terminated string                     */
/* ========================================================================== */

static char *t27_tok_strdup(T27Token tok) {
    char *s = (char *)malloc(tok.text_len + 1);
    if (s) {
        memcpy(s, tok.text, tok.text_len);
        s[tok.text_len] = '\0';
    }
    return s;
}

/* ========================================================================== */
/* Internal: collect type tokens into string                                   */
/* ========================================================================== */

static char *t27_collect_type(T27ParseState *state, size_t *out_len) {
    T27StrBuf sb = t27_strbuf_create();
    while (t27_peek(state).kind == TOK_IDENT   ||
           t27_peek(state).kind == TOK_L_BRACKET ||
           t27_peek(state).kind == TOK_R_BRACKET ||
           t27_peek(state).kind == TOK_STAR     ||
           t27_peek(state).kind == TOK_QUESTION)
    {
        T27Token tok = t27_next(state);
        t27_strbuf_append(&sb, tok.text, tok.text_len);
    }
    if (out_len) *out_len = sb.len;
    return t27_strbuf_take(&sb);
}

/* ========================================================================== */
/* Internal: collect value tokens until delimiter                              */
/* ========================================================================== */

static char *t27_collect_value(T27ParseState *state, T27TokenKind delim,
                               T27TokenKind delim2, size_t *out_len) {
    T27StrBuf sb = t27_strbuf_create();
    bool first = true;
    while (t27_peek(state).kind != delim &&
           t27_peek(state).kind != delim2 &&
           t27_peek(state).kind != TOK_EOF)
    {
        if (!first) t27_strbuf_append_char(&sb, ' ');
        T27Token tok = t27_next(state);
        t27_strbuf_append(&sb, tok.text, tok.text_len);
        first = false;
    }
    if (out_len) *out_len = sb.len;
    return t27_strbuf_take(&sb);
}

/* ========================================================================== */
/* Internal: skip brace-delimited body                                         */
/* ========================================================================== */

static void t27_skip_body(T27ParseState *state) {
    if (t27_eat(state, TOK_L_BRACE)) {
        int depth = 1;
        while (depth > 0 && t27_peek(state).kind != TOK_EOF) {
            T27Token tok = t27_next(state);
            if (tok.kind == TOK_L_BRACE) depth++;
            if (tok.kind == TOK_R_BRACE) depth--;
        }
    }
}

/* ========================================================================== */
/* AST Node List                                                               */
/* ========================================================================== */

static T27AstNodeList t27_ast_list_create(void) {
    T27AstNodeList list;
    list.nodes = (T27AstNode *)malloc(INITIAL_DECL_CAPACITY * sizeof(T27AstNode));
    list.count = 0;
    list.capacity = INITIAL_DECL_CAPACITY;
    return list;
}

static void t27_ast_list_append(T27AstNodeList *list, T27AstNode node) {
    if (list->count >= list->capacity) {
        list->capacity *= 2;
        list->nodes = (T27AstNode *)realloc(list->nodes,
                                             list->capacity * sizeof(T27AstNode));
    }
    list->nodes[list->count++] = node;
}

/* ========================================================================== */
/* Parser Functions                                                            */
/* ========================================================================== */

T27AstNode t27_parse_const_decl(T27ParseState *state, bool is_pub) {
    T27Token name_tok = t27_next(state);
    t27_eat(state, TOK_COLON);

    size_t type_len = 0;
    char *type_ref = t27_collect_type(state, &type_len);

    t27_eat(state, TOK_ASSIGN);

    size_t value_len = 0;
    char *value = t27_collect_value(state, TOK_SEMICOLON, TOK_SEMICOLON, &value_len);
    t27_eat(state, TOK_SEMICOLON);

    T27AstNode node;
    node.kind = AST_CONST_DECL;
    node.data.const_decl.is_pub       = is_pub;
    node.data.const_decl.name         = t27_tok_strdup(name_tok);
    node.data.const_decl.name_len     = name_tok.text_len;
    node.data.const_decl.type_ref     = type_ref;
    node.data.const_decl.type_ref_len = type_len;
    node.data.const_decl.value        = value;
    node.data.const_decl.value_len    = value_len;
    return node;
}

T27AstNode t27_parse_enum_decl(T27ParseState *state) {
    char *name = NULL;
    size_t name_len = 0;

    if (t27_peek(state).kind == TOK_IDENT) {
        T27Token name_tok = t27_next(state);
        name = t27_tok_strdup(name_tok);
        name_len = name_tok.text_len;
    }

    t27_eat(state, TOK_L_PAREN);
    T27Token backing_tok = t27_next(state);
    t27_eat(state, TOK_R_PAREN);
    t27_eat(state, TOK_L_BRACE);

    size_t values_cap = INITIAL_VALUE_CAPACITY;
    T27EnumValueNode *values = (T27EnumValueNode *)malloc(
        values_cap * sizeof(T27EnumValueNode));
    size_t values_count = 0;

    while (t27_peek(state).kind != TOK_R_BRACE &&
           t27_peek(state).kind != TOK_EOF) {
        T27Token val_name_tok = t27_next(state);
        t27_eat(state, TOK_ASSIGN);

        size_t val_len = 0;
        char *val = t27_collect_value(state, TOK_COMMA, TOK_R_BRACE, &val_len);

        if (values_count >= values_cap) {
            values_cap *= 2;
            values = (T27EnumValueNode *)realloc(values,
                values_cap * sizeof(T27EnumValueNode));
        }
        values[values_count].name      = t27_tok_strdup(val_name_tok);
        values[values_count].name_len  = val_name_tok.text_len;
        values[values_count].value     = val;
        values[values_count].value_len = val_len;
        values_count++;

        t27_eat(state, TOK_COMMA);
    }
    t27_eat(state, TOK_R_BRACE);

    T27AstNode node;
    node.kind = AST_ENUM_DECL;
    node.data.enum_decl.name         = name ? name : (char *)calloc(1, 1);
    node.data.enum_decl.name_len     = name_len;
    node.data.enum_decl.backing      = t27_tok_strdup(backing_tok);
    node.data.enum_decl.backing_len  = backing_tok.text_len;
    node.data.enum_decl.values       = values;
    node.data.enum_decl.values_count = values_count;
    return node;
}

T27AstNode t27_parse_struct_decl(T27ParseState *state) {
    T27Token name_tok = t27_next(state);
    t27_eat(state, TOK_L_BRACE);

    size_t fields_cap = INITIAL_FIELD_CAPACITY;
    T27FieldNode *fields = (T27FieldNode *)malloc(
        fields_cap * sizeof(T27FieldNode));
    size_t fields_count = 0;

    while (t27_peek(state).kind != TOK_R_BRACE &&
           t27_peek(state).kind != TOK_EOF) {
        T27Token field_name_tok = t27_next(state);
        t27_eat(state, TOK_COLON);

        size_t type_len = 0;
        char *type_ref = t27_collect_type(state, &type_len);

        if (fields_count >= fields_cap) {
            fields_cap *= 2;
            fields = (T27FieldNode *)realloc(fields,
                fields_cap * sizeof(T27FieldNode));
        }
        fields[fields_count].name         = t27_tok_strdup(field_name_tok);
        fields[fields_count].name_len     = field_name_tok.text_len;
        fields[fields_count].type_ref     = type_ref;
        fields[fields_count].type_ref_len = type_len;
        fields_count++;

        t27_eat(state, TOK_COMMA);
    }
    t27_eat(state, TOK_R_BRACE);

    T27AstNode node;
    node.kind = AST_STRUCT_DECL;
    node.data.struct_decl.name         = t27_tok_strdup(name_tok);
    node.data.struct_decl.name_len     = name_tok.text_len;
    node.data.struct_decl.fields       = fields;
    node.data.struct_decl.fields_count = fields_count;
    return node;
}

T27AstNode t27_parse_fn_decl(T27ParseState *state) {
    T27Token name_tok = t27_next(state);
    t27_eat(state, TOK_L_PAREN);

    size_t params_cap = INITIAL_FIELD_CAPACITY;
    T27ParamNode *params = (T27ParamNode *)malloc(
        params_cap * sizeof(T27ParamNode));
    size_t params_count = 0;

    while (t27_peek(state).kind != TOK_R_PAREN &&
           t27_peek(state).kind != TOK_EOF) {
        T27Token param_name_tok = t27_next(state);
        t27_eat(state, TOK_COLON);

        size_t type_len = 0;
        char *type_ref = t27_collect_type(state, &type_len);

        if (params_count >= params_cap) {
            params_cap *= 2;
            params = (T27ParamNode *)realloc(params,
                params_cap * sizeof(T27ParamNode));
        }
        params[params_count].name         = t27_tok_strdup(param_name_tok);
        params[params_count].name_len     = param_name_tok.text_len;
        params[params_count].type_ref     = type_ref;
        params[params_count].type_ref_len = type_len;
        params_count++;

        t27_eat(state, TOK_COMMA);
    }
    t27_eat(state, TOK_R_PAREN);

    /* Return type */
    const char *return_type = "void";
    size_t return_type_len = 4;
    char  *return_type_alloc = NULL;

    if (t27_eat(state, TOK_ARROW)) {
        T27Token ret_tok = t27_next(state);
        return_type_alloc = t27_tok_strdup(ret_tok);
        return_type = return_type_alloc;
        return_type_len = ret_tok.text_len;
    } else if (t27_peek(state).kind == TOK_IDENT) {
        /* Return type without arrow (spec style) */
        T27Token ret_tok = t27_next(state);
        return_type_alloc = t27_tok_strdup(ret_tok);
        return_type = return_type_alloc;
        return_type_len = ret_tok.text_len;
    }

    /* Skip body */
    t27_skip_body(state);

    T27AstNode node;
    node.kind = AST_FN_DECL;
    node.data.fn_decl.name           = t27_tok_strdup(name_tok);
    node.data.fn_decl.name_len       = name_tok.text_len;
    node.data.fn_decl.params         = params;
    node.data.fn_decl.params_count   = params_count;
    if (return_type_alloc) {
        node.data.fn_decl.return_type     = return_type_alloc;
    } else {
        /* Duplicate the static "void" so caller can always free */
        node.data.fn_decl.return_type     = strdup("void");
    }
    node.data.fn_decl.return_type_len = return_type_len;
    return node;
}

T27AstNode t27_parse_invariant_decl(T27ParseState *state) {
    T27Token name_tok = t27_next(state);
    t27_skip_body(state);

    T27AstNode node;
    node.kind = AST_INVARIANT_DECL;
    node.data.invariant_decl.name     = t27_tok_strdup(name_tok);
    node.data.invariant_decl.name_len = name_tok.text_len;
    return node;
}

T27AstNode t27_parse_test_decl(T27ParseState *state) {
    T27Token name_tok = t27_next(state);
    t27_skip_body(state);

    T27AstNode node;
    node.kind = AST_TEST_DECL;
    node.data.test_decl.name     = t27_tok_strdup(name_tok);
    node.data.test_decl.name_len = name_tok.text_len;
    return node;
}

T27AstNode t27_parse_bench_decl(T27ParseState *state) {
    T27Token name_tok = t27_next(state);
    t27_skip_body(state);

    T27AstNode node;
    node.kind = AST_BENCH_DECL;
    node.data.bench_decl.name     = t27_tok_strdup(name_tok);
    node.data.bench_decl.name_len = name_tok.text_len;
    return node;
}

T27AstNode t27_parse_decl(T27ParseState *state, bool *valid) {
    bool is_pub = t27_eat(state, TOK_PUB);
    *valid = true;

    if (t27_eat(state, TOK_CONST)) {
        return t27_parse_const_decl(state, is_pub);
    } else if (t27_eat(state, TOK_ENUM)) {
        return t27_parse_enum_decl(state);
    } else if (t27_eat(state, TOK_STRUCT)) {
        return t27_parse_struct_decl(state);
    } else if (t27_eat(state, TOK_FN)) {
        return t27_parse_fn_decl(state);
    } else if (t27_eat(state, TOK_INVARIANT)) {
        return t27_parse_invariant_decl(state);
    } else if (t27_eat(state, TOK_TEST)) {
        return t27_parse_test_decl(state);
    } else if (t27_eat(state, TOK_BENCH)) {
        return t27_parse_bench_decl(state);
    }

    /* Skip unknown token */
    t27_next(state);
    *valid = false;

    T27AstNode node;
    memset(&node, 0, sizeof(node));
    return node;
}

T27ModuleNode t27_parse_module(T27ParseState *state) {
    T27ModuleNode mod;
    mod.name     = "";
    mod.name_len = 0;

    /* Optional: module NAME; */
    if (t27_eat(state, TOK_MODULE)) {
        T27Token name_tok = t27_next(state);
        mod.name     = t27_tok_strdup(name_tok);
        mod.name_len = name_tok.text_len;
        t27_eat(state, TOK_SEMICOLON);
    }

    T27AstNodeList decls = t27_ast_list_create();

    while (t27_peek(state).kind != TOK_EOF) {
        bool valid = false;
        T27AstNode decl = t27_parse_decl(state, &valid);
        if (valid) {
            t27_ast_list_append(&decls, decl);
        }
    }

    mod.decls       = decls.nodes;
    mod.decls_count = decls.count;
    return mod;
}

T27ModuleNode t27_parse(const char *source, size_t source_len) {
    T27TokenList tlist = t27_tokenize(source, source_len);

    T27ParseState state;
    state.tokens = tlist.tokens;
    state.count  = tlist.count;
    state.pos    = 0;

    T27ModuleNode mod = t27_parse_module(&state);

    t27_token_list_destroy(&tlist);
    return mod;
}

/* ========================================================================== */
/* JSON Output                                                                 */
/* ========================================================================== */

static void t27_json_write_escaped(T27StrBuf *sb, const char *s, size_t len) {
    size_t i;
    for (i = 0; i < len; i++) {
        switch (s[i]) {
            case '"':  t27_strbuf_append(sb, "\\\"", 2); break;
            case '\\': t27_strbuf_append(sb, "\\\\", 2); break;
            case '\n': t27_strbuf_append(sb, "\\n", 2);  break;
            case '\r': t27_strbuf_append(sb, "\\r", 2);  break;
            case '\t': t27_strbuf_append(sb, "\\t", 2);  break;
            default:   t27_strbuf_append_char(sb, s[i]);  break;
        }
    }
}

static void t27_json_write_node(T27StrBuf *sb, const T27AstNode *node) {
    size_t i;
    switch (node->kind) {
        case AST_CONST_DECL:
            t27_strbuf_append(sb, "{\"kind\":\"const\",\"pub\":", 21);
            t27_strbuf_append(sb,
                node->data.const_decl.is_pub ? "true" : "false",
                node->data.const_decl.is_pub ? 4 : 5);
            t27_strbuf_append(sb, ",\"name\":\"", 9);
            t27_json_write_escaped(sb, node->data.const_decl.name,
                                   node->data.const_decl.name_len);
            t27_strbuf_append(sb, "\",\"type\":\"", 10);
            t27_json_write_escaped(sb, node->data.const_decl.type_ref,
                                   node->data.const_decl.type_ref_len);
            t27_strbuf_append(sb, "\",\"value\":\"", 11);
            t27_json_write_escaped(sb, node->data.const_decl.value,
                                   node->data.const_decl.value_len);
            t27_strbuf_append(sb, "\"}", 2);
            break;

        case AST_ENUM_DECL:
            t27_strbuf_append(sb, "{\"kind\":\"enum\",\"name\":\"", 22);
            t27_json_write_escaped(sb, node->data.enum_decl.name,
                                   node->data.enum_decl.name_len);
            t27_strbuf_append(sb, "\",\"backing\":\"", 13);
            t27_json_write_escaped(sb, node->data.enum_decl.backing,
                                   node->data.enum_decl.backing_len);
            t27_strbuf_append(sb, "\",\"values\":[", 12);
            for (i = 0; i < node->data.enum_decl.values_count; i++) {
                if (i > 0) t27_strbuf_append_char(sb, ',');
                t27_strbuf_append(sb, "{\"name\":\"", 9);
                t27_json_write_escaped(sb,
                    node->data.enum_decl.values[i].name,
                    node->data.enum_decl.values[i].name_len);
                t27_strbuf_append(sb, "\",\"value\":\"", 11);
                t27_json_write_escaped(sb,
                    node->data.enum_decl.values[i].value,
                    node->data.enum_decl.values[i].value_len);
                t27_strbuf_append(sb, "\"}", 2);
            }
            t27_strbuf_append(sb, "]}", 2);
            break;

        case AST_STRUCT_DECL:
            t27_strbuf_append(sb, "{\"kind\":\"struct\",\"name\":\"", 24);
            t27_json_write_escaped(sb, node->data.struct_decl.name,
                                   node->data.struct_decl.name_len);
            t27_strbuf_append(sb, "\",\"fields\":[", 12);
            for (i = 0; i < node->data.struct_decl.fields_count; i++) {
                if (i > 0) t27_strbuf_append_char(sb, ',');
                t27_strbuf_append(sb, "{\"name\":\"", 9);
                t27_json_write_escaped(sb,
                    node->data.struct_decl.fields[i].name,
                    node->data.struct_decl.fields[i].name_len);
                t27_strbuf_append(sb, "\",\"type\":\"", 10);
                t27_json_write_escaped(sb,
                    node->data.struct_decl.fields[i].type_ref,
                    node->data.struct_decl.fields[i].type_ref_len);
                t27_strbuf_append(sb, "\"}", 2);
            }
            t27_strbuf_append(sb, "]}", 2);
            break;

        case AST_FN_DECL:
            t27_strbuf_append(sb, "{\"kind\":\"fn\",\"name\":\"", 20);
            t27_json_write_escaped(sb, node->data.fn_decl.name,
                                   node->data.fn_decl.name_len);
            t27_strbuf_append(sb, "\",\"params\":[", 12);
            for (i = 0; i < node->data.fn_decl.params_count; i++) {
                if (i > 0) t27_strbuf_append_char(sb, ',');
                t27_strbuf_append(sb, "{\"name\":\"", 9);
                t27_json_write_escaped(sb,
                    node->data.fn_decl.params[i].name,
                    node->data.fn_decl.params[i].name_len);
                t27_strbuf_append(sb, "\",\"type\":\"", 10);
                t27_json_write_escaped(sb,
                    node->data.fn_decl.params[i].type_ref,
                    node->data.fn_decl.params[i].type_ref_len);
                t27_strbuf_append(sb, "\"}", 2);
            }
            t27_strbuf_append(sb, "],\"return\":\"", 12);
            t27_json_write_escaped(sb, node->data.fn_decl.return_type,
                                   node->data.fn_decl.return_type_len);
            t27_strbuf_append(sb, "\"}", 2);
            break;

        case AST_INVARIANT_DECL:
            t27_strbuf_append(sb, "{\"kind\":\"invariant\",\"name\":\"", 27);
            t27_json_write_escaped(sb, node->data.invariant_decl.name,
                                   node->data.invariant_decl.name_len);
            t27_strbuf_append(sb, "\"}", 2);
            break;

        case AST_TEST_DECL:
            t27_strbuf_append(sb, "{\"kind\":\"test\",\"name\":\"", 22);
            t27_json_write_escaped(sb, node->data.test_decl.name,
                                   node->data.test_decl.name_len);
            t27_strbuf_append(sb, "\"}", 2);
            break;

        case AST_BENCH_DECL:
            t27_strbuf_append(sb, "{\"kind\":\"bench\",\"name\":\"", 23);
            t27_json_write_escaped(sb, node->data.bench_decl.name,
                                   node->data.bench_decl.name_len);
            t27_strbuf_append(sb, "\"}", 2);
            break;

        default:
            t27_strbuf_append(sb, "{\"kind\":\"unknown\"}", 18);
            break;
    }
}

char *t27_module_to_json(const T27ModuleNode *mod) {
    T27StrBuf sb = t27_strbuf_create();
    size_t i;

    t27_strbuf_append(&sb, "{\"module\":\"", 11);
    t27_json_write_escaped(&sb, mod->name, mod->name_len);
    t27_strbuf_append(&sb, "\",\"decls\":[", 11);

    for (i = 0; i < mod->decls_count; i++) {
        if (i > 0) t27_strbuf_append_char(&sb, ',');
        t27_json_write_node(&sb, &mod->decls[i]);
    }

    t27_strbuf_append(&sb, "]}", 2);
    return t27_strbuf_take(&sb);
}

void t27_json_free(char *json) {
    if (json) free(json);
}

/* ========================================================================== */
/* Cleanup                                                                     */
/* ========================================================================== */

void t27_module_node_destroy(T27ModuleNode *mod) {
    size_t i, j;
    if (!mod) return;

    /* Free name if dynamically allocated (not empty string literal) */
    if (mod->name && mod->name_len > 0) {
        free((void *)mod->name);
    }

    for (i = 0; i < mod->decls_count; i++) {
        T27AstNode *node = &mod->decls[i];
        switch (node->kind) {
            case AST_CONST_DECL:
                free((void *)node->data.const_decl.name);
                free((void *)node->data.const_decl.type_ref);
                free((void *)node->data.const_decl.value);
                break;
            case AST_ENUM_DECL:
                free((void *)node->data.enum_decl.name);
                free((void *)node->data.enum_decl.backing);
                for (j = 0; j < node->data.enum_decl.values_count; j++) {
                    free((void *)node->data.enum_decl.values[j].name);
                    free((void *)node->data.enum_decl.values[j].value);
                }
                free(node->data.enum_decl.values);
                break;
            case AST_STRUCT_DECL:
                free((void *)node->data.struct_decl.name);
                for (j = 0; j < node->data.struct_decl.fields_count; j++) {
                    free((void *)node->data.struct_decl.fields[j].name);
                    free((void *)node->data.struct_decl.fields[j].type_ref);
                }
                free(node->data.struct_decl.fields);
                break;
            case AST_FN_DECL:
                free((void *)node->data.fn_decl.name);
                for (j = 0; j < node->data.fn_decl.params_count; j++) {
                    free((void *)node->data.fn_decl.params[j].name);
                    free((void *)node->data.fn_decl.params[j].type_ref);
                }
                free(node->data.fn_decl.params);
                free((void *)node->data.fn_decl.return_type);
                break;
            case AST_INVARIANT_DECL:
                free((void *)node->data.invariant_decl.name);
                break;
            case AST_TEST_DECL:
                free((void *)node->data.test_decl.name);
                break;
            case AST_BENCH_DECL:
                free((void *)node->data.bench_decl.name);
                break;
            default:
                break;
        }
    }
    free(mod->decls);
    mod->decls = NULL;
    mod->decls_count = 0;
}
