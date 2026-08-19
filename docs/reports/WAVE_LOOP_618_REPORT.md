# Wave Loop 618 — all three variants, and a fourth state a failing test can be in

**Date:** 2026-08-11 · **Predecessor:** [`WAVE_LOOP_617_REPORT.md`](WAVE_LOOP_617_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
A  instrumented the struct-method gap   -> two hypotheses ELIMINATED
B  `no field named` = 51 errors, ~8 structs, one dominant
C  the board                            -> cable not found, verified

T9  UNSATISFIABLE is a fourth state, distinct from UNDERDETERMINED
```

---

## 1. Variant A — instrumented, as W617 said to

`t27c parse` already dumps the AST, so no code change was needed to *look*.

| Question | Answer |
|---|---|
| Is `parse_struct_body` reached for `struct W { fn f() … }`? | **yes** — traced |
| Does the loop see the method's token? | **yes** — exactly one `KwFn "fn"` |
| Does an `else if KwFn` branch in that chain fire? | **no** — a probe inside it never prints |
| Does a `FnDecl` child appear? | **no** — the `StructDecl` has zero children |
| Does the loop iterate again? | **no** — one token, then exit |

**This eliminates the two hypotheses W617 could not choose between:** the parser
*is* reached, and the emitter was never the issue. What remains is a precise,
reproducible anomaly — the loop sees `KwFn`, a branch matching `KwFn` does not
fire, and the whole method is consumed in that one iteration.

Reverted with `git checkout`. **No compiler change survives this wave.**

### Two instrumentation errors, recorded

- The first trace landed in **`parse_enum_body`** — a non-unique `while` anchor
  with a first-match replace. A probe that prints nothing may be in the wrong
  function; verify placement before concluding.
- `2>&1 >/dev/null` binds **stderr to the terminal** and stdout to the void — the
  opposite of the intent. Write `>/dev/null 2>file`.

## 2. Variant B — `no field named 'X' in struct 'Y'`

**51 errors across ~8 structs** — unlike the `TernaryWeight` class, not one
shape:

| Field | struct | errors |
|---|---|---:|
| `quality_score` | `DataSample` | **25** |
| `a1` | `SystolicState` | 11 |
| `pass` | `BenchResult` | 3 |
| 5 others | | 12 |

`specs/igla/coder/dataset.t27` declares

```t27
pub const DataSample = struct { prompt : string, rtl : string, template : string };
```

and **its own tests** construct `DataSample { rtl: …, quality_score: …, … }` —
**50 errors in that one file.** A second, unrelated `DataSample` exists in
`training.t27` with a completely different field set.

## 3. Variant C — the board

```
dlc10 idcode  ->  DLC10 cable not found (VID=0x03FD)
```

Run, not assumed. Phase 0 of
[`IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md) remains complete.

## 4. The science — T9, and a fourth state

> **T9.** Let struct `S` be declared with field set `F`, and a test construct
> `S { g: v, … }` with `g ∉ F`. Then **no implementation of `S` satisfies that
> test.** A struct literal in a nominally-typed language denotes a value whose
> field set is exactly `F`; `S { g: … }` is ill-typed, not merely
> unconstrained, and no function body changes `F`. ∎

This completes a **four-state taxonomy**, and the states have different owners:

| State | Example | Remedy | Decidable by |
|---|---|---|---|
| **false assertion** | `K(12) > K(8)` (T5) | fix the test | measurement |
| **real gap** | `cordic_sin(π)` (T6) | write the code | nobody — it is work |
| **underdetermined** | `throughput` (P25), `encode` (P29) | *choose* a contract | an owner |
| **unsatisfiable** | `DataSample { quality_score }` | **drop one of the two** | an owner |

> **Underdetermined admits many implementations; unsatisfiable admits none.**
> Reporting both as "needs a decision" hides that the second **cannot be closed
> by writing code.**

### Where this sits in the literature

The corpus's largest remaining blockers are **schema divergence**, which is well
studied:

- **Nominal vs structural typing** — Cardelli's record calculi, treated
  systematically in Pierce's *Types and Programming Languages*. **T9 depends on
  nominality**; under structural typing the two `DataSample`s would be distinct
  row types and the conflict would surface at the use site.
- **Schema evolution** — the classic treatment of schema modification in
  object-oriented databases (Banerjee et al., 1987) covers exactly the
  unmigrated-instance case: a schema changed on one side and not the other.
- **Wire-format compatibility** — Protocol Buffers and Apache Avro make this
  explicit policy, defining forward/backward compatibility so a reader written
  against one schema can process another's data.

**The gap here is not the type system.** Nominal typing is right for a language
that lowers to Verilog, where a struct *is* a bit layout. The gap is **process**:
two generations of artefacts diverged with no compatibility rule and no
migration step — and it is visible only because the corpus is now compiled
end to end.

Recorded as **T9**, **P33**, and decision-register entries **16** and **17**.

## 5. Verification

| Gate | Result |
|---|---|
| `lex-conform` / `parse-conform` | 34 / 34 · 15 / 15 |
| `parse-complete` | 402 / 608, 0 truncating |
| working tree | clean — no compiler change survives |

---

## 6. Three cooperation variants for W619

### Variant A (recommended) — Migrate `DataSample`, the largest unsatisfiable case

T9 says this cannot be closed by writing code, but it *can* be closed by a
migration once someone says which schema is canonical — and the register now
states the question with both field sets side by side. **50 errors in one file**,
and the remedy is mechanical after the one sentence.

### Variant B — The struct-method anomaly, with a debugger rather than a probe

Three waves have now approached it by editing and observing. The instrumented
answer (P33) narrowed it to one iteration of one loop; what it needs next is
**stepping through that iteration**, not another hypothesis.

### Variant C — Flash the board

Unchanged. Phase 0 complete; the FPGA spec family remains at 300/300.

---

## Recommendation

**Variant A.** It is the largest single item that a one-sentence answer converts
into mechanical work — and T9 is what makes that claim precise rather than
hopeful.

---

*φ² + φ⁻² = 3 | TRINITY*
