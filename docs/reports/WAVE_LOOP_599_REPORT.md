# Wave Loop 599 — a failing assertion now says what it saw

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_598_REPORT.md`](WAVE_LOOP_598_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

For the whole life of this project, every generated assertion lowered to

```zig
if (!(cond)) @panic("assertion failed");
```

A failing test named itself and stopped. It never said what it saw — which is
why W598's swapped sin/cos took a hand-written probe program, a re-export of
every function as `pub`, and an hour, to find something that one line of output
would have made obvious.

```
before   1/1 test.cordic_sin_exact_pi ... assertion failed

after    1/1 test.cordic_sin_exact_pi ...
           assertion failed:
             @abs(s) = 0.98524404
```

---

## 1. What changed

One lowering site in `bootstrap/src/compiler.rs`. When an assertion's condition
compares printable operands, the operands are printed on failure.

```rust
let mut cmps: Vec<&Node> = Vec::new();
Self::assert_comparisons(&node.children[0], &mut cmps);   // leaves of && / ||
…
self.write(&format!(")) __t27_assert_fail(\"\\n  {}:{}\\n\", .{{ {} }})", …));
```

Three design points, each forced by something that broke first:

| Decision | Why |
|---|---|
| a `noreturn` **helper fn**, not an inline block | the assert site is emitted as an *expression*; `if (c) { … };` is not a Zig statement, and the first attempt failed to compile on exactly that |
| operands **re-evaluated** in the failure branch | binding them first needs a type for each temporary. `then` clauses are pure and the branch runs once, immediately before the process dies |
| **literals and repeats suppressed** | `0.01 = 0.01` is noise, and `g > lo && g < hi` would otherwise print `g` twice |

`{any}` was checked against every type the corpus asserts on — f32, ints, bool,
strings, slices, enums — before the lowering was written.

## 2. The falsification conditions, checked before the edit

**F1 — `std.debug.print` is not comptime-callable, and some of this corpus's
assertions fold at comptime.** If a folded-false condition analysed the print, it
would become a *compile* error instead of a test failure, and the change would
break more than it fixed.

Checked with a two-case probe before touching the frozen file: a
constant-folded condition inside a `test` body still executes at **runtime**, so
the print is legal. **Recorded as cleared — wrongly. See below.**

**F2 — the corpus must not regress.** `cordic.t27` re-measured at **330 / 336
with the identical six failures**; `adder_tree.t27` at **335 / 335**;
`cc-gate` unchanged at 101. **Cleared.**

**F1 was NOT actually cleared, and the second measurement caught it.** The probe
tested an assertion inside a `test` body — where a constant-folded condition
still runs at runtime. But this corpus *also* folds assertions at **comptime**:
T4's and T5's disproved invariants are exactly that, and `std.debug.print` is
illegal there. The first version turned `cordic_top`'s clear

```
cordic_top.zig:522:32: error: encountered @panic at comptime: assertion failed
```

into an opaque error inside `std.Io.Threaded`. **I tested the case I thought of,
not the case the corpus contains** — the seventh time in this chain that the
instrument, not the code, was the thing that was wrong.

The fix is `@inComptime()`, which keeps each context's own diagnostic:

```zig
fn __t27_assert_fail(comptime fmt: []const u8, args: anytype) noreturn {
    if (@inComptime()) {
        @compileError("assertion failed");
    } else {
        std.debug.print(fmt, args);
        @panic("assertion failed");
    }
}
```

**F3 — the C backend must be untouched.** `cc-gate`: **101** headers compile,
identical to W597. **Cleared.**

## 3. What it shows on the six known failures

Each of `cordic.t27`'s six remaining failures now names, in its own output, the
number that falsifies it:

| Test | Output | Diagnosis it hands you |
|---|---|---|
| `cordic_sin_exact_pi` | `@abs(s) = 0.98524404` | **T6** — π is outside the convergence domain |
| `cordic_gain_boundary` | `g = 0.60725296` | below the asserted window (0.6073, 0.6074) |
| `cordic_arctan_table_entry_fifth` | `val = 0.06241881` | `atan(2⁻⁴)`, not the expected 0.03–0.04 |

**All three diagnoses took a hand-written probe in W598. They are now in the test
output.** Had this existed one wave earlier, `cordic_cos_zero` would have printed
`c[0] = 0.007032` and the swap would have been visible without any of it.

## 4. Verification

| Gate | Result |
|---|---|
| `cordic.t27` per-test | **330 / 336** — identical to W598 — unchanged by the new lowering |
| `lex-conform` / `parse-conform` | 29/29 · 13/13 |
| `cc-gate` | 101 compile (unchanged) |
| FROZEN_HASH | resealed — `e99eebd1…` → the sealed value in `bootstrap/stage0/FROZEN_HASH` |
| suite | superseded by `test-report` — see §5 |

---

## 5. Three cooperation variants for W600

### Variant A — **delivered this wave**, so W600 inherits its consequence

`t27c test-report <spec>` exists. It compiles **once** with a custom runner that
executes one test per process, selected by index, then spawns it per test.

```
$ t27c test-report specs/igla/race/cordic.t27
  FAIL  cordic_gain_increases_with_iters
  … 5 more …
  tests   336
  pass    330
  FAIL    6
  rate    98.2%
```

| | W597's shell loop | `t27c test-report` |
|---|---:|---:|
| wall clock | ~45 min | **5 s** |
| compilations | 336 | **1** |
| build cache | **6.1 GB — it filled the disk** | one binary |

`zig test --test-filter` recompiles the whole file per filter; that is why the
loop cost what it did. **The disk exhaustion was the clue** — a measurement
whose cost scales with the number of tests is one that stops being taken.

**What W600 should do with it:** run it over the whole corpus. Every spec that
compiles now has a per-test rate available for five seconds of work, and nobody
has ever seen that table.

### Variant B — Argument reduction for CORDIC (T6)

T6 states exactly what to write and why: map θ into [−π/2, π/2] via
`θ' = θ − k·π`, negating both outputs for odd *k*. Unlike the five false
assertions no judgement call is involved — only the work. It changes the
algorithm, so it wants an owner's sign-off.

### Variant C — Flash the board

Unchanged: `verdict : BLOCKED -- no programmer on USB`. Needs the QMTech Wukong
V1 and the Digilent HS2 cable physically present.

---

## Recommendation

**Variant A** — now meaning *use* the command, not build it. This wave removed
both the reason a diagnosis needed a probe and the reason a measurement needed a
loop. What it did not do is spend the resulting five seconds per spec on the
other 396, and that table has never existed.

---

*φ² + φ⁻² = 3 | TRINITY*
