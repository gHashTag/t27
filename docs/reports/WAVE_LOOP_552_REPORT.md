# Wave Loop 552 Report — making L2 mean something

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_551_REPORT.md`](WAVE_LOOP_551_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W551 found that L2's seal mechanism certified files the compiler rejects, and
that the documented command doing it was silently destructive — a fact that
wave-chain demonstrated by inflicting it on 30 of its own seals.

W552 closed that hole. `t27c seal --save` now refuses to write a seal that
would certify nothing; `seal --verify` treats such a seal as failure rather
than a match; and a new `t27c seal-audit` reports the state of the whole seal
store.

This track was chosen because it is the only one not blocked on a human
decision, more RAM, or hardware — and because `bootstrap/build.rs` watches
`compiler.rs` but **not** `main.rs`, where the seal code lives, so it could be
done at all while the LANG-EN gate remains unresolved.

---

## 1. What changed

| Behaviour | Before | After |
|---|---|---|
| `seal --save` on a spec that generates nothing | writes all-`none` seal, exit 0, **overwrites real hashes** | refuses with an explanation, exit 1; `--force` to override |
| `seal --verify` against an all-`none` seal | `all hashes MATCH` | `VERIFICATION FAILED — the saved seal is vacuous`, exit 1 |
| Seal-store health | no view | `t27c seal-audit`, `--strict` for a hard gate |

The refusal message names the spec, states that the seal would verify green
while certifying nothing, warns that it would overwrite recorded hashes, and
points at `t27c parse` as the check to run first.

---

## 2. Falsification, run before enforcing

The W551 plan said this rule needed a carve-out if any legitimate spec class is
*expected* to seal all-`none`. Checked: of the all-`none` seals present, **zero**
had a spec that parses. No carve-out is warranted.

Had one existed, the correct move would have been an exemption, not
enforcement — and enforcing first would have broken a valid workflow. This is
the fourth consecutive wave where the check was worth running.

---

## 3. Audit of the current seal store

```
seals total          : 1714
healthy              : 1621
VACUOUS (all 'none') :    2
spec file missing    :   91
```

**The 91 orphans are a new finding.** These are seals referencing spec paths
that do not exist *and* have no git history at that path — e.g.
`specs/numeric/binary16.t27`, `specs/network/d2d_conformance.t27`. They are
neither stale-after-deletion nor renamed-with-history; the paths appear never
to have existed as committed files. Something wrote seals for specs that were
never in the tree. Cause unknown; carried to W553 Variant A.

---

## 4. Regression check

Healthy specs still verify through the normal path. Three sampled failures
(`boards/arty_a7`, `fpga/mac`, `base/types`) are **pre-existing and unrelated**:
they report `spec_hash: MATCH` with differing gen hashes — the backends evolved
since those seals were recorded — and the W549 full-suite run already counted
**1,035** such failures. None mentions the new vacuous check.

Every seal touched during testing was reverted. The commit modifies **0** seal
files.

---

## 5. What this does not fix

- The 1,035 stale seals whose specs still parse but whose generated output has
  drifted. That is a mass-reseal decision, not a defect, and it should happen
  only once the corpus stops moving.
- The 326 specs that do not parse. Their seals will now correctly refuse to be
  refreshed into a green state, which is the point.
- Nothing about the FPGA gates, which remain blocked on memory and hardware.

---

## 6. Three cooperation variants for W553

### Variant A (recommended) — Explain and clear the 91 orphaned seals

**Hypothesis.** 91 seals reference spec paths with no file and no git history.
Either the seal writer accepts an arbitrary `spec_path`, or specs were created,
sealed and removed without ever being committed. Both are integrity problems in
the same law W552 just hardened, and neither has an explanation yet.

**Deliverables.**
1. Determine the origin: check whether `seal_file_path`/`compute_seal_hashes`
   can be pointed at a path outside `specs/`, and whether any of the 91 appear
   in reflog or in other branches.
2. Depending on the answer: either delete the orphans as garbage, or fix the
   writer so a seal cannot be created for a non-existent spec.
3. Wire `t27c seal-audit --strict` into the suite once the count is zero.

**Validation.** `seal-audit` reports 0 vacuous and 0 orphaned; `--strict`
passes; no healthy seal changes.

**What would falsify it.** If the 91 correspond to specs that legitimately live
outside `specs/` (e.g. `compiler/parser/parser.t27`, which is one of the two
vacuous seals), then "orphan" is the wrong label and the audit needs a
path-scope fix instead.

### Variant B — Clear the LANG-EN gate, then the syntax gaps

Unchanged since W550 and still **blocked on a human decision**. Six committed
documents violate L3 and are not allowlisted, so `build.rs` panics on any
`compiler.rs` edit. Once cleared: the float-cast patch (~16 specs), a
block-expression production (~40), struct-literals-in-expression (~28).

This remains the largest available win — roughly 84 of the 326 remaining parse
failures — and it cannot start without the decision.

### Variant C — Resolve the `given`/`when`/`then` question

Unchanged since W550. 327 specs use a test form with no parser production that
is nonetheless specified in `SOUL.md`, the language RFC and the TDD contract.
Whether those blocks are recognised as tests, skipped, or mis-parsed determines
whether every "N tests in spec X" figure in the repository is meaningful.

Given that W549 showed the vacuity counts were inflated and W551 showed the
seals were, this is the third integrity claim in the same family and the one
still unexamined.

---

## Recommendation

**Variant A.** It finishes what W552 started, it is small, and it is unblocked.
B is the biggest prize and is purely waiting on the LANG-EN decision. C is the
one most likely to invalidate published numbers, which argues for doing it
before more numbers are published.

---

*φ² + φ⁻² = 3 | TRINITY*
