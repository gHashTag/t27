# Wave Loop 606 — one missing disjunct, and a string that appears in no source file

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_605_REPORT.md`](WAVE_LOOP_605_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
W605 said dataset.t27 was blocked by "a module-qualified call
splicing cannot satisfy".  That string appears in NO SPEC FILE.

The compiler synthesises it, use_resolve's rewrite had one missing
disjunct, and the fix is one line.

IGLA CODER parse failures:  4 -> 1
```

---

## 1. Tracing the string beat theorising about it

W605's Variant A was *"module-qualified references — `use_resolve` splices
contents into the namespace but creates no module object, so a qualified call
has nothing to bind to."* A confident architectural story.

**`eval.has_substring` appears in no `.t27` file.** The source writes

```t27
then eval::has_substring(prompt, "counter", 0) == true
```

`use_resolve` is *supposed* to rewrite that to the bare name; codegen lowers any
surviving `::` to `.`. So the question was never "does splicing create a
namespace" — it was "why did the rewrite not fire".

### The rewrite had one missing disjunct

```rust
.filter(|(_, name)| pulled_names.contains(name))                          // before
.filter(|(_, name)| pulled_names.contains(name) || local.contains(name))  // after
```

`dataset.t27` declares its **own** `has_substring` — its header says *"inline
copies of eval.t27 templates to avoid circular imports"* — and the fixpoint
**skips local names by design**, so the name never entered `pulled_names`.

**Three other qualified references in the same file, whose declarations were
pulled, rewrote correctly.** One file, two outcomes, one missing disjunct. When
a rule works for some sites and not others *in the same file*, the predicate is
incomplete, not the design.

Rewriting to the bare name is safe *precisely because* the fixpoint skips
locals: a local name is never also pulled, so the bare spelling has exactly one
definition to bind to.

## 2. The population, counted three times before being believed

| Count | What it actually measured |
|---:|---|
| 1538 | `mod.fn()` anywhere — **1381 of them Zig's `testing.expect`** |
| 29 | `mod.fn()` where the file imports `mod` — **missed the `::` spelling entirely** |
| **616** | `mod::fn()`, of which **187** are imported modules |

The other 429 are **type**-qualified — `TernaryWeight::from`, `HybridBigInt::…`,
`Vec::…` — and must **not** be rewritten. **Fourth consecutive wave in which the
first count was wrong and checking caught it before it was acted on.**

## 3. Two brace defects in `arch.t27`, found in sequence

```
line  666   `rag_retrieve_architecture` has NO CLOSING BRACE
line 2352   a stray `}` closes nothing -- brace depth goes negative there
```

**The second was invisible until the first was fixed.** Compute the running
brace depth over the whole file rather than trusting the first error location.

Both are the W569 class, and the parser reports them as *errors* rather than
truncating silently **because W569 and W577 made it do so** — the same
instrument, many waves later, diagnosing a file it was not built for.

## 4. IGLA CODER, start of wave to end of wave

| | start | end |
|---|---:|---:|
| **parse** failures | 4 | **1** — only `weights.t27` |
| **compile** failures | 6 | 9 |
| `parse-complete` | 397 | **400** of 608 |
| **measurable specs** | **0** | **0** |

`prm` moved off `BeamCandidate` — the `arch` dependency resolved. **No IGLA
CODER spec produces a test binary yet, and that remains the headline.** Nine of
ten now parse; every remaining blocker is in codegen or a missing declaration.

Recorded as **P22**, with **P21 annotated at its head**.

## 5. Verification

| Gate | Result |
|---|---|
| `lex-conform` | 34 / 34 |
| `parse-conform` | 15 / 15 |
| `parse-complete` | **400** / 608, 0 truncating |
| `catalog-gate` | 83 records, 1 known finding |
| `cc-gate` | 101 |
| `use_resolve` unit tests | **4 / 4** (1 new regression test) |
| suite Phase 7 | verified by breaking it — corrupting an emitted field yields `GATE FAILURES: 1` |

---

## 6. Three cooperation variants for W607

### Variant A (recommended) — `eval.t27` is missing an import

`eval.t27` uses `SimResult` and imports only `base::types` and
`math::constants`. `SimResult` is declared in `specs/fpga/simulator.t27` and
`specs/igla/coder/prm.t27`. **It is a missing `use`, not a compiler gap** — but
which module it should import is a specification decision, because importing
`prm` when `prm` already imports `eval` would be circular, and `dataset.t27`'s
own header records that the author avoided exactly that.

The honest framing: one import line, one decision about direction, and `eval`
plausibly becomes the **first measurable IGLA CODER spec**.

### Variant B — `weights.t27`, the last parse failure

Line 690, `int4_dequantize_bank`. It is the only CODER spec that still fails to
parse; the other nine have moved to codegen. Whatever it is, it is the last of
its class in this family.

### Variant C — Flash the board

Unchanged, backed by
[`IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md). Phase 0 complete.

---

## Recommendation

**Variant A.** It is the smallest remaining step that could produce the thing
this family has never had — a CODER spec with a measured pass rate — and this
wave's lesson is that the small mechanical explanation usually beats the
architectural one.

---

*φ² + φ⁻² = 3 | TRINITY*
