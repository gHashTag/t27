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

## Constitutional Law #2: Git Integration with Tri Cell

**Status**: MANDATORY for P0/P1 episodes

### Statement

Any P0/P1 episode in `--strict` mode is considered complete **ONLY** after successful `tri git push` to `github.com/gHashTag/t27` with:
- A bound sealed cell
- Non-toxic verdict
- Required artifacts per Policy Matrix

### Rationale

1. **Traceability**: Every change must be traceable to an issue (GitHub issue ID).

2. **Quality Gate**: The sealed cell and verdict mechanism prevents toxic changes from entering the codebase.

3. **Policy Enforcement**: Different cell types (recovery, hotfix) have different artifact requirements.

### Enforcement

1. **`tri git commit`**: Requires active or sealed cell with bound issue.
2. **`tri git push`**: Requires sealed cell with non-toxic verdict and proper artifacts.
3. **Strict Mode**: Only allows pushes to `github.com/gHashTag/t27`.

### Policy Matrix

| Cell Kind | Min Checkpoints | Required Artifacts | Verdict |
|-----------|-----------------|---------------------|---------|
| Recovery | 3 | spec, docs, checkpoints | NOT TOXIC |
| Hotfix | 1 | checkpoint (fix-only areas) | NOT TOXIC |
| Feature | 1 | spec (with tests) | NOT TOXIC |
| Bugfix | 1 | spec (with tests) | NOT TOXIC |

### Workflow

```bash
# Start work
tri cell begin --issue N
# ... work on spec (with tests!) ...

# Seal cell
tri cell seal

# Commit (adds issue:N to message automatically)
tri git commit --all -m "description"

# Push (validates sealed cell + verdict + artifacts)
tri git push origin HEAD
```

### Violations

1. **Commit without cell**: `NO-COMMIT-WITHOUT-ISSUE violated` — run `tri cell begin --issue N` first.

2. **Commit without issue binding**: Cell must have `issue` field in registry.

3. **Push toxic cell`: Cannot push cells with `verdict = "TOXIC"` — fix or supersede.

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
