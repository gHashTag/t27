/* Auto-generated from compiler/codegen/zig/codegen.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/codegen/zig/codegen.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "codegen.h"

T27ZigCodegen t27_zig_codegen_new(T27ZigCodegenOptions opts) {
    T27ZigCodegen gen;
    gen.indent_level = 0;
    gen.options = opts;
    return gen;
}

void t27_zig_codegen_indent(T27ZigCodegen *gen) {
    if (gen) gen->indent_level++;
}

void t27_zig_codegen_dedent(T27ZigCodegen *gen) {
    if (gen && gen->indent_level > 0) gen->indent_level--;
}
