/* Auto-generated from compiler/runtime/validation.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/runtime/validation.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "validation.h"
#include <string.h>

T27ValidationResult t27_validate_tdd_contract(bool has_tests, bool has_invariants) {
    T27ValidationResult r;
    if (!has_tests && !has_invariants) {
        r.valid = false;
        r.error_msg = "TDD contract violated: spec must contain at least one test or invariant";
        r.hint = "See: docs/TDD-CONTRACT.md";
    } else {
        r.valid = true;
        r.error_msg = "";
        r.hint = "";
    }
    return r;
}

T27ValidationResult t27_validate_language_policy(const char *content, size_t len, bool is_docs) {
    T27ValidationResult r = { .valid = true, .error_msg = "", .hint = "" };
    if (is_docs) return r;
    for (size_t i = 0; i < len; i++) {
        if ((unsigned char)content[i] > 127) {
            r.valid = false;
            r.error_msg = "Language policy violated: non-ASCII characters found";
            r.hint = "Source files must be ASCII-only";
            break;
        }
    }
    return r;
}

T27ValidationResult t27_validate_naming(const char *name) {
    T27ValidationResult r;
    if (!name || name[0] == '\0') {
        r.valid = false;
        r.error_msg = "Name cannot be empty";
        r.hint = "Use snake_case for identifiers";
    } else {
        r.valid = true;
        r.error_msg = "";
        r.hint = "";
    }
    return r;
}
