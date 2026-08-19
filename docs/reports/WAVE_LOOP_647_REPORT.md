# Wave Loop 647 — the stratum I built to catch it sees 3 of 144

**Date:** 2026-08-13 · **Predecessor:** [`WAVE_LOOP_646_REPORT.md`](WAVE_LOOP_646_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
T58  T57's falsification condition -- "no static check could distinguish
     %%0d from %0d without modelling $display's grammar" -- is met by
     three lines. The generator never intends a literal percent.

T59  and then the measurement reversed my own W646 recommendation:

       specs emitting [BENCH]                    144
         compile under iverilog                    3
         actually print a [BENCH] line             3

       static coverage  144/144      output coverage  3/144  (2%)

     The strata are INCOMPARABLE, not ordered.
```

---

## 1. T58 — I wrote the impossibility too strongly

T57 ended with a falsification condition asserting that a static check would
"require the checker to model `$display`'s format grammar, i.e. to be a Verilog
interpreter."

**Three lines meet it.** The relevant fact is not about Verilog's grammar but
about *this generator*: **it never intends a literal percent.** Over the corpus,
the only `%`-bearing text it emits is `%0d cycles`. So `%%` in emitted Verilog is
unconditionally a defect, and deciding that needs no grammar.

Implemented as `verilog-no-double-percent` and verified by reintroducing the bug:

```
FAIL verilog-no-double-percent: 3 emitted line(s) contain `%%` …
  line 4858: $display("[BENCH] mac_accumulator_nonneg : %%0d cycles", …
```

> **An impossibility argument transfers from the general setting to a *generated*
> one only if the generator is adversarial** — and this one is the thing being
> audited. Its own invariants collapse the problem.

**Second falsification condition I have met myself within a wave** — T53's
"a third unescaped site is the way to bet" was collected by T54's gate.
**A condition the author can satisfy next wave was not a prediction; it was an
unfinished task with a question mark.**

---

## 2. T59 — and the measurement reversed my own recommendation

W646 recommended building the output stratum *over* finishing the gate audit,
because T57 lived there. Both gates now exist, so the comparison is measured:

| | static (`%%` in emitted text) | output (run it, read the print) |
|---|---:|---:|
| specs emitting `[BENCH]` | **144** | 144 |
| of those, compile under `iverilog` | — | **3** |
| of those, actually print a `[BENCH]` line | — | **3** |
| **coverage** | **144 / 144** | **3 / 144 (2%)** |

**48× narrower**, because the output stratum is conditioned on the artefact
*compiling and executing* — and 141 of 144 do not compile.

> **T59 — execution-level checking is not a strengthening of static checking;
> the two are incomparable.** Static sees code that is generated and never run;
> execution sees behaviour no static shape reveals. In a corpus where most
> artefacts do not build, **the dynamic stratum's coverage is bounded by the
> build rate** — T21's reachability conditioning, one level out.

**The recommendation was made before either was measured.** The stratum is still
worth having — it is the only place a *wrong value* can be caught rather than a
*malformed shape* — but its 3-spec reach is a fact about this corpus's build
rate, not about the technique. **Its value is back-loaded:** it grows exactly as
the 173 parse failures and the remaining iverilog rejections are repaired.

---

## 3. A measurement I got wrong first

I first measured "0 BENCH lines in simulation output" and nearly built T59 on
it. The two specs I sampled **have no bench blocks at all** — my own population
error, the same one T35 and T49 name. Re-measured over the specs that *do* emit
`[BENCH]`, the answer is 3 of 144, not 0.

**Fourth population error of this session, and the third one I caught before
publishing.** The rule that keeps working: *measure the denominator, do not
assume it.*

---

## 4. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| static gate, bug reintroduced | **FAILS**, names all 3 lines |
| static gate, corpus | **609 clean, 0 with `%%`** |
| output stratum coverage | **3 of 144** — measured, not assumed |
| ratchet | **CLEAN**, 332/332, rc 0, 599 s |
| gates now in the suite | 9 phases + the ratchet |

---

## 5. What was NOT done

- **The output stratum is not wired into the phase list.** It exists as
  `cmd_simulated_output_wellformed` and is bounded-and-named by design (T55);
  with 2% coverage, adding it to the 5-minute gate buys little today.
- **Four gates remain unaudited** for their totality claims:
  `no-vacuous-invariant` (Zig only), `no-vacuous-verilog-test`,
  `backends-declare-omissions` (3 of 5 backends), and the ratchet itself.
- **`restore_bdd_fallback` is still uninstrumented** — its two `advance()` calls
  were not clearly content-discarding and I did not resolve it rather than guess.
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a provider
  error for this entire session; everything named is described from general
  knowledge and **no citation was fabricated**.

---

## 6. Three ways to continue (pick one for W648)

### Option 1 — **Finish the gate audit: the remaining four**

T55's method, applied to `no-vacuous-invariant`, `no-vacuous-verilog-test`,
`backends-declare-omissions` and the ratchet. **T56 found the first audited gate
measuring one channel of three** — a 22% understatement — and the prior on the
other four is not good.

- **Cost:** medium; four enumerations.
- **Pays off in:** every figure those gates produce is currently a lower bound
  of unknown tightness, and this document quotes several of them as measurements.
- **Risk:** each widening reddens a gate and may need a bless.
- **Confirming measurement:** per gate, the T55 table — population enumerated
  from the artefact against what the gate parses.

### Option 2 — **Repair the remaining iverilog rejections, and watch the output stratum's coverage grow**

Five remain: 2 missing-function, 2 undeclared loop variable, 1 empty-identifier
declaration. **T59 says the output stratum's reach is bounded by the build rate**,
so each repair widens a gate that currently sees 2%.

- **Cost:** medium; four distinct defects.
- **Pays off in:** correctness *and* coverage — the only option that does both.
- **Risk:** T19 — expect unmasking; retriage after each.
- **Confirming measurement:** iverilog rejections 6 → n **and** the output
  stratum's 3-of-144 moving.

### Option 3 — **T50's 754 tests that run and check nothing**

Still the only population where a repair increases what is *verified* rather than
what is honestly *reported*.

- **Cost:** medium; a lowering gap in the `then` clause.
- **Pays off in:** 754 simulations that drive the circuit and assert nothing
  become real checks.
- **Risk:** some `then` clauses may be inexpressible in Verilog.
- **Confirming measurement:** vacuous blocks 754 → n, residue characterised.

**Recommendation: Option 2.** T59 changed what the backlog is worth. The build
rate is now a *multiplier* on the output stratum, so repairing rejections buys
correctness and coverage together — and it is the only item on the list where
fixing one thing improves a gate's reach rather than just its verdict.

---

## Appendix — reproduction

```bash
./target/release/t27c suite --repo-root . --corpus-only 2>&1 | grep 'Format strings'
```

For the stratum comparison: count specs whose generated Verilog contains
`[BENCH]`, then how many `iverilog -g2012` accepts, then how many print a
`[BENCH]` line under `vvp`. **Measure the denominator; do not assume it.**

**φ² + φ⁻² = 3 | TRINITY**
