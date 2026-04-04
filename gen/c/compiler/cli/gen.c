/* Auto-generated from compiler/cli/gen.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/cli/gen.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "gen.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

/* ========================================================================== */
/* Command: tri gen <spec>                                                     */
/* ========================================================================== */

int t27_gen(const char *spec_path, T27GenOptions options) {
    (void)options;

    if (!spec_path || spec_path[0] == '\0') {
        fprintf(stderr, "ERROR: spec path required\n");
        return 1;
    }

    /* TDD CONTRACT ENFORCEMENT: No prototype mode */
    /* 1. Check spec exists */
    /* 2. Parse spec */
    /* 3. Count tests + invariants -- reject if zero */
    /* 4. Generate implementation code for backend */
    /* 5. Optionally generate test code */
    /* 6. Optionally generate conformance JSON */

    return 0;
}

int t27_gen_all(T27GenOptions options) {
    (void)options;
    /* Glob specs, call t27_gen for each */
    return 0;
}

/* ========================================================================== */
/* Helpers                                                                     */
/* ========================================================================== */

const char *t27_basename_without_ext(const char *path) {
    if (!path) return "";

    /* Find last separator */
    const char *last_sep = strrchr(path, '/');
    if (!last_sep) last_sep = strrchr(path, '\\');
    const char *start = last_sep ? last_sep + 1 : path;

    /* Find extension */
    const char *dot = strrchr(start, '.');

    /* Return basename without extension */
    static char buf[256];
    size_t len = dot ? (size_t)(dot - start) : strlen(start);
    if (len >= sizeof(buf)) len = sizeof(buf) - 1;
    memcpy(buf, start, len);
    buf[len] = '\0';

    return buf;
}
