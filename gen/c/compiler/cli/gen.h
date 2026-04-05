/* Auto-generated from compiler/cli/gen.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/cli/gen.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef T27_CLI_GEN_H
#define T27_CLI_GEN_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ========================================================================== */
/* Options                                                                     */
/* ========================================================================== */

typedef struct {
    const char *backend;
    const char *output_dir;
    bool        emit_tests;
    bool        emit_conformance;
    uint8_t     optimize_level;
    bool        no_prototype;
} T27GenOptions;

typedef struct {
    bool        emit_comments;
    bool        emit_debug;
    uint8_t     optimize_level;
    const char *target_triple;
    bool        include_runtime;
} T27CodegenOptions;

typedef struct {
    const char *backend;
    bool        emit_comments;
    bool        emit_benchmarks;
    const char *output_format;
} T27TestGenOptions;

/* ========================================================================== */
/* Commands                                                                    */
/* ========================================================================== */

int t27_gen(const char *spec_path, T27GenOptions options);
int t27_gen_all(T27GenOptions options);

/* ========================================================================== */
/* Helpers                                                                     */
/* ========================================================================== */

const char *t27_basename_without_ext(const char *path);

#ifdef __cplusplus
}
#endif

#endif /* T27_CLI_GEN_H */
