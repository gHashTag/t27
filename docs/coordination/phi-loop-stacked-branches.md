# PHI LOOP as GitButler Stacked Branches

## Overview

The PHI LOOP (t27's canonical development workflow) maps to GitButler's stacked branches feature. This document describes the branching pattern for systematic spec-first development.

## Branch Naming Convention

All branches follow the pattern:
```
ring-NNN-<phase>-<description>
```

Where:
- `ring-NNN`: Ring/issue number (e.g., ring-001, ring-072)
- `phase`: One of `issue-spec`, `tdd`, `impl`, `seal`
- `description`: Brief, ASCII-only description

## Stacked Branches Template

```
ring-NNN-issue-spec      ← Phase 1-2 (Issue, Spec)
  └── ring-NNN-tdd       ← Phase 3 (TDD)
        └── ring-NNN-impl  ← Phase 4-5 (Code, Gen)
              └── ring-NNN-seal  ← Phase 6-8 (Seal, Verify, Land)
```

### Phase Mapping

| Phase | Branch | Description |
|-------|--------|-------------|
| 1-2 | `ring-NNN-issue-spec` | Issue definition and spec creation |
| 3 | `ring-NNN-tdd` | Test-driven development |
| 4-5 | `ring-NNN-impl` | Implementation and code generation |
| 6-8 | `ring-NNN-seal` | Seal, verification, and landing |

## Creating a New PHI LOOP Stack

Using GitButler CLI:

```bash
# Start with issue/spec phase
but branch create ring-NNN-issue-spec

# After spec is complete, stack TDD branch
but branch create ring-NNN-tdd

# After TDD, stack implementation branch
but branch create ring-NNN-impl

# After implementation, stack seal branch
but branch create ring-NNN-seal
```

## Commit Message Format

All commits must follow the L1 TRACEABILITY invariant:

```
ring-NNN-<type>: <description>

Closes #N
```

Examples:
- `ring-072-feat: add GitHub SSOT integration\n\nCloses #72`
- `ring-001-fix: enforce L1 TRACEABILITY in pre-commit\n\nCloses #1`

## Landing the Stack

When all phases are complete:

1. Verify all tests pass: `./scripts/tri test`
2. Verify invariant laws: `./scripts/tri check`
3. Seal the final branch: `./scripts/tri seal`
4. Create PR with stack summary

## Invariant Law Enforcement

Each branch phase enforces specific laws:

- **L1 TRACEABILITY**: All commits must include `Closes #N`
- **L2 GENERATION**: No direct edits to `gen/` directory
- **L3 PURITY**: ASCII-only source files with English identifiers
- **L4 TESTABILITY**: Every `.t27` spec must contain `test`/`invariant`/`bench`
- **L5 IDENTITY**: φ² = φ + 1 checks with proper tolerance
- **L6 CEILING**: Numeric SSOT in `FORMAT-SPEC-001.json` and `gf16.t27`
- **L7 UNITY**: No new `*.sh` on critical path; use `tri`/`t27c`

## Branch Scatter Prevention

To reduce branch scatter (BSI < 0.3):

1. Delete completed branches after landing
2. Use GitButler's virtual branches for local experimentation
3. Merge duplicate branches before creating PRs
4. Clean up abandoned branches weekly

## Example: ring-072

```
ring-072-issue-spec
├── ring-072-issue-spec: create issue #72
├── ring-072-issue-spec: draft spec for GitHub SSOT
  └── ring-072-tdd
      ├── ring-072-tdd: add tests for GitHub API integration
      ├── ring-072-tdd: add tests for SSOT validation
        └── ring-072-impl
            ├── ring-072-impl: implement GitHub API client
            ├── ring-072-impl: implement SSOT validator
              └── ring-072-seal
                  ├── ring-072-seal: run test suite
                  ├── ring-072-seal: verify invariant laws
                  └── ring-072-seal: seal and prepare for PR
```

## Tools and Commands

```bash
# List all stacked branches
but branch list

# Show stack visualization
but branch tree

# Move to parent branch
but branch checkout <parent-name>

# Merge a completed phase
but branch integrate <branch-name>
```

## References

- [TASK_PROTOCOL.md](./TASK_PROTOCOL.md) — Task coordination rules
- [T27-CONSTITUTION.md](../T27-CONSTITUTION.md) — Invariant laws
- [GitButler Commit Skill](../../.claude/skills/gitbutler-commit.md) — Commit message conventions
