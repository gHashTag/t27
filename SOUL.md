# SOUL — T27 Constitutional Law

**Canonical location:** this file at the **repository root** is the **single source of truth** for Trinity constitutional law.

> *"A specification without tests is a lie told in the future tense."*
> — Trinity TDD Axiom #1

---

## Preamble

T27 is a **spec-first** architecture where `.t27` compiles **itself via `.tri`**. The compilation chain is:

```
.t27 (human source) → [t27c compiler] → .tri (canonical IR) → [tri runtime] → execution
```

**Critical invariant:** `.t27` compiles itself via `.tri`. There is NO "dogfooding" or self-feeding. We do NOT write `.tri` by hand, and we do NOT compile `.t27` directly in other languages.

---

## Article I: The Language Food Chain

### §1.1. Single Source of Truth

**`.t27` is the ONLY source of truth.** All Trinity logic lives in `.t27` specifications.

Everything else is an **output artifact** generated from `.t27`:

| Artifact | Status | Generation method |
|---------|--------|------------------|
| `.tri` | ⚙️ Generated | Compiled from `.t27` by `t27c` |
| Zig | ❌ Consumed | Generated from `.tri` by `tri gen-zig` |
| C | ❌ Consumed | Generated from `.tri` by `tri gen-c` |
| Verilog | ❌ Consumed | Generated from `.tri` by `tri gen-verilog` |
| Rust | 🔒 Frozen | Only `t27c` bootstrap compiler — never edited |

**Never write `.tri`, Zig, C, Verilog by hand.** Write only `.t27` specs.

### §1.2. The Golden Rule

> **`.t27` eats `.tri` — it does NOT feed itself its own output.**

This is the **no-self-feeding invariant**:
- `.t27` compiles to `.tri` via `t27c`
- `.tri` is consumed by `tri runtime`
- The cycle is: `.t27` → `.tri` → execution, NOT `.t27` → `.t27`

**Violations:**
- Writing `.tri` by hand — FORBIDDEN
- Writing Zig directly for Trinity logic — FORBIDDEN
- Writing Python/C/Verilog for Trinity logic — FORBIDDEN
- Editing `t27c` bootstrap compiler — FORBIDDEN

### §1.3. Frozen Toolchain

Rust code is **frozen**:
- `bootstrap/src/compiler.rs` is the ONLY editable Rust file
- All other Rust is bootstrap infrastructure
- Purpose: Compiles `.t27` → `.tri`
- NEVER add domain logic to Rust directly

### §1.4. ASCII-Only Source Files

All source files MUST be ASCII-only. Identifiers and comments MUST be English.

**FORBIDDEN in source files:**
- **Cyrillic** (U+0400–U+04FF) and other non-Latin scripts in identifiers and comments
- **Non-Latin scripts**: Greek, Arabic, Chinese, Japanese, Korean, etc.

### §1.5. Rationale

1. **Universality**: ASCII is universally supported across all platforms and tools
2. **Clarity**: English is single review language
3. **Self-compilation**: `.t27` compiles itself, eliminating dogfooding
4. **Clear boundaries**: Single source of truth (`specs/*.t27`)

---

## Article II: The TDD Mandate

### §2.1. The Iron Law

Every `.t27` specification MUST contain at least one of:
- A `.test` section with one or more test cases
- An `.invariant` section with one or more invariant declarations
- A `.bench` section with one or more benchmark declarations

**No exceptions.** A spec without tests is not a specification—it is a draft.

### §2.2. Test Format (Assembly-Style)
```t27
.test
    ; test_bind_unbind_identity
    ; Verify: bind and unbind are inverses for all trits
    ; Setup: create vector a, bind with b, unbind with b
    ; Expected: result == a

.invariant
    ; no_trit_overflow
    ; All trit operations stay within {-1, 0, +1}
    ; Rationale: Balanced ternary requires strict bounds

    ; similarity_non_negative
    ; For all vectors a, b: similarity(a, b) >= 0
    ; Rationale: Similarity is a distance metric variant

.bench
    ; bench_bind_latency_cycles
    ; Measure: cycles for single bind operation
    ; Target: < 50 cycles on typical hardware
```

### §2.3. Test Format (Spec-Style)
```t27
spec vsa_ops {
    test bind_unbind_identity {
        given a = vector_random(27)
        given b = vector_random(27)
        when bound = bind(a, b)
        when result = unbind(bound, b)
        then result == a
    }

    invariant no_trit_overflow {
        assert forall v: vector, all_trits_in_range(v, -1, 1)
    }

    rule similarity_bounds {
        expect forall a, b: 0 <= similarity(a, b) <= 1
    }
}
```

---

## Article III: No Prototype Mode

### §3.1. The Ban on Temporary Code

T27 **does not have** a prototype mode. There is no `--allow-no-tests` flag. There is no "I'll add tests later" grace period.

If you write a spec without tests, parser **will reject it** with:

```
TDD contract violated: spec must contain at least one 'test' or 'invariant' block
```

### §3.2. The Rationale

Tests written after implementation are not tests—they are retroactive justification. True TDD requires test to exist **before** implementation, serving as:
1. A contract between specifier and implementer
2. Executable documentation
3. A guard against regression
4. A design tool (the test tells you what API should be)

---

## Article IV: Validation Requirements

### §4.1. Parser-Level Enforcement

The **parser** MUST call `validate_spec()` after parsing. This function:
1. Checks for presence of test_section, invariant_section, or bench_section
2. Emits a hard error ("TDD contract violated") if none exist

### §4.2. Codegen-Level Emission

All code generators MUST emit test code:
- **Zig**: `test "test_name" {}` and `test "invariant_name" {}`
- **C**: `void test_name(void)` and `void invariant_name(void)`
- **Verilog**: `task test_name();` and `assert (property)`

### §4.3. Build-Time Execution

Generated tests MUST be executed at build time:
- `.tri` → `tri test` runs automatically
- Zig/C/Verilog: tests run via their respective test frameworks

---

## Article V: Amendment Process

### §5.1. What Can Be Amended

This constitution may be amended by:
1. Opening an ADR (Architectural Decision Record)
2. Documenting proposed change with full rationale
3. Obtaining **unanimous consent** from all architectural stewards

### §5.2. What Cannot Be Amended

The following are **immutable** and may never be changed:
- The Language Food Chain (Article I) — `.t27` → `.tri` → backends
- The TDD Mandate (Article II)
- The Ban on Prototype Mode (Article III)
- The Validation Requirements (Article IV)

---

## Article VI: Enforcement

### §6.1. Agent Compliance

All AI agents working on T27 MUST:
1. Check for test/invariant/bench sections before creating new specs
2. Reject specs without tests with a hard error
3. Add test blocks when retrofitting existing code
4. Never bypass or disable validation
5. NEVER write `.tri` files by hand

### §6.2. Human Compliance

Human contributors MUST:
1. Review test coverage in every PR
2. Request tests for any spec lacking them
3. Treat test failures as blocking issues
4. NEVER edit generated backends directly

### §6.3. Automated Enforcement

The CI/CD pipeline MUST:
1. Run all generated tests on every commit
2. Block PRs where any test fails
3. Block PRs where any spec lacks tests
4. Validate that no hand-written backend code exists

---

## Article VII: Sacred Trinity

T27 rests on three pillars. Violating any violates the whole:

1. **φ² + 1/φ² = 3** — The mathematical foundation
2. **Ternary Computation** — The computational substrate
3. **TDD-Inside-Spec** — The verification mechanism

Additionally, **Language Food Chain** (Article I) ensures single source of truth and no self-feeding.

---

## Appendix: Quick Reference

| Command | Action |
|---------|--------|
| `tri parse <spec>` | Parse .t27 specification |
| `tri gen-zig <spec>` | Generate Zig backend from .tri |
| `tri gen-c <spec>` | Generate C backend from .tri |
| `tri gen-verilog <spec>` | Generate Verilog backend from .tri |
| `tri test` | Run generated tests |
| `tri seal <spec> --verify` | Verify a seal |
| `tri soul` | Display this document |

---

**Enacted**: 2026-04-16
**Version**: 2.0
**Status**: Immutable
