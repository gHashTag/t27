# Wave Loop 574 Report — the falsification check fired, so I built the arbiter instead

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_573_REPORT.md`](WAVE_LOOP_573_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W573 recommended unifying `ternary_mac`'s calling convention on `(a, w, acc)`, on the
authority of the machine-checked golden RTL — and attached a falsification condition:

> *"Verilog ports are named, not positional. Check whether `prove_ternary_mac.ys` binds
> by name or by position before relying on it."*

**It binds by name.** `yosys miter -equiv` pairs ports by identifier, so the golden
model's port *order* carries no meaning at all, and the RTL does not settle the
question. Variant A's premise was false and the wave did not do it.

What it did instead: made the whole defect class visible, corpus-wide, without
deciding anything.

```
t27c check-calls          38 findings   (35 arity, 3 aggregate-vs-scalar)
suite Phase 6             reports them on every run
tests executing/passing   615      (unchanged)
non-scratch parse OK      351      (0 regressions vs W568)
T1 / T2 / T3              re-proved
```

---

## 1. The check that fired

```yosys
miter -equiv -flatten -make_assert ternary_mac_top ternary_mac_golden miter_mac
```

`miter -equiv` builds its equivalence relation from **port names**. `a`, `w_code`,
`acc_in` in one module are matched to `a`, `w_code`, `acc_in` in the other regardless
of declaration order; reordering either module's port list would not change what T1
proves. So the RTL says what a ternary MAC *computes* — and says nothing about the
order a t27 function should take its arguments in.

W573's conclusion that "the machine-checked hardware makes the `.t27` declaration the
outlier" is withdrawn. It was based on reading port order as normative, and it is not.

## 2. What the evidence actually says

Re-counted by argument **type** rather than position — a call whose second argument is
a weight is `(a, w, acc)`; one whose third is a weight is `(acc, a, w)`:

| Spec | `(acc, a, w)` — as declared | `(a, w, acc)` |
|---|---:|---:|
| `specs/igla/race/ternary_mac.t27` — the owning module | **91** | **80** |
| `specs/igla/race/ternary_gemm.t27` | 0 | 72 |
| `specs/igla/race/systolic_ternary.t27` | 1 | 0 |

**The module that declares the function is itself split, 91 to 80.** This is not one
file drifting from a convention; both conventions have substantial test bases inside
the owning spec, and choosing either makes roughly eighty tests wrong.

With the RTL out as arbiter, and both call-site populations large, **this is a
specification decision.** It is recorded here and left to the maintainer.

## 3. So build the thing that would have caught it

The reason this survived is not that it was subtle. It is that **nothing in this
project ever compared a call to the declaration it targets.** Before W569 made `use`
resolution real, a foreign callee was simply absent from every generated file, so a
wrong call and a missing one were indistinguishable. And even now, a mismatch is only
caught in specs that compile all the way through Zig — 23 of 199.

`t27c check-calls` closes that gap. It reports only what is decidable from the AST,
with no inference and no semantic choices:

- **arity** — a call passing a different number of arguments than the declaration
  takes. Sound.
- **aggregate-vs-scalar** — a struct literal passed where the declaration names a
  scalar, or the reverse. This is exactly what distinguishes the two `ternary_mac`
  conventions, without deciding between them.

Signatures are resolved from the spec itself and from every module it `use`s.

### What it found

```
  aggregate-vs-scalar    3
  arity                 35
  TOTAL                 38
```

Full output: [`data/W574-call-site-check.txt`](data/W574-call-site-check.txt).

The arity findings are unambiguous defects — no convention debate is possible about
passing seven arguments to a four-parameter function:

| Site | Finding |
|---|---|
| `specs/physics/gamma_conjecture.t27:211` | `verify_gamma_conjecture` — 7 passed, 4 declared |
| `specs/vsa/sdk.t27:445` | `hypervector_set` — 2 passed, 3 declared |
| `specs/server/mdns.t27:238` | `concat` — 3 passed, 2 declared |
| `specs/tri/trees/avl_tree.t27:58` and 5 others | `init(input)` — 1 passed, 0 declared |

Six of the 35 are `init(input)` — the `default_input()` template scaffold calling
zero-argument constructors with an argument. That is the same scaffold that blocks 110
specs, now visible as a *type* error rather than only as a missing helper.

Call nodes also carried no line number, so a finding could name only the file; they do
now.

### Made permanent

Suite Phase 6 prints it on every run, alongside the other integrity metrics:

```
  call-site mismatches: 35 arity, 3 aggregate-vs-scalar (t27c check-calls)
```

Reporting only, per the standing rule that turning these into hard gates is the
maintainer's decision.

---

## 4. Verification

| Gate | Result |
|---|---|
| Parse, 608 non-scratch specs, per-file vs W568 | `341 → 351`, **0 regressions** |
| Harness | `ALL_PASS 23, TEST_FAIL 0, COMPILE_FAIL 178` |
| Tests executing and passing | 615 |
| Generated Verilog, FPGA + board specs vs W568 | 17 byte-identical, 1 strictly larger |
| T1 / T2 / T3 | re-proved |
| Freeze ceremony | performed on the `compiler.rs` edit |

---

## 5. Three cooperation variants for W575

### Variant A (recommended) — Fix the 35 arity mismatches

Every one is a defect with no design question attached, each is one line, and the
check re-runs in seconds to confirm. Six are the `init(input)` scaffold, which is the
`default_input` family showing up as a type error; the rest are scattered and
individually decidable.

**Metric:** `t27c check-calls` total, 38 → target 3 (the `ternary_mac` findings, which
are the specification decision below).

### Variant B — Decide the `ternary_mac` convention

Both conventions are live inside the owning module, 91 against 80, and the RTL cannot
arbitrate. Whichever is chosen, the other ~80 call sites and their tests move. The one
useful piece of new evidence would be the *host-side* driver or ISA documentation, if
either states an argument order — neither was found in this wave.

**This is a maintainer's call.** It changes what a large part of the RACE corpus
asserts.

### Variant C — Flash the board

Unchanged. Bitstream at 150.63 MHz, preflight correctly reporting
`BLOCKED -- no programmer on USB`, three theorems re-proved.

---

## Recommendation

**Variant A.** Thirty-five unambiguous defects, newly visible, with a re-runnable
metric — and none of them requires anyone to decide anything.

---

*φ² + φ⁻² = 3 | TRINITY*
