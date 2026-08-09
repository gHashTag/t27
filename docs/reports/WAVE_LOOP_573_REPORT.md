# Wave Loop 573 Report — 335 tests green, and the corpus is split down the middle on how to call its own MAC

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_572_REPORT.md`](WAVE_LOOP_572_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
tests executing and passing   280  ->  615     (+335)
specs fully passing            22  ->   23
TEST_FAIL                       1  ->    0
non-scratch parse OK          341  ->  351     (0 regressions vs W568)
T1 / T2 / T3                  re-proved
```

`specs/igla/race/adder_tree.t27` is the first IGLA RACE kernel to go **fully green:
All 335 tests passed.** It more than doubles the project's executing test count on its
own.

And answering the question that made it possible turned up a bigger one: **the corpus
is split roughly evenly between two incompatible calling conventions for
`ternary_mac`**, and the machine-checked RTL says which is right.

---

## 1. The overflow question, settled from the record

W572 left one blocking question and named `FORMAT-SPEC-001.json` + `gf16.t27` (the L6
numeric SSOT) as the deciding artefacts. Reading them:

- **`FORMAT-SPEC-001.json` says nothing about integer overflow.** Its keys are the
  φ-format family, value formulas, and sacred constants.
- **`gf16.t27` specifies *float* overflow** (saturate to Inf), not integer.

By W572's own falsification condition that would make this a constitutional amendment.
It is not — because the language already decided, and the decision is recorded in
[`docs/NOW.md`](../NOW.md):

> *"Adds the Zig-style wrapping-operator family … `+%`/`-%`/`*%` … Rust →
> `wrapping_add/sub/mul`; Verilog collapses to `+`/`-` (HW wraps by width) … Checked
> `+`/`-`/`*` stay infix → same overflow-panic semantics as the Zig backend."*

So: **plain `+` traps, `+%` wraps, and both already work end to end.** The 43
overflow-named tests were not asserting the wrong thing about the language — the
*functions they exercise* were spelled with the trapping operator while modelling
hardware that wraps.

## 2. The fix, and why it is provably free in hardware

`adder_tree`, `ternary_mac` and `systolic_ternary` model FPGA datapaths. There is no
trap in an LUT carry chain, and their own tests say so:

```t27
test adder_tree_4_i32_max_overflow
    given a = 2147483647
    given b = 2
    when sum = adder_tree_4(a, b, 0, 0)
    then sum == -2147483647
```

Their arithmetic now uses `+%` / `-%`. **The Verilog backend collapses those to `+` /
`-`, so the generated RTL does not change** — verified by regenerating each spec with
and without the change:

| Spec | RTL diff |
|---|---|
| `adder_tree.t27` | **byte-identical** (3,641 lines) |
| `ternary_mac.t27` | one line: `-a` → `(0 - a)` — the same operation, same width |
| `systolic_ternary.t27` | one temporary's line-numbered name (`__tup_l43` → `__tup_l54`), from the added doc comment |

Only the software backends change, and only for inputs that overflow. Non-overflowing
arithmetic is bit-identical, which is why no existing test could break — and none did.

**Result: `adder_tree.t27` — All 335 tests passed.**

---

## 3. The finding: two conventions for `ternary_mac`

`ternary_mac.t27` declares

```t27
fn ternary_mac(acc: i32, a: i8, w: TernaryWeight) -> i32
```

and `ternary_gemm.t27` calls it as

```t27
let c00 = ternary_mac(a[0], w[0], ternary_mac(a[1], w[2], 0));
```

— activation, weight, accumulator. Counting every call site in the corpus:

| First argument looks like | Sites |
|---|---:|
| an **accumulator** — matches the `.t27` declaration `(acc, a, w)` | 117 |
| an **activation** — matches `(a, w, acc)` | 126 |

(The classifier scores a bare numeric literal as accumulator-like, so the second row is
if anything under-counted.) **The corpus is split down the middle on how to call its
own most important function**, with tests written for both conventions, and it was
completely undetectable until W569 made `use` resolution real — before that, every
generated file simply had no `ternary_mac` in it at all.

### The deciding artefact, read

`fpga/formal/ternary_mac_golden.v` is the model T1 (exact equivalence) and T2 (zero
DSP48) are proved against — the authoritative statement of what a ternary MAC *is* in
this project:

```verilog
module ternary_mac_golden (
    input  wire        clk,
    input  wire        rst_n,
    input  wire        en,
    input  wire signed [7:0]  a,        // activation
    input  wire        [1:0]  w_code,   // weight
    input  wire signed [31:0] acc_in,   // accumulator
    output reg  signed [31:0] acc_out
);
```

**`(a, w, acc)`.** The machine-checked hardware disagrees with the spec's declaration
and agrees with the larger half of its call sites.

That makes the `.t27` declaration the outlier, and the repair determined: change the
signature to `fn ternary_mac(a: i8, w: TernaryWeight, acc: i32) -> i32`, then fix the
117 call sites and the tests written against the old order. It is a large, mechanical,
fully-determined edit — and it is W574, not a footnote to this wave.

---

## 4. Verification

| Gate | Result |
|---|---|
| Harness | `ALL_PASS 23, TEST_FAIL 0, COMPILE_FAIL 178` |
| Tests executing and passing | **280 → 615** |
| Parse, 608 non-scratch specs, per-file vs W568 | `341 → 351`, **0 regressions** |
| Generated RTL, per changed spec, with vs without `+%` | identical or provably equivalent (§2) |
| Generated Verilog, FPGA + board specs vs W568 | 17 byte-identical, 1 strictly larger |
| T1 / T2 / T3 | re-proved |

---

## 5. Three cooperation variants for W574

### Variant A (recommended) — Unify the `ternary_mac` calling convention

Determined by `fpga/formal/ternary_mac_golden.v`: the signature becomes
`(a, w, acc)`. Then 117 call sites and their tests move to match.

**Deliverables.**
1. Flip the declaration in `ternary_mac.t27`.
2. Rewrite `(acc, a, w)` call sites — including the nested ones in `ternary_gemm.t27`,
   where the accumulator is itself a `ternary_mac(...)` call.
3. Re-run the harness. `ternary_mac.t27`, `ternary_gemm.t27` and `systolic_ternary.t27`
   are **833 substantive assertions** between them.

**What would falsify it.** If the golden model's port *order* is incidental rather than
normative — Verilog ports are named, not positional — then the RTL does not settle the
question and it returns to being a specification decision. Check whether
`prove_ternary_mac.ys` binds by name or by position before relying on it.

### Variant B — Extend wrapping arithmetic to the rest of the family

`systolic_array.t27` and `ternary_gemm.t27` carry the same wrap-asserting tests
(`systolic_step_i32_max_accumulation_wrap`, `booth_mul_i16_both_negative_overflow`) and
the same trapping `+`. The pattern and the RTL-identity check are established; this is
the same edit applied four more times, gated the same way.

### Variant C — Flash the board

Unchanged. Bitstream at 150.63 MHz, preflight correctly reporting
`BLOCKED -- no programmer on USB`, all three theorems re-proved this wave.

---

## Recommendation

**Variant A**, with its falsification check done first. A project cannot have two
conventions for calling the function its own name is built on, and for once the
disagreement has a machine-checked arbiter sitting in the repository.

---

*φ² + φ⁻² = 3 | TRINITY*
