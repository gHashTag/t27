# Wave Loop 640 — the repairs, and the third cause underneath

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_639_REPORT.md`](WAVE_LOOP_639_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
Three defects repaired, in dependency order:

  1. T31 bless-on-absence  -- PRECONDITION, not follow-up
  2. gen-c inflated count  -- "All %d checked tests passed (%d empty)"
  3. gen-verilog false PASSED -- "NOT CHECKED (empty body)"

  Verilog blocks printing PASSED with no check:  3,429 (28%) -> 754 (6%)
  2,675 repaired. Yield 78%.

T50  the 754 that remain have a THIRD cause: setup lowered, assertion
     did not. 631 `x = x;`, 475 `x = x + x;`, 83 clock waits, no check.
     Neither authored-empty nor discarded. No wave predicted it.
```

---

## 1. T31 first, because it is a precondition

`cmd_icarus_simulate_with_baseline` compared against a stored baseline when one
existed and **otherwise wrote one and returned `Ok`**. Regenerating oracles while
that path is live means a missing baseline blesses itself, unaudited — so this
had to land *before* any emit change, not after.

Acquisition is now `--bless-baselines`. Verification with no oracle is a hard
failure that names the remedy:

```
no Icarus baseline at <path> -- run with --bless-baselines to record one,
and review the diff before committing it (T31)
```

---

## 2. The two dishonest emit sites

**`gen-c`** counted authored-empty blocks in its success line. The claim was
*sound* — the emitted `assert(...)` traps, so the line is only reached when
nothing failed — but the denominator was wrong:

```c
printf("All %d checked tests passed (%d empty, NOT CHECKED).\n", 1, 1);
```

**`gen-verilog`** printed `PASSED` for a block with no lowered statements. Now:

```verilog
$display("[TEST] authored_empty : NOT CHECKED (empty body)");
```

| | before | after |
|---|---:|---:|
| Verilog blocks printing `PASSED` with no check | **3 429** (28%) | **754** (6%) |

**And the fix restores discriminating power to the baselines.**
`normalize_icarus_output` keeps only `[TEST]`/`[BENCH]` lines, so a passing test
records `starting` + `PASSED` — **previously indistinguishable from a vacuous
block recording the same two lines.** `NOT CHECKED (empty body)` also starts with
`[TEST]`, so it survives the normaliser: the golden files can now tell the two
apart. Verified on `w371_verilog_keyword`, whose block does contain
`if (!(1'b1)) … FAILED …` and whose baseline is therefore an honest pass.

---

## 3. T50 — the residue is the next cause

**2 675 of 3 429 repaired; 754 remain**, and their bodies are *not* empty:

| statements inside a still-vacuous block | count |
|---|---:|
| `x = x;` | 631 |
| `x = x + x;` | 475 |
| `@x(x);` (clock wait) | 83 |
| assignments from calls | 92 |

**Setup lowered; the assertion did not.** A `given`/`when` clause becomes signal
assignments, the `then` clause produces nothing, and the block prints `PASSED`
having exercised the circuit and checked no result. **Neither authored-empty
(T45) nor discarded (T42)** — a third cause, and no wave predicted it.

> **T50 — the residue of a repair is not noise; it is the next cause, made
> visible by removing the first.** Its size is the yield's complement and its
> *shape* is only observable once the dominant cause stops masking it. T19 said
> fixing a defect can expose another as a diagnostic; **T50 says the same holds
> for populations — a partial repair is also a measurement instrument.**

**And I could have forecast the 78% and did not.** T44 established the test: a
population is forecastable when the classifier is *per-item* rather than
first-failure, and `children.is_empty()` is per-block. **I applied that rule in
the wave that stated it and not in the wave after** — T49's pattern at one wave's
remove.

---

## 4. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| `cargo test --bins suite::` | 26 passed |
| probe: empty Verilog test | `NOT CHECKED (empty body)` |
| probe: C runner | `All 1 checked tests passed (1 empty, NOT CHECKED)` |
| corpus vacuous Verilog blocks | **3 429 → 754** |
| `NOT CHECKED` survives the baseline normaliser | yes — it is a `[TEST]` line |
| iverilog / vvp / yosys available | yes (13.0) — the oracle *can* be regenerated here |

---

## 5. What was NOT done — and one thing deliberately left uncommitted

- **22 newly acquired Icarus baselines are uncommitted.** `--bless-baselines`
  created them for scratch specs that had none. **They are acquisitions I have
  not reviewed**, and committing 22 unreviewed golden files would contradict the
  discipline this wave just built. They are left in the working tree for a human
  to review or discard. *The T31 rule applied to my own output.*
- **No existing baseline was modified** by the run that completed within this
  wave — the icarus regression set is `specs/scratch/w5*`/`w3*`, and the run was
  still walking it at close.
- **The 754 remain.** They are T50's third cause and Option 1 below.
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a provider
  error for this entire session; everything named is described from general
  knowledge and **no citation was fabricated**.

---

## 6. Three ways to continue (pick one for W641)

### Option 1 — **Close T50's third cause: tests whose `then` clause does not lower**

754 blocks assign signals and check nothing. The `given`/`when` lowering works;
the `then` does not. Find out why — and **forecast the yield first**, since
"does this block contain a lowered assertion" is a per-item classifier and
therefore forecastable (T44).

- **Cost:** medium; it is a lowering gap, not a reporting one.
- **Pays off in:** these are tests that *run* — they drive the circuit — and
  report nothing. Repairing them turns 754 exercised-but-unchecked simulations
  into real checks, which is the first time this chain increases what is
  actually verified rather than what is honestly reported.
- **Risk:** the `then` clauses may reference values the Verilog backend cannot
  express, in which case the honest outcome is `NOT CHECKED (then not lowered)`
  rather than a check — a reporting fix wearing a repair's clothes. Say which
  it turned out to be.
- **Confirming measurement:** vacuous blocks 754 → n, with the residue's shape
  characterised as T50 requires.

### Option 2 — **Review and land the 22 acquired baselines, then re-run the Icarus phase**

The acquisition mode works; nothing has yet verified the artefacts it produced.
Read the 22, confirm each records a real check, commit or discard, then run the
Icarus phase in verification mode and see whether the emit change moved any
existing baseline.

- **Cost:** low, mostly reading.
- **Pays off in:** closes the loop on this wave's own output, and produces the
  first measurement of how many baselines the emit change actually invalidates.
- **Risk:** if many existing baselines changed, the re-bless is large and needs
  the same review discipline — which is the work, not a side effect.
- **Confirming measurement:** every one of the 22 corresponds to a block with an
  `if (…) FAILED` branch; the Icarus phase reports the count of modified
  baselines explicitly.

### Option 3 — **Lower the 250 non-`forall` invariants**

Still the cheapest unclaimed item, pre-forecast in W635: shapes like `x > y;`
and `let x = f()` look lowerable by the machinery that already handles
`invariant name: <expr>`.

- **Cost:** low-medium.
- **Pays off in:** the first spec assertions to become executable rather than
  better-labelled.
- **Risk:** T37 — 250 is a shape-grouping, not a cause-grouping; expect finer
  classes and a yield below 250.
- **Confirming measurement:** `NOT CHECKED` invariants fall from 1 087 toward
  837, and the ratchet reports that many unexpected passes.

**Recommendation: Option 1.** Every repair so far has made the artefacts
*honest*; none has made them *check more*. The 754 are the first population
where the two coincide — the tests already run, and only the assertion is
missing.

---

## Appendix — reproduction

```bash
./target/release/t27c gen-verilog <spec> | grep -c ': PASSED'
```

Scan for vacuous blocks: for each `initial begin : X` … `end`, flag a body
containing `PASSED` and no `if (`, `assert` or `FAILED`. The residue's shape is
the non-`$display` statements inside those blocks, normalised.

**φ² + φ⁻² = 3 | TRINITY**
