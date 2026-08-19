# Wave Loop 610 — 82% of what blocks IGLA is functions nobody wrote

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_609_REPORT.md`](WAVE_LOOP_609_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
W609 recommended the usize/u32 class as "the largest remaining".
Measured first, as W609's own rule demands:  ~7 errors.  Not a class.

The real picture:  1458 errors across specs/igla/**
                    886 (61%) are `use of undeclared identifier`
                    728 of those (82%) are functions DECLARED NOWHERE

gemm.t27:  90 -> 2 errors
```

---

## 1. Applying the measure-first rule to my own recommendation

W609 ended by recommending the `usize`/`u32` cast class. Measured:

| Spec | errors | mention `usize` |
|---|---:|---:|
| `eval` | 27 | 4 |
| `ternary_inference` | 37 | 1 |
| `prm` | 86 | 2 |
| `formal` | 175 | **0** |
| `bram_weights` | 86 | **0** |
| `dataset` | 116 | **0** |

**~7 errors total.** Not a class.

> **The recommendation you wrote last wave is exactly as unmeasured as any other
> guess.** W609's own rule — *measure the class before sizing the work* — applies
> to W609's conclusion.

## 2. The real distribution

| Error class | n | share |
|---|---:|---:|
| **`use of undeclared identifier`** | **886** | **61 %** |
| `expected type 'X', found 'Y'` | 208 | 14 % |
| `assertion failed` (comptime) | 87 | 6 % |
| `no field named 'X'` | 50 | |
| `struct 'X' has no member 'Y'` | 40 | |
| others | 187 | |
| **total** | **1 458** | |

### The 886 decompose into 76 names

| | errors | names |
|---|---:|---:|
| declared somewhere — **import/resolve** | 158 | 13 |
| **declared NOWHERE — unwritten** | **728** | **63** |

**82% of the dominant class is functions that are called and never written.**

This is W586's *unwritten* category — established there at **spec** granularity
— measured for the first time at **function** granularity. It is the largest
single fact about why IGLA does not compile, and it is **not** a compiler
defect, a missing lowering, or an import-graph problem.

Concentrated: `booth_mul_i32` 84, `throughput` 60, `is_prefix` 55,
`param_bounds_saturate` 53, `bram_weights_depth` 50, `smt_check_bool` 43 — the
top six are **345 errors**.

## 3. Two written from their tests

**`is_prefix`** (55 → 0) and **`booth_mul_i32`** (84 → 0) are fully determined
by their own tests, and were written to match the neighbours they sit beside —
`strings_equal` and `booth_mul_i16` respectively, including the sign/magnitude
decomposition and the shift-and-add unsigned core.

## 4. And one that could not be

`throughput` (60 errors) has four tests:

```
throughput(0, 1000) == 0.0      throughput(10, 1000)  == 10.0
throughput(1, 1)    == 1.0      throughput(100, 1000) == 100.0
```

They are satisfied **only** by `f(ops, ns) = ops` — a function that ignores its
duration argument, and therefore is not a throughput. No scaled form fits all
four: `ops·1000/ns` gives 1000 for the last, `ops/ns` gives 0.01 for the second.

> **The tests determine a projection, not a throughput.** Reported, not
> written — the same treatment as `ternary_mac`'s argument order and
> `systolic_ternary_array`'s contradictory tests. **Do not write a degenerate
> implementation to make a number go down.**

## 5. `gemm.t27`: 90 → 2

`booth_mul_i32` plus three spec repairs:

| Repair | Sites |
|---|---:|
| untyped `sign` — Zig reads `if (…) -1 else 1` as `comptime_int` under runtime control flow | 2 (one **pre-existing** in `booth_mul_i16`) |
| `i32` × `u32` product mismatch | 1 |
| lowercase `mat2x2` against the declared `Mat2x2` | 2 |

The remaining two errors are genuine design questions — `booth_mul_i16` returns
`i32` while `Mat2x2`'s fields are `i16`, and one function takes `*Matrix` where
a `Mat2x2` is passed. **Whether the product matrix should widen is a
specification decision**, left as one.

Recorded as **P25**.

## 6. Verification

| Gate | Result |
|---|---|
| `gemm.t27` | **90 → 2** errors |
| `yosys.t27` | `is_prefix` 55 → 0 |
| `lex-conform` / `parse-conform` | 34 / 34 · 15 / 15 |
| `parse-complete` | 401 / 608, 0 truncating |
| `cc-gate` | 101 |
| `cordic.t27` | 330 / 336 |

---

## 7. Three cooperation variants for W611

### Variant A (recommended) — The next four unwritten functions

`param_bounds_saturate` (53), `bram_weights_depth` (50), `smt_check_bool` (43),
`bram_weights_width` (28) — **174 errors from four names**, and this wave
established the method: read the tests, check they determine the function, write
it to match its neighbours, and **report rather than write when they do not**.

That last clause is the important one — `throughput` proved one of six is
under-determined, so expect roughly one of the four to come back as a decision.

### Variant B — Finish `gemm.t27`: two design questions

`booth_mul_i16` returns `i32`; `Mat2x2` holds `i16`. Either the product matrix
widens to `i32` or the multiply narrows. **It is one decision affecting both
remaining errors**, and it would make `gemm.t27` the first newly-compiling IGLA
RACE spec since `adder_tree`.

### Variant C — Flash the board

Unchanged, backed by
[`IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md). Phase 0 complete.

---

## Recommendation

**Variant A.** It is the largest measured class in the project, the method is
established and verified twice this wave, and each function is independent —
so the work parallelises and partial progress is real progress.

---

*φ² + φ⁻² = 3 | TRINITY*
