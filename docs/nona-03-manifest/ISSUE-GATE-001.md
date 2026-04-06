# ISSUE-GATE-001: No Byte Without Issue

**Status:** ACTIVE
**Effective:** 2026-04-04

## Law

No byte enters `master` without:

1. An open GitHub Issue describing the work
2. A Pull Request linked to that issue via `Closes #N`
3. CI passing (issue-gate + phi-loop-ci)

## Enforcement

The `issue-gate.yml` workflow runs on every PR targeting `master`.
It queries GitHub GraphQL for `closingIssuesReferences` on the PR.
If no linked issues are found, the check fails and the PR cannot merge.

## Rationale

Every change to the Trinity codebase must be traceable to a declared intent.
This prevents drive-by commits, undocumented changes, and scope creep.
The issue is the contract; the PR is the delivery; the seal is the proof.

## Issue Templates

- **Seed Ring** (`seed-ring.yml`): New language capability rings
- **AR Task** (`ar-task.yml`): CLARA Argumentation & Reasoning tasks

Blank issues are disabled. All work flows through templates.

## Sacred Invariant

```
phi^2 + 1/phi^2 = 3 | TRINITY
```
