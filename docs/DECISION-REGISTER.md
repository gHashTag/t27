# Decision register — what only a maintainer can settle

**Date:** 2026-08-10 · **Waves:** W568–W612 · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Why this file exists

Across forty-five waves, sixteen wave reports have ended by handing something
back with the phrase *"this is a specification decision"*. Each was recorded
where it was found and nowhere else. **This is the first time they are in one
place.**

Every entry below has the same shape:

- it is **not** a compiler defect, a missing lowering, or a parse gap — those
  categories have been eliminated for these items and the elimination is cited;
- the **arithmetic is already done**, so no investigation is needed to answer it;
- it is **one sentence from someone who owns the spec**.

Together they block more than any remaining compiler change.

---

## 1. `ternary_mac` — argument order

**The largest decidable-by-a-human item in the project.**

| | |
|---|---:|
| Call sites in declared order `(acc, a, w)` | **91** |
| Call sites in the other order `(a, w, acc)` | **80** |
| Both inside the module that **declares** it | ✔ |
| Substantive assertions blocked | **849**, across 3 specs |

Found in W574, the first wave in which call sites were ever compared to the
signatures they call. **The RTL cannot settle it** — T1 binds ports by name, so
both orders synthesise identically. What would settle it is a host-side driver
or an ISA document; neither exists.

**Question:** which order is `ternary_mac(a, b, c)`?

---

## 2. `bram_weights_depth` — contradictory tests

| | |
|---|---:|
| Test points | **30** |
| Consistent with `depth == len` | **24** |
| Consistent with `depth == len / 2` | **6** |
| Consistent with neither | 0 |

| input length | expects |
|---:|---|
| 1 | **{0, 1}** |
| 2 | **{1, 2}** |
| 4 | **{2, 4}** |

**Three lengths carry both expectations, so no function satisfies the suite.**
The 24–6 split suggests identity was intended; noting that is not the same as
deciding it (W611).

**Question:** is `bram_weights_depth(data)` the element count, or the row count
at width 2?

---

## 3. `throughput` — the tests do not describe a throughput

Four test points:

```
throughput(0, 1000) == 0.0      throughput(10, 1000)  == 10.0
throughput(1, 1)    == 1.0      throughput(100, 1000) == 100.0
```

Satisfied **only** by `f(ops, ns) = ops` — a function that ignores its duration
argument. No scaled form fits all four: `ops·1000/ns` gives 1000 for the last,
`ops/ns` gives 0.01 for the second (W610).

**Question:** what is the intended formula, and which test is wrong?

---

## 4. `systolic_ternary_array` — output length

An invariant says the output length equals `size`; a test says it is `0` for
`size == 2`. W571 refused to write it for exactly this reason.

**Question:** does the array return one element per PE, or an empty result at
size 2?

---

## 5. `OP_ADD` / `OP_SUB` versus the sacred opcode set

Both are asserted to pass `is_sacred_opcode`, but the sacred set is eleven
**named** opcodes and neither is among them (W571).

**Question:** does the sacred set grow, or do the assertions go?

---

## 6. `PpaMetrics` — field mismatch

The struct's declared fields do not match the fields its constructors and tests
use (W592).

**Question:** which field set is canonical?

---

## 7. The five false CORDIC assertions

All arithmetic is written down in **T5**, **T6** and P13/P14:

| Assertion | Fact |
|---|---|
| `K(12) > K(8)` | **false** — `K(n) = ∏(1+2⁻²ⁱ)^(−½)` decreases to 0.6072529… |
| `K(16) > K(8)` | false, same reason |
| `K(16) ∈ (0.6073, 0.6074)` | false — K(16) = 0.6072529…, below the window |
| `arctan_table_entry(4) ∈ (0.03, 0.04)` | the table is 0-based; `entry(4) = atan(2⁻⁴) = 0.0624188` |
| `arctan_table_entry(16) > 0.0` | the table ends at 15 |

**Question:** correct the assertions, or state a different intent?

**Separately — and this one is a real gap, not a false assertion:**
`cordic_sin(π)` cannot work. **T6** proves the residual reaches zero only when
`|z₀| ≤ Σ atan(2⁻ⁱ) = 1.7432866…`, and π exceeds that by 80.1°. The remedy is
standard argument reduction and involves **no judgement** — only the work.

---

## 8. `gemm.t27` — widen the product matrix, or narrow the multiply?

`booth_mul_i16` returns `i32`; `Mat2x2`'s fields are `i16`. **One decision
covers both of the spec's two remaining compile errors** (W610), and would make
`gemm.t27` the first newly-compiling IGLA RACE spec since `adder_tree`.

---

## 9. The 25 stub specs

25 files of ~327 bytes each — a module header and an empty `TDD: Tests` banner —
in `specs/tri/` (17), `specs/sacred/` (7) and `specs/ml/` (1). W586 proved the
`.tri` sources they name **do not exist**.

**Question:** write them, or delete them? They are *unwritten*, not *untested*,
and reporting them as L4 violations overstates the debt by nearly double (W601).

---

## 10. The 15 Markdown files named `*.t27`

7% of everything that fails to parse. Renaming changes provenance —
`MANIFEST.json` carries 104 references.

**Question:** rename, or exclude from the parse census?

---

## 11. `backend.t27` ↔ `eval.t27` — a genuine import cycle

`backend.t27` uses `eval::has_substring(...)` **4 times** and does not import
`eval`. Adding the import would close a cycle: **`eval.t27` imports
`igla::race::backend`** (added in W608, so that `substring_match` — declared in
`backend` — would resolve).

The three other consumers of `eval::` were resolved this wave without a
decision, because none of them is imported *by* eval:

| Consumer | `eval::` refs | resolved |
|---|---:|---|
| `yosys.t27` | 14 | ✔ |
| `rtl.t27` | 6 | ✔ |
| `eda.t27` | 2 | ✔ |
| **`backend.t27`** | **4** | **blocked — cycle** |

**Question:** which direction is the real dependency? Either `substring_match`
moves out of `backend` (so `eval` need not import it), or the four
`eval::has_substring` calls in `backend` are replaced by something local.

---

## What is *not* on this list

Every item this chain could settle itself has been settled. The compiler-side
categories are eliminated and the eliminations are measured:

- **P12** — no remaining IGLA RACE blocker is a compiler defect;
- **P25/P26** — 82% of the dominant error class is unwritten functions, and of
  nine examined, **seven were determined by their own tests and written**;
- **T1–T3** — the RTL is machine-checked equivalent, multiplier-free, and
  timing-closed.

---

*φ² + φ⁻² = 3 | TRINITY*
