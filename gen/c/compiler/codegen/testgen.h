/* Auto-generated from compiler/codegen/testgen.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/codegen/testgen.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef T27_CODEGEN_TESTGEN_H
#define T27_CODEGEN_TESTGEN_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    const char *backend;
    bool        emit_comments;
    bool        emit_benchmarks;
    const char *output_format;
} T27TestGenOptions;

typedef struct {
    const char *name;
    const char *verify_description;
    const char *expected_outcome;
} T27TestGenCase;

typedef struct {
    const char *name;
    const char *formal_statement;
    const char *rationale;
} T27TestGenInvariant;

typedef struct {
    const char *source_file;
    size_t      test_count;
    size_t      invariant_count;
} T27TestGenProgram;

const char *t27_testgen_generate(T27TestGenProgram *prog, T27TestGenOptions opts);
const char *t27_testgen_generate_zig(T27TestGenProgram *prog);
const char *t27_testgen_generate_c(T27TestGenProgram *prog);
const char *t27_testgen_generate_verilog(T27TestGenProgram *prog);
const char *t27_testgen_generate_json(T27TestGenProgram *prog);

#ifdef __cplusplus
}
#endif

#endif /* T27_CODEGEN_TESTGEN_H */
