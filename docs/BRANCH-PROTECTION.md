# Branch Protection Rules

This document defines the branch protection settings for the `master` branch.

## Required Settings

Configure in **Settings → Branches → Add rule** → `master`:

### General

| Setting | Value | Reason |
|---------|-------|--------|
| **Require a pull request before merging** | ✓ | All changes go through PR review |
| **Require approvals** | 1 | At least one maintainer review |
| **Dismiss stale PR approvals** | ✓ | New commits require re-review |
| **Require review from CODEOWNERS** | ✓ | Ensures domain experts review |
| **Allow auto-merge** | ✗ | Manual merge control |
| **Require status checks to pass** | ✓ | CI must pass |
| **Require branches to be up to date** | ✓ | Avoid merge conflicts |

### Required Status Checks

Mark these workflows as **required** before merging:

| Workflow | File | Description |
|----------|------|-------------|
| **PHI Loop CI** | `.github/workflows/phi-loop-ci.yml` | Main test suite, L5 identity, L8 FPGA-safety |
| **Seal Coverage** | `.github/workflows/seal-coverage.yml` | All specs have valid seals |
| **Schema Validation** | `.github/workflows/schema-validation.yml` | JSON schema conformance |
| **Issue Gate** | `.github/workflows/issue-gate.yml` | L1 TRACEABILITY (Closes #N) |
| **NOW Sync Gate** | `.github/workflows/now-sync-gate.yml` | docs/NOW.md date freshness |

### Restrict Settings

| Setting | Value | Reason |
|---------|-------|--------|
| **Require signed commits** | ✗ (optional) | GPG signing not enforced yet |
| **Restrict who can push** | ✓ (maintainers) | Prevent direct pushes |
| **Allow force pushes** | ✗ | Prevent history rewrites |
| **Do not allow bypassing** | ✗ (optional) | Allow admin bypass for emergencies |
| **Require linear history** | ✓ | Prefer rebase/squash merges |

---

## Merge Methods

Recommended merge method for PRs: **Squash and merge**

This keeps `master` history clean with one commit per PR. The commit message should follow the format:

```
<type>(<scope>): <subject>

Closes #N

φ² + 1/φ² = 3 | TRINITY
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

---

## Emergency Bypass

In emergencies, maintainers with admin privileges can bypass branch protection:

1. Disable "Do not allow bypassing the above settings" temporarily
2. Merge critical fix directly
3. Re-enable protection immediately
4. File follow-up issue to address root cause

---

## Related Policies

- **L1 TRACEABILITY**: All PRs must reference an issue (`Closes #N`)
- **L7 UNITY**: Use `tri` CLI instead of ad-hoc shell scripts on critical paths
- **Issue Gate**: Automated check via `.github/workflows/issue-gate.yml`
- **CODEOWNERS**: `.github/CODEOWNERS` defines reviewer routing

---

**φ² + 1/φ² = 3 | TRINITY**
