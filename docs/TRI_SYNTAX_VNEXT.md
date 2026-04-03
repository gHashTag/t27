# TRI_SYNTAX_VNEXT.md — TRI-27 Language Syntax vNext

**Version**: 1.0
**Date**: 2026-04-04
**Status**: Draft

> *This document defines the syntax for TRI-27 assembly and high-level spec notation.*

---

## Overview

TRI-27 has two syntax styles:

1. **Assembly-style** — Low-level assembly with `.const`, `.data`, `.code`, `.test`, `.invariant`, `.bench` sections
2. **Spec-style** — High-level notation with `spec`, `test` (given/when/then), `invariant` (assert), `rule` (expect)

Both styles compile to the same AST and can coexist in the same codebase.

---

## Assembly-Style Syntax

### Section Directives

```t27
.use base::types

.const NAME value
.data
    label: .dword value
.code
label:
    instruction operands
    HALT

.test
    ; test_name
    ; Verify: description
    ; Setup: setup description
    ; Expected: expected outcome

.invariant
    ; invariant_name
    ; Formal statement
    ; Rationale: explanation

.bench
    ; bench_name
    ; Measure: what to measure
    ; Target: target value
```

### Instructions

```
MOV dst, src         ; Move
ADD dst, src1, src2  ; Add
SUB dst, src1, src2  ; Subtract
MUL dst, src1, src2  ; Multiply
JZ test_reg, label   ; Jump if zero
JNZ test_reg, label  ; Jump if non-zero
JMP label            ; Unconditional jump
BIND reg, vsa        ; VSA bind
BUNDLE regs          ; VSA bundle
HALT                 ; Stop execution
```

### Operands

```
r0-r26               ; Registers (Coptic alphabet mapping)
#value               ; Immediate literal
label                ; Label reference
[reg + offset]       ; Memory reference
```

---

## Spec-Style Syntax (High-Level TDD)

### Spec Declaration

```t27
spec spec_name
    ; Constants
    const EPS = 0.001
    const PHI = 1.6180339887498948

    ; Test blocks
    test test_roundtrip_phi
        given x = PHI
        when encoded = gf16_encode_f32(x)
        and decoded = gf16_decode_to_f32(encoded)
        then abs(decoded - x) < EPS

    ; Invariants
    invariant phi_squared_property
        assert PHI * PHI = PHI + 1

    ; Rules
    rule phi_range
        expect PHI > 1.6 and PHI < 1.7
```

### Test Block Syntax

```t27
test test_name
    given variable = expression
    when result = function_call(variable)
    and intermediate = some_operation(result)
    then assertion_expression
```

### Invariant Syntax

```t27
invariant invariant_name
    assert logical_expression
    ; Rationale: explanation (optional comment)
```

### Rule Syntax

```t27
rule rule_name
    expect expectation_expression
```

---

## TDD Keywords

| Keyword | Assembly | Spec-Style | Purpose |
|---------|----------|------------|---------|
| `spec` | — | ✓ | Module header declaration |
| `test` | ✓ (.test) | ✓ | Test case declaration |
| `invariant` | ✓ (.invariant) | ✓ | Invariant/assert declaration |
| `rule` | — | ✓ | Business rule declaration |
| `bench` | ✓ (.bench) | — | Benchmark declaration |
| `given` | — | ✓ | Test setup clause |
| `when` | — | ✓ | Test action clause |
| `and` | — | ✓ | Test continuation clause |
| `then` | — | ✓ | Test assertion clause |
| `expect` | — | ✓ | Rule expectation clause |
| `assert` | — | ✓ | Invariant assertion |
| `; Verify:` | ✓ | — | Test description |
| `; Expected:` | ✓ | — | Expected outcome |
| `; Setup:` | ✓ | — | Test setup |
| `; Rationale:` | ✓ | — | Invariant rationale |
| `; Measure:` | ✓ | — | Benchmark measure |
| `; Target:` | ✓ | — | Benchmark target |

---

## Comments

```t27
; Single-line comment (semicolon at start of line)

; Inline comment after code
MOV r0, #1  ; This is an inline comment

; Section separator
; ═══════════════════════════════════════════════════════════════
```

---

## Literals

```t27
; Integer literals
42
0x2A        ; Hexadecimal (prefix 0x)
0b101010    ; Binary (prefix 0b)

; Float literals
1.618
1.618e-3

; String literals
"hello world"
```

---

## Example: Complete Spec

```t27
; gf16.t27 — GoldenFloat16 Encode/Decode
; φ² + 1/φ² = 3 | TRINITY

.use base::types

.const SIGN_SHIFT 15
.const EXP_SHIFT 9
.const MANT_MASK 0x01FF

.data
    .const GF16_ZERO_POS 0x0000
    .const GF16_INF_POS 0x7E00

.code
gf16_encode_f32:
    ; Encoding logic here
    HALT

.test
    ; gf16_roundtrip_phi
    ; Verify: encoding PHI to GF16 and decoding preserves value
    ; Expected: |decoded - PHI| < 0.001

    ; gf16_zero_encoding
    ; Verify: zero encodes to correct pattern
    ; Expected: 0.0 → 0x0000, -0.0 → 0x8000

.invariant
    ; gf16_roundtrip_symmetry
    ; For all normal values x: |decode(encode(x)) - x| < epsilon
    ; Rationale: Encoding/decoding are inverses within precision

.bench
    ; gf16_encode_throughput
    ; Measure: encodes per second
    ; Target: > 10M encodes/sec
```

---

## Grammar Summary

```
program         ::= (assembly_spec | spec_decl)*

assembly_spec   ::= const_section? data_section? code_section
                   (test_section | invariant_section | bench_section)*

spec_decl       ::= 'spec' identifier (const_decl | test_decl
                   | invariant_decl | rule_decl)*

test_decl       ::= 'test' identifier
                   ('given' identifier '=' expression)+
                   ('when' identifier '=' expression)*
                   ('and' (identifier '=' expression | expression))*
                   ('then' expression)+

invariant_decl  ::= 'invariant' identifier
                   'assert' expression

rule_decl       ::= 'rule' identifier
                   ('expect' expression)+

expression      ::= identifier | literal | binary_op | function_call
```

---

## Migration Notes

1. **Legacy assembly files** (.t27 with only .const/.data/.code) must add .test/.invariant sections
2. **New specs** should use spec-style notation for better TDD integration
3. **Mixed syntax** is supported — assembly-style and spec-style can coexist
