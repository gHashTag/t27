# Wave Loop 645 — the totality claim covered 2 of 7

**Date:** 2026-08-13 · **Predecessor:** [`WAVE_LOOP_644_REPORT.md`](WAVE_LOOP_644_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
T54 argued artefact checks beat site audits BECAUSE they are total.
W644's scanner parsed reg, wire, integer.

Enumerated from the backend's OUTPUT:
  reg 965 | input 59 | function 17 | integer 14
  localparam 12 | task 5 | output 3 | wire 0

  2 of 7 forms in use, plus one that is never emitted.

T55  a totality claim is itself a claim.

Widened to 9 forms -> first count was 49 FALSE POSITIVES
  (`localparam real ZERO = 0.0;` -- `real` is the type, ZERO the name)
-> fixed, pinned -> 609 clean, 0 bare keywords, ratchet CLEAN 332/332.
```

---

## 1. T55 — the argument was sound and the instance was not

T54's case for checking the artefact rests entirely on **totality**. So the
totality is the thing to verify, and W644's scanner covered three declaration
keywords.

**Enumerating the forms by running the backend and counting what it emits:**

| form | occurrences | covered by W644 |
|---|---:|---|
| `reg` | 965 | yes |
| `input` | 59 | **no** |
| `function` | 17 | **no** |
| `integer` | 14 | yes |
| `localparam` | 12 | **no** |
| `task` | 5 | **no** |
| `output` | 3 | **no** |
| `wire` | **0** | yes — for a form never emitted |

**29% coverage, in a gate whose whole argument is that it covers everything.**

> **T55 — a totality claim is a proposition about the checker**, with the same
> evidential status as any other. It is not established by the checker's design
> intent, its name, or the soundness of the argument that motivated it.
> *"Artefact checks are total, therefore this artefact check is total"* is the
> composition fallacy — **recorded here in a checker written specifically to
> embody the principle it violates.**

**The coverage must come from the artefact.** The table above took one command.
The alternative — listing the forms one remembers — is what produced 2-of-7.

Widened to `reg`, `wire`, `integer`, `input`, `output`, `localparam`, `genvar`,
`function`, `task`, with the remaining limits **written into the doc comment**:
multi-name declarations (`reg a, b;`) yield only the first name; a declaration
split across lines is invisible. Neither occurs in this backend today, and the
comment is the record of what stops being true if that changes.

---

## 2. And then the widened gate reported 49 false positives

```verilog
localparam real ZERO = 0.0;
```

**`real` is the type; `ZERO` is the name.** The qualifier skip-list held
`signed`, `unsigned`, `reg`, `wire`, `integer` — storage and sign, **not type**.

**This is the third detector in this session whose count needed checking before
it was believed:**

| wave | detector | first measurement |
|---|---|---|
| T47 | truncation scanner | **50% false positives** |
| W636 | ledger scrape from the run log | **2 short** (truncated list) |
| T49 | backend coverage table | **pooled over empty outputs** |
| **W645** | declaration scanner | **49 false positives** |

**Always the same failure: a syntactic discriminator standing in for a semantic
one.** Fixed, with `localparam real ZERO = 0.0 → ZERO` pinned by a test.

---

## 3. And I nearly shipped T29 again

I first wrote the nine declaration-form cases into a Python table and **printed
it** — which asserts nothing, and is exactly T29's defect, **in the wave whose
subject is checkers that do not check.** Four unit tests now call
`verilog_declared_names` directly, including the negative cases: an
already-escaped name, an ordinary identifier, a non-declaration line, and the
type-qualifier case.

---

## 4. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| scanner unit tests | **4 passed**, all calling the production extractor |
| corpus, widened + uncorrected | 560 clean, 49 flagged — **all false** |
| corpus, corrected | **609 clean, 0 bare keywords** |
| ratchet | **CLEAN**, 332/332, rc 0, 449 s — no ledger growth |

---

## 5. What was NOT done

- **Multi-name and split-line declarations remain uncovered** — documented in
  the code rather than fixed, because the backend does not emit them.
- **Five iverilog rejections remain** (2 missing function, 2 undeclared loop
  variable, 1 empty-identifier declaration).
- **Icarus has not been re-run** since W643/W644's fixes; the only measurement
  is still the pre-fix 31.
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a provider
  error for this entire session; everything named is described from general
  knowledge and **no citation was fabricated**.

---

## 6. Three ways to continue (pick one for W646)

### Option 1 — **Re-run Icarus and measure what W643+W644 bought**

Two keyword fixes have landed since the only Icarus measurement (31 failures).
Re-run, re-triage by rejected construct, and separate *moved* from *vanished*.

- **Cost:** ~100 minutes of waiting, near-zero of work.
- **Pays off in:** the first number in this chain measuring a repair's effect on
  *simulation* rather than on a static scan — and the 171-vs-4 gap predicts a
  large drop.
- **Risk:** T19 — expect unmasking; the residue's shape is the finding (T50).
- **Confirming measurement:** Icarus failures 31 → n, retriaged, with movement
  distinguished from disappearance.

### Option 2 — **Apply T55 to every other gate written this session**

Seven phases were added: `parse-no-discard`, `no-vacuous-invariant`,
`no-vacuous-verilog-test`, `backends-declare-omissions`,
`verilog-no-keyword-decl`, and the ratchet itself. **Each carries an implicit
totality claim and not one has been audited the way W645 audited this one.**

- **Cost:** medium; six gates, one enumeration each.
- **Pays off in:** T55 says the claim needs evidence, and six claims currently
  have none. The session's own record — every detector wrong on first
  measurement — says the expected yield is high.
- **Risk:** it will find holes, each needing a widening and a re-measure; budget
  for the gates getting *redder* before they get right.
- **Confirming measurement:** for each gate, the population it claims to cover,
  enumerated from the artefact, against what it actually parses.

### Option 3 — **T50's 754 tests that run and check nothing**

Still the only population where a repair increases what is *verified* rather
than what is honestly *reported*.

- **Cost:** medium; a lowering gap in the `then` clause.
- **Pays off in:** 754 simulations that drive the circuit and assert nothing
  become real checks.
- **Risk:** some `then` clauses may be inexpressible in Verilog, making the
  honest outcome a `NOT CHECKED` marker.
- **Confirming measurement:** vacuous blocks 754 → n, residue characterised.

**Recommendation: Option 2.** W645 audited one gate's totality claim and found
it 29% true. **Six more gates carry the same unexamined claim**, and this
session's record is that every detector was wrong on first measurement. Auditing
them while the method is fresh is cheaper than discovering each hole the way
this one was discovered.

---

## Appendix — reproduction

Enumerate the forms, do not recall them:

```bash
./target/release/t27c gen-verilog-for-simulation <spec> \
  | grep -oE '^\s*[a-z]+\b' | sort | uniq -c | sort -rn
```

Then check the scanner parses each. **Read a detector's hits before quoting its
count** — four for four this session needed it.

**φ² + φ⁻² = 3 | TRINITY**
