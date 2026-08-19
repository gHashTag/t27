# Wave Loop 572 Report — the first real test failure this project has ever produced

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_571_REPORT.md`](WAVE_LOOP_571_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
harness   ALL_PASS 22   TEST_FAIL 1   COMPILE_FAIL 178
                        ^^^^^^^^^^^
non-scratch parse OK   341 -> 351   (0 regressions vs W568)
generated Verilog      17 identical, 1 strictly larger
T1 / T2 / T3           re-proved
```

`TEST_FAIL 1` is the point of this wave. Since W549 this chain has been chasing one
number — *how many of the project's tests fail once they can actually run* — and every
wave has answered "unknown, they don't compile yet." **`specs/igla/race/adder_tree.t27`
now compiles, runs 335 tests, passes 32, and fails the 33rd.**

---

## 1. What it fails on

```
33/335  adder_tree_4_i32_max_overflow ... thread panic: integer overflow
```

```t27
test adder_tree_4_i32_max_overflow
    given a = 2147483647
    given b = 2
    when sum = adder_tree_4(a, b, 0, 0)
    then sum == -2147483647
```

The test asserts **two's-complement wrap-around**. The Zig backend emits `+`, which
traps on overflow in safe builds. Both are defensible and they cannot both be right:

- The FPGA target wraps by construction — an adder in `fpga/verilog/` has no trap.
- A trapping `+` is what catches the overflow bugs W565 found in the simulator.

**206 specs mention overflow or wrapping, and 43 tests are named for it.** This is a
numeric-semantics question for the whole language, not an `adder_tree` bug.

**Deciding artefact:** `docs/T27-CONSTITUTION.md` L6 names `FORMAT-SPEC-001.json` and
`gf16.t27` as the numeric SSOT. Whichever they specify — wrapping (`+%`), trapping, or
an explicit `wrapping_add` — is what the backend should emit, and the 43 tests should
be read against it. Nothing here should be decided by whichever choice makes the
harness green.

## 2. The two bugs that got it there

### The receiver-losing postfix call — a silent-wrong-code defect

```
ternary_gemm([...], [...]).len() == 4      ->  emitted  len()
```

`flatten_field_access_name` folds a receiver chain into a dotted callee *name*. It
walks identifiers and field accesses, and on anything else — a call, an index — it
**stopped and dropped the receiver without a diagnostic**. In this corpus:

| Receiver shape | Count |
|---|---:|
| `<call>.method()` — no arguments | **198** (161 of them `.len()`) |
| `<call>.method(args)` | ~40 |
| `<index>.method()` | 18 |

It failed loudly here only because `len` happens to be undeclared. **With a method name
that resolves, it would have called the wrong thing on nothing.** A receiver that
cannot be folded into a name is now kept as the call's first child and emitted as a
real method call.

This is the second time this chain has found the compiler discarding input silently;
W569's stray-brace truncation was the first.

### An array literal that cannot be a slice

`.{ 0, 0, 0, 0 }` coerces to `[4]i32` but never to `[]i32`, and the element type is not
in the literal — only the callee's signature has it. Codegen now records declared
parameter types and uses them in two places:

1. **A literal argument** → `@constCast(&[_]i32{ … })`. `&[_]T{…}` is `*const [N]T`,
   which reaches `[]const T` but not the mutable `[]T` these signatures declare, and a
   temporary has no addressable `var` to point at.
2. **A local bound to a literal and later passed as a slice** → declared
   `var x = [_]i32{ … };` and passed `&x`. It must be `var` for exactly the same
   const-ness reason.

---

## 3. What this exposed next: a cross-module argument-order mismatch

With `use` resolution live (W569), `ternary_gemm.t27` and `ternary_mac.t27` now fail
with `expected type 'i8', found 'TernaryWeight'`. `ternary_mac.t27` declares

```t27
fn ternary_mac(acc: i32, a: i8, w: TernaryWeight) -> i32
```

and `ternary_gemm.t27` calls it as `ternary_mac(a[0], w[0], acc)` — activation, weight,
accumulator. **The caller's argument order does not match the callee's signature.**

This was undetectable before W569, because nothing crossed a module boundary: each
spec generated a file in which `ternary_mac` was simply undeclared. The definition is
authoritative — it lives in the module that owns the function — so the call sites are
what is wrong, and the fix is mechanical.

---

## 4. Verification

| Gate | Result |
|---|---|
| Parse, 608 non-scratch specs, per-file vs W568 | `341 → 351`, **0 regressions** |
| Harness | `ALL_PASS 22, TEST_FAIL 1, COMPILE_FAIL 178` |
| Assertions emitted, 201 BDD specs | 4,393 |
| Generated Verilog | 17 byte-identical, 1 strictly larger |
| T1 / T2 / T3 | re-proved |
| Freeze ceremony | performed on every `compiler.rs` edit |

`ALL_PASS` is still 22 and that is correct: `adder_tree` is not passing, it is
**failing**, which is strictly more informative than not compiling.

---

## 5. Three cooperation variants for W573

### Variant A (recommended) — Settle integer overflow semantics

The one blocking question, now backed by a running test. Read
`FORMAT-SPEC-001.json` and `gf16.t27` (the L6 numeric SSOT), decide whether t27's `+`
on a sized integer wraps or traps, and then:

- if **wrapping**: emit `+%`/`-%`/`*%` for sized integer arithmetic, and re-run the 43
  overflow-named tests as a suite — several will move from "cannot run" to a real
  verdict at once;
- if **trapping**: the 43 tests assert the wrong thing and should be rewritten against
  a `wrapping_add` the language would then need.

**What would falsify the premise:** if the SSOT is silent on overflow, this is a
constitutional amendment rather than an implementation choice, and it should be raised
as one.

### Variant B — The `ternary_mac` argument-order mismatch

Reorder the call sites in `ternary_gemm.t27` (and audit `ternary_inference.t27`) to
match the owning module's signature. Mechanical, determined by the definition, and it
unblocks two of the heaviest RACE kernels — **545 substantive assertions**.

Worth doing as an audit rather than a patch: `use` resolution has only just made
cross-module call signatures checkable at all, and this is unlikely to be the only one.

### Variant C — Flash the board

Unchanged. Bitstream at 150.63 MHz, preflight correctly reporting
`BLOCKED -- no programmer on USB`, all three theorems re-proved this wave.

---

## Recommendation

**Variant A.** A running test that fails is worth more than nine that cannot run, and
this one asks a question the whole corpus depends on. Answer it from the numeric SSOT,
not from what makes the gate green.

---

*φ² + φ⁻² = 3 | TRINITY*
