# Conflict Resolution Strategy for NOW.md

**Ring:** 057 | **Issue:** #209 | **Status:** ACTIVE

## Problem

Multiple PRs modifying `docs/NOW.md` create merge conflicts. This is expected behavior as NOW.md is the single rolling snapshot.

## Root Cause

All Rings update NOW.md with their progress, creating parallel changes when PRs are open simultaneously.

## Solution: Sequential Merge Strategy

### Merge Order

PRs must be merged in this order to minimize conflicts:

```
1. #198  Ring 051: VERDICT_SCHEMA
2. #200  Ring 052: Brain seals
3. #202  Ring 053: Property-test template
4. #206  Ring 055: EXPERIENCE_SCHEMA
5. #208  Ring 056: Schema validation CI
```

*Note: PR #204 (Ring 054: META_DASHBOARD) already merged to master.*

### Conflict Resolution Procedure

When a conflict is detected in NOW.md:

1. **Checkout and update:**
   ```bash
   git checkout <feature-branch>
   git pull origin master
   ```

2. **Resolve NOW.md conflict:**
   - Keep the **newest** "Last updated" timestamp
   - Merge revision text: append new rings to existing list
   - Keep all badges and headers

3. **Test the resolution:**
   ```bash
   ./scripts/tri check-now
   ```

4. **Commit and push:**
   ```bash
   git add docs/NOW.md
   git commit -m "resolve: NOW.md conflict with latest master"
   git push origin <feature-branch>
   ```

### Automated Conflict Detection

Use the following to detect conflicts before merging:

```bash
#!/bin/bash
# check-conflicts.sh

BRANCHES=(
  "feat/ring-051-verdict-schema"
  "feat/ring-052-lotus-automation"
  "feat/ring-053-property-test"
  "feat/ring-055-experience-schema"
  "feat/ring-056-schema-validation-ci"
)

for branch in "${BRANCHES[@]}"; do
  git merge-base --is-ancestor master "origin/$branch"
  if [ $? -ne 0 ]; then
    echo "$branch: NOT merged to master"
  else
    echo "$branch: already merged"
  fi
done
```

## Prevention: Alternative Approaches

### Option A: Separate NOW.md per Ring (REJECTED)

- Creates multiple sources of truth
- Violates SSOT principle
- Abandoned

### Option B: NOW.md Updates Only on Merge (ADOPTED)

- Branches don't update NOW.md directly
- Update happens in PR merge commit or immediately after
- Reduces conflicts but delays handoff
- Requires careful merge process

### Option C: NOW.md as Append-Only Log (FUTURE)

- Structure as chronological log instead of status snapshot
- New entries appended, existing entries never modified
- Eliminates conflicts completely
- Requires refactoring NOW.md structure

## Current Recommendation

Use **Sequential Merge Strategy** with **Option B** (update on merge):

1. Author PR without updating NOW.md
2. Maintainer merges PR
3. Maintainer updates NOW.md in follow-up commit or as part of merge
4. Conflicts resolved by maintainer

## Next Actions

1. Merge PRs in order: #198 → #200 → #202 → #206 → #208
2. Update NOW.md after each merge
3. Consider Option C for future Rings (Ring 060+)

## Refs

- NOW.md §1.1 (Agent handoff)
- NOW.md §9 (Error patterns)
- T27-CONSTITUTION.md L1 TRACEABILITY

---

**Last updated:** 2026-04-07

φ² + 1/φ² = 3 | TRINITY
