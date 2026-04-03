# SOUL — T27 Constitutional Law

** Immutable Document — Amendments Require Unanimous Architectural Consent **

> *"A specification without tests is a lie told in the future tense."*
> — Trinity TDD Axiom #1

---

## Preamble

T27 is a **spec-first** architecture where mathematical truth, not implementation, is the source of truth. This document establishes the constitutional principles that govern all T27 specifications, compilation, and verification.

---

## Article I: The TDD Mandate

### §1.1. The Iron Law
Every `.t27` specification MUST contain at least one of:
- A `.test` section with one or more test cases
- An `.invariant` section with one or more invariant declarations
- A `.bench` section with one or more benchmark declarations

**No exceptions.** A spec without tests is not a specification—it is a draft.

### §1.2. Test Format (Assembly-Style)
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

### §1.3. Test Format (Spec-Style)
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

## Article II: No Prototype Mode

### §2.1. The Ban on Temporary Code
T27 **does not have** a prototype mode. There is no `--allow-no-tests` flag. There is no "I'll add tests later" grace period.

If you write a spec without tests, the parser **will reject it** with:

```
TDD contract violated: spec must contain at least one 'test' or 'invariant' block
```

### §2.2. The Rationale
Tests written after implementation are not tests—they are retroactive justification. True TDD requires the test to exist **before** the implementation, serving as:
1. A contract between specifier and implementer
2. Executable documentation
3. A guard against regression
4. A design tool (the test tells you what the API should be)

---

## Article III: Validation Requirements

### §3.1. Parser-Level Enforcement
The **parser** (`compiler/parser/parser.t27`) MUST call `validate_spec()` after parsing. This function:
1. Checks for presence of test_section, invariant_section, or bench_section
2. Checks spec_decl.test_blocks or spec_decl.invariants for spec-style
3. Emits a hard error ("TDD contract violated") if none exist

### §3.2. Codegen-Level Emission
All code generators MUST emit test code:
- **Zig**: `test "test_name" {}` and `test "invariant_name" {}`
- **C**: `void test_name(void)` and `void invariant_name(void)`
- **Verilog**: `task test_name();` and `assert (property)`

### §3.3. Build-Time Execution
Generated tests MUST be executed at build time:
- Zig: `zig test` runs automatically in build.tri
- C: tests compiled into test binary and run
- Verilog: assertions verified in simulation

---

## Article IV: Amendment Process

### §4.1. What Can Be Amended
This constitution may be amended by:
1. Opening an ADR (Architectural Decision Record)
2. Documenting the proposed change with full rationale
3. Obtaining **unanimous consent** from all architectural stewards

### §4.2. What Cannot Be Amended
The following are **immutable** and may never be changed:
- The TDD Mandate (Article I)
- The Ban on Prototype Mode (Article II)
- The Validation Requirements (Article III)

---

## Article V: Enforcement

### §5.1. Agent Compliance
All AI agents working on T27 MUST:
1. Check for test/invariant/bench sections before creating new specs
2. Reject specs without tests with a hard error
3. Add test blocks when retrofiting existing code
4. Never bypass or disable validation

### §5.2. Human Compliance
Human contributors MUST:
1. Review test coverage in every PR
2. Request tests for any spec lacking them
3. Treat test failures as blocking issues

### §5.3. Automated Enforcement
The CI/CD pipeline MUST:
1. Run all generated tests on every commit
2. Block PRs where any test fails
3. Block PRs where any spec lacks tests

---

## Article VI: Sacred Trinity

T27 rests on three pillars. Violating any violates the whole:

1. **φ² + 1/φ² = 3** — The mathematical foundation
2. **Ternary Computation** — The computational substrate
3. **TDD-Inside-Spec** — The verification mechanism

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
