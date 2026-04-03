# TDD-CONTRACT.md — TDD-Inside-Spec Contract Enforcement

**Version**: 1.0
**Date**: 2026-04-04
**Status**: Active

---

## Overview

This document defines the TDD (Test-Driven Development) contract for Trinity t27 specifications. The contract enforces that **every specification must include tests** — tests are not optional, they are mandatory.

---

## The Contract

### Rule #1: No Spec Without Tests

**Every `.t27` specification file MUST contain at least one `test` or `invariant` block.**

**Status**: MANDATORY — No exceptions, no prototype mode.

### Rule #2: Tests Are Source of Truth

Tests defined in `.t27` files are the source of truth. Conformance JSON files in `conformance/` directory are **generated artifacts**, not hand-written source.

### Rule #3: TDD Applies to All Spec Types

The TDD contract applies to:
- Assembly-style specs (`.const`/`.data`/`.code` with `.test`/`.invariant` sections)
- Spec-style specs (`spec` keyword with `test`/`invariant`/`rule` blocks)
- Both high-level and low-level specifications

---

## Enforcement

### 1. Parser-Level Enforcement

The parser (`compiler/parser/parser.t27`) validates the TDD contract:

```
fn Parser.validate_spec() -> bool {
    if self.context.ast_root.test_blocks.len() == 0
       and self.context.ast_root.invariant_blocks.len() == 0 {
        self.context.add_error(
            "TDD contract violated: spec must contain at least one 'test' or 'invariant' block",
            self.context.ast_root.line, 1);
        return false;
    }
    return true;
}
```

**Error message**:
```
error: TDD contract violated: spec must contain at least one 'test' or 'invariant' block
```

### 2. CLI-Level Enforcement

The `tri gen` command fails if TDD contract is violated:

```
$ tri gen specs/my_spec.t27
error: TDD contract violated: no tests in spec

The t27 project follows Test-Driven Development where:
  1. Every spec MUST have at least one 'test' or 'invariant' block
  2. Tests are written BEFORE or WITH the implementation
  3. Conformance JSON is generated FROM tests, not hand-written

To fix this error:
  1. Add a .test section with test cases to your spec
  2. Or add a .invariant section with invariant declarations
  3. Or use the high-level TDD syntax with 'test' and 'invariant' blocks

NOTE: There is NO --allow-no-tests flag (prototype mode is disabled per policy)
```

### 3. Git-Level Enforcement

The `tri git commit` command requires TDD compliance:

```
$ tri git commit --all -m "my changes"
error: TDD contract violated: spec has no tests
```

---

## Test Syntax

### Assembly-Style Tests

```t27
.test
    ; test_name
    ; Verify: description of what is being tested
    ; Setup: how to set up the test
    ; Expected: the expected outcome

.invariant
    ; invariant_name
    ; Formal statement of the invariant
    ; Rationale: why this invariant must hold
```

### Spec-Style Tests

```t27
test test_name
    given variable = value
    when result = function_call(variable)
    then result == expected

invariant invariant_name
    assert logical_expression
```

---

## Conformance JSON Generation

Conformance JSON is **GENERATED** from spec tests, not hand-written:

```bash
# Generate conformance JSON from spec tests
tri gen specs/numeric/gf16.t27 --emit-conformance

# Output: conformance/gf16.json (generated artifact)
```

The generated JSON includes:
- Test names from `.test` blocks
- Verify/Expected descriptions from assembly-style tests
- Given/When/Then clauses from spec-style tests
- Invariant statements

**DO NOT** hand-edit `conformance/*.json` files. They will be overwritten on next generation.

---

## Migration Guide

### For Existing Specs

1. **Check your spec** for test/invariant blocks:
   ```bash
   tri spec validate specs/my_spec.t27
   ```

2. **Add tests** if missing:
   ```t27
   .test
       ; my_feature_test
       ; Verify: feature works correctly
       ; Expected: correct result

   .invariant
       ; my_invariant
       ; For all valid inputs: output is valid
   ```

3. **Validate** after adding tests:
   ```bash
   tri spec validate specs/my_spec.t27
   ```

### For New Specs

Use `tri spec create` to generate a template with TDD blocks:

```bash
tri spec create my_feature

# Creates: specs/my_feature.t27 with template .test/.invariant sections
```

---

## Common Violations and Fixes

### Violation #1: Spec Without Tests

**Error**:
```
TDD contract violated: spec must contain at least one 'test' or 'invariant' block
```

**Fix**: Add at least one test or invariant:
```t27
.test
    ; basic_test
    ; Verify: basic functionality
    ; Expected: returns expected value
```

### Violation #2: Empty Test Section

**Error**: Parser detects `.test` section with no test cases

**Fix**: Add test case inside `.test` section

### Violation #3: Hand-Written Conformance JSON

**Error**: Tests in spec don't match conformance JSON

**Fix**: Regenerate conformance JSON from spec tests:
```bash
tri gen specs/my_spec.t27 --emit-conformance
```

---

## Verification

### Verify TDD Compliance

```bash
# Check single spec
tri spec validate specs/my_spec.t27

# Check all specs
tri spec list

# Show TDD status for all specs
tri gen --check-tdd
```

### Verify Generated Tests

```bash
# Generate code and tests
tri gen specs/my_spec.t27 --emit-tests

# Run generated tests
zig test backend/zig/my_spec_test.zig
```

---

## Policy Summary

| Policy | Enforcement | Exception |
|--------|-------------|-----------|
| No spec without tests | Parser, CLI, Git | NONE |
| Conformance JSON generated | CLI validation | NONE |
| Tests in spec | Parser validation | NONE |
| Invariants in spec | Parser validation | NONE |

---

## References

- [SOUL.md](SOUL.md) — Constitutional Law #1: TDD-Inside-Spec
- [TRI_SYNTAX_VNEXT.md](TRI_SYNTAX_VNEXT.md) — Syntax definition
- [ADR-003-tdd-inside-spec.md](architecture/ADR-003-tdd-inside-spec.md) — Architecture decision
