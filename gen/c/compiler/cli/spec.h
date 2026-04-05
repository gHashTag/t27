/* Auto-generated from compiler/cli/spec.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/cli/spec.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef T27_CLI_SPEC_H
#define T27_CLI_SPEC_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

int  t27_spec_create(const char *name, const char *path);
int  t27_spec_validate(const char *path);
int  t27_spec_list(void);
bool t27_is_valid_spec_name(const char *name);
const char *t27_generate_spec_template(const char *name);

#ifdef __cplusplus
}
#endif

#endif /* T27_CLI_SPEC_H */
