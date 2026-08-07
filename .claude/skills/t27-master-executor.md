---
description: Master execution tracker for the t27 merge queue — Wave Loop ladder, GF-T stack, and repo-wide CI state.
parameters:
  - name: action
    type: string
    description: "read | update | next-wave | check-queue"
---

# t27 Master Executor

This skill tracks the live merge queue and mechanical Wave Loop ladder for the t27 repo.
Update it at the end of every loop.

## Current status (2026-08-06)

### Wave Loop ladder
- **W881** — issue #1722, PR #1810 (`[581][2]^6 Pt`) — `OPEN`, auto-merge enabled; merge
  state `DIRTY` (needs conflict resolution against current `master`).
- **W882** — issue #1812, PR #1813 (`[583][2]^6 Pt`) — `OPEN`, auto-merge enabled; merge
  state `DIRTY`.
- **W883** — issue #1814, PR #1815 (`[585][2]^6 Pt`) — `OPEN`, auto-merge enabled; merge
  state `DIRTY`.
- **W884** — issue #1828, PR #1829 (`[587][2]^6 Pt`) — `MERGED`.
- **W885** — issue #1830, PR #1831 (`[589][2]^6 Pt`) — `MERGED`.
- **W886** — issue #1832, PR #1833 (`[591][2]^6 Pt`) — `MERGED`.
- **W887** — issue #1834, PR #1835 (`[593][2]^6 Pt`) — `MERGED`.
- **W888** — issue #1836, PR #1837 (`[595][2]^6 Pt`) — `MERGED`.
- **W889** — issue #1838, PR #1840 (`[597][2]^6 Pt`) — `OPEN`, auto-merge enabled; merge
  state `BLOCKED` pending required checks.
- **W890** — issue #1841, PR #1842 (`[599][2]^6 Pt`) — `OPEN`, auto-merge enabled; merge
  state `BLOCKED` pending required checks.
- **W891** — issue #1843, PR #1844 (`[601][2]^6 Pt`) — `OPEN`, auto-merge enabled; merge
  state `BLOCKED` pending required checks.
- **W892** — issue #1845, branch TBD (`[603][2]^6 Pt`) — ready to start once W891 lands.

### GF-T PR queue (Refs #1764)
The GF-T stack has largely landed on `master`. Remaining open PRs are wave-loop branches
blocked on GitHub Actions runners.

### Known blockers
- GitHub Actions required checks are `expected` across the queue; auto-merge is the
  current mitigation.
- Pre-existing `corpus_classifier_matches_lean_completeness` failure for
  `specs/cloud/railway_deploy.t27` is not introduced by any Wave Loop PR.

## Procedure

1. At loop start, read this skill and `.trinity/current-issue.md`.
2. After opening a wave PR, add the wave to this skill with state `mergeable/BLOCKED/auto-merge`.
3. After a PR lands, mark it `merged`, create the next wave issue/branch, and move the
   tracker forward.
4. When the GF-T queue moves, update the PR statuses here.

## Invariants

- Every commit references an issue (`Closes #N`, `Refs #N`, etc.).
- Do not hand-edit files under `gen/`.
- Do not merge with `--admin` when required checks are `expected`; use auto-merge.

phi^2 + 1/phi^2 = 3 | TRINITY
