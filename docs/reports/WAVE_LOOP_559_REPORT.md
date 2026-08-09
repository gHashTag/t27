# Wave Loop 559 Report — 7,623 inert tests now execute

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_558_REPORT.md`](WAVE_LOOP_558_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

The largest finding of this chain is fixed. W555 measured that **65.3 % of test
blocks assert nothing**; 7,623 of them were braceless `given`/`when`/`then`
bodies that the parser discarded. W558 attempted the lowering and reverted on
19 regressions. W559 diagnosed the three mechanisms behind those regressions and
landed it.

```
tests that assert nothing:  9,788 of 14,996 (65.3 %)   ->   2,165 of 14,996 (14.4 %)
full census:                PARSE OK=746 FAIL=317  (baseline 317)
REGRESSIONS:                0
```

---

## 1. The lowering

```
given | when | and   x = expr   ->  StmtLocal x = expr
then  | assert          expr    ->  StmtExpr( assert(expr) )
```

Proven end-to-end. The false-assertion fixture:

```t27
test bdd_obviously_false
    given x = two()        // two() returns 2
    then x == 999
```

now generates `if (!(x == 999)) @panic("assertion failed")`, and `zig test`
**aborts**. Before W559 it generated `test "bdd_obviously_false" {}` and
reported *"All 2 tests passed"*.

---

## 2. The three mechanisms behind W558's 19 regressions

W558 kept the failing set as a fixture, which is what made this wave cheap.
Diagnosing rather than guessing found three distinct shapes:

1. **`and` continuation clauses** — a binding list may continue:
   ```
   given p35  = FPGA_PART_35T
   and   p100 = FPGA_PART_100T
   ```
2. **`assert <expr>` as a bare clause** — **525 occurrences** repo-wide. The
   loop broke on it and left the parser mid-block. This was the mechanism
   behind most of the 19.
3. **Comma-separated bindings** —
   `given clk = true, rst_n = false, angle = 4096`. The loop treated the `,`
   as end-of-block.

### The general fix

Mechanisms 2 and 3 share a root cause: the loop's exit condition assumed *any*
non-clause token ended the block. It now uses a **boundary predicate** — the
block ends only on a token that can legitimately follow it (`Eof`, `RBrace`,
`KwTest`, `KwFn`, `KwInvariant`, `KwBench`, `KwPub`, `KwConst`, `KwUse`,
`KwModule`). Anything else means the loop stopped mid-clause, and the whole
block restores its entry checkpoint and falls back to the original skip.

**Safety contract: this may only add assertions, never break a file.** Every
shape not fully understood restores and skips, so a spec that parsed before
still parses. The census confirms it: zero regressions.

---

## 3. Verification

| Gate | Result |
|---|---|
| W558's 19-spec regression fixture | **0 failing** |
| Full census | `PARSE OK=746 FAIL=317` |
| Baseline | 317 |
| **Regressions** | **0** |
| False-assertion fixture | `zig test` **aborts** (was: passed) |

Assertion generation, sampled across 49 parsing BDD specs: **166 real
assertions** where there were previously zero. `specs/boards/arty_a7.t27` alone
went from 0 to 16.

Freeze ceremony performed with the `t27c frozen-digest` tool built in W558 —
the first wave in which that ceremony was routine rather than an obstacle.

---

## 4. A correction to my own metric, in the opposite direction

After landing the fix, `validate-vacuity` still reported *"assertions
DISCARDED"* and 65.3 %. The tool had become stale in the **opposite**
direction — understating the fix instead of the problem.

Corrected: only `assert true` bodies remain vacuous, giving **2,165 of 14,996
(14.4 %)**. The message now also notes that shapes the lowering cannot model
still fall back to a skip, so **the figure is a lower bound on what actually
executes** — a static scan cannot distinguish a lowered block from a fallback
one.

That distinction matters for the next wave: the honest number is not "everything
now runs", it is "everything the lowering understands now runs, and we do not
yet know the exact fraction".

---

## 5. What this unlocks, and the number nobody has yet

7,623 tests that could not fail can now fail. **Nobody knows how many do.**

That number — how many previously-inert tests fail once executed — is the real,
previously-hidden defect count of this project, and it is now obtainable for the
first time. It is the single most valuable measurement available and it is
Variant A below.

---

## 6. Three cooperation variants for W560

### Variant A (recommended) — Run the newly-executing tests and count the failures

**Deliverables.**
1. For every spec that parses and contains BDD tests, generate and run the Zig
   (and/or C/Rust) test binary; record pass/fail per test.
2. Report the failure count and classify the failures — genuine spec bugs,
   missing functions, or lowering artefacts.
3. Feed the classification back: lowering artefacts are W560 compiler bugs;
   genuine failures are the defect backlog this project has never seen.

**Why it is first.** Everything else in the queue is speculative work; this is
measurement of a fault surface that was invisible until yesterday.

**What would falsify it.** If the great majority fail for a single mechanical
reason (e.g. an undefined helper), the number is about the corpus's
completeness rather than its correctness, and should be reported that way.

### Variant B — Lower keyword-form invariants the same way

`parse_invariant_block` has the identical `skip_to_next_top_level()` discard,
affecting **5,163 invariants** that currently emit
`// invariant: X verified (no statements)` — a comment claiming verification.
The lowering pattern from W559 applies directly, and the same fixture-and-census
discipline should gate it.

### Variant C — Datapath root cause

W554 established that generated modules have a fixed `clk/rst_n/en/ready`
interface, drive only `assign ready = 1'b1;`, and produce **0 logic cells**
across every spec sampled. This decides whether spec→synthesisable RTL is
achievable at all, and it is unblocked.

---

## Recommendation

**Variant A.** The fix is landed and verified; the value is in finding out what
it exposed. A defect count from 7,623 newly-live tests is worth more than any
further feature work, and it cannot be estimated — only run.

---

*φ² + φ⁻² = 3 | TRINITY*
