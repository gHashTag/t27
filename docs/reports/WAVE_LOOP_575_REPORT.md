# Wave Loop 575 Report — the new check found a lexer bug on its first run

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_574_REPORT.md`](WAVE_LOOP_574_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
t27c check-calls          38  ->  32     (every remaining finding is a pending decision)
assertions emitted      4,393 -> 4,403
non-scratch parse OK      341 ->  351     (0 regressions vs W568)
tests executing/passing            615
T1 / T2 / T3              re-proved
```

W574 built `t27c check-calls` and reported 38 findings. Working through them, one
turned out not to be a spec defect at all:

> **`specs/physics/gamma_conjecture.t27:211` — `verify_gamma_conjecture` — 7 arguments
> passed, 4 declared.**

The call passes four. **The lexer was splitting scientific notation.**

---

## 1. `1e6` was three tokens

```t27
f(1e6, 2.5e-3)
```
```zig
f(1, e6, 2.5, e - 3)      // before
f(1e6, 2.5e-3)            // after
```

The number lexer consumed digits, `.`, `x`, `b`, `_` and hex letters, and stopped at
`e` — so an exponent became an identifier, and a *negative* exponent became an
identifier, a minus and another number. A four-argument call parsed as seven arguments;
elsewhere it silently changed the value being asserted.

**486 occurrences across 62 specs**, and it had never been reported, because a wrong
value is only visible if something checks it. The arity check is what made it visible,
and it did so on its first run against the corpus.

The fix consumes `e`/`E` plus an optional sign only when a digit actually follows, so
`0x1e` stays hex and a bare identifier `e6` is untouched. **19 of the 62 specs now
generate different code** — the other 43 use scientific notation only where the split
happened to reparse to the same value.

## 2. The four genuine arity defects, each determined

| Site | Was | Now | What determines it |
|---|---|---|---|
| `igla/race/cordic.t27:555` | `cordic_gain()` | `cordic_gain(16)` | declared `fn cordic_gain(iterations: u32)`; the file's own invariant bounds `n` at 1..16 and the test asserts `0.6 < g < 0.61` |
| `queen/lotus.t27:493-494` | `generate_plan()`, `execute_action()` | `generate_plan_()`, `execute_action_()` | both take one argument; the zero-argument wrappers carry a trailing underscore, and the neighbouring field already calls `evaluate_()` |
| `server/mdns.t27:238` | `concat(a, ".", b)` | `concat(concat(a, "."), b)` | `fn concat(a: []u8, b: []u8)` takes two |
| `vsa/sdk.t27:445` | `hypervector_set(a, 1)` | `hypervector_set(a, 1, Trit::pos)` | declared `(self, index, value)`; the test asserts only that the dimension is preserved, so the value is free |

## 3. What is left, and why none of it is mine

```
  aggregate-vs-scalar    3
  arity                 29
  TOTAL                 32
```

Every one of the 32 belongs to a decision already recorded and awaiting the maintainer:

- **29 arity** — all of the form `f(input)` against `fn f()`, from the
  `default_input()` template scaffold. Dropping the argument would make the call
  well-formed and change nothing: the test still calls the undefined `default_input`
  and the spec still does not compile. These are a *facet* of the pending
  `default_input` decision (open since W561, 110 specs), not independent defects.
- **3 aggregate-vs-scalar** — the `ternary_mac` calling convention, split 91-to-80
  inside the module that declares it (W574 §2). The RTL cannot arbitrate; the golden
  model's proof binds ports by name.

**The check has been driven to the point where every remaining finding is a question
for a human.** That is the correct place to stop.

---

## 4. Verification

| Gate | Result |
|---|---|
| Parse, 608 non-scratch specs, per-file vs W568 | `341 → 351`, **0 regressions** |
| Harness | `ALL_PASS 23, TEST_FAIL 0, COMPILE_FAIL 178` |
| Assertions emitted | 4,393 → **4,403** |
| `t27c check-calls` | 38 → **32** |
| Generated Verilog, FPGA + board specs vs W568 | 17 byte-identical, 1 strictly larger |
| T1 / T2 / T3 | re-proved |
| Freeze ceremony | performed on the `compiler.rs` edit |

---

## 5. Three cooperation variants for W576

### Variant A (recommended) — Audit what else the lexer silently reshapes

`1e6` was wrong for the entire life of the project and nobody saw it, because a
mis-lexed *value* only shows up if something checks it. The same lexer accepts `x`,
`b` and `_` **anywhere inside a number**, not just as a prefix:

```
1x2   1b0   1_2_3   0b12   0x
```

Some of those are legal, some are nonsense that currently lexes as a single Number
token. Write a lexer conformance spec — a table of inputs and the token sequence each
must produce — and run it as a test. This is the class of defect that survives longest,
and it is the one the project has the least instrumentation for.

**Metric:** number of inputs in the table whose actual tokenisation differs from the
declared one.

### Variant B — Extend `check-calls` to return types

Arity and aggregate-vs-scalar are sound but shallow. The next sound step needs no
inference either: a call used as the initializer of a binding with a declared type, or
returned from a function with a declared return type, can be checked against the
callee's declared return. That would catch the class where a `void` function's result
is bound and asserted on — which the `default_input` scaffold does 571 times.

### Variant C — Flash the board

Unchanged. Bitstream at 150.63 MHz, `fpga-flash --dry-run` correctly reporting
`BLOCKED -- no programmer on USB`, three theorems re-proved.

---

## Recommendation

**Variant A.** A lexer that silently reshapes literals is the most dangerous component
in this compiler, and it is the only one with no conformance test at all. The `1e6`
bug was found by accident, by a checker built for something else — that is not a
strategy.

---

*φ² + φ⁻² = 3 | TRINITY*
