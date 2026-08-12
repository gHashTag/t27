# Wave Loop 636 — 28% of the generated tests pass by printing so, and I read my own truncated list

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_635_REPORT.md`](WAVE_LOOP_635_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
T45  3,429 of 12,067 generated Verilog test blocks (28%) print
     PASSED with no check. 1,792 trace to AUTHORED-EMPTY
     `test X { /* verify baseline */ }` -- not the W633 discard.

     The control: the SAME AST gives `test "X" {}` in Zig -- honest.
     Same source, honest in one backend, false in the other.

     164 of 373 lines in the 108 committed Icarus baselines are PASSED.

T46  the gate said "UNEXPECTED FAILURES: 27" and listed 25.
     I built a ledger from that list. take(25), no "and 2 more".
     T26 committed inside the tool written to enforce T26.
```

---

## 1. T45 — the same node, honest and false

W635 recommended auditing the other backends. The first one checked reproduces
T43 somewhere worse:

```verilog
initial begin : ternary_mac_w321_batch_depth_invariant_1_test
    $display("[TEST] ternary_mac_w321_batch_depth_invariant_1 : starting");
    $display("[TEST] ternary_mac_w321_batch_depth_invariant_1 : PASSED");
end
```

**Nothing between "starting" and "PASSED".** Corpus-wide: **3 429 of 12 067
generated test blocks (28%)**.

**And it is not the W633 discard.** The source is an *authored-empty* test:

```t27
test ternary_mac_w321_batch_depth_invariant_1 { /* verify baseline */ }
```

**1 792 such blocks — 38% of all brace-form tests — every one carrying the
identical comment**, 64 per file across many files: generator output.

**The control is what makes it sharp.** From the *same AST*, the Zig backend
emits `test "X" {}` — empty, claiming nothing.

> **T45 — if two backends over one node disagree in *epistemic content*, at most
> one is faithful, and the disagreement localises the defect without any
> reasoning about the node.** Differential backend testing is an oracle for
> report honesty, and this repository has five backends over one AST — an oracle
> it was not using.

**The sting is downstream.** `.trinity/icarus-baselines/` holds 108 files
recording 373 expected simulation lines, **164 (44%) of them `PASSED`**.
Unconditional successes are frozen into the regression suite's golden output,
and `Icarus simulation fails: 0` in every run is — for these blocks — true
because nothing was checked.

**Gated, not changed.** `no-vacuous-verilog-test` reports the population; the
emitted text is deliberately left alone, because correcting it invalidates 108
committed baselines and re-blessing an oracle is an explicit human step (T31).
The phase found **27 primary** failures — specs that parse cleanly, discard
nothing, and still emit false PASSED — plus 129 already blocked. Unlike W635's
phase, which was fully subsumed, this one is not: **two different causes, partly
overlapping populations**, exactly as T45 predicts.

---

## 2. T46 — and then I did it too

The gate reported `UNEXPECTED FAILURES: 27` and listed them. I extracted the
list, blessed the entries, raised the cap — and got a ledger of **328** against
an observed **330**.

**The printer stops at 25.** I wrote it that way in W628:

```rust
for f in v.unexpected_failures.iter().take(25) { … }
```

The count says 27. The list shows 25. **There is no "and 2 more".**

> **T46 — a report presenting a set as a count plus a prefix is individually
> correct in both parts and jointly misleading.** The count and the list are two
> channels; their disagreement is detectable only by comparing them — which is
> precisely what a reader *using* the list does not do, because the list is what
> they came for.

**This is T26 committed inside the tool built to enforce T26**, by the person
who wrote the lesson, using a truncation authored twelve waves earlier. T41 said
a ratchet is as blind as its phase predicates; **T46 adds that it is as honest
as its printer.**

**The rule is not "print everything"** — 330 lines is unreadable. It is that a
**lossy view must be self-describing**: it must carry, in the same channel as
the data, the fact that it is lossy and by how much. `head`, `take(n)`, `limit`,
`--max-count`, a truncating table — all this hazard.

Fixed; and the hand-built ledger was **reverted and rebuilt with
`--bless-expectations`** — the tool, not the transcript. The tool wrote 330 and
confirmed the scrape was two short.

**§4's table** lists components that accepted input, produced less than they
should, and reported success. `take(25)` accepted 27, produced 25, reported
nothing. W588's entry is *"my own measurement"*; this is *"my own report"* — and
W588's measurement was **wrong**, while this one was **right and truncated**,
which is harder to see.

---

## 3. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| vacuous Verilog test blocks | **3 429 of 12 067 (28%)** |
| authored-empty `test X { }` | **1 792 of 4 751 (38%)** |
| Zig control for the same node | `test "X" {}` — honest |
| Icarus baselines recording PASSED | **164 of 373 (44%)** |
| new phase | **27 primary**, 129 blocked |
| ledger | 303 → **330**, cap hand-raised |
| printer truncation | now reports "and N more NOT SHOWN" |

---

## 4. What was NOT done

- **The Verilog emit was not corrected.** Deliberate: it invalidates 108
  baselines. Option 1 below.
- **`gen-rust`, `gen-c`, `gen-verilog-hir` were not audited** — one backend of
  four was checked, and it was the first one tried.
- **The 1 792 placeholder tests were not touched.** They are generator output
  and deleting or filling them is a corpus decision.
- **Still no web literature.** `WebSearch`/`WebFetch` failed with a provider
  error for the entire session; everything named is from general knowledge and
  **no citation was fabricated**.

---

## 5. Three ways to continue (pick one for W637)

### Option 1 — **Correct the Verilog emit and re-bless the 108 baselines**

Make an empty test block print `[TEST] X : NOT CHECKED (empty body)` instead of
`PASSED`, then regenerate the Icarus baselines in the same change, with the diff
reviewed rather than auto-recorded.

- **Cost:** medium — one emit site, then a reviewed re-bless of 108 files.
- **Pays off in:** simulation logs stop reporting 3 429 successes that were
  never checked. It is the actual repair of T45.
- **Risk:** `save_icarus_baseline` still records on absence (T31), so a missing
  baseline would self-bless during the regeneration. Fix that in the same
  change or the re-bless is unaudited.
- **Confirming measurement:** the vacuous-block count falls to 0, the 108
  baselines change by exactly the number of affected lines, and no baseline is
  created rather than modified.

### Option 2 — **Finish the backend audit: `gen-rust`, `gen-c`, `gen-verilog-hir`**

One backend of four was checked, and it was the first one tried. T45's oracle —
compare backends' epistemic content on the same node — applies to all pairs.

- **Cost:** medium; three backends, and the method is now mechanical.
- **Pays off in:** the systematic version. T43 and T45 were both accidents; a
  third would be evidence the class is everywhere, and a clean result would
  bound it.
- **Risk:** each finding needs a phase, a ledger class and a cap raise, and the
  ledger is already 330.
- **Confirming measurement:** a table of backend × success-claiming string ×
  the predicate that produces it, with every unconditional one named.

### Option 3 — **Audit every truncating view in the toolchain**

T46 was one `take(25)`. Grep the whole tree for `take(`, `head`, `[..N]`,
`.limit(`, `--max-count` and any table that caps rows, and make each one
self-describing.

- **Cost:** low-medium, mechanical.
- **Pays off in:** T46 is the fourth instrument-produced observation this
  session (T26, T41, T46, and the W632 verification attempt). This is the only
  option that addresses the class rather than the instance.
- **Risk:** low. Mostly it will find truncations that are harmless, and the
  work is telling them apart.
- **Confirming measurement:** a list of every truncating site with its cap and
  whether it announces elision; all reader-facing ones announce it.

**Recommendation: Option 3.** T45 is a real defect and Option 1 repairs it, but
this session has now produced **four** findings where my own instrument created
the observation. That rate says the class is the problem, not the instances —
and Option 3 is cheap.

---

## Appendix — reproduction

```bash
./target/release/t27c gen-verilog specs/igla/race/ternary_mac.t27 | grep -B2 ': PASSED' | head
```

Scan for vacuous blocks: for each `initial begin : X` … `end`, flag any body
containing `PASSED` and no `if (`, `assert` or `FAILED`. Compare against
`./target/release/t27c gen <same spec>` for the Zig rendering of the same node.
**Bless from the tool, never from a run log** (T46).

**φ² + φ⁻² = 3 | TRINITY**
