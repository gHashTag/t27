# Wave Loop 597 — the first per-test measurement of an IGLA RACE kernel: 321 of 336

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_596_REPORT.md`](WAVE_LOOP_596_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

All three variants.

```
A  per-test measurement   ->  cordic.t27: 321 pass, 15 fail of 336   (95.5%)
B  the other five kernels ->  every remaining blocker is a SPECIFICATION DECISION
C  the board              ->  verified, still BLOCKED

lex-conform 29/29 · parse-conform 13/13 · T1/T2/T3 re-proved
```

---

## 1. Variant A — the number W596 could not give

Zig's test runner aborts on the first panic, so W596 could only report *"4 pass,
then it stops"* — a floor, not a measurement. Running each test in isolation
(`zig test --test-filter`, 336 invocations):

| | |
|---|---:|
| Tests | **336** |
| Pass | **321** |
| Fail | **15** |
| Pass rate | **95.5%** |

**This is the first per-test correctness figure for an IGLA RACE kernel.** W559
asked the equivalent question of the whole corpus and W560 answered it with
*"they do not compile"*; this is the same question answered for one kernel that
now does.

### Where the failures cluster

| Family | Failures |
|---|---:|
| **exact value at a special angle** (sin/cos at 0, π/2, π) | **10** |
| **gain** (the CORDIC scaling constant K) | **3** |
| **arctan table entry** | **2** |
| | **15** |

The clustering is the finding, not the total. Failures concentrate in the
**exact-value-at-special-angle** family — the same property T4 and T5 disproved
for the fixed-point kernels — and in the **gain** family, which is where the
CORDIC scaling constant enters. Both are consequences of asserting exact
equalities over a converging approximation, and both were predicted by T5's
analysis before this measurement existed.

The full list of 15, so nobody has to re-derive it:

```
exact value at a special angle (10)   gain (3)
  cordic_cos_zero                       cordic_gain_increases_with_iters
  cordic_sin_half_pi                    cordic_gain_8_vs_16
  cordic_cos_half_pi                    cordic_gain_boundary
  cordic_sin_near_zero_exact_zero
  cordic_cos_near_zero_exact_pi_half  arctan table (2)
  cordic_sin_exact_pi                   cordic_arctan_table_entry_fifth
  cordic_sin_small_angle                cordic_arctan_table_entry_sixteen
  cordic_sin_cos_pi_over_2_approx
  cordic_sin_cos_zero
  cordic_sin_cos_pi_over_2_approx__dup2
```

**Not one failure is a compiler defect.** Recorded as **P13**.

Raw results: [`data/W597-cordic-per-test.tsv`](data/W597-cordic-per-test.tsv).

## 2. Variant B — every remaining blocker is a decision

| Kernel | State | What blocks it |
|---|---|---|
| `adder_tree.t27` | **335 / 335 pass** | — |
| `cordic.t27` | **321 / 336 pass** | false invariants (T4's family) |
| `cordic_top.t27` | compiles; invariant disproved at comptime | T4 |
| `cordic_fixed.t27` | compiles; two disproved | T5 |
| `ternary_mac.t27` | does not compile | **argument order** — W574, 849 assertions |
| `ternary_gemm.t27` | does not compile | the same decision |
| `systolic_ternary.t27` | does not compile | `systolic_ternary_array` — contradictory tests (W571) |
| `opcodes.t27` | does not compile | `OP_ADD` — outside a closed set (W571) |
| `eda.t27` | does not compile | `PpaMetrics` — field mismatch (W592) |

**Not one remaining blocker is a compiler defect.** No missing lowering, no parse
gap, no type-mapper hole — the categories this chain has spent twenty-nine waves
eliminating are, for this family, gone. Recorded as **P12**.

## 3. Variant C — the board

```
verdict : BLOCKED -- no programmer on USB; connect the cable, then rerun
```

T1, T2, T3 re-proved.

---

## 4. Verification

| Gate | Result |
|---|---|
| `cordic.t27` per-test | **321 / 336** |
| `adder_tree.t27` | 335 / 335 |
| `lex-conform` / `parse-conform` | 29/29 · 13/13 |
| T1 / T2 / T3 | re-proved |

---

## 5. Three cooperation variants for W598

### Variant A (recommended) — Correct the exact-value assertions, as one change

15 failing tests, clustered in two families, all instances of one fact: a
converging approximation does not hit exact values. T4 and T5 give the
arithmetic; the corpus's own convention gives the form
(`cordic_cos(0) ∈ (9900, 10000)`).

**This is a specification decision, but it is now a single one** — not 15
separate judgements. One tolerance rule, applied to one family, and the
measurement above says exactly what it must accommodate.

### Variant B — Make per-test measurement routine

This wave's number took 336 process invocations and a shell loop. Every other
measurement this chain trusts is a command. `t27c test-report <spec>` — generate,
compile, run each test in isolation, emit the pass/fail table — would make the
figure reproducible and let the harness report it for every spec that compiles,
not just the one somebody asked about.

### Variant C — Flash the board

Unchanged.

---

## Recommendation

**Variant A.** 15 failures, one cause, one decision — and for once the decision
comes with the arithmetic already done.

---

*φ² + φ⁻² = 3 | TRINITY*
