/* Auto-generated from compiler/codegen/c/codegen.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/codegen/c/codegen.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef T27_CODEGEN_C_CODEGEN_H
#define T27_CODEGEN_C_CODEGEN_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

#define T27_C_INDENT_SIZE       4
#define T27_C_MAX_LINE_LENGTH 100

typedef enum {
    C_NODE_PROGRAM = 0, C_NODE_CONST = 1, C_NODE_VAR = 2,
    C_NODE_BINOP = 3, C_NODE_UNOP = 4, C_NODE_CALL = 5,
    C_NODE_BLOCK = 6, C_NODE_IF = 7, C_NODE_LOOP = 8,
    C_NODE_RET = 9, C_NODE_FN = 10
} T27CNodeType;

typedef enum {
    C_BINOP_ADD = 0, C_BINOP_SUB = 1, C_BINOP_MUL = 2,
    C_BINOP_DIV = 3, C_BINOP_MOD = 4, C_BINOP_AND = 5,
    C_BINOP_OR = 6, C_BINOP_XOR = 7, C_BINOP_SHL = 8,
    C_BINOP_SHR = 9, C_BINOP_EQ = 10, C_BINOP_NE = 11,
    C_BINOP_LT = 12, C_BINOP_LE = 13, C_BINOP_GT = 14,
    C_BINOP_GE = 15
} T27CBinOp;

typedef enum {
    C_UNOP_NEG = 0, C_UNOP_NOT = 1, C_UNOP_DEREF = 2, C_UNOP_ADDR = 3
} T27CUnOp;

typedef struct {
    const char *trit_type;
    const char *word_type;
    const char *float_type;
} T27CMapping;

typedef struct {
    uint32_t indent_level;
} T27CCodeGen;

T27CCodeGen t27_c_codegen_new(void);
void t27_c_codegen_emit_header(T27CCodeGen *gen);
void t27_c_codegen_emit_includes(T27CCodeGen *gen);
void t27_c_codegen_indent(T27CCodeGen *gen);
void t27_c_codegen_dedent(T27CCodeGen *gen);

#ifdef __cplusplus
}
#endif

#endif /* T27_CODEGEN_C_CODEGEN_H */
