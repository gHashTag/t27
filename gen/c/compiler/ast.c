/* Auto-generated from compiler/ast.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/ast.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "ast.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <assert.h>

/* ========================================================================== */
/* AST Node Creation                                                           */
/* ========================================================================== */

T27ASTNode t27_ast_node_new(T27NodeType type, uint32_t line, uint32_t col,
                             const char *source_file) {
    T27ASTNode node;
    node.node_type   = type;
    node.line        = line;
    node.column      = col;
    node.source_file = source_file;
    return node;
}

/* ========================================================================== */
/* Program Creation / Destruction                                              */
/* ========================================================================== */

T27Program *t27_program_new(const char *source_file) {
    T27Program *prog = (T27Program *)calloc(1, sizeof(T27Program));
    if (!prog) return NULL;
    prog->node = t27_ast_node_new(NODE_PROGRAM, 1, 1, source_file);
    return prog;
}

void t27_program_free(T27Program *prog) {
    if (!prog) return;
    free(prog);
}

/* ========================================================================== */
/* ConstDef                                                                    */
/* ========================================================================== */

T27ConstDef t27_const_def_new(const char *name, int64_t value,
                               uint32_t line, uint32_t col,
                               const char *source_file) {
    T27ConstDef def;
    def.node  = t27_ast_node_new(NODE_CONST_DEF, line, col, source_file);
    def.name  = name;
    def.value = value;
    return def;
}

/* ========================================================================== */
/* DataDecl                                                                    */
/* ========================================================================== */

T27DataDecl t27_data_decl_new(uint8_t size, int64_t initial_value,
                               const char *label,
                               uint32_t line, uint32_t col,
                               const char *source_file) {
    T27DataDecl decl;
    decl.node          = t27_ast_node_new(NODE_DWORD, line, col, source_file);
    decl.size          = size;
    decl.initial_value = initial_value;
    decl.label         = label;
    return decl;
}

/* ========================================================================== */
/* Instruction                                                                 */
/* ========================================================================== */

T27Instruction t27_instruction_new(T27Opcode opcode,
                                    uint32_t line, uint32_t col,
                                    const char *source_file) {
    T27Instruction instr;
    instr.node          = t27_ast_node_new((T27NodeType)(NODE_MOV + (int)opcode),
                                           line, col, source_file);
    instr.opcode        = opcode;
    instr.operands      = NULL;
    instr.operand_count = 0;
    return instr;
}

/* ========================================================================== */
/* Compiler Error                                                              */
/* ========================================================================== */

T27CompilerError t27_error_new(const char *message, uint32_t line,
                                uint32_t col, const char *source_file,
                                T27CompilationPhase phase) {
    T27CompilerError err;
    err.message     = message;
    err.line        = line;
    err.column      = col;
    err.source_file = source_file;
    err.phase       = phase;
    return err;
}

/* ========================================================================== */
/* Utility                                                                     */
/* ========================================================================== */

const char *t27_node_type_name(T27NodeType type) {
    switch (type) {
        case NODE_PROGRAM:       return "Program";
        case NODE_DATA_SECTION:  return "DataSection";
        case NODE_CODE_SECTION:  return "CodeSection";
        case NODE_CONST_DEF:     return "ConstDef";
        case NODE_DWORD:         return "DWord";
        case NODE_DSPACE:        return "DSpace";
        case NODE_DTRIT:         return "DTrit";
        case NODE_MOV:           return "Mov";
        case NODE_JZ:            return "Jz";
        case NODE_JNZ:           return "Jnz";
        case NODE_JMP:           return "Jmp";
        case NODE_HALT:          return "Halt";
        case NODE_TEST:          return "Test";
        case NODE_INVARIANT:     return "Invariant";
        case NODE_BENCH:         return "Bench";
        case NODE_SPEC_DECL:     return "SpecDecl";
        default:                 return "Unknown";
    }
}

const char *t27_opcode_name(T27Opcode op) {
    switch (op) {
        case OP_MOV:    return "MOV";
        case OP_JZ:     return "JZ";
        case OP_JNZ:    return "JNZ";
        case OP_JMP:    return "JMP";
        case OP_MUL:    return "MUL";
        case OP_ADD:    return "ADD";
        case OP_SUB:    return "SUB";
        case OP_BIND:   return "BIND";
        case OP_BUNDLE: return "BUNDLE";
        case OP_HALT:   return "HALT";
        default:        return "UNKNOWN";
    }
}
