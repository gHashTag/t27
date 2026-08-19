# Wave Loop 557 Report — making the findings permanent, and both documented test formats are broken

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_556_REPORT.md`](WAVE_LOOP_556_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W556 established that every substantive track now routes through the LANG-EN
approval. W557 did the two things still available: made seven waves of
measurement permanent, and told the truth in the documents that promise
unimplemented syntax.

It also sharpened W555's finding. **Both canonical documents specify a test
format, and neither works — in two different ways.**

---

## 1. Suite Phase 6 — the numbers can no longer be lost

Every measurement in this chain was invisible until someone ran a one-off
command, and would become invisible again the moment nobody did. The suite now
prints them on every run:

```
--- Phase 6: Integrity metrics (reporting only) ---
  NOT ANALYSED: 17 `.tri` file(s). SOUL.md documents .tri as a spec
  BDD-form tests (given/when/then, assertions DISCARDED): 7623
  tests that assert nothing: 9788 of 14996 (65.3%)
  VACUOUS (all 'none') : 2
  spec file missing    : 91
  (reporting only -- not counted in TOTAL FAILURES)
```

**Deliberately excluded from `TOTAL FAILURES`.** The values are large; turning
them into hard gates would fail the suite immediately, and that is a
maintainer's decision, not the suite's. What the suite *can* do is make them
impossible to lose.

`suite.rs` is outside `build.rs`'s watch list (`FROZEN_HASH`, `compiler.rs`,
the LANG-EN allowlist, `build.rs`), so this landed despite the gate.

---

## 2. Both documented test formats are broken, in different ways

W555 established that the braceless `given`/`when`/`then` form parses and then
discards its body. W557 checked the *other* documented form and found it worse.

| Document | Form shown | Reality |
|---|---|---|
| **`SOUL.md` §2.3** | `test name { given … when … then … }` | **does not parse at all** — `t27c parse` rejects it with *"unexpected token after expression statement"* at the first `given` |
| **`TDD-CONTRACT.md`** | braceless `test name` / `given` / `then` | parses, but `parse_test_block` calls `skip_to_next_top_level()`, so the body never reaches the AST and codegen emits `test "…" {}` |

SOUL.md is the canonical law of this repository. Its documented test example is
a hard parse error. The contract document's example compiles to an empty test
that always passes.

Repo-wide impact of the second: **7,623 test blocks** and — through the same
path in `parse_invariant_block` — **5,163 invariants**, which emit
`// invariant: X verified (no statements)`.

### What was done

Both documents now carry a clearly-marked **implementation-status note** with
the verification date, the mechanism, the repo-wide counts, a pointer to
[`WAVE_LOOP_555_REPORT.md`](WAVE_LOOP_555_REPORT.md), and the practical advice
to use brace-form tests with ordinary `assert` statements — the form the
backends actually emit.

**The specifications themselves are unchanged.** Recording what is true today
is not the same as amending the law, and which way to close the gap — implement
the syntax, or drop it — is the maintainer's call. Both notes are ASCII-clean,
so they do not add to the LANG-EN problem.

---

## 3. Where the project stands after eight waves

| Track | Blocked by |
|---|---|
| BDD parser fix (7,623 tests + 5,163 invariants) | `compiler.rs` → LANG-EN |
| Hollow-synthesis / datapath root cause | `compiler.rs` → LANG-EN |
| Syntax gaps (~84 specs) | `compiler.rs` → LANG-EN |
| `.tri` migration (needs `pub type`) | `compiler.rs` → LANG-EN |
| G2/G3 flash | a physical board |

Everything achievable without those two inputs has now been done. The FPGA
track reached its hardware boundary in W553 with a routed bitstream at
150.63 MHz; the software tracks have reached their approval boundary here.

What eight waves produced, in one list:

- **Three machine-checked theorems** about the shipped ternary MAC (T1 exact
  equivalence, T2 zero DSP48 vs one, T3 unbounded invariant by induction).
- **A working bitstream** and a `t27c fpga-chipdb` command that reproduces it.
- **Four new measurement commands** — `fpga-flash`, `fpga-chipdb`,
  `validate-vacuity`, `synth-gate`, `seal-audit` — and a hardened `seal`.
- **37 specs repaired** (65.9 % → 69.3 % parse rate).
- **Six integrity findings**, the largest being that 65.3 % of test blocks
  assert nothing.
- **Three blind spots found in my own instrumentation**, each corrected.

---

## 4. Three cooperation variants for W558

### Variant A (recommended) — Clear the LANG-EN gate

Six documents violate L3 and are not in `docs/.legacy-non-english-docs`, so
`build.rs` panics on any `compiler.rs` edit. Either translate them or approve
allowlisting them.

**This unblocks four tracks at once.** It is a one-line decision and it is now
the only thing standing between a queue of evidence-backed fixes and their
implementation. Every other variant below is smaller.

**Deliverable after unblocking, in priority order:** the BDD parser fix
(largest measured impact), then the datapath investigation (decides whether
spec-to-RTL is real), then the syntax gaps.

### Variant B — Turn the reporting gates into real gates

Phase 6 reports; it does not fail. Once the maintainer accepts the current
numbers as a baseline, pin them: `synth-gate --min-pass-rate`,
`validate-vacuity --max-ratio`, `seal-audit --strict`. Then the ratios can only
improve.

**Unblocked** — all three flags already exist and `suite.rs` is outside the
watch list. Needs only a decision on what the baseline should be.

### Variant C — Hygiene the maintainer must decide

Four items, each small, each affecting provenance so none is mine to take:

1. 15 Markdown files with a `.t27` extension (104 references) — rename or
   exclude from the corpus.
2. 91 orphaned seals (89 for deleted specs, 2 for specs never committed).
3. 585 redundant seals from a filename-convention change.
4. The 2 IGLA CODER source specs missing from the repo — re-import from
   `gHashTag/trinity`, or mark those pairs withdrawn in `MANIFEST.json`.

---

## Recommendation

**Variant A.** Eight waves of measurement have produced a queue of well-scoped,
evidence-backed fixes, and four of them are behind one approval. The measuring
is done; what remains needs a decision.

---

*φ² + φ⁻² = 3 | TRINITY*
