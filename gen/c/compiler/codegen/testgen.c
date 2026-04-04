/* Auto-generated from compiler/codegen/testgen.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/codegen/testgen.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "testgen.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

/* ========================================================================== */
/* Test Generation                                                             */
/* ========================================================================== */

const char *t27_testgen_generate(T27TestGenProgram *prog, T27TestGenOptions opts) {
    if (!prog) return "// No program";

    if (strcmp(opts.backend, "zig") == 0) {
        return t27_testgen_generate_zig(prog);
    } else if (strcmp(opts.backend, "c") == 0) {
        return t27_testgen_generate_c(prog);
    } else if (strcmp(opts.backend, "verilog") == 0) {
        return t27_testgen_generate_verilog(prog);
    } else if (strcmp(opts.backend, "json") == 0) {
        return t27_testgen_generate_json(prog);
    }
    return "// Unsupported backend";
}

const char *t27_testgen_generate_zig(T27TestGenProgram *prog) {
    (void)prog;
    return "// Zig tests generated from spec\n"
           "const std = @import(\"std\");\n";
}

const char *t27_testgen_generate_c(T27TestGenProgram *prog) {
    (void)prog;
    return "// C tests generated from spec\n"
           "#include <assert.h>\n";
}

const char *t27_testgen_generate_verilog(T27TestGenProgram *prog) {
    (void)prog;
    return "`timescale 1ns/1ps\n"
           "module tb_spec();\n"
           "endmodule\n";
}

const char *t27_testgen_generate_json(T27TestGenProgram *prog) {
    (void)prog;
    return "{\"spec\": \"\", \"generated_at\": \"\", \"test_vectors\": []}\n";
}
