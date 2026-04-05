/* Auto-generated from compiler/skill/registry.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/skill/registry.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef T27_SKILL_REGISTRY_H
#define T27_SKILL_REGISTRY_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define T27_REGISTRY_PATH ".trinity/skills/registry.json"

typedef enum {
    SKILL_ACTIVE = 0, SKILL_SEALED = 1, SKILL_PAUSED = 2,
    SKILL_BLOCKED = 3, SKILL_COMPLETED = 4
} T27SkillStatus;

typedef enum {
    KIND_FEATURE = 0, KIND_BUGFIX = 1, KIND_HOTFIX = 2,
    KIND_RECOVERY = 3, KIND_REFACTOR = 4
} T27SkillKind;

typedef enum {
    VERDICT_NOT_TOXIC = 0, VERDICT_TOXIC = 1
} T27SkillVerdict;

typedef struct {
    const char *title;
    const char *author;
    const char *priority;
} T27SkillMetadata;

typedef struct {
    const char      *id;
    T27SkillStatus   status;
    T27SkillKind     kind;
    const char      *issue;
    const char      *branch;
    const char      *created_at;
    const char      *updated_at;
    const char      *sealed_at;
    const char      *commit;
    bool             pushed;
    T27SkillVerdict  verdict;
    const char      *seal_hash;
    const char      *artifacts;
    T27SkillMetadata metadata;
} T27Skill;

typedef struct {
    const char  *version;
    T27Skill    *skills;
    size_t       skill_count;
} T27SkillRegistry;

#ifdef __cplusplus
}
#endif

#endif /* T27_SKILL_REGISTRY_H */
