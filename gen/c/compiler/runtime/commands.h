/* Auto-generated from compiler/runtime/commands.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/runtime/commands.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef T27_RUNTIME_COMMANDS_H
#define T27_RUNTIME_COMMANDS_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    CMD_SPEC = 0, CMD_GEN = 1, CMD_COMPILE = 2,
    CMD_GIT = 3, CMD_LINT = 4, CMD_SKILL = 5, CMD_HELP = 6
} T27Command;

int t27_spec_create(const char *name, const char *kind);
int t27_spec_validate(const char *path);
int t27_spec_list(void);
int t27_lint(const char *path);
void t27_help(const char *topic);

#ifdef __cplusplus
}
#endif

#endif /* T27_RUNTIME_COMMANDS_H */
