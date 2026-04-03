# SOUL.md — Trinity Constitutional Laws

**Version**: 1.0
**Date**: 2026-04-04
**Status**: Sacred — Changes require consensus

> *SOUL = System of Universal Laws (Система Универсальных Законов)*

---

## Constitutional Law #1: No Cyrillic in Source Files

**Status**: MANDATORY (no exceptions)

### Statement

Source files (`.t27`, `.tri`, `.zig`, `.c`, `.v`, `.verilog`) **MUST NOT** contain Cyrillic characters (Unicode range U+0400–U+04FF).

Documentation files (`docs/*.md`, `AGENTS.md`, `README.md`, `*.md`) MAY contain Cyrillic.

### Rationale

1. **Code Consistency**: Source code must be universally readable without encoding issues.

2. **Tool Compatibility**: Many tools have issues with non-ASCII in source files.

3. **Clear Separation**: Code = ASCII-only, Docs = any language (including Cyrillic).

### Allowed Characters in Source Files

- **ASCII** (U+0000–U+007F): All printable ASCII characters
- **Latin-1 Supplement** (U+0080–U+00FF): For non-English identifiers (if needed)
- **Comments**: Must follow the same rule as code

### Forbidden in Source Files

- **Cyrillic** (U+0400–U+04FF): Русские буквы А-Я а-я Ё ё
- **Cyrillic Extended** (U+0500–U+052F)
- **Non-Latin scripts**: Greek, Arabic, Chinese, Japanese, Korean, etc.

### Exceptions

- **Documentation files** (`docs/`, `*.md`) can use any language including Cyrillic
- **String literals** in source files should also be ASCII-only for portability

### Enforcement

1. **Parser Validation**: The parser rejects source files containing Cyrillic with error:
   ```
   error: source file contains forbidden characters (Cyrillic U+0400–U+04FF)
   ```

2. **CLI Validation**: `tri lint` and `tri gen` fail on files with Cyrillic:
   ```
   $ tri gen specs/my_spec.t27
   error: spec contains Cyrillic characters - not allowed in source files
   ```

3. **Pre-commit Hook**: Git pre-commit hook checks for Cyrillic in staged source files

### Violation Example

```t27
; ❌ VIOLATION: Cyrillic in comment
; Это комментарий на русском

; ✅ CORRECT: ASCII-only comment
; This is a comment in English

; ❌ VIOLATION: Cyrillic in identifier
const КОЭФФИЦИЕНТ = 1.0

; ✅ CORRECT: ASCII identifier
const COEFFICIENT = 1.0
```

---

## Constitutional Law #2: TDD-Inside-Spec

**Status**: MANDATORY (no exceptions)

### Statement

Every specification in Trinity **MUST** include at least one `test` or `invariant` block. Specifications without tests are **INVALID** and **WILL NOT** be accepted by the compiler.

### Rationale

1. **Single Source of Truth**: The `.t27` spec file is the only source of truth. Conformance JSON vectors are **generated artifacts**, not hand-written.

2. **Test-First Development**: Tests define the contract. Implementation follows tests. Without tests, there is no contract.

3. **Architecture Bottleneck**: The #1 bottleneck in Trinity was the separation of specs and conformance vectors. This law eliminates that bottleneck.

### Enforcement

1. **Parser Level**: The parser (`compiler/parser/parser.t27`) rejects specs without tests with error:
   ```
   TDD contract violated: spec must contain at least one 'test' or 'invariant' block
   ```

2. **CLI Level**: `tri gen` fails with TDD violation if spec has no tests. No `--allow-no-tests` flag exists (prototype mode is disabled per policy).

3. **Commit Level**: `tri git commit` requires at least one test or invariant in the spec.

### Syntax

**Assembly-style TDD** (for `.t27` assembly specs):
```t27
.test
    ; my_test
    ; Verify: functionality works correctly
    ; Setup: initialize with given values
    ; Expected: returns correct result

.invariant
    ; my_invariant
    ; For all valid inputs: output is in valid range
    ; Rationale: ensures correctness
```

**Spec-style TDD** (for high-level specs):
```t27
spec my_spec
    test my_test
        given x = INPUT_VALUE
        when result = process(x)
        then result == EXPECTED_VALUE

    invariant my_invariant
        assert |PHI - 1.6180339887498948| < 1e-12
```

### Violations

The following are **VIOLATIONS** of TDD-Inside-Spec Law:

1. **Spec without tests**: A `.t27` file with only `.const`/`.data`/`.code` sections and NO `.test`/`.invariant` blocks.

2. **Empty test sections**: A `.test` section with no test cases.

3. **Conformance JSON as source**: Hand-editing `conformance/*.json` files. These MUST be generated via `tri gen --emit-conformance`.

### Penalties

1. **Compiler Error**: Specs without tests fail to compile.

2. **Git Block**: `tri git commit` and `tri git push` block if TDD contract is violated.

3. **CI Failure**: Any CI pipeline must reject specs without tests.

### Exceptions

**NONE**. There is no prototype mode, no `--allow-no-tests` flag. TDD is mandatory for all specs.

---

## Constitutional Law #2: Git Integration with Tri Skill

**Status**: MANDATORY for P0/P1 episodes

### Statement

Any P0/P1 episode in `--strict` mode is considered complete **ONLY** after successful `tri git push` to `github.com/gHashTag/t27` with:
- A bound sealed skill
- Non-toxic verdict
- Required artifacts per Policy Matrix

### Rationale

1. **Traceability**: Every change must be traceable to an issue (GitHub issue ID).

2. **Quality Gate**: The sealed skill and verdict mechanism prevents toxic changes from entering the codebase.

3. **Policy Enforcement**: Different skill types (recovery, hotfix) have different artifact requirements.

### Enforcement

1. **`tri git commit`**: Requires active or sealed skill with bound issue.
2. **`tri git push`**: Requires sealed skill with non-toxic verdict and proper artifacts.
3. **Strict Mode**: Only allows pushes to `github.com/gHashTag/t27`.

### Policy Matrix

| Skill Kind | Min Checkpoints | Required Artifacts | Verdict |
|------------|-----------------|---------------------|---------|
| Recovery | 3 | spec, docs, checkpoints | NOT TOXIC |
| Hotfix | 1 | checkpoint (fix-only areas) | NOT TOXIC |
| Feature | 1 | spec (with tests) | NOT TOXIC |
| Bugfix | 1 | spec (with tests) | NOT TOXIC |

### Workflow

```bash
# Start work
tri skill begin --issue N
# ... work on spec (with tests!) ...

# Seal skill
tri skill seal

# Commit (adds issue:N to message automatically)
tri git commit --all -m "description"

# Push (validates sealed skill + verdict + artifacts)
tri git push origin HEAD
```

### Violations

1. **Commit without skill**: `NO-COMMIT-WITHOUT-ISSUE violated` — run `tri skill begin --issue N` first.

2. **Commit without issue binding**: Skill must have `issue` field in registry.

3. **Push toxic skill**: Cannot push skills with `verdict = "TOXIC"` — fix or supersede.

4. **Push to wrong remote**: In strict mode, only `github.com/gHashTag/t27` allowed.

---

## Constitutional Law #3: De-Zigfication

**Status**: MANDATORY

### Statement

AI agents MUST see `.tri` context and write `.tri`/`.t27` files, never Zig directly.

### Rationale

1. **Spec-First Philosophy**: `.tri` and `.t27` files are the single source of truth for mathematical, numerical, and formal logic.

2. **Zig as Backend**: Zig code is a generated backend, not a language to author in.

3. **Migration Path**: Legacy Zig code is migrated to `.t27` specs, not the reverse.

### Enforcement

1. **Agent Training**: All agents are trained to check `.tri` context before writing code.

2. **Documentation**: `CANON_DE_ZIGFICATION.md` and `ADR-001-de-zigfication.md` define the migration path.

3. **Codegen**: Zig is only generated via `compiler/codegen/zig/codegen.t27`.

### Violations

1. **Writing Zig directly**: AI agents writing Zig without `.tri` spec source.

2. **Modifying Generated Zig**: Hand-editing generated Zig files (marked `DO NOT EDIT`).

3. **Skipping Spec**: Implementing features without corresponding `.t27` spec.

---

---

## Constitutional Law #4: De-Zig Strict

**Status**: MANDATORY (no exceptions)

### Statement

> **No new Trinity business logic in Zig by hand.**
>
> 1. **Source of Truth**: All new Trinity logic (CLI, runtime, numeric, physics, graph, agents) MUST be written only in `.t27/.tri` specifications.
> 2. **Backends Only**: Zig, C, Verilog, Rust may exist ONLY as **generated backends** from `.t27/.tri` via `tri gen`.
> 3. **Temporary Bootstrap**: Any new `.zig` file is permitted ONLY as temporary bootstrap layer (I/O, process startup). Domain logic in Zig is forbidden.
> 4. **Migration Debt**: Any existing handwritten Zig code with domain logic MUST have an explicit migration task to `.t27/.tri`. Creating new debt is forbidden.
> 5. **Enforcement**:
>    - `tri lint` fails if it detects new `.zig` files without `generated` marker
>    - `tri git push --strict` blocks push if there is diff in `src/` Zig files that did not pass validation

### Rationale

1. **Spec-First Philosophy**: `.tri` and `.t27` files are the single source of truth. Zig is a generated backend, not an authoring language.

2. **Multi-Target Code Generation**: Same spec generates Zig, C, Verilog, Rust. Writing Zig directly breaks this capability.

3. **AI Agent Alignment**: Agents must see `.tri` context and write `.tri` files, never Zig directly.

### Allowed Zig Files

Zig is ONLY permitted for:
1. **Generated backends** - From `.t27` specs with `DO NOT EDIT` header
2. **Bootstrap layer** - Temporary I/O and process startup (no domain logic)
3. **Legacy quarantine** - Existing code awaiting migration (with TODO comment)
4. **Hardware bridge** - FPGA bindings and external system interfaces

### Forbidden Zig Files

Writing Zig directly is FORBIDDEN for:
- CLI commands and routing
- Runtime domain logic
- Numeric/mathematical operations
- Sacred physics formulas
- Graph algorithms
- Agent orchestration
- Any Trinity-specific business logic

### Generated Header Requirement

All Zig files generated from `.t27` specs must have this header:

```zig
// This file is generated from <spec_path>
// DO NOT EDIT - Changes will be overwritten on next tri gen
// Generated at: <timestamp>
// Source spec: <spec_path>
```

Files without this header are considered handwritten and will be blocked.

### Correct Workflow

```bash
# CORRECT: Spec-first
1. Write spec in .t27
2. Run 'tri gen spec.t27'
3. Use generated Zig

# INCORRECT: Writing Zig directly
1. Write Zig code ← FORBIDDEN
2. No .t27 source ← FORBIDDEN
```

### Enforcement

1. **Linter**: `tri lint` fails on Zig files without generated header
2. **Git Push**: `tri git push --strict` blocks commits with handwritten Zig
3. **CI/CD**: GitHub Actions reject PRs with new handwritten Zig

### Violations

1. **Writing Zig directly**: Creating or modifying Zig without `.t27` source
2. **Modifying Generated Zig**: Hand-editing files marked `DO NOT EDIT`
3. **Skipping Spec**: Implementing features without corresponding `.t27` spec

### Penalties

1. **Linter Error**: Handwritten Zig detected, migration required
2. **Git Block**: Push blocked in strict mode
3. **CI Failure**: PR rejected in CI/CD pipeline

---

## Amendment Process

To amend SOUL.md:

1. Submit an ADR (Architecture Decision Record) proposing the change.
2. Must have consensus from agents A (Architecture), S (Standards), and T (Queen Trinity).
3. Update `SOUL.md` and create `docs/SOUL-v<new_version>.md` snapshot.

---

## Sacred Truths

These are the immutable truths of Trinity:

1. **φ² + 1/φ² = 3** — The golden ratio identity is the foundation of sacred physics.

2. **27 = 3³** — The trinity manifests as cube of trinity (27 agents, 27 registers, 27 letters).

3. **TDD Inside Spec** — Tests live inside specs, not as separate artifacts.

4. **No Spec Without Tests** — This is the law, not a guideline.

---

*"The law of Trinity is the law of φ: what is whole is found in the parts, and what is in the parts makes the whole."* — SOUL Law #0
