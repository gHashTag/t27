/* Auto-generated from compiler/runtime/validation.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/runtime/validation.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef T27_RUNTIME_VALIDATION_H
#define T27_RUNTIME_VALIDATION_H

#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    bool        valid;
    const char *error_msg;
    const char *hint;
} T27ValidationResult;

T27ValidationResult t27_validate_tdd_contract(bool has_tests, bool has_invariants);
T27ValidationResult t27_validate_language_policy(const char *content, size_t len, bool is_docs);
T27ValidationResult t27_validate_naming(const char *name);

#ifdef __cplusplus
}
#endif

#endif /* T27_RUNTIME_VALIDATION_H */
