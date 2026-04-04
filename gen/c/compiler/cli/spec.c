/* Auto-generated from compiler/cli/spec.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/cli/spec.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "spec.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <ctype.h>

int t27_spec_create(const char *name, const char *path) {
    (void)path;
    if (!name || !t27_is_valid_spec_name(name)) {
        fprintf(stderr, "ERROR: invalid spec name\n");
        return 1;
    }
    /* Create spec with TDD template */
    return 0;
}

int t27_spec_validate(const char *path) {
    (void)path;
    /* Parse spec, check TDD compliance */
    return 0;
}

int t27_spec_list(void) {
    /* Glob specs, print list */
    return 0;
}

bool t27_is_valid_spec_name(const char *name) {
    if (!name || name[0] == '\0') return false;

    /* Must start with lowercase letter */
    if (name[0] < 'a' || name[0] > 'z') return false;

    /* Only lowercase alphanumeric and underscores */
    for (size_t i = 1; name[i] != '\0'; i++) {
        char c = name[i];
        if ((c < 'a' || c > 'z') && (c < '0' || c > '9') && c != '_') {
            return false;
        }
    }

    return true;
}

const char *t27_generate_spec_template(const char *name) {
    (void)name;
    return "; spec template\n.test\n.invariant\n.bench\n";
}
