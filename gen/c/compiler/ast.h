/* Auto-generated from compiler/ast.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/ast.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef T27_COMPILER_AST_H
#define T27_COMPILER_AST_H

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
    TOK_NEWLINE = 1,
    TOK_DOT = 2,
    TOK_COLON = 3,
    TOK_SEMICOLON = 4,
    TOK_COMMA = 5,
    TOK_HASH = 6,
    TOK_LPAREN = 7,
    TOK_RPAREN = 8,
    TOK_LBRACKET = 9,
    TOK_RBRACKET = 10,
    TOK_PLUS = 11,
    TOK_MINUS = 12,
    TOK_STAR = 13,
    TOK_SLASH = 14,
    TOK_PERCENT = 15,
    TOK_AND = 16,
    TOK_OR = 17,
    TOK_XOR = 18,
    TOK_TILDE = 19,
    TOK_LT = 20,
    TOK_GT = 21,
    TOK_EQ = 22,
    TOK_EXCL = 23,
    /* Keywords */
    TOK_USE = 24,
    TOK_CONST = 25,
    TOK_DATA = 26,
    TOK_CODE = 27,
    TOK_DWORD = 28,
    TOK_DSPACE = 29,
    TOK_DTRIT = 30,
    /* TDD sections */
    TOK_TEST = 31,
    TOK_INVARIANT = 32,
    TOK_BENCH = 33,
    TOK_VERIFY = 34,
    TOK_EXPECTED = 35,
    TOK_SETUP = 36,
    TOK_RATIONALE = 37,
    TOK_MEASURE = 38,
    TOK_TARGET = 39,
    /* Literals */
    TOK_INTEGER = 40,
    TOK_FLOAT = 41,
    TOK_STRING = 42,
    TOK_IDENTIFIER = 43,
    TOK_REG = 44,
    TOK_LABEL = 45,
    /* Opcodes */
    TOK_MOV = 60,
    TOK_JZ = 61,
    TOK_JNZ = 62,
    TOK_JMP = 63,
    TOK_HALT = 78,
    /* Spec-style */
    TOK_SPEC = 93,
    TOK_RULE = 94,
    TOK_GIVEN = 95,
    TOK_WHEN = 96,
    TOK_THEN = 97,
    TOK_ASSERT = 98,
    TOK_EXPECT = 100
} T27TokenType;

/* ========================================================================== */
/* Node Types                                                                  */
/* ========================================================================== */

typedef enum {
    NODE_PROGRAM = 0,
    NODE_DATA_SECTION = 1,
    NODE_CODE_SECTION = 2,
    NODE_CONST_DEF = 10,
    NODE_DWORD = 20,
    NODE_DSPACE = 21,
    NODE_DTRIT = 22,
    NODE_MOV = 30,
    NODE_JZ = 31,
    NODE_JNZ = 32,
    NODE_JMP = 33,
    NODE_MUL = 34,
    NODE_ADD = 35,
    NODE_SUB = 36,
    NODE_BIND = 37,
    NODE_BUNDLE = 38,
    NODE_HALT = 39,
    NODE_REG_OPERAND = 40,
    NODE_IMM_OPERAND = 41,
    NODE_LABEL_OPERAND = 42,
    NODE_MEM_OPERAND = 43,
    NODE_TEST = 50,
    NODE_INVARIANT = 51,
    NODE_BENCH = 52,
    NODE_TEST_CASE = 53,
    NODE_INVARIANT_DECL = 54,
    NODE_BENCH_DECL = 55,
    NODE_SPEC_DECL = 60,
    NODE_RULE_BLOCK = 61,
    NODE_GIVEN_CLAUSE = 62,
    NODE_WHEN_CLAUSE = 63,
    NODE_THEN_CLAUSE = 64,
    NODE_AND_CLAUSE = 65,
    NODE_EXPECT_CLAUSE = 66,
    NODE_ASSERT_STMT = 67,
    NODE_TEST_BLOCK_HL = 68,
    NODE_RULE_DECL = 69
} T27NodeType;

/* ========================================================================== */
/* Opcode Enum                                                                 */
/* ========================================================================== */

typedef enum {
    OP_MOV = 0,
    OP_JZ = 1,
    OP_JNZ = 2,
    OP_JMP = 3,
    OP_MUL = 4,
    OP_ADD = 5,
    OP_SUB = 6,
    OP_BIND = 7,
    OP_BUNDLE = 8,
    OP_HALT = 9
} T27Opcode;

/* ========================================================================== */
/* Operand Type                                                                */
/* ========================================================================== */

typedef enum {
    OPERAND_REGISTER = 0,
    OPERAND_IMMEDIATE = 1,
    OPERAND_LABEL_REF = 2,
    OPERAND_MEMORY = 3
} T27OperandType;

/* ========================================================================== */
/* Compilation Phase                                                           */
/* ========================================================================== */

typedef enum {
    PHASE_PARSING = 0,
    PHASE_SEMANTIC_ANALYSIS = 1,
    PHASE_CODE_GENERATION = 2,
    PHASE_OPTIMIZATION = 3
} T27CompilationPhase;

/* ========================================================================== */
/* AST Node                                                                    */
/* ========================================================================== */

typedef struct {
    T27NodeType node_type;
    uint32_t    line;
    uint32_t    column;
    const char *source_file;
} T27ASTNode;

/* ========================================================================== */
/* Structures                                                                  */
/* ========================================================================== */

typedef struct {
    T27ASTNode  node;
    const char *name;
    int64_t     value;
} T27ConstDef;

typedef struct {
    T27ASTNode  node;
    uint8_t     size;
    int64_t     initial_value;
    const char *label;
} T27DataDecl;

typedef struct {
    T27ASTNode         node;
    const T27DataDecl *declarations;
    size_t             decl_count;
} T27DataSection;

typedef struct {
    T27ASTNode      node;
    T27OperandType  operand_type;
} T27Operand;

typedef struct {
    T27ASTNode        node;
    T27Opcode         opcode;
    const T27Operand *operands;
    size_t            operand_count;
} T27Instruction;

typedef struct {
    T27ASTNode              node;
    const T27Instruction   *instructions;
    size_t                  instr_count;
} T27CodeSection;

typedef struct {
    T27ASTNode  node;
    const char *name;
    const char *verify_description;
    const char *expected_outcome;
    const char *setup_description;
    const char *rationale;
} T27TestCase;

typedef struct {
    T27ASTNode  node;
    const char *name;
    const char *formal_statement;
    const char *rationale;
} T27InvariantDecl;

typedef struct {
    T27ASTNode  node;
    const char *name;
    const char *measure_description;
    const char *target_value;
    const char *units;
} T27BenchDecl;

typedef struct {
    T27ASTNode            node;
    const T27TestCase    *test_cases;
    size_t                case_count;
} T27TestSection;

typedef struct {
    T27ASTNode                node;
    const T27InvariantDecl   *invariants;
    size_t                    inv_count;
} T27InvariantSection;

typedef struct {
    T27ASTNode           node;
    const T27BenchDecl  *benchmarks;
    size_t               bench_count;
} T27BenchSection;

typedef struct {
    T27ASTNode             node;
    const T27ConstDef     *constants;
    size_t                 const_count;
    T27DataSection        *data_section;
    T27CodeSection        *code_section;
    T27TestSection        *test_section;
    T27InvariantSection   *invariant_section;
    T27BenchSection       *bench_section;
} T27Program;

/* ========================================================================== */
/* Compiler Error / Warning                                                    */
/* ========================================================================== */

typedef struct {
    const char          *message;
    uint32_t             line;
    uint32_t             column;
    const char          *source_file;
    T27CompilationPhase  phase;
} T27CompilerError;

typedef struct {
    const char *message;
    uint32_t    line;
    uint32_t    column;
    const char *source_file;
} T27CompilerWarning;

/* ========================================================================== */
/* Type Info / Symbol                                                          */
/* ========================================================================== */

typedef struct {
    const char *name;
    uint8_t     size_bits;
    bool        is_signed;
} T27TypeInfo;

typedef struct {
    const char *name;
    T27ASTNode  node;
    const char *scope;
    bool        is_exported;
    bool        is_defined;
} T27Symbol;

#ifdef __cplusplus
}
#endif

#endif /* T27_COMPILER_AST_H */
