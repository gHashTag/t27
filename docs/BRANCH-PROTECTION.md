# Branch Protection Rules

What protects `master`, as read from the repository's settings on 2026-10-03.

Settings are not files, so nothing in the tree can notice when this page stops
being true. It did: until 2026-10-03 it listed five required workflows, of which
two were. Re-measure instead of trusting it:

```
tri gates required          # the ruleset's required contexts, against every claim in the tree
tri gates preview           # ask each required context locally, before you push
gh api repos/gHashTag/t27/rules/branches/master
```

## Where the rules live

- One ruleset, `t27-master-protection` (id 14813367), targets `refs/heads/master`
  with enforcement `active`. Its last edit is 2026-09-19 15:06 UTC.
- Classic branch protection is off: `gh api repos/gHashTag/t27/branches/master`
  reports `"protection": {"enabled": false}`, and the branch reads as protected
  because of the ruleset. `branches/master/protection` answers 404, which proves
  nothing either way: it answers 404 to anyone without admin rights.
- The repository belongs to a user account, so no organization ruleset applies.
- Bypass actors are visible only to admins and are not listed here.

## Rules

| Rule | Setting |
|------|---------|
| Restrict deletions | on |
| Block force pushes | on (`non_fast_forward`) |
| Require a pull request before merging | on |
| Required approving reviews | 0 |
| Dismiss stale approvals on push | on |
| Require review from code owners | off |
| Require approval of the most recent push | off |
| Require conversation resolution | off |
| Extra approval for unattributed changes | on |
| Allowed merge methods | merge, squash, rebase |
| Require status checks to pass | on -- the three contexts below |
| Require branches to be up to date | off (`strict_required_status_checks_policy: false`) |

Outside the ruleset, the repository allows auto-merge and does not delete head
branches on merge.

## Required status checks

GitHub matches a required check by CONTEXT -- the job's `name:`, or its id when
it has none -- not by workflow file or workflow name. Renaming one of these jobs
does not make it optional: the context stops posting, and every pull request
waits on it forever.

| Context | Workflow | What it asks |
|---------|----------|--------------|
| `validate` | `.github/workflows/schema-validation.yml` | every tracked JSON file parses (known exceptions ledgered in `tools/json_parse_baseline.txt`) |
| `check-linked-issue` | `.github/workflows/issue-gate.yml` | the pull request title, or its body outside code fences and `>` quotes, references an issue (L1 TRACEABILITY) |
| `parse-ratchet` | `.github/workflows/spec-parse-ratchet.yml` | no `.t27` spec that parsed at the base stops parsing |

`.github/required-contexts.txt` names the same three contexts. It is generated
from the ruleset by `tri gates required --write`, never edited by hand, and
`tri gates preview` reads it only when the ruleset cannot be read.
`tri gates required` and `tri gates preview` both report any difference between
the file and the ruleset.

History: on 2026-09-06 the required contexts were `check`, `check-now-freshness`,
`validate` and `check-linked-issue`. The ruleset's last edit, 2026-09-19 15:06 UTC,
came 21 seconds after #4277 merged the parse ratchet, and left the three above.

## Run on every pull request, and not required

These post on every pull request and their red is worth reading, but none of
them can block a merge. They are named here without their directory on
purpose: `tri gates required` reads every `.github/workflows/` path in this
file as a claim that the workflow is required.

| Context | Workflow file | What it asks |
|---------|---------------|--------------|
| `check` | `check-now-freshness.yml` | the `docs/now/` entry the pull request adds has a heading and real bullets; a `fix(` title in a compiler scope carries a source file |
| `check-now-freshness` | `now-sync-gate.yml` | the pull request adds a `docs/now/` entry dated within a day of today |
| `coverage` | `seal-coverage.yml` | every spec's seal matches it |
| `phi-loop-check` | `phi-loop-ci.yml` | the main test suite, L5 identity, L8 FPGA-safety |

The `docs/now/` entry is still this repository's rule (AGENTS.md); it is just
not one GitHub enforces.

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

Only a repository admin can change the ruleset. In an emergency an admin can
set its enforcement to `disabled` (or add a bypass actor), merge the critical
fix, restore the ruleset immediately, and file a follow-up issue for the root
cause. `current_user_can_bypass` in `gh api repos/gHashTag/t27/rulesets/14813367`
says whether the caller can bypass today.

---

## Related Policies

- **L1 TRACEABILITY**: All PRs must reference an issue (`Closes #N` or `Refs #N`)
- **L7 UNITY**: Use `tri` CLI instead of ad-hoc shell scripts on critical paths
- **Issue Gate**: Automated check via `.github/workflows/issue-gate.yml`
- **CODEOWNERS**: `.github/CODEOWNERS` routes review requests; the ruleset does not require code-owner approval

---

**φ² + 1/φ² = 3 | TRINITY**
