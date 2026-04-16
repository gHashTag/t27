# ADR-005: De-Zig Strict

**Status**: Accepted
**Date**: 2026-04-16
**Decision Level**: Constitutional (SOUL Law #4)
**Context**: T27 Self-Compilation Chain Architecture

---

## Context

Trinity follows self-compilation philosophy where `.t27` is the single source of truth. The compilation chain is:

```
.t27 (human source) → [t27c compiler] → .tri (canonical IR) → [tri runtime] → execution
```

**Critical invariant:** `.t27` compiles **itself via .tri** — there is NO "dogfooding" or self-feeding. We do NOT write `.tri` by hand, and we do NOT compile `.t27` directly in Rust/Zig/etc.

### The Language Food Chain

| Language | Status | Role |
|----------|--------|------|
| `.t27` | ✅ write by hand | Single source of truth |
| `.tri` | ⚙️ generated | Canonical IR, runtime target |
| Rust | 🔒 frozen | Only bootstrap toolchain `t27c`, never edited |
| Zig | ❌ consumed | Generate from `.tri` if needed, never write by hand |
| Python | ❌ consumed | Forbidden by L4 (except legacy) |
| C, JS, others | ❌ consumed | Generate from `.tri` if needed |

**Golden rule:** DO NOT dogfood. `.t27` eats `.tri`. We don't feed `.t27` its own output.

---

## Decision

### SOUL Law #4: Language Food Chain

> **Write ONLY .t27 files. t27c compiles .t27 → .tri automatically. .tri is NOT edited by hand — it is output. If Zig/C/other target needed — generate from .tri, don't write by hand.**

> 1. **Single source of truth**: All Trinity logic lives in `.t27` specs only.
> 2. **Compiler chain**: `t27c` compiles `.t27` → `.tri` automatically. `.tri` is canonical IR.
> 3. **No manual .tri editing**: `.tri` files are generated artifacts — never edited by hand.
> 4. **Frozen Rust**: Rust code is ONLY the `t27c` bootstrap compiler. Never edit directly.
> 5. **Consumed languages**: Zig, Python, C, JS, etc. are generated from `.tri` via `tri gen`. Never write by hand.
> 6. **No self-feeding**: `.t27` compiles itself via `.tri`. No dogfooding.

### Allowed Zig Files

Zig files are ONLY permitted in these cases:

1. **Generated backends** - Files with the mandatory header:
   ```zig
   // This file is generated from <spec_path>
   // DO NOT EDIT - Changes will be overwritten on next tri gen
   // Generated at: <timestamp>
   // Source spec: <spec_path>
   ```

2. **Bootstrap layer** - Temporary I/O and process startup code:
   - `src/bootstrap/*.zig` - OS integration, file I/O basics
   - No domain logic (no Trinity-specific algorithms, math, physics, etc.)

3. **Legacy quarantine** - Existing Zig being migrated:
   - `contrib/backend/zig/legacy/*.zig` - Handwritten code awaiting migration
   - Each file must have `TODO: migrate to .t27 spec` comment

4. **Hardware bridge** - FPGA bindings and external system interfaces:
   - `contrib/backend/bridges/*.zig` - Foreign function interfaces only (if present)

### Forbidden Zig Files

Writing Zig directly is FORBIDDEN for:
- CLI commands and routing
- Runtime domain logic (beyond basic I/O)
- Numeric/mathematical operations
- Sacred physics formulas
- Graph algorithms
- Agent orchestration
- Test frameworks
- Any Trinity-specific business logic

### Correct Workflow

```bash
# CORRECT: Spec-first
1. Write spec in .t27
   $ vim specs/numeric/gf16.t27
   # (include test blocks!)

2. Generate backend
   $ tri gen specs/numeric/gf16.t27
   # Generates: src/numeric/gf16.zig with DO NOT EDIT header

3. Use generated code
   $ zig build test

# INCORRECT: Writing Zig directly
$ vim src/numeric/gf16.zig  # FORBIDDEN
```

---

## Enforcement Mechanisms

### 1. Linter Validation

`tri lint` enforces:
```bash
$ tri lint src/numeric/gf16.zig
error: Zig file lacks generated header
  Expected: "// This file is generated from..."
  File: src/numeric/gf16.zig
  Hint: Write spec in .t27 first, then run tri gen

$ tri lint src/bootstrap/main.zig
ok: bootstrap file (no domain logic detected)

$ tri lint contrib/backend/zig/legacy/old_code.zig
warning: legacy file detected
  Status: awaiting migration to .t27
  Hint: Create migration task for this file
```

### 2. Git Push Strict Mode

`tri git push --strict` blocks:
```bash
$ tri git push --strict
error: strict mode violation
  Modified Zig files without generated header:
  - src/cli/commands.zig (handwritten)
  - src/runtime/executor.zig (handwritten)

  Action required:
  1. Write .t27 specs for this logic
  2. Run 'tri gen' to generate Zig
  3. Commit generated files (with DO NOT EDIT header)
```

### 3. CI/CD Gate

GitHub Actions reject:
- PRs with new handwritten Zig in `src/` (outside `bootstrap/`)
- PRs modifying generated Zig without corresponding `.t27` changes
- PRs with legacy Zig lacking migration task

---

## Consequences

### Positive

1. **Single source of truth** - All logic lives in `.t27` specs
2. **Multi-target codegen** - Same spec generates Zig, C, Verilog
3. **AI agent alignment** - Agents always check `.tri` context first
4. **Clear boundaries** - No ambiguity about where logic belongs

### Negative

1. **Initial migration cost** - Existing handwritten Zig needs migration
2. **Bootstrap complexity** - Temporary layer adds indirection
3. **Toolchain dependency** - Requires functional `tri gen` for all work

### Migration

All existing handwritten Zig with domain logic must be migrated:
1. Create `.t27` spec describing the logic
2. Add test blocks to spec (TDD-Inside-Spec)
3. Run `tri gen` to produce Zig
4. Delete handwritten Zig (move to `legacy/` if needed for reference)
5. Update imports and build system

---

## Alternatives Considered

### Alternative 1: Allow Zig in `src/` with warnings
- **Rejected**: Too easy to ignore warnings, defeats the purpose

### Alternative 2: Separate repo for handwritten Zig
- **Rejected**: Fragmentation, makes single-source-of-truth unclear

### Alternative 3: Runtime-only exception for CLI
- **Rejected**: CLI has significant domain logic (validation, routing)

---

## Status

- [x] ADR accepted
- [x] SOUL Law #4 added to SOUL.md
- [ ] Linter implementation
- [ ] Git push --strict implementation
- [ ] CI/CD gate implementation
- [ ] Legacy Zig audit and migration plan

---

## References

- ADR-001: De-Zigfication (original high-level principle)
- SOUL.md: Constitutional Laws
- docs/nona-03-manifest/GENERATED-HEADER-POLICY.md: Header specification
- compiler/runtime/runtime.t27: CLI runtime specification (source of truth)
