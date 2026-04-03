# ADR-003: TDD-Inside-Spec — Inline Tests as Architectural Law

## Status
**Accepted** — Enforced by SOUL.md as immutable constitutional law

## Context

Historical problem: Tests written **after** implementation serve as retroactive justification rather than design guidance. In specification-driven development, the spec is the source of truth—but without embedded tests, the spec is incomplete.

**Prior State (Before ADR-003):**
- Specs lived as `.t27` files with `.data`, `.code`, `.const` sections
- Tests existed as separate files (Zig `test {}` blocks, Python test suites)
- No guarantee that spec and tests stayed in sync
- "I'll add tests later" was a common anti-pattern
- Conformance vectors generated separately, not from source truth

**The Gap:**
A spec is a contract. A contract without verification clauses is not a contract—it's a wish list.

## Decision

### Core Principle
**Every T27 specification MUST contain inline test/invariant/bench blocks.**

This is enforced at the parser level: a `.t27` file without tests is rejected as invalid.

### Syntax Options

Two complementary syntaxes are supported:

#### 1. Assembly-Style (For Low-Level Specs)
```t27
.test
    ; test_name
    ; Verify: what is being tested
    ; Setup: test setup
    ; Expected: expected outcome

.invariant
    ; invariant_name
    ; Formal statement of invariant
    ; Rationale: why this matters

.bench
    ; bench_name
    ; Measure: what is measured
    ; Target: performance target
```

#### 2. Spec-Style (For High-Level Specs)
```t27
spec module_name {
    test test_name {
        given setup
        when action
        then assertion
    }

    invariant invariant_name {
        assert property
    }

    rule rule_name {
        expect property
    }
}
```

### Codegen Emission

All three codegen backends MUST emit test code:

| Backend | Test Output | Invariant Output | Bench Output |
|---------|-------------|------------------|--------------|
| Zig | `test "name" {}` | `test "invariant_name" {}` | `test "bench_name" {}` |
| C | `void test_name(void)` | `void invariant_name(void)` | `void bench_name(...)` |
| Verilog | `task test_name();` | `assert (property)` | Task with timing |

### Validation

The parser (`compiler/parser/parser.t27`) calls `validate_spec()` after parsing, which:

1. Checks for `test_section`, `invariant_section`, or `bench_section` (assembly-style)
2. Checks for `spec_decl.test_blocks` or `spec_decl.invariants` (spec-style)
3. Emits hard error if none exist: `"TDD contract violated: spec must contain at least one 'test' or 'invariant' block"`

**No `--allow-no-tests` flag exists.**

## Consequences

### Positive

1. **Design-First Development**: Tests guide implementation, not the reverse
2. **Living Documentation**: Tests are always co-located with the code they verify
3. **Drift Prevention**: Tests and spec cannot diverge—they're in the same file
4. **Automated Verification**: Generated tests run at build time
5. **Spec Completeness**: A spec is considered "complete" only when it has tests

### Negative

1. **Higher Barrier to Entry**: Cannot create "quick draft" specs without tests
2. **Initial Overhead**: Every spec requires test thought upfront
3. **No Prototype Mode**: No escape hatch for "I'll test later"

### Mitigations

The negative consequences are intentional features, not bugs:
- The "higher barrier" ensures quality
- The "initial overhead" is design work that would happen anyway
- The "no prototype mode" prevents technical debt

## Examples

### Example 1: VSA Bind Operation (Assembly-Style)
```t27
; specs/vsa/ops.t27

.test
    ; test_bind_unbind_identity
    ; Verify: bind and unbind are inverses for all trit vectors
    ; Setup: create random 27-trit vectors a and b
    ; Expected: unbind(bind(a, b), b) == a

    ; test_bind_distributes_over_bundle
    ; Verify: bind distributes over bundle
    ; Setup: create vectors a, b, c
    ; Expected: bind(a, bundle(b, c)) == bundle(bind(a, b), bind(a, c))

.invariant
    ; bind_result_trit_count
    ; bind(a, b) always produces 27 trits
    ; Rationale: Output dimension must match input dimension

    ; bind_neutral_element
    ; bind(a, zeros) == a for all a
    ; Rationale: Zero vector is neutral element for bind

.bench
    ; bench_bind_latency_cycles
    ; Measure: cycles for single 27-trit bind
    ; Target: < 100 cycles on t27-hardware

    ; test_bind_parallel_throughput
    ; Measure: binds per second across all MAC units
    ; Target: > 100M binds/sec on 8-unit MAC
```

### Example 2: Sacred Constants (Spec-Style)
```t27
; specs/math/constants.t27

spec sacred_constants {
    test phi_squared_relationship {
        given phi = PHI
        given phi_squared = PHI_SQUARED
        given result = phi_squared - phi - 1
        then result == 0
    }

    test gamma_reciprocal {
        given gamma = GAMMA
        given reciprocal = 1.0 / gamma
        then abs(gamma - 1.0/reciprocal) < epsilon
    }

    invariant phi_positive {
        assert PHI > 0
    }

    invariant phi_not_rational {
        assert !is_rational(PHI)
    }

    rule phi_bounds {
        expect 1.6 < PHI < 1.7
    }
}
```

## Alternatives Considered

### Alternative 1: External Test Files
Tests live in separate files (`spec.test.t27`).

**Rejected:** Allows drift between spec and tests.

### Alternative 2: Test Annotation Comments
Tests are comments that a post-processor extracts.

**Rejected:** Comments are not parsed—no parser validation.

### Alternative 3: Optional `--allow-no-tests` Flag
Allow bypassing test requirement with a flag.

**Rejected:** Violates constitutional mandate (see SOUL.md Article II).

## Related Decisions

- **ADR-001**: De-Zigfication — Spec is source of truth, Zig is generated
- **ADR-002**: Trinity Architecture — Sacred math + ternary + orchestration
- **SOUL.md**: Constitutional law establishing TDD mandate as immutable

## Implementation Status

- [x] AST supports test/invariant/bench nodes
- [x] Lexer recognizes test/invariant/bench tokens
- [x] Parser validates spec has tests
- [x] Zig codegen emits test blocks
- [x] Verilog codegen emits assertions and tasks
- [x] C codegen emits test functions
- [x] SOUL.md establishes constitutional mandate
- [ ] All 9 retrofitted specs have test blocks
- [ ] CI enforces test execution on every PR

## References

1. SOUL.md — Constitutional law
2. compiler/ast.t27 — AST node definitions
3. compiler/parser/parser.t27 — Validation logic
4. compiler/codegen/*/codegen.t27 — Test emission

---

**Accepted**: 2026-04-04
**Author**: T27 Architecture
**Status**: Immutable (per SOUL.md Article IV)
