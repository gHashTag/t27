# Wave Loop 646 — auditing my own gates, and a bug no gate could catch

**Date:** 2026-08-13 · **Predecessor:** [`WAVE_LOOP_645_REPORT.md`](WAVE_LOOP_645_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
T56  parse-no-discard counted ONE of the parser's four walk-past paths.
     Instrumenting two more:  55,563 tokens / 130 specs
                          ->  68,039 tokens / 132 specs   (+22%)

T57  `%%` is not an escape in Rust's format!.
     "$display(\"[BENCH] {} : %%0d cycles\", {})" reached Verilog verbatim.
     Measured:  "%%0d cycles", n  ->  `%0d cycles         42`
                "%0d cycles",  n  ->  `42 cycles`
     439 malformed lines across 144 specs.

And the ledger showed a MIGRATION -- 2 failures + 2 passes, same files,
different phase. A path-keyed ledger would have seen nothing.
```

---

## 1. T56 — the first gate audited was measuring one channel of three

T55 says a totality claim needs evidence, and six gates from this session carry
one unaudited. Starting with the load-bearing one: **`parse-no-discard` counts
tokens dropped by `skip_to_next_top_level`.** The parser has four functions that
walk past tokens:

| discard path | `advance()` calls | counted |
|---|---:|---|
| `skip_to_next_top_level` | 7 | **1** |
| `skip_brace_body` | 7 | **0** |
| `recover_to_stmt_boundary` | 4 | **0** |
| `restore_bdd_fallback` | 2 | **0** |

Instrumenting the two that discard *content*:

| | T42's figure | corrected |
|---|---:|---:|
| specs discarding | 130 | **132** |
| tokens discarded | **55 563** | **68 039** |

> **A gate that counts a phenomenon by instrumenting one of its producers
> reports `|φ ∩ P₁|`, not `|φ|`** — and the gap is invisible from inside the
> gate, whose count stays internally consistent, monotone and reproducible.
> **T55 generalises: every gate's totality claim is a claim about a producer
> enumeration, and producer enumerations are what this codebase does not
> maintain.**

---

## 2. T57 — a bug no gate in this session could have caught

Found while auditing gate 3's coverage of `[BENCH]` blocks:

```rust
"$display(\"[BENCH] {} : %%0d cycles\", {});"
```

**Rust's `format!` escapes `{{` and `}}`. It does not escape `%`.** Measured
through `iverilog` + `vvp` on a four-line probe:

```
"%%0d cycles", n   ->   [BENCH] a : %0d cycles         42
"%0d cycles",  n   ->   [BENCH] b : 42 cycles
```

**The cycle count was never formatted into the sentence** — it was appended
afterwards in default form, behind the literal text `%0d cycles`.
**439 lines across 144 specs.**

> **An escape convention borrowed from the wrong language is invisible to every
> check that does not *execute* the output.** The string is well-formed Rust,
> well-formed Verilog, compiles and runs in both, and is wrong only when a human
> reads what it printed.

**Static checks stratify: shape, type, and output.** This session built gates in
the first two strata. **T57 lives in the third**, and is the complement of T54:
T54 said check the artefact rather than the producers; T57 is the case where the
property is decidable only on the artefact's *behaviour*.

---

## 3. The ledger showed a migration

Re-running after the instrumentation:

```
UNEXPECTED FAILURES: 2   + specs/ar/coa_planning.t27 [parse-no-discard]
                         + specs/ar/restraint.t27    [parse-no-discard]
UNEXPECTED PASSES  : 2   - specs/ar/coa_planning.t27 [backends-declare-omissions]
                         - specs/ar/restraint.t27    [backends-declare-omissions]
```

**One failure and one pass per file, at different phases.** The instrumentation
changed *which* phase first attributes the defect, and nothing else.

> **T33's identity choice paying off.** Keyed by `path` alone the migration
> would have been **silent** — the file fails before and after. A count would
> have seen nothing either. **`(path, phase)` makes a change of *attribution*
> observable.**

Entries migrated rather than added or removed; ledger unchanged at 332.

---

## 4. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| discard measurement | 55 563 → **68 039** tokens, 130 → **132** specs |
| `[BENCH]` format string | `%0d` verified against `iverilog`+`vvp` |
| malformed lines fixed | **439 across 144 specs** |
| ledger | 332, **unchanged** — 2 entries migrated by phase |
| ratchet | **CLEAN**, 332/332, rc 0, 519 s |
| keyword gate | 609 clean, 0 bare keywords |

---

## 5. What was NOT done

- **Four gates remain unaudited**: `no-vacuous-invariant`,
  `no-vacuous-verilog-test`, `backends-declare-omissions`, and the ratchet
  itself. T55's method applies to each and none has had it.
- **`restore_bdd_fallback` is still uninstrumented** — its two `advance()` calls
  were not clearly content-discarding, and I did not resolve it rather than
  guess.
- **`no-vacuous-invariant` checks the Zig backend only.** The C backend's
  commented-out `_Static_assert(1, …)` is vacuous by the same standard and is
  not covered.
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a provider
  error for this entire session; everything named is described from general
  knowledge and **no citation was fabricated**.

---

## 6. Three ways to continue (pick one for W647)

### Option 1 — **Finish the gate audit: the remaining four**

`no-vacuous-invariant` (Zig only — C and Verilog have their own inert forms),
`no-vacuous-verilog-test` (scans `initial begin :` — what else emits a test?),
`backends-declare-omissions` (3 of 5 backends), and the ratchet (corpus-only by
design, but is the *population* it walks the one it claims?).

- **Cost:** medium; four enumerations, one per gate.
- **Pays off in:** T56 found the first audited gate measuring a third of its
  phenomenon. The prior on the other four is not good, and each hole is a number
  in this document that is currently overstated as a measurement.
- **Risk:** each widening makes a gate redder and may need a bless; budget for
  the ledger to grow before it shrinks.
- **Confirming measurement:** per gate, the population enumerated from the
  artefact against what the gate parses — the T55 table.

### Option 2 — **Add an output stratum: run what the generators emit**

T57 is invisible to shape and type checks. A phase that *executes* a small
generated artefact — `iverilog` + `vvp` on one representative spec, asserting
the printed lines are well-formed — closes a stratum no current gate reaches.

- **Cost:** medium; the tools are present (`iverilog` 13.0, `vvp`, `yosys`).
- **Pays off in:** the only stratum where format-string, printf-argument and
  encoding defects live, and this session found one there by accident.
- **Risk:** execution is slow and flaky relative to static checks; scope it to
  one spec and a fixed set of assertions, not the corpus.
- **Confirming measurement:** re-introduce `%%0d` and confirm the phase fails.

### Option 3 — **T50's 754 tests that run and check nothing**

Still the only population where a repair increases what is *verified* rather
than what is honestly *reported*.

- **Cost:** medium; a lowering gap in the `then` clause.
- **Pays off in:** 754 simulations that drive the circuit and assert nothing
  become real checks.
- **Risk:** some `then` clauses may be inexpressible in Verilog.
- **Confirming measurement:** vacuous blocks 754 → n, residue characterised.

**Recommendation: Option 2.** Option 1 finds more of what this session is
already good at finding. **T57 showed a whole stratum with no coverage at all**,
and the tools to reach it are installed and were used this wave to characterise
the bug. Adding the stratum is the higher-value move; the remaining gate audits
will still be there.

---

## Appendix — reproduction

```bash
./target/release/t27c parse-complete | tail -6
```

For T57: write a four-line Verilog module printing `"%%0d"` and `"%0d"` with the
same argument, run it through `iverilog -g2012` and `vvp`, and read the output.
**The defect is only visible in the third stratum — run it.**

**φ² + φ⁻² = 3 | TRINITY**
