# Wave Loop 627 — the JSON said zero, and eight specs pass

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_626_REPORT.md`](WAVE_LOOP_626_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W626 recommended Option 1 — give the suite back its signal. Implemented, and it
turned up two latent bugs on the way in.

```
T29  suite_summary.json reported total_failures: 0 for every run that
     printed 2614 -- three fields declared and never assigned. The test
     that "covers" one of them recomputes its rule on local variables.
T30  attribution implemented: every downstream failure is BLOCKED, zero
     are primary. T27's identity now measured by the tool over all 1064.
T31  a golden-file gate that WRITES the golden file when it is missing
     cannot fail on a new item (`cmd_icarus_simulate_with_baseline`).
T32  where the ratchet idea actually comes from: the coarse half and the
     fine half, and why skip lists are not members of the family.

Eight. Of 1064 specs, exactly eight pass every phase of the suite.
```

---

## 1. The bug the research turned up, and the test that hid it

`SuiteSummary` declares `total_failures`, `passed`, `acceptable`
([suite.rs:919](bootstrap/src/suite.rs)). **None was ever assigned.** Both
`--json` files written this session, from runs that printed
`TOTAL FAILURES: 2614`:

```json
{ "total_failures": 0, "passed": false, "acceptable": false }
```

**The human output and the machine output of the same process disagreed by
2614.** Any CI consumer reading `total_failures` saw a clean run. `passed: false`
was right only because `false` is the `Default` for `bool` — and `ACCEPTABLE: no`
printed for the same reason, not because anything computed it.

**A test appears to cover exactly this.**
`test_suite_summary_acceptable_computation` builds a `HashSet` baseline and a
`known` vector, then asserts `known_set.is_subset(&baseline)`. Every value is a
local. **The test calls nothing under test.**

> **T29 — a test that recomputes its subject's rule from locally constructed
> values establishes that rule about its own arithmetic, not about the
> function.** It is *total*: it passes for every implementation of `f`, including
> the empty one — which is the one that shipped.

This is **T16 with the population shrunk to one**: the check and the checked have
a common cause, so agreement is entailed and the green is indistinguishable from
a sound one.

**Fixed.** The three fields are assigned from the run; verified end-to-end —
`suite_w627.json` now reports `total_failures: 2614`. Four new tests call
`is_scratch`, `PhaseSplit::from_failures` and `PhaseAttribution::attribute`
directly instead of restating their rules. 19 suite tests pass.

---

## 2. The partition, measured

Every spec-walking phase now records *which* files failed. A failure on a file
already failing an earlier gating phase is **BLOCKED**, not *failed*.

```
--- Population split (W627) ---
phase              corpus  scratch   blocked
parse                 206       43         0
typecheck               0        0       249
gen-zig                 0        0       249
gen-rust                0        0       249
gen-verilog             0        0       249
gen-c                   0        0       249
seal-verify           395      412       249

PRIMARY (corpus):        206
PRIMARY (scratch):       43
BLOCKED (gated upstream):1494
DISTINCT FAILING SPECS:  1056
  of them, corpus:       206
```

**Every downstream phase reports zero primary failures.** T27 established this by
re-running five subcommands over 609 specs and diffing with `comm -3`; the
production tool now reproduces it over all 1064, and `BLOCKED = 1494` is exactly
T27's "one fact counted six times". **There is not a single genuine codegen-only
defect in the corpus** — everything downstream of `parse` is a file that never
parsed.

**2614 decomposes into five facts:**

| | n | what it is |
|---|---:|---|
| corpus parse failures | **206** | the only actionable defect population |
| scratch parse failures | 43 | generator fixtures, incl. deliberate `*_negative_*` |
| downstream re-reports | **1494** | the same 249, counted five more times + seal |
| stale seals on files that parse | 807 | golden-file drift, ~940 with unchanged `spec_hash` |
| smoke / FPGA / GF16 | 64 | |

**Two facts only the split makes visible.** Seal staleness divides 395 corpus /
412 scratch / 249 unparseable — so **601 of 609 corpus specs and all 455 scratch
specs carry a stale or unverifiable seal**. And `DISTINCT FAILING SPECS: 1056`
against 1064 total means **exactly eight specs in this repository pass every
phase.**

> **T30 — attribution must precede amnesty.** Without it one primary defect costs
> `k` ledger entries, so a ledger's size tracks pipeline *depth* rather than
> defect count, and its cap — the only mechanism resisting baseline rot —
> measures the wrong thing. With attribution the corpus ledger is exactly 206
> parse entries; without it, ~1236.

---

## 3. The second bug

`cmd_icarus_simulate_with_baseline` ([suite.rs:491](bootstrap/src/suite.rs)):

```rust
if baseline.exists() {
    if actual != expected { anyhow::bail!("does not match baseline …"); }
} else {
    save_icarus_baseline(&baseline, &actual)?;   // records whatever happened
}
```

265 baselines exist under `.trinity/icarus-baselines/`. For any spec without one,
the first run writes the file from its own output and returns `Ok(())`.

> **T31 — a comparison gate over a stored oracle is a no-op exactly once per
> item: on the only run in which that item's behaviour has never been reviewed.**
> The artefact it creates then makes every later run look earned, and leaves no
> trace distinguishing "verified against a reviewed oracle" from "blessed itself".

This is §4's list again with the artefact *created* rather than discarded.
Documented, not yet changed — the fix is that acquisition must be an explicit
`--bless` mode and a missing oracle in verify mode must be a hard failure.

---

## 4. Where this sits (T32)

The mechanism W626 asked for has been reinvented by most mature toolchains, and
splits into two halves that get conflated.

**Coarse — a scalar.** A *static threshold* is a number nothing updates (ESLint's
`--max-warnings`, in practice set to zero). A *true ratchet* rewrites it downward
on an improving run (`betterer`; RuboCop's `--auto-gen-config`) — and **both real
auto-tightening tools store per-item or per-class counts, not one global
integer.** Diff-scoped gates (`golangci-lint --new-from-rev`, SonarQube new-code)
are a different mechanism, not a member.

**Fine — an identity paired with an expected outcome**, which is what T27's
situation requires. DejaGnu separates XFAIL from XPASS as distinct counts; GDB
added KFAIL/KPASS to split a bug-tracked failure from a platform limit. LLVM
`lit`'s `XFAIL:` still *runs* the test, and **an XPASS is a failure**. `lit`'s
`UNSUPPORTED:` is not the same mechanism — the test is skipped, so a fix can
never be detected. pytest's `xfail_strict` exists because its default tolerates
XPASS. The idea reappears in type systems with the dual explicit:
`@ts-expect-error` errors when the next line has none, mypy's
`warn_unused_ignores`, Rust's `#[expect(lint)]` /
`unfulfilled_lint_expectations`.

**The invariant:** the unit of amnesty is an *identity*, and the verdict is
observed-vs-expected per identity. **None of them reports a total and asks a
human to remember what it used to be** — which is what this suite did, and why
T27 found it carries no signal. Skip lists (CTS `--exclude-filter`, `[ Skip ]`)
are excluded from the family for the same reason T31 is a bug: an item that never
runs can never report that it was fixed.

The named failure mode is **normalisation of deviance** (Vaughan, from the
Challenger analysis): each individually reasonable decision to accept an
out-of-spec observation accumulates until out-of-spec *is* the standard. In test
infrastructure it is baseline rot. The countermeasures are policy, not code — an
owner and issue per entry, an expiry that fails the gate when past due, a
monotone-downward cap on list size, periodic forced re-derivation.

**Competitor verdict.** t27's unusual axis is not the arithmetic or the
architecture but the *fan-out*: one `.t27` lowered to Zig, Rust, C **and**
Verilog, with the RTL then machine-checked against a golden model by a yosys SAT
miter. Chisel/FIRRTL, SpinalHDL, Amaranth, Bluespec and Veryl all target RTL from
one host language; Calyx, Filament and Dahlia target accelerators with stronger
static guarantees; none routinely emits three software backends *and* synthesised
hardware from the same source. **On the axis measured here, though, a 33.8%
corpus parse-failure rate is far outside the norm for a language toolchain** —
the comparable projects keep their own corpora at or near 100% parseable and use
per-item XFAIL for the residue. That number, not the fan-out, is the state of the
repository.

---

## 5. What was NOT done

- **No ledger file yet.** W627 built the *prerequisite* (attribution +
  partition + honest JSON). The expectations file, `--bless` mode and the
  ratchet exit code are Option 1 below.
- **T31 documented, not fixed.** Changing the Icarus phase to fail on a missing
  baseline would fail the run for every spec lacking one; that needs the ledger
  first.
- **The 206 were not touched.** Classifying them is Option 2.
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a provider
  error for the whole session; §4 is named from general knowledge under the
  standing rule and **no citation was fabricated**. Several claims in the first
  draft were refuted by adversarial verifiers and corrected — ESLint's
  `--max-warnings` is a threshold not a ratchet, and CTS `--exclude-filter` is a
  skip list not an XFAIL mechanism.

---

## 6. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| `cargo test --bins suite::` | **19 passed**, 4 new |
| `cargo test --bins len_` | 8 passed |
| full suite, W627 binary | 2614, term for term unchanged — no regression |
| `suite_summary.json` | **`total_failures: 2614`** (was `0`) |
| partition arithmetic | 206+43+1494 = 1743; +807 seal +64 = 2614 ✓ |
| wall time | 4112 s (68.5 min) |

---

## 7. Three ways to continue (pick one for W628)

### Option 1 — **Land the ledger: 206 entries, `--bless`, and a ratchet exit code**

`docs/reports/suite_expectations.json`, set-based on `(path, phase)` over the
corpus population only; scratch stays advisory and gates nothing. Each entry
carries an owner, an issue and a mandatory `expires` date that **fails the run
when past due**. An unexpected pass is a failure (the `xfail_strict` rule).
`--bless-expectations` is the only writer; a missing entry in verify mode is a
hard failure, which also fixes **T31**.

- **Cost:** medium. The hard part is policy, not code.
- **Pays off in:** the exit code starts moving on change — the thing nothing in
  this repository can currently do.
- **Risk:** normalisation of deviance; the file becomes where defects go to die.
  The expiry date and the monotone cap are the only defences and must land with
  it, not after.
- **Confirming measurement:** introduce a one-line parse break in a corpus spec;
  the run must exit non-zero *and* name that file as unexpected. Then fix a
  ledger entry; the run must exit non-zero for the **unexpected pass**.

### Option 2 — **Classify the 206: parser gap or spec defect?**

Three classes cover 81 (`KwInvariant` in expression position 30, `KwStruct` at
module level 27, `Ident` after an expression statement 24). For each, read the
offending line and decide — do not infer. Close the parser gaps; migrate the spec
defects.

- **Cost:** highest. Three independent language-surface questions.
- **Pays off in:** the only option that increases how much of the corpus is
  verifiable at all, and it is now known to be the *whole* actionable population
  — every other failure is downstream of it.
- **Risk:** T19 (expect unmasking; budget for a class table, not a total) and
  T20 (probe each class by construction first — the corpus only shows the
  positions it happens to contain).
- **Confirming measurement:** `PRIMARY (corpus)` falls from 206 by the size of
  the classes closed, and `BLOCKED` falls by 5× that.

### Option 3 — **Re-seal, with the acquisition path made explicit**

807 stale seals on files that parse, ~940 with an unchanged `spec_hash`. Either
re-seal (a ~1046-file tracked rewrite — a maintainer decision) or drop the phase
from the total until it can be maintained. Either way, fix **T31**'s
bless-on-absence in the same change, since both are the same defect: an oracle
that updates itself.

- **Cost:** low effort, high review surface.
- **Pays off in:** removes ~40% of the failure total and makes the rest legible.
- **Risk:** re-sealing blesses whatever the compiler emits *now*, including any
  regression already in the tree. **Must come after Option 1**, or it freezes an
  unverified state into the baseline.
- **Confirming measurement:** `seal --verify` over all 1064 → 0 mismatches, after
  a first `--save` for the 18 `gft_*` specs that have no seal at all.

**Recommendation: Option 1.** It is the prerequisite for checking the other two,
and W626's argument still holds: nothing here can be *verified* until the suite
can tell a change from the status quo. W627 built the attribution that makes a
206-entry ledger possible instead of a 1236-entry one; the ledger itself is the
next step.

---

## Appendix — reproduction

```bash
cargo test --release -p t27c --bins suite::
```

Full run — **redirect, never pipe through `tail`** (T26):
`./target/release/t27c suite --repo-root . --json out.json > run.log 2>&1`,
then read the `--- Population split (W627) ---` block and compare
`total_failures` in the JSON against `TOTAL FAILURES` in the log. They agreed for
the first time this wave.

**φ² + φ⁻² = 3 | TRINITY**
