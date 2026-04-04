/* Auto-generated from specs/compiler/parser.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/compiler/parser.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef T27_COMPILER_PARSER_H
#define T27_COMPILER_PARSER_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ========================================================================== */
/* Token Types                                                                 */
/* ========================================================================== */

typedef enum {
    TOK_EOF = 0,
    TOK_IDENT,
    TOK_LITERAL,
    /* Keywords */
    TOK_PUB,
    TOK_CONST,
    TOK_EXTERN,
    TOK_STRUCT,
    TOK_ENUM,
    TOK_FN,
    TOK_RETURN,
    TOK_IF,
    TOK_ELSE,
    TOK_USING,
    TOK_TEST,
    TOK_BENCH,
    TOK_INVARIANT,
    TOK_MODULE,
    TOK_TYPE,
    /* Operators */
    TOK_PLUS,
    TOK_MINUS,
    TOK_STAR,
    TOK_SLASH,
    TOK_PERCENT,
    TOK_EQ,
    TOK_NE,
    TOK_LT,
    TOK_LE,
    TOK_GT,
    TOK_GE,
    TOK_AND,
    TOK_OR,
    TOK_XOR,
    TOK_NOT,
    TOK_ASSIGN,
    TOK_ARROW,
    /* Delimiters */
    TOK_L_PAREN,
    TOK_R_PAREN,
    TOK_L_BRACE,
    TOK_R_BRACE,
    TOK_L_BRACKET,
    TOK_R_BRACKET,
    TOK_COMMA,
    TOK_COLON,
    TOK_DOT,
    TOK_SEMICOLON,
    TOK_QUESTION
} T27TokenKind;

typedef struct {
    T27TokenKind kind;
    const char  *text;
    size_t       text_len;
    uint32_t     line;
    uint32_t     col;
} T27Token;

/* ========================================================================== */
/* Token List                                                                  */
/* ========================================================================== */

typedef struct {
    T27Token *tokens;
    size_t    count;
    size_t    capacity;
} T27TokenList;

/* ========================================================================== */
/* AST Node Types                                                              */
/* ========================================================================== */

typedef enum {
    AST_MODULE = 0,
    AST_CONST_DECL,
    AST_ENUM_DECL,
    AST_STRUCT_DECL,
    AST_FN_DECL,
    AST_INVARIANT_DECL,
    AST_TEST_DECL,
    AST_BENCH_DECL
} T27AstNodeKind;

typedef enum {
    BINOP_ADD = 0,
    BINOP_SUB,
    BINOP_MUL,
    BINOP_DIV,
    BINOP_MOD,
    BINOP_EQ,
    BINOP_NE,
    BINOP_LT,
    BINOP_LE,
    BINOP_GT,
    BINOP_GE,
    BINOP_AND,
    BINOP_OR,
    BINOP_XOR,
    BINOP_ASSIGN,
    BINOP_ARROW
} T27BinaryOp;

/* ========================================================================== */
/* AST Nodes                                                                   */
/* ========================================================================== */

typedef struct {
    const char *name;
    size_t      name_len;
    const char *value;
    size_t      value_len;
} T27EnumValueNode;

typedef struct {
    const char *name;
    size_t      name_len;
    const char *type_ref;
    size_t      type_ref_len;
} T27FieldNode;

typedef struct {
    const char *name;
    size_t      name_len;
    const char *type_ref;
    size_t      type_ref_len;
} T27ParamNode;

typedef struct {
    bool        is_pub;
    const char *name;
    size_t      name_len;
    const char *type_ref;
    size_t      type_ref_len;
    const char *value;
    size_t      value_len;
} T27ConstDeclNode;

typedef struct {
    const char       *name;
    size_t            name_len;
    const char       *backing;
    size_t            backing_len;
    T27EnumValueNode *values;
    size_t            values_count;
} T27EnumDeclNode;

typedef struct {
    const char  *name;
    size_t       name_len;
    T27FieldNode *fields;
    size_t        fields_count;
} T27StructDeclNode;

typedef struct {
    const char  *name;
    size_t       name_len;
    T27ParamNode *params;
    size_t        params_count;
    const char  *return_type;
    size_t       return_type_len;
} T27FnDeclNode;

typedef struct {
    const char *name;
    size_t      name_len;
} T27InvariantDeclNode;

typedef struct {
    const char *name;
    size_t      name_len;
} T27TestDeclNode;

typedef struct {
    const char *name;
    size_t      name_len;
} T27BenchDeclNode;

typedef struct {
    T27AstNodeKind kind;
    union {
        T27ConstDeclNode     const_decl;
        T27EnumDeclNode      enum_decl;
        T27StructDeclNode    struct_decl;
        T27FnDeclNode        fn_decl;
        T27InvariantDeclNode invariant_decl;
        T27TestDeclNode      test_decl;
        T27BenchDeclNode     bench_decl;
    } data;
} T27AstNode;

typedef struct {
    const char *name;
    size_t      name_len;
    T27AstNode *decls;
    size_t      decls_count;
} T27ModuleNode;

/* ========================================================================== */
/* AST Node List                                                               */
/* ========================================================================== */

typedef struct {
    T27AstNode *nodes;
    size_t      count;
    size_t      capacity;
} T27AstNodeList;

/* ========================================================================== */
/* Parse State                                                                 */
/* ========================================================================== */

typedef struct {
    const T27Token *tokens;
    size_t          count;
    size_t          pos;
} T27ParseState;

/* ========================================================================== */
/* Character Classification                                                    */
/* ========================================================================== */

bool         t27_is_ident_start(char c);
bool         t27_is_ident_continue(char c);
bool         t27_is_digit(char c);
bool         t27_is_whitespace(char c);

/* ========================================================================== */
/* Keyword Lookup                                                              */
/* ========================================================================== */

T27TokenKind t27_keyword_kind(const char *text, size_t len);
T27TokenKind t27_single_char_token_kind(char c);

/* ========================================================================== */
/* Tokenizer                                                                   */
/* ========================================================================== */

T27TokenList t27_token_list_create(void);
void         t27_token_list_destroy(T27TokenList *list);
bool         t27_token_list_append(T27TokenList *list, T27Token token);
T27TokenList t27_tokenize(const char *source, size_t source_len);

/* ========================================================================== */
/* Parser                                                                      */
/* ========================================================================== */

T27Token     t27_peek(T27ParseState *state);
T27Token     t27_next(T27ParseState *state);
bool         t27_eat(T27ParseState *state, T27TokenKind kind);

T27ModuleNode t27_parse(const char *source, size_t source_len);
T27ModuleNode t27_parse_module(T27ParseState *state);
T27AstNode    t27_parse_decl(T27ParseState *state, bool *valid);
T27AstNode    t27_parse_const_decl(T27ParseState *state, bool is_pub);
T27AstNode    t27_parse_enum_decl(T27ParseState *state);
T27AstNode    t27_parse_struct_decl(T27ParseState *state);
T27AstNode    t27_parse_fn_decl(T27ParseState *state);
T27AstNode    t27_parse_invariant_decl(T27ParseState *state);
T27AstNode    t27_parse_test_decl(T27ParseState *state);
T27AstNode    t27_parse_bench_decl(T27ParseState *state);

/* ========================================================================== */
/* JSON Output                                                                 */
/* ========================================================================== */

char *t27_module_to_json(const T27ModuleNode *mod);
void  t27_json_free(char *json);

/* ========================================================================== */
/* Cleanup                                                                     */
/* ========================================================================== */

void t27_module_node_destroy(T27ModuleNode *mod);

#ifdef __cplusplus
}
#endif

#endif /* T27_COMPILER_PARSER_H */
