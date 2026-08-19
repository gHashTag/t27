# Wave Loop 641–642 — a skipped phase read as passing, and the shape underneath all of it

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_640_REPORT.md`](WAVE_LOOP_640_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
T51  every summary this session said `Icarus simulation fails: 0`
     for a phase that never ran. With the flag: 124 passed, 31 FAILED.

T52  five artefacts, one shape -- the empty case renders identically
     to the verified case:

       T43  invariant body discarded -> "verified (no statements)"
       T45  test block with no stmts -> "[TEST] X : PASSED"
       T48  authored-empty test      -> "All 2 tests passed"
       T51  phase never ran          -> "Icarus fails: 0"
       T52  nothing ever recorded    -> baseline {"lines": []}

     152 of 282 Icarus baselines (54%) record NO expected output.
```

---

## 1. T51 — nine waves of reading a zero that meant "not run"

The Icarus phase is opt-in, and every suite invocation this session omitted the
flag. All of them printed `Icarus simulation fails: 0`. **With the flag
(6 113 s): 124 passed, 31 failed.**

Three lines of Rust:

```rust
let mut p3d_fail = 0usize;                        // initialised to zero
if opts.icarus_simulate { … p3d_fail = p3df; }    // assigned ONLY if run
println!("Icarus simulation fails:  {}", p3d_fail);  // prints 0 either way
```

**Zero is the identity for "failures", so the absence of a measurement is
indistinguishable from a measurement of zero.**

**It contaminated this document's own arithmetic.** W626 decomposed
`TOTAL FAILURES: 2614` into five facts, two of which — Icarus and Cocotb — were
never measured. The *total* is unaffected: a skipped phase contributes 0 either
way, **which is exactly why the error stayed invisible for nine waves.** But the
inventory was wrong, and the inventory is what anyone reads to decide what to
work on.

Fixed: both lines now print `SKIPPED (not run -- pass the flag to enable)`.
W626's report is annotated in place.

---

## 2. Triaging the 31

| n | class |
|---:|---|
| **16** | `iverilog rejected generated Verilog` — a real backend defect |
| 9 | Verilog generation error, module-level parse (T42's discard class) |
| 3 | Verilog generation error in a fn — includes deliberate `*_negative_*` fixtures |
| **2** | output does not match baseline |
| 1 | genuine simulation failure |

**The 2 mismatches are good news wearing a failure's clothes.**
`w373_struct_field_keyword`'s baseline is `{"lines": []}`; its Verilog now
contains a real check:

```verilog
if (!((sum_word(item) == 7))) begin
    $display("[TEST] w373_struct_field_keyword_sum : FAILED");
end
```

**The spec improved and the golden file never caught up.**

| of 282 Icarus baseline files | count |
|---|---:|
| record **no expected output at all** | **152 (54%)** |
| record something | 130 |
| **not valid JSON** | **5** |

A baseline of `{"lines": []}` passes exactly when the simulation produces
nothing — recorded under T31's bless-on-absence (closed in W640) at a moment
when the spec produced nothing. Sampling 45, **6 (13%)** belong to specs whose
Verilog now emits `[TEST]`/`[BENCH]`: the oracle says *expect silence* and the
artefact speaks.

---

## 3. T52 — the shape

> In five independent artefacts, written by different code over different media,
> **`R(nothing was done) = R(verified)`.**

**Why it recurs.** Success vocabularies are **absorbing**: `0` is the identity
for failure counts, the empty set matches any empty observation, "passed" is what
you print when no assertion fired, and an empty golden file diffs clean against
empty output. **The empty case is the fixed point of the success encoding**, so a
system that says nothing about emptiness reports it as success *by
construction*.

**The defect is not carelessness.** It is that the honest value has no natural
representation unless one is deliberately reserved.

> **The remedy is a reserved symbol, not more care.** Every fix this session was
> the same move — introduce a value success cannot produce:
> `NOT CHECKED -- body was not lowered`; `NOT CHECKED (empty body)`;
> `(%d empty, NOT CHECKED)`; `SKIPPED (not run)`. **One edit, applied four
> times.** The fifth — an empty baseline — is unfixed.

---

## 4. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| skipped phases now report | `SKIPPED (not run …)`, verified on the throwaway repo |
| Icarus with the flag | 124 passed, **31 failed**, 6 113 s |
| baselines recording nothing | **152 of 282 (54%)** |
| malformed baselines | **5** |
| `w373` — real check, empty baseline | confirmed by reading the generated Verilog |

---

## 5. What was NOT done

- **None of the 31 was repaired.** This wave classified them; the 16 iverilog
  rejections are the largest real defect and are Option 1.
- **The 152 empty baselines stand.** Their reserved symbol — a baseline that
  records *"this spec is expected to emit no test output"* as distinct from
  *"no baseline content"* — is not designed.
- **22 acquired baselines remain uncommitted** from W640, still unreviewed.
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a provider
  error for this entire session; everything named is described from general
  knowledge and **no citation was fabricated**.

---

## 6. Three ways to continue (pick one for W643)

### Option 1 — **The 16 `iverilog rejected generated Verilog` failures**

The largest real defect in the triage: the backend emits Verilog that `iverilog`
refuses. Unlike everything else this session, this is not a reporting flaw — the
output is wrong.

- **Cost:** medium; classify the rejections by `iverilog`'s message, then fix.
- **Pays off in:** it is the first purely-correctness defect found since T18.
  Everything between has been about what artefacts *claim*.
- **Risk:** T37 — group by the *rejected construct*, not by `iverilog`'s message
  text, or the class sizes will be inflated.
- **Confirming measurement:** Icarus failures 31 → n, with the residue
  characterised as T50 requires.

### Option 2 — **Give the empty baseline its reserved symbol**

152 of 282 golden files assert nothing. Introduce an explicit
`"expects_no_output": true` distinct from an absent/empty `lines`, make the
verifier reject the ambiguous form, and re-bless deliberately.

- **Cost:** low in code, high in review — 152 files need a human decision each.
- **Pays off in:** closes T52's fifth site, the only one still open, and
  completes the "reserved symbol" programme this session ran four times.
- **Risk:** re-blessing 152 baselines is exactly the bulk-acquisition T31 warns
  about; it must be per-file and reviewed, or it recreates the problem.
- **Confirming measurement:** zero baselines with an ambiguous empty `lines`;
  every one either records output or declares that none is expected.

### Option 3 — **Close T50's third cause: 754 tests that run and check nothing**

Still the only population where a repair increases what is *verified* rather
than what is honestly *reported*. Forecast the yield first — the classifier is
per-item, so by T44 it is forecastable.

- **Cost:** medium; a lowering gap.
- **Pays off in:** 754 simulations that drive the circuit and assert nothing
  become real checks.
- **Risk:** some `then` clauses may be inexpressible in Verilog, making the
  honest outcome `NOT CHECKED (then not lowered)` — a reporting fix wearing a
  repair's clothes. Say which it turned out to be.
- **Confirming measurement:** vacuous blocks 754 → n, residue characterised.

**Recommendation: Option 1.** Twenty waves have now been spent on what artefacts
*claim*; the 16 rejections are the first thing in a long time that is simply
*wrong*, and a backend emitting Verilog its own simulator refuses is a defect no
amount of honest reporting improves.

---

## Appendix — reproduction

```bash
./target/release/t27c suite --repo-root . --icarus-simulate
```

Budget ~100 minutes. Compare its `Icarus simulation fails:` line against a run
without the flag — the two used to be identical. For the baselines:
count `.trinity/icarus-baselines/**/*.json` whose `lines` array is empty.

**φ² + φ⁻² = 3 | TRINITY**
