# Wave Loop 602 — the SSOT the compiler could not see

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_601_REPORT.md`](WAVE_LOOP_601_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary — all three variants, plus two theorems

```
A  t27c catalog-gate     83 records, 7 check kinds, 5 real findings
B  the stub population   L4 debt corrected: 25 of 38 were never specs
C  the board             dlc10 idcode -> cable not found (VERIFIED, not assumed)

T7  the GF rule is exact before rounding, optimal on every published rung,
    and NOT a minimiser in general -- it fails at N = 5, 73, 1293
T8  why 1/phi, and what phi^2 + phi^-2 = 3 has to do with the field split
P17 five records assert a field layout they do not have
```

Plus [`docs/fpga/IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md) —
the standing FPGA request, answered as a plan rather than as "blocked".

---

## 1. Variant A — the payload no compiler can see

W601 recommended `formats_catalog.t27`: 83 functions, 32 KB, zero tests. The
falsification check found something better than a plan:

```t27
fn binary16() -> str { return "binary16"; }
fn binary32() -> str { return "binary32"; }
```

**All 83 functions are that.** The entire payload is in structured comments:

```
// CATALOG: id=gf10 name="GF10 (rule-derived)" bits=10 s=1 e=3 m=6 bias=3
//          phi_distance=0.118 cluster=GoldenFloat source="specs/numeric/gf10.t27"
```

The file's own header explains why — struct literals were not parseable when it
was written, so "per-format records live as fn getters that the codegen reads
from the AST". **The consequence is that W601's approach cannot work here.**
Invariants check `const` declarations; there are none. This needed a command.

### The exceptions are the deliverable

The obvious check, `s + e + m == bits`, reports **13 violations — and twelve are
not violations**:

| Class | n | Why the rule does not apply |
|---|---:|---|
| **Tapered** — `posit*`, `takum*` | 8 | variable-length regime; `e` is the *es* parameter and **there is no fixed *m*** |
| **Parametric** — `bits=0` | 4 | `q_format`, `minifloat`, `unum_i`, `tapered_fp` are families, not formats |
| **Alphabet** — `bits < 4` | 1 | `gfternary` is the 3-value set {−φ, 0, +φ} |

A gate emitting thirteen false alarms is a gate switched off within a wave. The
classification **is** the work; the arithmetic is four lines.

### Skipping a case is worse than a false alarm

The first version simply `continue`d past the non-fixed shapes and reported
**zero findings**. That converts a false alarm into a *silent exemption*, which
is strictly worse — the data is still there and still wrong. Replacing "skip"
with **"must not claim a layout it does not have"** found five real defects:

```
gfternary   Alphabet    bits=2  s=1 e=0 m=2  (sum 3)   status=Verified
q_format    Parametric  bits=0  s=1          (sum 1)
minifloat   Parametric  bits=0  s=1          (sum 1)
unum_i      Parametric  bits=0  s=1          (sum 1)
tapered_fp  Parametric  bits=0  s=1          (sum 1)
```

A 3-value alphabet has no exponent/mantissa decomposition, so `gfternary`'s
`s=1 m=2` is data no reader can act on — in a record marked `status=Verified`.
**What these should say is a specification decision**, so they are reported, not
silently changed. Recorded as **P17**.

### What the gate proves clean

| Check | Population | Findings |
|---|---:|---:|
| `mandatory-field` | 83 | 0 |
| `widths-partition` | 65 | 0 |
| `gf-closed-form` | 21 | 0 |
| **`gf-ratio-optimal`** (T7) | 21 | 0 |
| `gf-phi-distance` | 21 | 0 |
| `source-agrees` | 10 | 0 |
| `no-spurious-layout` | 10 | **5** |

## 2. T7 — check the property, not the procedure

The GF rule is `e = round((N−1)/φ²)`. **Part 1: that is not an approximation of
the design goal — it is the goal.** Solving `e/m = 1/φ` with `m = N−1−e`:

```
phi*e = N-1-e  =>  e*(phi+1) = N-1  =>  e = (N-1)/phi^2      [phi^2 = phi+1, L5]
```

**Part 2: rounding the root is not minimising the error.** `|e/(N−1−e) − 1/φ|`
is nonlinear, so the nearest integer to the root need not minimise it.
Exhaustive over every integer *e* for every `N ∈ [4, 4000]`:

| | |
|---|---:|
| widths tested | 3 997 |
| where the rule is **not** the minimiser | **3** |
| | **N = 5, 73, 1293** |

**Part 3: every published rung is outside that set**, so all 21 fixed-layout GF
records are ratio-optimal, not merely rule-conformant.

**Part 4, and this is the interesting half:** all three exceptions sit near a
half-integer with fractional part above ½ — but that is **necessary, not
sufficient**. `N = 3877` has fractional part 0.500260, *nearer* to ½ than
`N = 73`'s 0.501553, and is **not** an exception. There is no simple predicate.

**So the gate checks the property the ladder wants — ratio-optimality — by
searching every integer *e*, rather than re-running the formula.** A rung added
at N = 73 by applying the published rule would be suboptimal by the ladder's own
criterion, and nothing before this wave would have noticed.

## 3. T8 — why 1/φ

For `e + m = N−1` with `e/m = 1/φ`, we get `m = φe` and `e + m = eφ²`, so

```
(e + m)/m = phi = m/e
```

— the defining proportion of the golden section: **the whole is to the larger
part as the larger part is to the smaller.** The anchor `φ² + φ⁻² = 3` is the
same identity in reciprocal form, and it is what lets the ladder's arithmetic be
stated without a transcendental constant. All 21 rungs' recorded `phi_distance`
agrees with `|e/m − 1/φ|` to within 0.0015.

## 4. Variant B — the L4 debt was overstated by nearly double

`test-report` now separates **STUB** (declares nothing at all — no `fn`, no
`const`, no `struct`) from **NO TESTS** (has declarations, checks none).

```
specs/tri/    17 STUBS    2 NO TESTS
```

matching W601's manual count exactly. A 327-byte file with a module header and
an empty `TDD: Tests` banner is **unwritten** — W586's category — and the remedy
is to write the spec, not to add a test to a file with nothing in it.

Five populations now, deliberately not merged: MEASURED / INVARIANTS ONLY /
NO TESTS / STUB / BLOCKED.

## 5. Variant C — the board, verified with the real tool

```
$ dlc10 idcode
Error: open DLC10
Caused by: DLC10 cable not found (VID=0x03FD)
```

Not assumed — run. And this wave answers the standing FPGA request properly:
[`docs/fpga/IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md) records
that **Phase 0 is complete** (T1 equivalence, T2 multiplier-freedom, T3 timing,
FPGA specs 300/300), gives the ordered Phase 1 (`idcode` → `sram` → readback →
`flash` → `reload`, and *why* `idcode` must not be skipped), and states what
would falsify each claim once silicon is available — including that **T2 is a
netlist property and a DSP48 in the utilisation report would refute it on the
real toolchain.**

## 6. Literature — where GoldenFloat actually sits

There is a real and old literature on φ as a **radix**: Bergman's base-φ system
(1957), and Zeckendorf's theorem on unique non-consecutive Fibonacci sums.

**GF is not that, and the distinction bounds the claim in both directions:**

| | φ appears in | consequence |
|---|---|---|
| Bergman / Zeckendorf | the **radix** | non-standard arithmetic, no direct hardware analogue |
| **GoldenFloat** | the **field split** | **ordinary binary floats** — any existing FPU datapath shape applies |

GF is *less* novel than a new number system and *more* usable for it. What the
ladder demonstrably has is not a performance result — 9 of 22 GF entries are
`status=Open`, and `PHI_BIAS` is explicitly **retracted** in every rung's own
comment — but **internal consistency, now machine-checked end to end**: 21 rungs,
four independent properties each, across two files that state the constants
separately.

## 7. Verification

| Gate | Result |
|---|---|
| `catalog-gate` | 83 records · 83 getters · 7 checks · 5 findings |
| `cordic.t27` | 330 / 336 |
| `adder_tree.t27` | 335 / 335 |
| `lex-conform` / `parse-conform` | 29/29 · 13/13 |
| `cc-gate` | 101 |
| new unit tests | 6, incl. `the_rule_is_not_the_minimiser_at_5_73_and_1293` |

**Corpus-wide, with the corrected populations:**

```
specs MEASURED           30      (29 at 100%)     unchanged
specs INVARIANTS ONLY     9                       unchanged
specs with NO TESTS       4      <- was 38, then 29
specs that are STUBS     25      <- new population
specs BLOCKED           540                       unchanged

1024 tests / 1018 pass / 6 fail / 99.4%           unchanged
invariants proved       445                       unchanged
```

**The L4 debt is 4 files, not 38** — `formats_catalog.t27`, `clocked_counter.t27`,
`faculty_board.t27`, `error.t27`, exactly the four W601 predicted by hand, now
confirmed by the tool.

### One pre-existing failure set, verified not mine

`cargo test -p t27c` has **5 failing unit tests** in `compiler::tests_w458`
(Verilog backend — `set(1, 43981);` not emitted as a bare statement, array
parameter binding, ROM-style pragma). Verified pre-existing by stashing this
wave's work and re-running: **identical 5 failures**. They trace to the wave that
introduced them (`91ef62549`, W459). Reported here because nothing else reports
them — `./scripts/tri test` and the conformance gates are all green, so this set
is invisible to every check this chain has built.

---

## 8. Three cooperation variants for W603

### Variant A (recommended) — Settle the five `no-spurious-layout` records

Five records in the numeric SSOT assert field widths they do not have, one of
them `status=Verified`. Each needs one decision — *what should a parametric
family record for s/e/m?* — and the honest answers are probably "omit the
fields" and, for `gfternary`, "record the alphabet size, not a layout".

It is small, it is bounded, and it is the only open item in the catalog. The
gate already names all five and will confirm the fix.

### Variant B — Run `catalog-gate` against the six generated targets

The catalog exists to feed Markdown, JSON, Python, Rust, C and TypeScript via
`tools/gen_formats_catalog.py`. This wave verified the **source**. Nothing has
verified that the **emitted files still agree with it** — and W602's own finding
is that a payload the compiler cannot see is a payload nothing checks. The same
argument applies one layer down, with the same fix: make it a command.

### Variant C — Flash the board

Now backed by a written plan rather than a verdict:
[`IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md). Phase 0 is
complete; Phase 1 begins with `dlc10 idcode` and must not skip it.

---

## Recommendation

**Variant A.** B is worth doing and is larger than it looks; C needs hardware.
A is five decisions with the arithmetic already done — the same shape as every
other item this chain has handed back to an owner, and the smallest of them.

---

*φ² + φ⁻² = 3 | TRINITY*
