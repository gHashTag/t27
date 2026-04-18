# Constitutional Laws Reference

This document summarizes the constitutional laws that govern all t27 development.

## Core Documents

### SOUL.md
**Location:** `docs/nona-03-manifest/SOUL.md`

The supreme constitution of Trinity S³AI Framework. Contains:

- **De-Zig Strict**: Absolute prohibition on hand-editing generated Zig code
- **TDD-Inside-Spec**: Tests must live within spec files, not separate
- **PHI LOOP**: The canonical workflow (edit spec → seal hash → gen → test → verdict → save experience → skill commit → git commit)
- **Queen Trinity**: The spec files are the sovereign source of truth

### ADR-001: De-Zig-fication
**Location:** `architecture/ADR-001-de-zigfication.md`

Architecture Decision Record establishing:

- `.tri`/`.t27` files are the canonical source
- Generated backends (Zig, C, Verilog) are ephemeral
- All agents must check spec context before writing code
- Migration path from legacy Zig to spec-first

### ADR-003: TDD-Inside-Spec
**Location:** `architecture/ADR-003-tdd-inside-spec.md`

Defines TDD integration within spec files:

- Test cases embedded in `.t27` using `test { ... }` blocks
- Tests execute during `tri test` phase
- Conformance JSON stores expected results

### ADR-004: Language Policy
**Location:** `architecture/ADR-004-language-policy.md`

Article I of the Constitution:

- Official language: English for specs, Russian for policy
- Cyrillic validation for policy documents
- Language-specific validation rules

### ADR-005: De-Zig Strict
**Location:** `architecture/ADR-005-de-zig-strict.md`

Strengthens De-Zig-fication:

- Hand-editing generated code is forbidden
- All modifications must go through spec layer
- Violations cause toxic verdict

## Constitutional Hierarchy

1. **SOUL.md** — Supreme law
2. **Architecture ADRs** — Specific decisions
3. **Standards** — Implementation contracts
4. **Specs** — Source of truth
5. **Generated code** — Ephemeral output

## Enforcement

Violations of constitutional laws result in:
- **Toxic verdict** from `tri verdict --toxic`
- Automatic rollback
- Mistake recording in experience database
- No skill registration

## Quick Reference

| Law | Source | Key Command |
|------|---------|-------------|
| De-Zig Strict | SOUL.md + ADR-005 | Never edit .zig files |
| TDD-Inside-Spec | ADR-003 | Use `test {}` in .t27 |
| PHI LOOP | SOUL.md | Follow 5-step loop |
| Spec-First | ADR-001 | Check .tri before writing code |

## See Also

- `references/numeric-standards.md` — NUMERIC-STANDARD-001
- `references/sacred-physics.md` — SACRED-PHYSICS-001
- `references/graph-structure.md` — Canonical dependency graph
