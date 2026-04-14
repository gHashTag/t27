---
description: Execute tri pipeline commands for spec-first development
parameters:
  - name: command
    type: string
    description: tri command (gen, test, verify, seal)
  - name: spec
    type: string
    description: Path to .t27 spec file
---

# TRI Pipeline Skill

The tri pipeline is the primary tool for spec-first development in t27.

## Commands

### `tri gen <spec>`
Generate code from .t27 specification.
- Outputs to `gen/` directory
- Never hand-edit generated files
- Modify spec to change behavior

### `tri test`
Run conformance tests.
- Executes all .t27 specs
- Validates invariants and test cases
- Returns TAP format results

### `tri verify <spec>`
Verify a single specification.
- Checks test/invariant/bench sections
- Validates generated code matches spec

### `tri seal <spec>`
Seal specification hash.
- Creates cryptographic seal for traceability
- Required before merge

## Usage Flow

1. Write .t27 spec with test/invariant/bench
2. `tri gen <spec>` - generate code
3. `tri test` - run all tests
4. `tri seal <spec>` - seal hash
5. Create PR with `Closes #N` reference

## Important

- L2 (GENERATION): Never edit files under `gen/` directly
- L4 (TESTABILITY): Every spec must have test/invariant/bench
- Use L7 (UNITY): Prefer tri over shell scripts
