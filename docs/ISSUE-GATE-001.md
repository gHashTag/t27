# ISSUE-GATE-001: L1 TRACEABILITY Enforcement

**Version:** 1.0
**Status:** ENFORCED
**Date:** 2026-04-06
**Related:** L1 TRACEABILITY (T27-CONSTITUTION.md), Issue #128

---

## Purpose

Enforces **Law 1 (L1) TRACEABILITY**: Every commit must reference a GitHub Issue via `Closes #N`. This ensures all code changes are traceable to tracked work units.

## Enforcement Mechanism

The `issue-gate.yml` workflow runs on every pull request to `master` and **blocks merge** if no linked issue is found.

### Workflow

File: `.github/workflows/issue-gate.yml`

```yaml
check-linked-issue:
  runs-on: ubuntu-latest
  steps:
    - name: Check for linked issues via GraphQL
      env:
        GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      run: |
        ISSUES=$(gh api graphql ... --jq ...)

        if [ -z "$ISSUES" ]; then
          echo "::error::TOXIC: PR must reference an issue with 'Closes #N'"
          exit 1
        fi
```

## Acceptable Keywords

The following keywords in PR body or title link an issue:

| Keyword | Effect |
|---------|--------|
| `Closes #N` | Closes issue on merge |
| `Fixes #N` | Closes issue on merge |
| `Resolves #N` | Closes issue on merge |

## Blocking Behavior

- **With linked issue:** ✅ CI passes, PR can merge
- **Without linked issue:** ❌ CI fails, PR **cannot** merge

**Error message:**
```
TOXIC: PR must reference an issue with 'Closes #N', 'Fixes #N', or 'Resolves #N' (L1 TRACEABILITY)
See issue #128 for ISSUE-GATE requirements
```

## Exceptions

**None.** All code changes must be traceable. If you need to make a fix without an issue, create one first using the issue template.

## Related Laws

| Law | Name | Relation |
|-----|------|----------|
| L1 | TRACEABILITY | This gate enforces L1 directly |
| L4 | TESTABILITY | Issues should contain acceptance criteria |

## See Also

- [T27-CONSTITUTION.md](T27-CONSTITUTION.md) — L1–L7 laws
- [NOW.md](docs/NOW.md) — Current rings and issues
- Issue [#128](https://github.com/gHashTag/t27/issues/128) — Ring 033: ISSUE-GATE CI enforcement
