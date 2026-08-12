# Wave Loop 626 — the suite finished and refuted me five times; then it said something worth hearing

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_625_REPORT.md`](WAVE_LOOP_625_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
The run of `t27c suite` completed. It falsified FIVE observations in the theorem
I had just written about it -- and I produced four of them myself.

T25  "has not finished" is never evidence for "will not finish"
T26  the instrument produces the observation; `| tail -25` was the silence
T27  a total that sums GATED phases counts one defect once per phase
     -- 1494 of 2614 (57%) is one fact reported six times
T28  none of the 2614 is this session's doing -- three differential suite
     runs (pre-W623 / W624 / W625) agree term for term

And the number that was hiding inside the aggregate:
     33.8% of the hand-written spec corpus does not parse.
```

---

## 1. Five corrections to T24

The run finished, exiting non-zero: `TOTAL FAILURES: 2614`,
`GATE FAILURES: 0`, `ACCEPTABLE: no`.

| # | T24 said | truth | who caused the error |
|---|---|---|---|
| 1 | "stops terminating" | terminated with a verdict | **me** — published mid-run |
| 2 | "no output at all, no progress line" | streams `FAIL <phase> (<path>)` from Phase 1 | **me** — `\| tail -25` |
| 3 | glob = `icarus_regression_specs()` | glob = `collect_t27(repo/specs)` | me — read from memory |
| 4 | "89% about scaffolding" | **98.89%** by bytes; 88.99 **: 1** is the ratio | **me** — ratio written as a percent |
| 5 | "~52 minutes" | never measured; last `etime` seen was 50:11, and an uncontended run takes **79.7 min** | **me** — lower bound as point estimate |

**(2) is the sharpest.** `suite` had been reporting continuously for 47 minutes.
`tail -N` must read to end-of-stream before it knows which N lines are last, so
it consumed every line and emitted nothing until exit. A re-run logging to a file
had 159 `FAIL` lines while still running.

> **T26 — a measurement is `observe(instrument, subject)`, and an absence in the
> output has two preimages.** They are indistinguishable from the output alone,
> and the default attribution is to the subject, because the instrument was
> chosen for convenience and then dropped from the model.

This is §4's own rule — *a stage that cannot fail cannot be trusted; ask it to
account for its input* — committed by the observer. `tail -25` accepted 47
minutes of diagnostics, discarded all but 25, and reported success.

> **T25 — a finite observation can refute non-termination and can never
> establish it.** The likelihood ratio against "merely slow" is 1, so a finite
> wait carries exactly zero evidence. This is T18's rule with the quantifier
> flipped; the repository had it seven waves earlier.

**All five are one error: the apparatus treated as transparent** — memory,
waiting, `tail`, a ratio carried across a unit change, and a clock I stopped
reading. Not one is about the compiler; every one is about how I looked.

---

## 2. What 2614 actually is

`suite.rs:1484` defines the total as a plain sum. It reconciles with no residual:

| term | value |
|---|---:|
| Parse / Typecheck / Gen Zig / Gen Rust / Gen Verilog / Gen C | 249 **× 6** |
| Verilog yosys smoke | 62 |
| FPGA smoke · GF16 conformance | 1 · 1 |
| ~~Icarus · Cocotb~~ | ~~0 · 0~~ — **NOT RUN, not zero (T51, W641)** |
| **Seal mismatches** | **1056** |
| **total** | **2614** |

**The six 249s are the same 249 files.** Each downstream subcommand was re-run
over all 609 non-scratch specs; each failed on 206, and `comm -3` against the
parse-failure list returned **0 differing lines in all five cases**. Spot-checked
independently here: over the first 80 non-scratch specs `gen-c` fails on exactly
44, and exactly those 44 are in the parse-failure list.

> **2614 counters carry five facts.** 1494 of them — **57%** — is one fact
> reported six times, because later phases are gated on parse success.
> (**T27**.)

**And the operationally serious half.** `GATE FAILURES: 0` means the conformance
gates are clean; the non-zero exit is entirely accumulated drift. Any *new* parse
break or seal break lands inside 2614 and moves the exit code not at all.
**The suite cannot distinguish "nothing changed" from "you broke the compiler."**

---

## 3. The number that was hiding

Split `Parse failures: 249` by the glob:

| | ok | fail | rate |
|---|---:|---:|---|
| `specs/` outside `specs/scratch/` — **the hand-written corpus** | 403 | **206** | **33.8%** |

> **Corrected W628 (T34):** 24 of the 206 are not t27 source — 15 Markdown files
> carrying a `.t27` extension, 9 with no `module` declaration. Over actual source
> the rate is **182 / 585 = 31.1%**. The 33.8% is itself a count over a mixed
> population, one layer below the one this report was about.
| `specs/scratch/` — generator scaffolding | 412 | 43 | 9.5% |

**The compiler cannot parse a third of its own specification corpus**, and this
is not one bug: the 206 spread over **47 distinct error classes**. The top three
cover 81 specs:

| n | class | exemplar |
|---:|---|---|
| 30 | `Unexpected token in expression: KwInvariant` | `specs/fpga/boards/arty_a7_integration.t27` |
| 27 | `KwStruct` at module level | — |
| 24 | `unexpected token after expression statement: Ident` | `specs/api/c_api_contract.t27` |

By directory: `specs/fpga/testbench` 29, `specs/tri/collections` 18,
`specs/numeric` 11, `specs/isa` 11, `specs/physics` 9, and 40 more.

**The seals decompose too.** 1056 = 1037 hash mismatches + 18 with *no saved seal
at all* (`specs/ternary/gft_*.t27`) + 1 vacuous. Of the 1037, only **98** have a
changed `spec_hash`; the other ~940 are **pure compiler drift** — spec unchanged,
output changed. Seals last written 2026-08-06/09; **34 commits, +2719/−102 lines**
of `compiler.rs` since. **99.2% of the sealed surface is stale.**

---

## 4. Is any of it mine?

**No.** (**T28**.)

1. **1494 of 2614 are parse failures**, and the diff cannot reach the parser.
   `+276 / −4` in `compiler.rs`; every hunk lands inside `impl Codegen`
   (4305–7027) plus one in `mod tests_w458`. `Lexer` (237) and `impl Parser`
   (952) untouched. Parsing strictly precedes codegen.
2. **The seals are not Zig-attributable.** **Zero** specs mismatch on
   `gen_hash_zig` alone; all 1037 include `gen_hash_verilog` (1033),
   `gen_hash_c` (1011) or `gen_hash_rust` (790) — backends never touched here.
3. **Blast radius:** generated Zig byte-identical W623→W624, one line W624→W625.

4. **Differential runs — T28's falsification condition, executed.** `suite`
   invokes itself through `std::env::current_exe()` (`suite.rs:29`), so an older
   binary drives every phase. The pre-W623 build was kept before the rebuild.
   Three end-to-end runs:

| | pre-W623 | W624 | W625 |
|---|---:|---:|---:|
| Parse / Typecheck / Gen Zig / Gen Rust / Gen Verilog / Gen C | 249 ×6 | 249 ×6 | 249 ×6 |
| yosys smoke · FPGA smoke · GF16 | 62 · 1 · 1 | 62 · 1 · 1 | 62 · 1 · 1 |
| Seal mismatches · gate failures | 1056 · 0 | 1056 · 0 | 1056 · 0 |
| **TOTAL** | **2614** | **2614** | **2614** |
| wall time | **4782 s** | ≥ 3011 s (unmeasured) | **6205 s** |
| load | uncontended | moderate | 13-agent audit |

**Term for term identical across all three.** The condition —
*"a differential run of the pre-W623 compiler reporting a total below 2614"* —
was executed and **not met**. The exoneration is no longer structural; it is
measured, and it covers all 2614.

**Caveat, stated rather than buried:** these commits could have *added* to an
already-mismatching `gen_hash_zig` without moving any counter. That changes no
pass/fail outcome — every such spec already fails on a non-Zig backend — but the
suite could not have seen it either way. **T27's point about a non-zero baseline
arrives here as a limit on this very exoneration.**

**And a fifth correction.** "~52 min" for the first run, repeated in three
drafts, was never measured — it was the last `etime` I happened to see (50 min
11 s) turned into a point estimate. The uncontended run takes 79.7 min, so the
first run almost certainly ran *longer* than 52, not shorter. **A lower bound
reported as an estimate**; same family as the other four (**T25**, **T26**).

---

## 5. Method note

Four audit strands ran in parallel with adversarial verification of each
strand's top measurements. **Two of thirteen agents died on API stalls**, and
**two verifiers returned `refuted`** — one killed a derived "total bytes read"
figure of 7.35 GB (four independent counting errors in the reader inventory).
Those refuted numbers do not appear above. The byte totals that survived were
re-derived here independently: `find specs -name '*.t27' -exec stat -f%z {} +`
→ **612 924 235** total, **606 113 688** scratch.

**Still no web literature.** `WebSearch`/`WebFetch` have failed with a provider
error for the entire session. Nothing was cited that was not described from
general knowledge under §3's standing rule; **no citation was fabricated.**

---

## 6. Verification

| check | result |
|---|---|
| 2614 reconciliation | exact, no residual |
| six 249 sets identical | `comm -3` → 0 diff (×5); spot-check 44/80 here |
| non-scratch parse | **403 ok / 206 fail** |
| scratch parse | 412 ok / 43 fail |
| byte totals | 612 924 235 / 606 113 688, re-derived |
| regression, pre-W623 vs W624 vs W625 | **none** — three differential runs, 2614 term for term |
| suite wall time | 4782 s uncontended · 6205 s contended |
| `cargo test --bins len_` | 8 passed |

---

## 7. Three ways to continue (pick one for W627)

### Option 1 — **Give the suite back its signal: partition, then re-baseline**

Report every phase per population (`corpus` vs `scratch`), stop summing gated
phases into one total, and record an explicit expected-failure baseline so the
exit code moves when something *changes*. Today `TOTAL FAILURES: 2614` with
`GATE FAILURES: 0` cannot distinguish a clean tree from a broken compiler.

- **Cost:** low — a reporting change in `suite.rs` plus a baseline file.
- **Pays off in:** the constitution's own verification command becomes able to
  detect a regression, which nothing currently can.
- **Risk:** a baseline of known failures is how a project learns to ignore 2614
  failures permanently. Mitigate by baselining *per class with a count*, so a new
  instance of an old class still trips it.
- **Confirming measurement:** introduce a deliberate one-line parse break; the
  exit code and the corpus-partition count must both move.

### Option 2 — **Attack the 33.8%: the three parse classes covering 81 specs**

`KwInvariant` in expression position (30), `KwStruct` at module level (27),
`Ident` after an expression statement (24). Determine for each whether it is a
parser gap or a spec defect — read the offending line, do not infer — then close
the parser gaps and migrate the spec defects.

- **Cost:** highest of the three; three independent language-surface questions.
- **Pays off in:** the largest real defect population in the repository, and the
  only option that increases how much of the corpus is *verifiable at all*.
- **Risk:** T19 applies — expect unmasking, and budget for a per-class table
  rather than a total. Also T20: probe each class by construction before fixing,
  because the corpus only shows the positions it happens to contain.
- **Confirming measurement:** non-scratch parse failures fall from 206 by the
  number of specs in the classes closed, with the class histogram published.

### Option 3 — **Re-seal, or delete the seal phase**

1056 of 1064 is not a gate, it is noise; ~940 are pure compiler drift against
seals written six days and 34 compiler commits ago. Either re-seal the corpus
(a ~1046-file tracked rewrite — a maintainer decision, not an autonomous one) or
remove the phase from the total until it can be maintained.

- **Cost:** low in effort, high in review surface.
- **Pays off in:** removes 40% of the failure total in one move and makes the
  remaining number legible.
- **Risk:** re-sealing blesses whatever the compiler currently emits, including
  any regression already in the tree. It must come *after* Option 1, or it
  freezes an unverified state into the baseline.
- **Confirming measurement:** `seal --verify` over all 1064 → 0 mismatches
  (after first `--save` for the 18 `gft_*` specs with no seal, and regenerating
  the one vacuous seal).

**Recommendation: Option 1.** Options 2 and 3 both change the tree, and neither
can be *checked* until the suite can tell a change from the status quo. Option 1
is cheap, is a prerequisite for the other two, and directly answers T27: the
instrument is currently unable to report what it was built to report.

---

## Appendix — reproduction

```bash
find specs -name '*.t27' -exec stat -f%z {} + | awk '{s+=$1} END {print s}'
```

Per-population parse sweep: run `t27c parse` over
`find specs -name '*.t27' -not -path 'specs/scratch/*'` and over
`specs/scratch/` separately, counting non-zero exits.
Phase-identity check: run each of `typecheck`, `gen`, `gen-rust`, `gen-verilog`,
`gen-c` over the same list and `comm -3` the failing sets against the parse set.
Never pipe a long suite run through `tail` — redirect to a file (**T26**).

**φ² + φ⁻² = 3 | TRINITY**
