/* Auto-generated from compiler/cli/git.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/cli/git.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef T27_CLI_GIT_H
#define T27_CLI_GIT_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    const char *id;
    const char *status;
    const char *kind;
    const char *issue;
    const char *branch;
    const char *verdict;
    const char *seal_hash;
    const char *artifacts;
} T27Skill;

int t27_git_commit(bool all, const char *message, const char *mode);
int t27_git_push(const char *remote, const char *branch, const char *mode);
int t27_git_status_with_skill(void);
const char *t27_summarize_status(const char *status);
int t27_count_checkpoints(const char *artifacts);

#ifdef __cplusplus
}
#endif

#endif /* T27_CLI_GIT_H */
