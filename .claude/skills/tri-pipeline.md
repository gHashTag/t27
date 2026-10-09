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

<!-- only-t27-rule -->
> **Only t27: everything is written in `.t27`.** (Owner hard rule 2026-10-05; budget #7371; restated in every skill per #7588.)
> - New logic starts as a `.t27` spec with `test` / `invariant` blocks. It reaches Rust, C, Zig, JS or Python only through `t27c gen-*`, and generated files are never hand-edited (L2).
> - Hand-written code in any other language is denied by the hooks and by CI: `.rs .py .ts .js .sh .zig .c .go .v .yml .toml`, Dockerfile, Makefile; the full list is in `specs/policy/own_language.t27`.
> - Existing foreign code is debt that only shrinks. Delete it, or replace it with a spec and its generated output.
> - Thin hand-written glue is capped at 40 added lines per file and 80 per PR. No label lifts the cap. Never split a PR to slip under it.
> - What gen cannot express yet is a compiler defect. File it on #5980 with a minimal `.t27` repro instead of writing the code by hand.
> - This rule overrides any step below that says to write a script or code in another language. The full text is in `AGENTS.md`, "Only t27".

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
