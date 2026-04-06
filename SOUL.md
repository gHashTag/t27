# SOUL — T27 Constitutional Law

**Canonical location:** this file at the **repository root** is the **single source of truth** for Trinity constitutional law. **[`docs/SOUL.md`](docs/SOUL.md)** is an **expanded reference** (detail, examples); if anything disagrees, **this file wins**.

** Immutable Document — Amendments Require Unanimous Architectural Consent **

> *"A specification without tests is a lie told in the future tense."*
> — Trinity TDD Axiom #1

---

## Preamble

T27 is a **spec-first** architecture where mathematical truth, not implementation, is the source of truth. This document establishes the constitutional principles that govern all T27 specifications, compilation, and verification.

---

## Article I: The Language Policy

### §1.1. ASCII-Only Source Files
**Source files MUST be ASCII-only.** Identifiers and comments MUST be English.

All files in the following categories MUST contain only ASCII characters (U+0000–U+007F):
- `.t27` — TRI-27 assembly specifications
- `.tri` — TRI high-level specifications
- `.zig` — Zig source code
- `.c` / `.h` — C source/header files
- `.v` / `.verilog` — Verilog hardware descriptions
- Build scripts, makefiles, etc.

**FORBIDDEN in source files:**
- **Cyrillic** (U+0400–U+04FF) and other non-Latin scripts in identifiers and comments
- **Non-Latin scripts**: Greek, Arabic, Chinese, Japanese, Korean, etc., unless an Architect-approved exception exists

### §1.2. First-party documentation language
Markdown under `docs/`, `specs/`, `architecture/`, `clara-bridge/`, `conformance/`, and root project Markdown (`README.md`, `AGENTS.md`, `CLAUDE.md`, `TASK.md`) **MUST be English**, except paths listed in **`docs/.legacy-non-english-docs`** (grandfathered) and anything under **`external/`**.

### §1.3. Enforcement
The parser rejects Cyrillic in source with:
```
error: Language policy violation: source file contains Cyrillic characters (U+0400-U+04FF). Source files must be ASCII-only. See SOUL.md Article I.
```

CI runs `scripts/check-first-party-doc-language.sh` on pull requests.

**Compiler build:** `cargo build` in `bootstrap/` runs `build.rs`, which fails the build if Cyrillic appears in specs, bootstrap Rust sources, or unlisted first-party Markdown (this Article; expanded enforcement notes in `docs/SOUL.md` Law #1).

### §1.4. Rationale
1. **Universality**: ASCII is universally supported across all platforms and tools
2. **Clarity**: English is the single review language for Trinity first-party docs and specs
3. **Separation of Concerns**: Vendored locales live under `external/`; core repo stays English
4. **Git Compatibility**: No encoding issues in diffs, patches, or blame output

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

    ; test_bundle_idempotence
    ; Verify: bundling a vector with itself produces same vector
    ; Setup: create vector v, bundle v with v
    ; Expected: result == v

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

If you write a spec without tests, the parser **will reject it** with:

```
TDD contract violated: spec must contain at least one 'test' or 'invariant' block
```

### §3.2. The Rationale
Tests written after implementation are not tests—they are retroactive justification. True TDD requires the test to exist **before** the implementation, serving as:
1. A contract between specifier and implementer
2. Executable documentation
3. A guard against regression
4. A design tool (the test tells you what the API should be)

---

## Article IV: Validation Requirements

### §4.1. Parser-Level Enforcement
The **parser** (`compiler/parser/parser.t27`) MUST call `validate_spec()` after parsing. This function:
1. Checks for presence of test_section, invariant_section, or bench_section
2. Checks spec_decl.test_blocks or spec_decl.invariants for spec-style
3. Emits a hard error ("TDD contract violated") if none exist

### §4.2. Codegen-Level Emission
All code generators MUST emit test code:
- **Zig**: `test "test_name" {}` and `test "invariant_name" {}`
- **C**: `void test_name(void)` and `void invariant_name(void)`
- **Verilog**: `task test_name();` and `assert (property)`

### §4.3. Build-Time Execution
Generated tests MUST be executed at build time:
- Zig: `zig test` runs automatically in build.tri
- C: tests compiled into test binary and run
- Verilog: assertions verified in simulation

---

## Article V: Amendment Process

### §5.1. What Can Be Amended
This constitution may be amended by:
1. Opening an ADR (Architectural Decision Record)
2. Documenting the proposed change with full rationale
3. Obtaining **unanimous consent** from all architectural stewards

### §5.2. What Cannot Be Amended
The following are **immutable** and may never be changed:
- The Language Policy (Article I)
- The TDD Mandate (Article II)
- The Ban on Prototype Mode (Article III)
- The Validation Requirements (Article IV)

---

## Article VI: Enforcement

### §6.1. Agent Compliance
All AI agents working on T27 MUST:
1. Check for test/invariant/bench sections before creating new specs
2. Reject specs without tests with a hard error
3. Add test blocks when retrofiting existing code
4. Never bypass or disable validation

### §6.2. Human Compliance
Human contributors MUST:
1. Review test coverage in every PR
2. Request tests for any spec lacking them
3. Treat test failures as blocking issues

### §6.3. Automated Enforcement
The CI/CD pipeline MUST:
1. Run all generated tests on every commit
2. Block PRs where any test fails
3. Block PRs where any spec lacks tests

---

## Article VII: Sacred Trinity

T27 rests on three pillars. Violating any violates the whole:

1. **φ² + 1/φ² = 3** — The mathematical foundation
2. **Ternary Computation** — The computational substrate
3. **TDD-Inside-Spec** — The verification mechanism

Additionally, the **Language Policy** (Article I) ensures universality and clarity.

---

## Appendix: Quick Reference

| Command | Action |
|---------|--------|
| `tri validate <spec>` | Check spec has tests (enforced by parser) |
| `tri gen <spec>` | Generate code with tests embedded |
| `tri test <spec>` | Run generated tests |
| `tri soul` | Display this document |

---

**Enacted**: 2026-04-04
**Version**: 1.0
**Status**: Immutable
