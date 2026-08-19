# Wave Loop 564 Report — the first genuine test failures

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_563_REPORT.md`](WAVE_LOOP_563_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
                     W560   W561   W562   W563   W564
ALL_PASS                5      7      9     14     14
TEST_FAIL               0      0      0      0      2   <- first real failures
COMPILE_FAIL          194    192    190    184    183
tests passing          45     54     64    167    175
```

Sixteen waves after this chain started with *"the repo does not build on stable
Rust"*, **a test in this repository has failed for a real reason.**

---

## 1. The defect

`specs/fpga/ternary_isa.t27`, test `validate_r_type_format`:

```t27
test validate_r_type_format
    given fmt = r_type_format()
    then validate_instr_format(fmt) == 0
```

`r_type_format()` declares:

| field | bits |
|---|---:|
| `opcode_bits` | 6 |
| `rd_bits` | 5 |
| `rs1_bits` | 5 |
| `rs2_bits` | 5 |
| `imm_bits` | 0 |
| **sum** | **21** |
| `total_bits` | **32** |

`validate_instr_format` counts a field-width mismatch as an error, so it returns
**1**, and the assertion `== 0` fails.

**11 bits are unaccounted for in the R-type instruction encoding.** This is a
genuine specification defect, not a codegen artefact — and it is precisely the
class of finding this chain was built to surface. It could not fail before W559,
because the test body was discarded before it reached any backend.

`specs/fpga/simulator.t27` also aborts on an assertion; not yet characterised.

**Neither is fixed here.** Whether `total_bits` is wrong or the field widths are
is a specification decision, not a compiler one.

---

## 2. What unblocked it

`zig_ident` escaped primitive *type* names (`bool`, `u32`, …) but not Zig
*keywords*, and it was not applied to enum variants or struct fields at all. A
variant or field literally named `error` emitted:

```
error = 4,      ->  expected '.', found '='
error: bool,    ->  expected '.', found ':'
```

Extended `zig_ident` with the full Zig keyword set and applied it at both sites.
Yield was small in compile-failure terms (184 → 183) but it released the two
specs that then *ran* — and failed.

---

## 3. A third instrumentation correction

The harness reported those two specs as `UNKNOWN`, because it did not recognise
`terminated with signal ABRT` / `panic: assertion failed` as a test failure —
the same class of gap as W560's misclassification, in the opposite direction.
Fixed. The harness and all raw results are now committed under
[`data/`](data/) so the measurement is reproducible rather than ad hoc.

---

## 4. Where the corpus stands

| | |
|---|---:|
| Substantive assertion clauses written | 11,282 |
| **Tests passing** | **175** |
| **Tests failing for real reasons** | **2 specs** |
| Specs fully passing | 14 of 199 |
| Specs blocked by `default_input()` | 169 |

---

## 5. Three cooperation variants for W565

### Variant A (recommended) — Characterise and fix the two real failures

1. Decide the `ternary_isa` R-type question: is `total_bits = 32` wrong, or are
   the field widths? 11 bits are unaccounted for; the ISA documentation should
   settle it.
2. Characterise the `simulator.t27` abort the same way.
3. Fix both in the spec, and confirm the tests pass.

**This is the first time the project has had a real defect backlog from its own
tests.** Working it is what the whole chain was for.

### Variant B — Decide the fate of the 571 template tests

Unchanged since W562 and still the single biggest lever: **169 specs** cannot
compile because of `default_input()`, proved unfixable mechanically (48 uniform
types, 96 mixed, 25 calling functions that do not exist). Rewriting or deleting
them releases the largest remaining block. **Maintainer's call** — it changes
test intent.

### Variant C — Lower keyword-form invariants

Unchanged since W559: **5,163 invariants** still emit
`// invariant: X verified (no statements)`. Largest single remaining inert
population; the W559 lowering pattern and its fixture-and-census discipline
apply directly.

---

## Recommendation

**Variant A.** Two real defects are worth more than a hundred more compiling
specs, because they are the first evidence that this test suite can detect
anything. Then **C**, which is mechanical and large. **B** has needed a human
since W562.

---

*φ² + φ⁻² = 3 | TRINITY*
