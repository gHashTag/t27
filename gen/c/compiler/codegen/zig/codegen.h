/* Auto-generated from compiler/codegen/zig/codegen.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/codegen/zig/codegen.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef T27_CODEGEN_ZIG_H
#define T27_CODEGEN_ZIG_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    bool        emit_comments;
    bool        emit_debug;
    uint8_t     optimize_level;
    const char *target_triple;
    bool        include_runtime;
} T27ZigCodegenOptions;

typedef struct {
    uint32_t indent_level;
    T27ZigCodegenOptions options;
} T27ZigCodegen;

T27ZigCodegen t27_zig_codegen_new(T27ZigCodegenOptions opts);
void t27_zig_codegen_indent(T27ZigCodegen *gen);
void t27_zig_codegen_dedent(T27ZigCodegen *gen);

#ifdef __cplusplus
}
#endif

#endif /* T27_CODEGEN_ZIG_H */
