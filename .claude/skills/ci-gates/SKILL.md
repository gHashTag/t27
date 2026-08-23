---
name: ci-gates
description: How a CI gate lies. Six confirmed cases from one day in this repository — gates that cannot fail, gates blind to the files that matter, wrappers that swallow the tool's own message, and a crash reported as a numeric disagreement. Read before writing a gate, before trusting a green one, and before believing a failure message about which subsystem broke.
---

# How a gate lies

Every rule below is a specific failure found in this repository on 2026-08-18, with the
path. None is a principle someone liked.

The day's arithmetic: **one missing brace cost four days of a red gate**, and the four days
were spent because three layers of diagnostics each named a different subsystem and only
the innermost one was right.

---

## 1. A gate that cannot fail reads as coverage, and is worse than none

`docs/BRANCH-PROTECTION.md` named five required checks. Two had a body of exactly one
`echo`:

```yaml
- run: echo "Running SEAL coverage analysis..."      # seal-coverage.yml, 17 lines
- run: echo "Validating JSON schemas..."             # schema-validation.yml, 15 lines
```

A third, described in that same table as **"Main test suite"**, asserted

```python
assert abs(phi**2 + phi**-2 - 3) < 1e-10
```

which is a truth about arithmetic and holds for an empty repository.

**The test:** for each gate you rely on, write down the change that should make it fail.
If you cannot name one, the gate is decoration and the row in the required-checks table is
a claim that isn't true.

Replacing one `echo` with *does every tracked JSON parse* — the weakest question worth
asking, chosen because it carries no theory that could itself be wrong — immediately found
`clara-bridge/audit-trail/experience-schema.json` with a literal `...` on line 40, which
`clara-bridge/tests/run_tests.py:152` loads with `json.load()`. **3 of its 11 tests were
failing** and no workflow ran that suite at all.

## 2. A gate is green until proven otherwise — check its SCOPE separately from its result

`tools/check_withdrawn_live.py` shipped scanning `.md`/`.tex`/`.rst` and printed
`OK … 975 documents scanned`. It was green **because it could not see** the file that
mattered: `specs/numeric/formats_catalog.t27:228`, the canonical CATALOG row that feeds
the published dataset, still carried the withdrawn number. So did the metrics CSV and two
self-citations in a `.t27` benchmark.

Widening to `.t27`/`.csv`/`.json` took the scan from 975 to **4435** documents and found
seven live occurrences.

**A gate green because it is under-scoped is the same failure it was built to kill.**
Print what the gate covered — file count, extensions, exclusions — beside its verdict, and
read that number as carefully as the verdict.

## 3. Ask the exit code. Print what the tool said.

Two patterns, both found by AST scan over `tools/*.py`:

| pattern | what it does | sites found |
|---|---|---|
| `run(..., capture_output=True).returncode` | reads the exit code, discards the message | 9 |
| `run(..., capture_output=True).stdout` | reads neither the exit code nor stderr | 18 |

The second is worse. When `t27c gen-c` failed to *parse* a spec, `.stdout` was `""`, the
empty string flowed downstream, and the failure surfaced as

```
FAIL: C backend failed to build/run
```

naming a subsystem that had never been reached. The compiler's own message named the file,
the function, the line and the token — and `capture_output` collected it so it could be
thrown away.

**A diagnostic that names the wrong subsystem costs more than no diagnostic.** "The C and
Rust backends diverge" is a far more alarming claim than "a spec has a typo", and it is
where the four days went.

**Rule: any wrapper that captures a tool's output must print it on a non-zero exit.**

## 4. A crash is not a disagreement

`run(built_binary).stdout` without an exit-code check turns a **dead program** into a
**numeric mismatch between targets** — the most alarming reading a verification harness can
produce, and the wrong one. A program that died on signal 11 did not disagree about
arithmetic.

Same shape one layer out: a simulator that hits its own timeout yields a short output list,
reported as `step count RTL=0 PY=80`, which reads as *the RTL emitted nothing* — a design
fault. The testbench was printing `TIMEOUT` into the very stdout the script parsed, and
nothing looked for it.

**Distinguish, in the message: did not run / did not finish / ran and disagreed.** Three
different bugs, three different owners.

## 5. Re-scan after a pattern fix, before claiming the pattern is gone

Two partial repairs in a row, same day:

* fixed the wrong 9 sites (the `.returncode` ones) and the output **did not change at all**,
  because the failure was upstream in the `.stdout` calls the scan had not been written to
  find;
* fixed 6 sites in one file, reported the pattern handled — a re-scan found **12 more**,
  the same call in a multi-line form the regex missed, plus 8 binary invocations.

Both were caught by a **negative control** — plant the original fault, run, read the output
— not by reasoning and not by the scan.

**A regex matches a form, not a meaning.** Scan by AST, and run the scan *after* the fix as
well as before.

## 6. Renaming a CI job silently breaks branch protection

Replacing a gate's body and renaming its job from `validate` to `schema-validation` sent
the PR to **`mergeable: MERGEABLE`, `mergeStateStatus: BLOCKED`** with every visible check
green and no required review outstanding.

Branch protection matches on the **job id**, which is the status-check context. A renamed
required context does not report as failing — **it stops reporting**, and nothing says so.

The workflow's `name:` may change freely. `jobs.<id>:` may not.

When `repos/:owner/:repo/branches/master/protection` returns nothing useful, read the
contexts off a recently merged PR: `gh pr checks <N> | awk '{print $1}' | sort -u`.

## 7. When a scan says something alarming, re-derive it a second way before writing it down

Four times in one session an anomaly came from the instrument rather than the thing
measured. Every one would have been reported as a fact about the repository.

| what the instrument said | what was true | why it lied |
|---|---|---|
| "two of four optima are not identified" | the opposite — all four narrow, mutually incompatible | `depth` measured the gap to the nearest **grid point**, so it ranked the grids |
| "`tmul` has diverged across the BitNet family" | all 15 copies are one function | a regex ending at `\n}` swallowed five definitions in a spec written on one line; and `if(ta==1)` vs `if (ta == 1)` hash differently |
| Verilog arm silently narrower than C and Rust | caught before it ran | the testbench generator sliced every argument `[7:0]`, hardcoded, while `sign0` takes `i16` |
| "73 seals reference specs that never existed" | **15** | `git log --diff-filter=D -- <exact path>` only sees a deletion recorded at that path; by basename across all history the count is a fifth of that |

None was caught by reasoning. Three were caught by a **negative control** — plant the
fault, run, read the output — and one by re-deriving the same number a different way.

**The rule.** A scan that reports something bad about the tree is a claim like any
other, and the first version of it is usually a claim about the scan. Before writing it
down: derive it a second way, and prefer a way that shares no code with the first.
`tools/check_seal_coverage.py` does this in-line — `_ever_existed` asks git twice, by
path and by basename, because the one-way version overstated fivefold.

**The corollary about severity.** The more alarming the finding, the more likely it is
yours. "Seals reference specs that never existed" and "the backends diverge" are
accusations; "a spec has a typo" and "a regex is wrong" are not. The session's four
false alarms were all in the first category, and all four true causes were in the
second.

## 8. Name the kinds of a failure separately when their fixes differ

`check_seal_coverage.py` first reported 89 **dangling** seals. Splitting by whether the
spec ever existed gives two problems that share nothing but a symptom:

* **74 dangling** — the spec was committed and later deleted, 16 of them by one
  identifiable commit. Fix: remove the seal with the spec, or restore both.
* **15 phantom** — the spec appears in no commit and is nowhere on disk. Its
  `spec_hash` and four `gen_hash_*` name a file nobody can fetch, so the record has no
  checkable content. For four of them the seal file is the **only** trace of the module
  anywhere in the tree. Fix: find the spec, or drop the seal.

One word for both would have sent a reader to the wrong repair for 15 of 89 cases. A
gate's vocabulary is part of its output.

---

## Writing a gate here

1. **Negative control first, as its own CI step.** Plant the fault the gate exists to catch
   and prove it fires; prove it stays silent on a clean input. A gate nobody has seen fail
   is not a gate. Every gate added on 2026-08-18 carries `--self-check`, and the controls
   caught three of the author's own mistakes before merge.
2. **Data, not code, for the rule set.** `tools/withdrawn.txt` holds one regex per withdrawn
   number, so a row is added the moment a number is withdrawn rather than when the document
   is finally fixed — those two events were **ten days** apart.
3. **Baselines keyed to the line, not the file.** `path | pattern | sha1(line)` — keying on
   the file would silence every future occurrence in it, which for an append-only document
   like `docs/NOW.md` widens with every entry.
4. **Report what you could not establish.** `.trinity/seals/` holds 1714 files keyed on
   *type* names; scoring "seal coverage" against spec names produced "1668 orphans of 1714",
   a finding about the assumption rather than the repository. Neither a check nor a deletion
   was written, and the PR said so.
5. **Say what the gate does not cover.** `external/` is excluded from the JSON gate because
   tsconfig is JSONC by convention — flagging it would be the gate making the mistake it
   exists to catch.

## 9. A job that has always failed early contains a tail that has never run

`fpga-bitstream` had been red since its creation. Fixing it took **ten layers**, one
CI round-trip each, because every step after the historical point of failure had
*never executed* — each was a fresh landmine, not a regression:

| layer | defect | had it ever run? |
|---|---|---|
| 1 | cloned `YosysHQ/nextpnr`, which has no xilinx arch (it lives in `openXC7/nextpnr-xilinx`) | failed every run |
| 2 | `libboost-dev` is headers-only; cmake needs component libs + Eigen3 | never |
| 3 | `cp bba/bbasm` — in-tree cmake puts binaries at the build root *(mine)* | never |
| 4 | bare prjxray clone; its yaml-cpp is a submodule | never |
| 5 | `cp xc7frames2bit` from `build/` — the binary is in `build/tools/` | never |
| 6 | `cp` into a directory nobody ever created | never |
| 7 | `cd` inside an if-branch made the final copy path relative to the wrong tree *(mine)* | never |
| 8 | the driver binary looks for nextpnr at a hardcoded path; pass `--nextpnr` | never |
| 9 | my own 45-min ceiling killed a healthy 46-min job — **a ceiling must clear the honest worst case, not the median** | n/a |
| 10 | the emitted XDC named pin `C18`, which the device does not have | never |

Layer 10 is the important one. The job's "chipdb" had been **1 MB of `/dev/zero`**,
and a zeroed database cannot reject a wrong pin — so the placeholder was not merely
failing to produce a bitstream, it was **masking wrong design constants**. A fake
artefact in a pipeline hides real bugs downstream of it; that is the same class as
the ring-oscillator MHz figure, in infrastructure instead of a paper.

Rules that would have cut ten round-trips to about three:

- **Dry-run the whole job's shell locally before pushing, not just the part you are
  fixing.** Paths, `mkdir`, clone URLs and `cp` targets are checkable on any OS. The
  three layers I did test locally were found before CI; the six I did not each cost a
  round-trip.
- **Treat every line after the historical failure point as unreviewed code**, because
  it is: nothing has ever executed it.
- **When a fake artefact is found (placeholder chipdb, stub data), assume everything
  downstream of it is also unvalidated** — including constants that look unrelated,
  like pin assignments.
- Text anchors for patching workflows mis-hit on indentation constantly; **edit YAML
  by line number and re-validate with a parser**, never by matched-string replace.

## 10. The onion's last layers are contracts between tools — and defaults are contracts too

The fpga-bitstream job's first real runs (after §9's ten never-executed layers were
fixed) died four more times. None of the four was a defect in any tool; each was a
mismatch between what one side EMITS and what the other DEMANDS:

| layer | emitter said | consumer demanded | fix |
|---|---|---|---|
| 11 | prjxray staged in `~/` | t27c hardcodes `build/fpga/prjxray` + PYTHONPATH + `build/nextpnr-xilinx/.../prjxray-db` | stage at the paths the driver reads (grep the driver for `join(` paths BEFORE staging) |
| 12 | `requirements.txt` | fasm2frames' real import chain | requirements pinned editable submodules + incompatible tools; the honest set is what `--help` actually imports — verified by RUNNING it locally against the exact package list before pushing |
| 13 | metrics step demanded `bitstream.bit` | t27c names output after the top module (`zerodsp_top.bit`) | glob `*.bit`; print the real name |
| 14 | a parallel branch flipped the `--device` CLI default (200T SSOT) | this workflow relied on the old default AS AN ABSENCE — it never passed `--device` | state the device explicitly; an absence cannot conflict in a merge, so the flip landed textually clean and broke two steps later |

Rules distilled:
- **Read the consumer's source for its default paths before staging artifacts.** A
  driver that hardcodes paths defines a contract; the workflow satisfies it or passes
  flags — guessing standard locations satisfies neither.
- **Verify the import chain by executing, not by reading requirements.txt.** Archived
  projects pin dead versions and editable paths. `PYTHONPATH=clone python3 tool.py
  --help` on the exact candidate package list is a one-minute local negative control
  that pre-empts a 45-minute CI round-trip.
- **Pre-verify the NEXT layer while fixing this one.** Layers 11-13 were all checkable
  in advance (grep the driver, run --help, one GitHub-API dir listing). One push fixed
  all three; three pushes would have discovered them one at a time.
- **Two upload-artifact@v4 steps with the same name are a landmine that only passes
  while broken**: v4 errors on duplicate names, so the second step works only as long
  as the first finds nothing. Fixing the first ARMS the second. Audit artifact names
  whenever you fix a "file not found" upload.
- **A gate's regex is part of its contract: read it before writing the commit
  message.** L1 requires `(Closes|Fixes|Resolves|Refs|Updates)\s*#N`; a human-plausible
  "Part of #2215" fails. One grep of the gate's source beats one failed run.
- **A consumer that relies on a CLI default encodes that reliance invisibly.** A
  semantic default change auto-merges with zero textual conflict, passes every
  path-trusting step (the wrong value flows through unexamined), and fails at the
  first step that looks a NAME up in a table. State the dependency explicitly — one
  flag makes the assumption mergeable-visible.
- **apt on a dead mirror HANGS, it does not fail** — retries without `timeout` around
  each attempt never fire, and the job burns its whole ceiling doing nothing (5/5 jobs
  at once on 2026-08-19: that is mirror weather, not a per-job lottery). Bound each
  attempt (`timeout 420/600`), retry a bounded number of times, fail fast and loud.

## 11. The formal onion: seven layers, and what each one teaches

fpga-formal was green for its whole life and had never run a solver. Peeling it
took seven layers, each invisible until the previous one was cured:

| layer | defect | lesson |
|---|---|---|
| 1 | .sby "task blocks" were indented pseudo-syntax sby does not parse | a config dialect is a contract: read the tool's format docs, not a plausible-looking example |
| 2 | [files] paths escaped the workspace (../../../) | paths in configs are resolved by the TOOL's rules, not yours |
| 3 | `if sby \| tee` without pipefail tested tee | under `bash -e` every pipeline's exit is the LAST stage; pipefail or die |
| 4 | sby resolves [files] against the INVOCATION cwd, not the .sby location | run the tool from where its config assumes; verify with the workdir it creates |
| 5 | [script] read_verilog lacked -sv -DSIMULATION | every reader of generated code must use the repo's own dialect flags |
| 6 | the copy chain preferred April-vintage committed .v over the artifact generated minutes earlier | fresh generated output FIRST; stale fallback loud; absence fatal. Grep for committed copies of generated files — they shadow silently |
| 7 | props modules never instantiated the DUT, used SVA yosys cannot parse, and mirrored a port interface the generated modules never had | a property file that elaborates is not a property file that CHECKS anything: the DUT instantiation is the property layer's first assertion |

Rules distilled:
- **The engine error's name lives in the job's ARTIFACT (per-task logfile.txt), not
  the job log** — the log says only "engine did not return a status". Download and
  read before theorizing.
- **Run the tool's whole chain locally before the CI round-trip** (yosys prep +
  write_smt2 + yosys-smtbmc -s z3 reproduces sby's core without sby) — layers 5-7
  and the latch finding cost minutes locally vs 25-minute CI cycles.
- **Prove properties against a simulation cross-check first**: the uart result==1
  invariant was scanned exhaustively (256/256) before being asserted; a property
  you cannot cross-check is a guess with syntax.
- **A latch in a comb design blocks the SMT model AND is a design smell**: write_smt2
  rejects $dlatch; clk2fflogic exposing a "logic loop" means real combinational
  feedback through the latch. Park the config with a named issue rather than
  deleting it (.sby.blocked with a header).
- **Self-healing watches beat reporting watches**: BEHIND → server-side
  update-branch (works despite allow_update_branch:false); DIRTY on an append-only
  file → union-merge of every hunk, guarded to that one file and known shape;
  anything else → report and stop. Six NOW.md races were resolved by hand before
  the watch learned to; zero after.

## 12. The executed-vector registry: three verdicts, never two

When a corpus of "conformance vectors" has never been run, the repair is not
"run them all" — it is a REGISTRY that sorts every artifact into exactly one of
three verdicts, each with a different obligation:

| verdict | meaning | obligation |
|---|---|---|
| **executed** | a call template maps the case shape onto real RTL entry points | gate hard; a planted fault must fail (prove it once per renderer) |
| **named debt** | executable in principle, blocked by a NUMBERED defect | print it in the job summary; link the issue; never count it as covered |
| **aspirational** | describes behavior no current interface exposes (bit-level protocol vectors against a combinational model; prose-only cases with no data fields) | classify honestly and stop — executing them would test an invention, not the design |

Rules the first registry distilled (t27 #2241, mac 18 cases in one day):
- **State-writing calls poison "initially" reads.** Stage every case: fresh
  reads → stateless → stateful ops → post-op reads. The gate's own first catch
  was its own sequencing bug — a TB that ran ops before an "initially" read.
- **An observability gap is a SPEC defect**: if a vector checks state nothing
  reads back (reset → accumulator), add a pure reader to the spec rather than
  bending the TB.
- **Different entry points, same registries.** The one-line class kill of the
  day: the simulation path passed None as the spec path, so import-aware
  registries never loaded THERE while the plain path elaborated clean — 95
  errors from one unplumbed parameter. When a compiler grows a registry, grep
  every `compile_*` entry point for the parameter that feeds it.
- **A phantom type can survive for months on a fallback width**: mac referenced
  TernaryWord{raw} which NO file declared (the same-named struct elsewhere had
  different fields); the 32-bit fallback happened to be right. When a field
  access flattens to an unbound name, first ask whether the TYPE exists at all.
- **Symbolic dimensions are one resolver away from a whole feature**: array
  dims spelled as const names never parsed, so AoS declarations emitted TODOs
  and every access flattened. One pre-codegen substitution pass (integer-literal
  consts into type strings) unlocked declaration, literal and access at once.

## 13. Emitter-class repair: measure the radius, then measure each arm

One evening took the 32-module fpga set from 573 iverilog elaboration errors to
186 across three emitter classes. What made that possible was not insight about
Verilog — it was refusing to write a line of code before a number existed, and
refusing to believe the fix worked until a second number said so.

**Measure the radius before the change.** "A string field blocks lowering" is a
guess; "100 of 438 structs are rejected ONLY by string fields, every other field
being a primitive scalar" is a decision. The second sentence also tells you the
change is safe: a struct that never lowered cannot have its layout shifted.

**Measure each arm separately, and believe the zero.** The unannotated-local fix
has two arms — initializer is a call, initializer copies a param. The call arm
was written first because it is the obvious one, and it moved the count by
EXACTLY ZERO (213 → 213). The copy-of-parameter arm is what this corpus needed.
Had both landed together, the commit would have claimed a fix for something that
never fired.

**Verify ordering instead of assuming it.** The local-type registration reads
`param_types`; if params were registered after locals, the lookup would silently
read an empty map and the fix would "work" for the wrong reason. Four lines of
source settled it. A fix that works by accident regresses the moment the
accident stops.

**Fixing one flattening class can create another.** The first nested-path
version resolved exactly one trailing field and emitted
`cat[(0 + i*233 + 40) +: 160]_luts` — a part-select with an identifier glued to
it, which is the *same* defect one layer out. yosys caught it in one smoke run
(32/32 → 31/32). Always run the full smoke set on a change to the expression
emitter; the new defect will not be where you were looking.

**A control that does not fire is not a control.** A negative control for the
elaboration ratchet was written as a no-op string replacement; the gate "passed"
and proved nothing. Controls need their own verification: change the input, see
the output change, then restore. Twice this month a silently-inert control
almost certified a gate that could not fail.

## 14. Ten ways a gate lies, from auditing eleven of them at once

Every gate in `tools/` was put to five questions -- does it count what it
claims, can it pass having done nothing, does its scope match its prose, can
its ledger be lowered silently, and is anything it says about itself false
today. 63 candidate findings, **43 survived independent refutation**. The list
was the evidence; these classes are the product. Read them as a checklist for
the next gate you write.

**A. Departure scored as repair.** The most repeated shape -- six gates. A
thing that LEAVES the measured set produces a lower number, and a lower number
reads as progress: a deleted spec, an ungenerated module, an emptied vector
file, a re-sealed spec, a relabelled pack. Every one of these ratchets counted
in one direction and never asked whether the baseline still covers the corpus.
The repair is one line each: iterate `set(now) | set(base)`, compute
`known - now`, and report a disappearance as its own failing class.

**B. Exit status mistaken for the property.** `rc == 0` becomes "generates",
"parses", "the vectors ran". The artefact is never opened -- 264 specs emit
syntactically invalid C while counted as generating, and an empty file returns
0 from every backend. Twice the compiler already ships the stronger answer
(`parse-complete` distinguishes consume-all from DISCARD from fail) and no gate
calls it.

**C. A dated measurement written as a standing fact.** Fifteen findings, one
mechanism: measure once, write it in present tense, never re-derive. Seven had
already been copied outward into a workflow header, `NOW.md`, or a commit
message. The healthy cases all share one feature -- an explicit "state when
this was written" line. Where that phrase exists the snapshot is defensible;
where it does not, the number is a live lie.

**D. The broken ruler -- the instrument inside the failure domain.** A syntax
error truncates the parse that produces the error count, so breakage reads as
improvement. `dangling` vs `phantom` is decided from git history that CI's own
depth-1 checkout removes. A self-test built from the schema it validates cannot
reach the branch that drops other schemas.

**E. Negative controls that cannot fail for a defect in the gate.** Four
instances, under CI steps named "must be falsifiable". One asserts a
comprehension it wrote itself and survives three mutants of the real logic;
one plants only rows the gate can already parse; one never calls the function
whose filter is the bug. The correct pattern was already in the tree twice:
plant a fixture, call the REAL scan().

**F. No floor: zero work passes.** `0 == 0` prints "canonical". An empty
generated directory prints "no module gained errors". No gate asserted
`rows_checked > 0`. This is section 1 of this file violated by gates that cite
it.

**G. Anomaly swallowed as absence.** An unreadable file becomes `(0, 0)` --
"nothing to check". A missing hash key becomes "no freshness requirement". A
NaN comparison becomes "no mismatch". The safe reading of an anomaly is
failure; each of these chose silence. The correct pattern was in-tree here too.

**H. Scope declared by prefix or literal while the rationale is a count.** A
fixture exclusion covering 29 files for a reason enumerating 21. A "32-module
set" that is a hardcoded array, not a directory. A population defined by an
`int16_t` baked into a regex. Someone counted the things they were thinking
about, then wrote a selector catching a different set.

**I. CI reach narrower than gate reach.** A `paths:` filter omitting the gate's
own script and ledger -- so `return 1` -> `return 0` lands in a PR that never
runs it. A self-test in no workflow. A gate wired into nothing at all. A
checkout depth that changes the answer. "It is enforced" and "it runs on the PR
that changes it" are different claims, and the second is the one that matters.

**J. A self-declared label trusted where a measurement was available.** A pack
kind skips the row check without counting the rows. "bitexact" names a property
nothing verifies. Family distinctness asserted from mutually exclusive regexes,
i.e. by construction. The data needed to check was in the file the gate had
already parsed.

**And the audit itself was wrong once.** It proposed a `source=` uniqueness
check for the catalog; `source=` is a citation and 30 of 109 rows legitimately
share one, so that check would have failed on the clean tree the day it landed.
A finding survives refutation only in the direction it was checked -- verify a
proposed fix against the corpus before applying it, including one that arrived
with evidence attached.

**Your own gate is an instrument, and instruments lie the same way.** The
elaboration ratchet counted every stderr line containing `" error"`. iverilog
closes a failing file with `N error(s) during elaboration.` -- a TOTAL, which
matched. One phantom per failing module, 25 of them, and the number went into
a published post. What caught it was classifying the output by message shape
instead of counting lines. Do that once for every gate you write: print the
distribution, read the rows, and check that each row is a thing you meant to
count.

**A count is not a quality score, and fixing something can raise it.** A syntax
error TRUNCATES the file. Removing four of them revealed five real elaboration
errors that had never been reached, so the ratchet reported `WORSE 4 -> 5` for
a strict improvement. Two consequences: (a) the gate is right to demand an
explanation for every increase, which is exactly what it did; (b) the reason
must be written NEXT TO THE NUMBER in the baseline, not only in the commit
message -- the next reader sees the file, not your PR.

**The same defect shape returns wherever nobody swept the siblings.** A
keyword-named identifier was escaped at its declaration and printed bare at its
use. That had already been fixed once here, for a local array's declaration and
initialiser, with a note saying the expression paths were fine. The part-select
paths were never checked. When you fix "escaped here but not there", grep for
every OTHER place the same value is printed before you close it -- the root is
usually one variable serving as both a lookup key and emitted text.

**Classify the WHOLE list, not its head.** After three fixes I wrote that "the
remainder is two design decisions" — from the top six rows of the error output.
Checking all of it later gave 48 unique names and 68 references, and the claim
happened to hold: 56 string reads, 12 unsized-array reads, zero anything else.
It held; it was still stated before it was checked, which is the same sampling
habit that produced a five-fold overcount elsewhere in this repo. A remainder is
a claim like any other — enumerate it exhaustively, or say you sampled.

**Hold the win with a per-module ratchet.** 573 → 186 is invisible to a job that
executes two modules and lints the rest with a tool (yosys) that accepts what
iverilog rejects. The baseline records the count per module, fails naming the
module and both numbers, and does not demand zero — the remainder is two named
design decisions, not an oversight.

## 15. What the second batch of repairs added to the ten classes

Twenty-eight of the forty-three findings are closed. Eight more repairs landed
after section 14 was written, and they sharpened three of its classes and added
one habit that is not in the list.

**Class H sharpened -- a scope that conflates two purposes.** A fixture
exclusion held 29 files under one rationale enumerating 21. The eight extra
were not slop: five were parser CONTROLS with their own assertions, meant to
pass, and three were damaged inputs that had started generating. One list, two
opposite intents, and the gate could not tell them apart because the selector
never distinguished them. Split the list before you fix the count -- the count
was the symptom.

**Class J sharpened -- the row already carries the oracle.** Before writing a
second tool, look at what the data stores about itself. Every conformance row
carried a hex twin of each decimal it states; nothing compared them. Measured
first: 3795 rows, 7590 pairs, all agreeing -- so the check was free, in-corpus
and total. A second instrument you have to build is a reason to look harder for
one you already have.

**Class D sharpened -- an instrument fix looks exactly like drift.** After
giving a gate git history, fifteen ledger entries changed class. Nothing in the
tree moved; the answer improved. A ratchet that compares states must suppress
movement inside a pair whose distinction the instrument itself concedes it
cannot always make -- and must say why in the code, or the next reader deletes
the suppression as dead weight.

**The new habit: mutate every half of your own patch, not the halves you
expect to matter.** A three-part fix landed with two mutants. Reverting the
third part left the whole selftest green, because the fixture made that
assertion unobservable. The check was decoration for as long as it existed. If
a patch has N independent parts, it needs N mutants, and the one you are least
worried about is the one most likely to be untested.

**And a discipline for the reports themselves.** Three separate controls in
this batch failed because they exercised the wrong branch: one mutated a pack
id the fixture does not contain, one ran a tool from a directory where its
ROOT resolves to `/`, one planted a fault the OLD branch already caught. State
in the control which branch it exercises and how you know -- usually by naming
a string only that branch prints, and by asserting the neighbouring branch's
marker is ABSENT.

## 16. The control has a failure path too, and it is usually the exit code

Every §15 lesson is about which branch a control exercises. This one is about
what a control **omits by construction**, and it produced the two findings of
the fifth batch.

**A control that calls the checking function proves the function.** It does not
prove the wiring from that function to the process exit code, and CI reads only
the exit code. Measured on `check_catalog_integrity.py`, one variable changed:

```
main(): return 1 -> return 0
  gate on a catalog with a dangling source=  ->  "OK: 109 catalog rows...", exit 0
  --self-check                               ->  "all branches proven red", exit 0
```

The gate was completely dead. Seven per-branch cases all reported success. The
in-process design was not a mistake -- it exists so module-level `ROOT` can
never resolve to `/`, a trap this campaign fell into for real. So **add** an
end-to-end layer rather than replacing the precise one: copy the script into
the planted tree and spawn it there, and `ROOT` resolves to the planted tree by
the ordinary `parent.parent` rule -- no `--root` flag, no environment override,
no new way to aim a live gate at somewhere harmless.

**Controls plant data faults; gates also have preconditions.** A control builds
a well-formed world and then breaks one fact inside it. It never breaks the
world's *existence*: the missing baseline, the tool that would not run, the
unreadable directory. Nine of twelve gates here had every data branch covered
and no precondition branch covered. Those paths are one `return` away from
turning the gate into a silent pass that announces itself:

```
no baseline; run --update-baseline once      <- printed
exit 0                                       <- and green
```

**`tri gates mutate` is the check.** It flips each `return 1..4` outside the
control's own functions to `return 0`, one site at a time, and demands the
control notice. `sweep` reports whether a control EXISTS -- a label. `mutate`
reports whether it can FAIL -- the property. Do not let a count of controls
stand in for evidence; that substitution is §14 class J, and it was in the tool
written to find §14 class J.

**A survivor is not a broken gate.** It says nothing proves the gate will keep
working, not that it is wrong now. Check by hand before writing it up: all six
baseline-backed gates here exit 1 correctly today with their baseline moved
aside. "Nine gates are broken" would have been false and alarming; "nine
controls do not cover their preconditions" is true and actionable.

**Finally: do not mutate with one regex.** `return 1..4` matches the returns
inside `self_check` too. Doing that scored two sound controls as vacuous passes
-- they had detected the mutant correctly and merely lost the ability to say
so. Reading the printed output rather than the exit code is what caught it.
The broken-ruler error applies to the experiment you run on your own tools.

## 17. The audit tool is a gate, and it lies the same ten ways

§16 added `tri gates mutate` and reported nine gates with surviving mutants.
Using it for one more day found two defects **in the tool**, and both are
classes already in §14 — which is the point.

**It ran ONE control per gate, and invented a survivor.** The flag lookup took
the first match. `check_duplicate_agreement.py` declares two, and the one it
picked never reaches that gate's drift verdict; the other kills the mutant in a
line. The published table said `SURVIVED at 298`. Nothing in the repository was
wrong; the tool was. §14 class H, scope decided by a convenient rule rather
than by what the rule is for. **A gate can have more than one control. Read the
set, not the first element.**

**Its control map was 1:1, so a shared control could not be expressed.** Once
one control covered the precondition branch of six gates, those branches kept
reporting as survivors while the control that covers them sat in the tree.
Same "it exists but nothing connects it" defect §13 is about, in the auditor.

**Classify by reading the sites, not by a proxy for them.** The claim "six of
the nine share the precondition shape" came from *does this gate keep a
baseline file*. Reading all twenty sites: seven are preconditions, thirteen are
ordinary verdict branches, one of them a gate's MAIN verdict. The proxy was
cheap, plausible, and shipped in an issue and a blog post before anyone read
the thing it stood for.

### The precondition class, and how one control closes it

A control plants a fault INSIDE a well-formed world. It never breaks the
world's EXISTENCE. **An empty tree makes every precondition fail at once** --
no baseline, no compiler, no specs, no seals -- so one control covers every
gate that has one, and new gates join by a table row. Copy the script INTO the
empty tree so `ROOT` resolves there by `parent.parent`: no `--root` flag and no
env override, so covering the class adds no way to aim a live gate somewhere
harmless.

Two things it must do that an exit-code check does not:

- **Assert the message.** These gates reach one exit code from many branches. A
  gate that fails by CRASHING satisfies "it went red" and satisfies nothing.
- **Stage it.** Preconditions are checked in order and the first to fire hides
  the rest. A bare tree reaches "t27c not built"; a tree with t27c reaches "the
  scan matched nothing, the instrument is broken". The first version of the
  table expected the second message from the first stage, and the file's own
  control is what said so.

**Name what you do not cover in a constant, not in a count.** Two sites here
sit behind a tool check, so their message depends on the machine, and an
assertion whose expected value varies is not one. `UNCOVERED = 2` next to the
reason beats a reader inferring completeness from a green.

**Line numbers in comments go stale inside the same commit.** The note listing
those two sites said `:346` and `:390`; the fix eight lines above them moved
both before the branch was ever pushed. Name branches by their message.

## 18. Five defects in one audit tool, and the shape they share

`tri gates mutate` was written to find gates whose controls cannot fail. Over
three days of using it, five defects turned up **in it**. Listed together
because the list is the lesson: an instrument that measures coverage is itself
a thing whose coverage nobody measures.

1. **First flag, not the set.** A gate with two controls was measured by one of
   them, and a fully covered line was published as a survivor.
2. **A 1:1 control map.** One control covering six gates could not be
   expressed, so those branches reported as uncovered while the file that
   covers them sat in the tree.
3. **One syntactic form.** It matched a bare `return 1..4` and missed seven
   ternaries and a `raise SystemExit(3)` -- 34 of 42 sites. It reported a gate
   whose every verdict is a ternary as having "no failure path to break".
4. **No baseline.** A mutant is killed when the control exits non-zero, so a
   control that is red BEFORE any mutation scored a perfect kill on everything.
   The exact inverse of the defect the tool exists to find.
5. **No cache clear.** Python keys a `.pyc` on (mtime in whole seconds, size).
   `return 1` -> `return 0` preserves the size and the loop writes well inside
   one second, so an IMPORTED gate can be served the previous state's
   bytecode. The sibling command in the same crate had already solved this and
   this one did not call it.

**One shape: scope decided by what was convenient to write, rather than by what
the rule is for.** (1), (2) and (3) are §14 class H. (4) and (5) are the
measurement replaced by a constant -- §14 class B and class D.

**Three of the five were found by an adversarial reviewer of the tool's own
OUTPUT, not of its code.** The brief that produced them was "default to
REFUTED, and re-run every command in the report yourself". One of them came
from a reviewer who went looking in the neighbouring module and found a
solved-and-uncalled function there. Reviewing a tool by reading its source
finds different defects from reviewing it by distrusting its numbers.

### The asymmetry that should set the tool's bias

A **missed** site stays an open question. An **invented** one gets published as
a defect in somebody else's work -- which this tool did, in an issue and a blog
post, before anyone checked it. So the digit test that decides "is this a
verdict" is deliberately conservative and carries its own negative tests: a
digit inside an identifier is not a verdict, and a bare name is not one either.

### Closing a survivor: what a good case does

- **Assert the message AND the siblings' absence.** Several gates reach one
  exit code from three or four branches. Verified to matter, not assumed: a
  fault planted so a case reds through the WRONG branch is caught on the
  message alone, and an over-broad refusal that names every file rather than
  the one unreadable file is caught only by the forbid list.
- **Build the configuration the LIVE gate is in.** `check_seal_coverage` had
  four end-to-end cases and none had a ledger present AND something outside it
  newly broken -- which is every day in this repository. Planting "a ledger
  existing hides every new breakage" passes all four and reds only the fifth.
- **Name the helper so the tool cannot mutate it.** A planting helper called
  `_plant` is excluded only by the accident of holding no `return 1..4`.
  Rename it to contain `self_check`.
- **Say what you did not cover, in a constant.** And say when a guard is
  correct by construction but **has not been seen to fire** -- by this
  repository's own rule that is not the same as working.

## 19. A permanently-red gate is a question, not a verdict

Three red gates were opened this campaign. **Two were the instrument, one was the
record, none was the product.** That ratio is the finding.

- **`emit-bitexact`** said "6 of 8 targets did not agree or did not run". Its own
  testbench wrapper lifted a module body into a bare `module tb;` and declared
  four hardcoded port names; the module had nine. Three targets never ran. Run,
  they agree exhaustively.
- **`coverage`** reported 136 stale seals. `t27c seal --save` writes
  `<dir>_<Module>.json`; the old files are bare `<Module>.json`. 99 of them are
  duplicates left behind by the rename, 81 with a current twin.
- **`Corpus ratchet`** named two unexpected failures. Both are the braceless
  `given`-style `test` block, which is parsed to its header and loses its body.

**Open the gate before you conclude anything about the code it guards.** All
three had been red for weeks and read as backlog.

### "Could not measure" and "measured and found wrong" are different verdicts

`emit-bitexact` printed one sentence for both, and the six was entirely the
first. A reader has no way to tell a broken instrument from a broken product.
Where a checker already distinguishes them internally — `None` vs `False` —
a tally that collapses them throws away the distinction the code paid for.

**And splitting the tally is not the fix on its own.** My first attempt did
exactly that and was *more confidently wrong*: `verilog_digest` returned `False`
for build failures too, so a broken testbench would have been labelled
`DISAGREED` in bold. Fix the source of the distinction, then the report.

### Build the instrument when the question has no command behind it

`parse-no-discard` gates nineteen specs and reports pass/fail. The one question
a reader asks — WHERE does the parser stop consuming — had no command, so nobody
asked it. `t27c parse-accounted --bisect` answered it in two runs.

It also **killed two plausible causes** I had already written down: "invariant
inside a test is discarded" (fifteen such in a passing spec) and "it is the
expression form" (all seven forms behave identically). A report built on either
would have sent someone to the wrong file. When a diagnosis is cheap to test,
the tool that tests it pays for itself before it is merged.

### Ledger reading, in one line

Read an "unexpected passes" list as **headroom nobody has**: the corpus ledger
was at 221 of a 221 cap with three fixed entries still in it.

### Two operational rules earned the hard way

- **`git stash` is repository-global, not worktree-local.** With agents in
  worktrees of the same repo, a `stash`/`pop` pair pops somebody else's work.
- **Backticks inside a double-quoted shell string are command substitution.**
  This ate text out of a commit message and then out of a filed issue, in the
  same session, after being written down once. Pass long bodies through
  `--body-file` or a heredoc with a quoted delimiter, never `-m "…\`x\`…"`.

## 20. A number that is correct and misleading

The hardest finding to catch in this campaign was not a wrong number. It was a
right one attached to the wrong meaning.

**"26,546 tokens of specification never reach codegen."** Every part of that
sentence was measured. Two thirds of it is `forall`-quantified statements that
`compiler.rs` documents, in the function that skips them, as *"not
runtime-checkable"* — a decision the project made, wrote down, and half-fixed
already (the artefact stopped printing `verified` and now prints `NOT CHECKED`).
The actionable number was 9,547.

A wrong number gets caught by anyone who re-runs the command. A right number
with the wrong meaning survives re-running, because re-running reproduces it.

**Before publishing a corpus-wide count, grep the code that produces the
behaviour for a comment about it.** One `grep forall bootstrap/src/compiler.rs`
would have found the sentence. It cost nothing and it was not done, twice: once
before filing the issue and once before writing the census.

### Your own tool truncates, and a census inherits it

`--spans` printed the first 40 discarded lines per file. The census ran over
that output. Every file with more than 40 discarded lines was undercounted —
**exactly the files a census is about.** It reported `forall` on 415 lines;
uncapped it is 857, which then corroborated the compiler's own count of 837 and
made the whole thing legible.

Two rules from that:

- **A display limit is a measurement limit the moment anything reads the
  output.** Give it a `--limit 0`, and make the default SAY it truncated. A list
  that just stops looks like a list that ended.
- **The truncation bias always points the same way.** It hides the tail, and the
  tail is the population you are studying.

### Seven over-generalisations in three days, all measured away

"Invariant inside a test is discarded" — a passing spec has fifteen. "It is the
expression form" — seven forms behave identically. "It is the braceless test
block" — the biggest contributor is a `forall` property. "Six of nine gates
share the precondition shape" — a proxy for reading the sites. And three more.

The pattern is not carelessness about evidence; each had evidence. It is
**reaching for the general statement one step before the measurement supports
it**, because the general statement is what a report wants. The discipline is to
write the specific one and let the reader generalise.

## 21. Grep --help for the noun in your own commit message

Twice in two days I built a tool that already existed, one layer down each time.

- `parse_ast_dropped_spans` was in the compiler, returning every discarded token
  as `(line, lexeme)`. I wrote a bisection that reaches the same answer by
  removing items one at a time.
- `t27c parse-complete --show <spec>` was in the CLI, printing those tokens
  grouped by line **with the source line beside them**. I wrote
  `parse-accounted --spans`, which printed the tokens alone and capped the list
  at 40. Its no-argument form is also the corpus census I then hand-rolled in
  Python — same figures to the token.

§14 already says to look for the right sample in the tree before inventing a
coarser one. Knowing the rule did not fire it, because the moment you reach for
a tool you are thinking about the problem, not about the toolbox.

**The check that would have fired: take the noun out of the sentence you are
about to write in the commit message, and grep `--help` and the source for it.**
"How many tokens does the parser DISCARD" → `grep -i discard`. Ten seconds,
twice missed.

### Two commands answering one question is a drift generator

The consolidation is not tidiness. Two seal-naming conventions left 99 orphaned
records that a gate reported as stale work for weeks. Two commands for one
question do the same to knowledge: the next reader finds one of them, and which
one is chance.

Fold, and keep only what is genuinely new — here, `--bisect`, which says WHICH
construct the parser stops on where `--show` says WHAT it dropped.

### When the freeze says don't, file instead

Two real defects turned up in `compiler.rs` while doing this: a comment saying
"one of four discard channels" where five sites increment the counter, and a
`dropped_spans` cap of 20,000 against an uncapped count, so a large enough file
would make a summary and its detail view disagree with nothing saying which.

Neither is worth spending a stage0 freeze on. **Filing with the measurement is
the finished state of that work, not a postponement** — and the issue is the
right place for the remedy sketch, so whoever does spend the freeze does not
re-derive it.

## 22. A command cannot certify an answer to a question it does not ask

`t27c classify` reads the opening of a file to decide whether it is code. Its
closing sentence read *"Anything outside SOURCE cannot parse"*. Measured: 5 of
28 non-SOURCE files parse fine.

The **direction** was right — counting a Markdown document as a failing spec
does inflate a corpus ratio — and the **stated reason** was a different claim
entirely, about the rest of the file, which this command never looks at.

That is the shape to watch for: a tool that measures X, and a sentence in its
output that asserts Y because Y feels like it follows. It survives review
because the conclusion is correct and only the justification is invented.

**The repair is not to delete the sentence.** State what the number is for, say
explicitly what does NOT follow, and name the command that IS the authority on
the other question. Two commands cross-referenced then say what neither says
alone: of 154 specs that do not parse, 131 are SOURCE — so the not-code
correction is worth 23, not 28.

### Cross-reference before you build

Every number in that finding came from two existing commands. §21's lesson —
grep `--help` for the noun before writing a tool — has a second half: once you
have found the tools, **the answer is often the intersection of two of them**,
and neither author had a reason to compute it.

### Read the raw section before reporting an inconsistency

I nearly filed "the summary says 5, the detail view lists 7 — the command
contradicts itself". It does not. My line filter was catching the closing prose,
which contains the string `.t27`. The section itself lists exactly five.

An inconsistency inside one command's own output is a strong claim. It is also
the easiest kind of finding to manufacture with a sloppy filter, because you are
grepping output whose format you did not write and did not check. **Print the
section and count by eye before writing it up.** Six of my own instruments have
been broken in this campaign; this is the first one that would have accused
somebody else's working code.

## 23. Backlogs stack; find out which one is underneath

Three investigations in this campaign looked separate and were one.

```
coverage is red            121 stale seals
  99 orphaned by a rename       81 have a current twin  -> bookkeeping
  18 twins themselves stale     14 DO NOT PARSE         -> not bookkeeping
                                                            at all
```

Sealing requires generating; generating requires parsing. **The seal backlog is
downstream of the parse backlog**, so no amount of re-sealing moves the gate.
Measured: re-sealing all 18 twins moved the stale count by **zero** — 14
rejected by all four backends, 4 changed nothing but a timestamp.

Before proposing work on a red gate, ask what the failing items need in order to
succeed, and whether *that* is also failing. A plan that treats a downstream
symptom as the unit of work produces a diff and no movement.

### Two proposals, both refuted by running something that already existed

- "Split the 131 non-parsing specs by error class; if one dominates it is one
  fix." `t27c backlog` had measured it: **0 specs at depth 1**, and its help text
  records that removing the most frequent cause (435 sites, 140 specs) moved the
  compiling count 151 → 151.
- "Re-seal the 18 twins — pure improvement, no deletions." Zero movement, above.

§21 said to grep `--help` for the noun before writing a tool. Its stronger form:
**grep it before writing the PLAN.** Both of these were an iteration of work
each, and both were answered by a command whose entire purpose was that question.

### An iteration whose output is two refutations is a finished iteration

Nothing went into the tree here but a note. Two proposals were wrong, the
measurement says so, and a commit that made the numbers move would have had to
be invented. The temptation to produce a diff is strongest exactly when the
honest result is "the plan was wrong" — and a diff manufactured then is the
purest form of the thing this whole skill is about.

## 24. Check the regression before proposing the next thing

Seventy merges landed in the twenty-four hours after this campaign's work. None
of them was reviewed against the numbers the campaign produced. **Re-running the
measurements is the first item of the next iteration, not an afterthought** —
and it is the only way "the new cycle does not break the old work" is a fact
rather than a hope.

Everything held. The detail worth keeping is *why* the most fragile piece held:
`check_gate_preconditions.py` names its two uncovered branches **by message**,
and other people's edits moved those lines from `:359/:403` to `:451/:495`
without breaking the declaration. A line number in a note is a claim with an
expiry date nobody sets.

### The decision brief is part of the work, not a wrapper on it

Eight questions accumulated that are an owner's to answer: repository security
settings, deleting reproducibility records, corpus-wide language decisions. Left
as eight issues they are eight interruptions; collected with **options and the
cost of doing nothing**, they are one sitting.

Two properties make such a brief usable rather than decorative:

- **Every number comes from running a command**, named in the footer so it can
  be re-run. A brief that summarises other summaries inherits every error in
  them and adds none of the evidence.
- **It says how often its author has been wrong.** Seven claims in this campaign
  were published and then refuted by my own later measurement. A decision
  document whose author has that record should carry it, or the reader has to
  discover the reliability of the source by accident.

### When "fix it" is not on the menu, say what is

`coverage` is red for a real reason and cannot be moved by bookkeeping *or* by
one compiler fix — re-sealing moved it zero, and the backlog is zero specs deep
at depth 1. The honest options are a multi-month body of work or a decision to
call it accepted debt. Offering a third, cheaper-sounding option would be the
comfortable thing and there isn't one.
