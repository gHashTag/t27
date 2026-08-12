# Wave Loop 631 — green, at 2416 failures

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_630_REPORT.md`](WAVE_LOOP_630_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
t27c suite --repo-root . --ratchet      4057 s

  ledger:              173 / 173 cap
  observed (primary):  173
  UNEXPECTED FAILURES: 0
  UNEXPECTED PASSES:   0
  EXPIRED ENTRIES:     0
RATCHET: CLEAN
TOTAL FAILURES:    2416
rc = 0
```

**The first zero exit from `t27c suite` in this chain — while its own total is
2416.** T39.

---

## 1. What the green means

W626 measured that a gate whose baseline is already non-zero detects nothing: a
new break lands inside 2614 and moves the exit code not at all. Five waves later
the exit code is a function of **observed-versus-expected per identity**, and the
total is merely reported.

That is the whole arc in one line: **rc 0, TOTAL FAILURES 2416.** The suite is
green not because the corpus is clean — it is not — but because nothing changed
that was not expected to change, which is the only question a regression gate
should be answering.

---

## 2. The equivalence that was assumed for two waves, now measured

W629 and W630 ratcheted the ledger from **direct `t27c parse` measurements**
rather than by running the 70-minute suite with `--bless`. That was a
convenience, and it carried an unverified assumption: that the two produce the
same ledger.

The production `--ratchet` path independently observed **173** primary corpus
failures against a ledger of **173**, with **zero** unexpected failures and
**zero** unexpected passes. The assumption holds, and it is now a measurement.

**Practical consequence:** ratchet in the commit that does the fixing, and
confirm with one nightly run. No wave needs to block on 70 minutes.

---

## 3. T39 — the arithmetic confirms T27 to the unit

33 corpus specs were fixed across W629 and W630 (206 → 173). The total moved
2614 → 2416:

| phase | before | after | Δ |
|---|---:|---:|---:|
| parse | 249 | 216 | −33 |
| typecheck, gen-zig, gen-rust, gen-verilog, gen-c | 249 ×5 | 216 ×5 | **−165** |
| seal-verify | 1056 | 1056 | **0** |
| **total** | **2614** | **2416** | **−198** |

**`198 / 33 = 6.000` — exactly six counters per file fixed.**

T27 stated that one unparseable spec contributes once per gated phase. Here the
claim is inverted and confirmed: removing one contributes exactly −6.
**Seal-verify is unchanged because those 33 moved *within* it** — from `blocked`
to `primary`. They parse now, so they reach the seal check, and the seal is
stale. Nothing lost, nothing double-counted; the `blocked` bookkeeping accounts
for all 198.

> **T39 — the total is a linear function of repairs with a coefficient set by
> pipeline shape.** That is why it is a poor progress metric and a perfectly good
> *consistency check*: it must move by a clean multiple of the depth, and if it
> does not, the attribution is wrong.

**This turns T27's complaint into a tool.** A total that cannot detect a
regression can still detect a **bookkeeping error**, because its arithmetic is
over-determined once the ledger names the files. That is the only use it now has
here.

---

## 4. Where the chain stands

| | W622 | now |
|---|---:|---:|
| `t27c suite` exit code | non-zero, always | **0** |
| regression detectable? | **no** (T27) | **yes**, by name |
| corpus parse failures | 206 | **173** |
| ledger | — | 173, capped, expiring 2026-11-30 |
| `suite_summary.json` `total_failures` | **0** for a 2614 run (T29) | 2416, correct |
| corpus rate, honestly stated | "33.8%" (a 5-population mixture) | **29.8%** of 581 kind-1 files |

Nine theorems this session (T29–T39) and 33 specs fixed, but the load-bearing
change is none of those: it is that a wave can now be *told* whether it broke
something.

---

## 5. What was NOT done

- **The 173 remain**, and T37/T38 say they are a long tail of ~147 causes with
  yields below 1. There is no shortcut left to find.
- **The 26 non-`module` files still sit in `specs/`** (T35), so every aggregate
  is still a mixture.
- **The 5 standing `cargo test --bins` failures remain**, and `suite` still does
  not run them.
- **`--ratchet` is not in CI.** It works; nothing invokes it automatically.
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a provider
  error for this entire session. Everything named across T32–T39 is described
  from general knowledge under the standing rule, and **no citation was
  fabricated.**

---

## 6. Three ways to continue (pick one for W632)

### Option 1 — **Wire `--ratchet` into CI as a nightly**

The mechanism is built, verified on a throwaway repo, and now verified green on
the real corpus. Nothing runs it. A nightly job plus a documented
"how to bless a new expected failure" note is the remaining step.

- **Cost:** low — one workflow file and a paragraph in `CLAUDE.md`.
- **Pays off in:** the guarantee becomes automatic instead of depending on
  someone remembering to run a 68-minute command.
- **Risk:** a nightly that nobody reads is worse than nothing, because it
  manufactures the appearance of a gate. Route the failure somewhere a human
  sees it, or do not add it.
- **Confirming measurement:** deliberately break one corpus spec on a branch;
  the nightly must go red and name the file.

### Option 2 — **Work the tail by file, letting the ledger record each removal**

T37 (147 causes, largest 6) and T38 (yield below 1) together say class-based
planning does not work here. The honest approach is file-by-file, with the
ledger shrinking monotonically and `newly broken = 0` enforced each time.

- **Cost:** linear in files — and that is the point: measured progress rather
  than forecast progress.
- **Pays off in:** the only strategy the last two waves' evidence supports.
- **Risk:** the temptation to re-derive a class-based shortcut will return. T37
  is the standing answer.
- **Confirming measurement:** ledger size falls monotonically; each commit shows
  deletions in `suite_expectations.json` and `UNEXPECTED FAILURES: 0`.

### Option 3 — **Rehome the 26 non-`module` files and make every aggregate mean one thing**

15 Markdown → `.md`; 6 `spec {}` and 3 `algorithm {}` migrated or moved out of
`specs/`. Each is referenced 4–21 times, so the references move in the same
change.

- **Cost:** medium-high, almost all in reference updates.
- **Pays off in:** T35 stops applying. The corpus rate becomes a single number
  comparable across waves instead of a mixture needing a footnote.
- **Risk:** a missed reference rots a doc link silently — the §4 failure mode.
  Count each basename's references before and after and diff the counts.
- **Confirming measurement:** `specs/` contains only kind-1 files, and the
  aggregate rate equals the kind-1 rate.

**Recommendation: Option 1.** It is the cheapest, and it is the difference
between a mechanism that exists and a mechanism that is *in force*. Options 2
and 3 are both long and both benefit from the gate running first.

---

## Appendix — reproduction

```bash
./target/release/t27c suite --repo-root . --ratchet --json out.json > run.log 2>&1
```

Expect `RATCHET: CLEAN` and `rc 0` against
`docs/reports/suite_expectations.json`. Redirect, never pipe through `tail`
(T26). To re-derive the ledger without a full run, parse each entry's path and
keep the ones that still fail — verified equivalent to `--bless` this wave.

**φ² + φ⁻² = 3 | TRINITY**
