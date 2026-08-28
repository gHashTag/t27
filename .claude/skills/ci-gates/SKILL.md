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
   ternaries and a `raise SystemExit(3)`: 34 of the 42 sites it could see AT THE
   TIME. It reported a gate whose every verdict is a ternary as having "no
   failure path to break". That denominator is a **dated measurement** — §33
   tightened the predicate and 42 became 36, because the loose one had been
   counting helper functions.
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

## 25. A note naming what you did not cover is a work item

`check_gate_preconditions.py` said, of two branches it left uncovered:

> Covering them needs a stage that requires iverilog and skips loudly without
> it; that is a bigger change than this file is, and it is filed rather than
> faked.

That sentence **was the design**: the mechanism, the tool it needs, and the
failure mode to avoid. Building it took one stage and moved the campaign's
surviving mutants from 2 to 1.

It sat unread for a day because it was written as an admission. Write the same
content as a task and it gets picked up; write it as a confession and it reads
as closed. **Both halves are worth having — the admission is what stops a reader
inferring completeness from a green — so put the design sentence in it too.**

### A tool that cannot be planted is its own class

`iverilog` lives on PATH, not in the tree. A stage that needs it cannot plant
it, so it reports **UNRUN** when the tool is absent rather than passing or
guessing. Three states, all measured, none inferred:

```
with the tool      the gate reaches its branch and reds
without the tool   UNRUN, named
branch neutered    VACUOUS, caught
```

The temptation is a fourth behaviour — skip quietly when the tool is missing —
and that is the exact defect the file was written to catch, one level up.

### Some branches genuinely cannot be covered, and saying which is the work

The remaining uncovered branch needs a build that **succeeds** before it can
fail the way it fails. Planting a fake success would test the plant, not the
gate. `UNCOVERED = 1` next to that reason is a finished state; a control that
faked it would be worse than none, because it would report a number.

### Quote characters in `-m` end the string

`git commit -m "... \"t27c fpga-build --smoke failed\" ..."` failed with
`unknown option --smoke`: the inner quotes closed the argument. Same family as
backticks being command substitution, which cost text out of a commit message
and then out of a filed issue earlier in this campaign. **Long messages go
through `-F` or a quoted heredoc, every time — there is no version of this
that is worth retyping.**

## 26. A fix in the tool is not a fix at the terminal

A reviewer found that `tri gates mutate` never cleared Python's bytecode cache:
a `.pyc` is keyed on (source mtime in whole seconds, source size), `return 1` →
`return 0` preserves the size, and the loop writes well inside one second. That
was fixed in the tool.

Two days later I hit the identical defect **by hand**, on the same gate, while
checking somebody's recommendation. Five quick mutations left a stale `.pyc`,
and the control then reported five failures on a tree whose `git status` was
empty and whose source sha matched HEAD. I spent a round chasing a phantom.

**Fixing a hazard inside one tool leaves it live everywhere a person does the
same thing manually** — and the manual path is the one used while investigating,
which is exactly when a false red is most expensive.

The repair belongs in the artefact that can be reached both ways. Here it is
three lines in the control: drop the imported module's cached bytecode before
importing it.

### Every verdict taken before you found the broken instrument is suspect

Not wrong — **suspect**. The control could have been red for the right reason or
the wrong one, and nothing in the output distinguishes them. Re-run all of them
and say in the write-up that the numbers are a re-measurement. I did, and they
held; if they had not, publishing the first set would have been the finding.

### Refuting a recommendation needs the same rigour as making one

The recommendation was "the drift verdict is not covered; add a whole-process
case". Three mutations of increasing narrowness — the broadest, the wiring only,
and the drift path alone with bad-input untouched — plus a **no-op mutation as a
control on the control**. All three killed, the no-op silent. The
recommendation is refuted, and refuting it needed more measurements than
accepting it would have.

An unexamined recommendation costs an iteration of work. Accepting one because
the reviewer was right about four other things is how a review's authority
outlives its evidence.

## 27. Sweep for the class, not the instance

`tri gates mutate` was taught to clear Python's bytecode cache after a reviewer
found it there. I then hit the same defect by hand in a control. Fixing that one
control would have been the instance.

A grep for *files importing a sibling module* found **three**, and the one nobody
had tripped over was the worst:

```
wp18_selftest_gate.py                 imports wp18_conformance_gate
wp18_gate_selfconsistent_selftest.py  imports wp18_conformance_gate
verify_exhaustive.py                  imports ternary_model
```

`verify_exhaustive` is the gate that proves four backends agree with a
**separately written** model. A stale model means the agreement is measured
against a version nobody is looking at — and that gate's whole value is that its
fourth opinion is independent.

**After fixing a defect, spend one grep on the shape of it.** The query is
usually mechanical: the language feature that made it possible (`^import (\w+)`
against the local module names), not the symptom. Two of the three had never
produced a wrong answer, which is why nobody had found them.

### Prove the fix on the instance you did not stumble into

The guard was verified on `verify_exhaustive`, not on the file where I hit the
bug: a same-size edit to the model gives exit 1, and reverting then running
immediately gives exit 0. Testing the repair only where you already saw the
symptom confirms the diagnosis you already had.

### A same-size edit is the whole hazard

`return 1` → `return 0`. `else 2` → `else 0`. `return acc` → `return aaa`. Every
mutation a mutation-tester makes is size-preserving by design, so a mutation
harness and this cache are natural enemies. If you are writing anything that
edits source in a loop, clear the cache in the loop — and remember the loop is
sometimes a person.

## 28. Two cases with opposite requirements need two fixtures

`check_specs_parse.py` compares `now > was`. Its control chose, deliberately and
with a comment, a spec whose recorded debt is **zero** — because a spec owing
1,139 tokens would swallow a ten-token plant and make the *drift* case vacuous.

That reasoning is correct, and it is exactly what left `was`-versus-`0`
unmeasurable. With `was == 0` the expressions `now > was` and `now > 0` are
indistinguishable. Measured: `if now > 0` **passes the control** and turns the
live gate red — a mutation that ignores the ledger entirely, invisible to the
thing that exists to see it.

**One fixture chosen by one rule cannot serve two cases whose requirements
point opposite ways.** The drift case needs debt small enough not to swallow the
plant; the ledger case needs debt large enough that the plant sits under it.

### The economical distinguisher is often "assert silence"

The obvious repair is a bigger plant that raises a non-zero debt. The cheaper
one is a plant that sits **under** the recorded figure and asserts the gate
stays quiet: correct code is silent, `now > 0` raises a false alarm.

A control made entirely of cases that demand red has a blind spot shaped like
every mutation that makes a gate louder. **At least one case should require
silence**, and it will usually be the one that catches a constant substituted
for a variable.

### Close the loop on guards written "correct by construction"

#2469 added a `try` around a `json.loads` so garbage stdout would record a
failure instead of aborting the control with a traceback. It was reasoned, not
demonstrated, and it stayed on the open list for that reason alone. Planting a
gate that prints non-JSON: `[FAIL]`, exit 1, zero tracebacks — and without it
the control dies there and every later case silently never runs.

A guard nobody has seen fire is the same evidential state as a gate nobody has
seen go red. Both are one planted fault away from being real.

## 29. Do not quote the line you are about to mutate

The comment explaining a new silence case quoted the branch it was about,
verbatim. A text-replacing mutation harness then hit the **comment** instead of
the code: the gate ran unmutated, the case reported itself blind, and I nearly
wrote up "the silence case does not work".

`str.replace(needle, mut, 1)` takes the first occurrence in the file, and prose
usually comes before code.

**Describe the branch, do not quote it** — "the one-group branch rewritten to a
constant false" reads as well and cannot be hit. If a literal is genuinely
needed, mutate **by line number** rather than by text, which is what settled it
here.

### Applying yesterday's rule found exactly one gate, and reading found it

A count of "silence-shaped" strings per control flagged one of thirteen.
`check_duplicate_agreement` had two controls and **neither required silence** —
both assert `returncode == 1`, so a gate reporting a split on a tree where every
copy agrees satisfies both.

Measured before the fix: two such mutations were caught **only** by
`--self-check-drop`, a control written to exercise a different branch. The
primary control passed them. **Coverage by a sibling's accident is not
coverage**, and nothing in a green report distinguishes the two.

### Say what the sweep did not look at

The count is a proxy. One file was read and confirmed; the other twelve were
scored by a regex and left there. The write-up says so. A sweep reported without
that sentence reads as twelve clean results, and twelve unexamined counts is a
different claim.

## 30. An operator that only pushes one way measures one way

For six days `tri gates mutate` turned failures into passes. Every mutant asked
the same question — *can this gate still fail?* — so a control made entirely of
cases demanding RED satisfied all of them, and the **opposite** defect went
unmeasured across thirteen gates.

`--loud` flips `return 0` to `return 1`: does anything notice a gate that fails
on a **clean** tree? On its first run seven of thirteen had survivors; all are
closed now, which is a fact about a date and not about the tool.

The worst was the file that enforces this discipline for six other gates:

```
check_gate_preconditions.py, success return forced to 1
  OK: 9 precondition(s) across 6 gates fail loudly
  exit 1
  control: exit 0 -- nothing noticed
```

It prints a clean bill of health and reports failure. That is the exact mirror
of this campaign's **first** finding, and it survived every iteration since,
because the tool and the blind spot shared a direction.

**When you build a mutation operator, write down what it cannot express.** Ours
could not express "the gate got louder", and nothing in a green report says so.
The generalisation is not about mutation testing: *any* instrument that
perturbs a system perturbs it along some axis, and the axes you did not choose
are invisible in exactly the way a passing test is.

### The survivors were one class, which is what made them worth reporting

Four of the eight are the success return of `--update-baseline`, right after
"baseline written: N entries" — nothing exercises the ledger-writing path's exit
code in any gate. A `--update-baseline` that writes the ledger and then reports
failure is invisible to every control in the tree.

One gate scored 2/2, and only because a case added days earlier for an unrelated
reason happens to run `--update-baseline`. **Note when a clean score is
accidental** — it tells the next reader that the coverage is not load-bearing.

### Land the tool and the finding; fix in the next change

Eight sites across seven gates were left open. Fixing them in the same commit
that introduced the tool that found them would leave nobody able to tell whether
the tool or the fixes were doing the work — and the tool is the part that has to
survive being wrong.

## 31. Assert the exit AND the effect

Four gates had the same unguarded site: the success return of
`--update-baseline`, right after "baseline written: N entries". **Nothing in the
tree ran the ledger-writing path at all.**

A case for it needs both halves:

- **Exit alone** passes a run that returns 0 without writing anything.
- **The marker alone** passes a run that writes and then reports failure — which
  is precisely the mutation that found the gap.

One gate scored clean here only because a case added days earlier for an
unrelated reason happened to run `--update-baseline`. One already had an
`--update-baseline` case, and it asserts the **refusal** to grow the ledger —
the opposite branch. A command with two outcomes needs a case for each; having
"a case for `--update-baseline`" is not the same as having covered it.

### Distinguish report modes from verdicts before counting them

Three survivors were behind flags CI never passes, and each file says so in its
own words: `--all` is *"a report, not a gate"*; `--allow-missing-tools` is
*"reporting nothing, deliberately"*. Those are a different finding from an
unguarded verdict, and folding them into one number makes the number mean less.

Name them with their reason. Eight survivors was accurate and "one class of
four, one mirror, three report modes" is what a reader can act on.

### The tool groups; the reading classifies

`--loud` put the same line in four reports, which is what made the shape
visible. It could not tell that those four were one fix and the other four were
three different things. **A tool that surfaces N instances has done its job;
deciding whether N is one finding or four is still reading.**

## 32. An opt-out that fails anyway is worse than no opt-out

`check_elab_ratchet` reds when its tools are missing, and offers
`--allow-missing-tools` to accept that locally. A sibling control proved the
red. **Nothing proved that the flag then returns success**, and the loud
operator rewrote that return to a failure with no assertion noticing.

An opt-out that fails anyway teaches that the flag does not work — and the next
person removes the guard instead of passing it. **Every escape hatch needs a
case proving it escapes**, not only one proving the thing it escapes from fires.

### A report is still a program with an exit code

Three of the last survivors were report modes behind flags CI never passes, and
each file said so: *"a report, not a gate"*, *"reporting nothing, deliberately"*.
The distinction is real and worth keeping. **Leaving the report's exit code
unmeasured is not part of it** — a report that prints its table and then reports
failure breaks any script reading it, and nothing would have said so.

### "No path to break" is unmeasured, not clean

**RETRACTED, see §33.** This section originally said two gates were in that
state because "the operator takes only bare returns on purpose, since forcing a
ternary's whole line to 1 is the silent operator seen backwards". **That reason
was wrong and I wrote it.** Silent forces the line to 0 — the gate never fails.
Loud forces it to 1 — the gate always fails. With ternaries allowed one of the
two measures fine; the other has no literal success return at all, its success
being `return code`, so the row is true there for a different reason.

The rule the section is for still stands: a row saying `no path to break` is an
**absence of measurement**, and the report has to say
which. A row that reads as a clean score when it is an absence of measurement is
the same substitution this whole skill is about — and it is easiest to commit in
your own tool's output, where you know what the row means and the next reader
does not.

### Name branches by message, not by line

The one remaining silent survivor is declared uncovered with its reason. Its
line has moved **four times** — `:390`, `:403`, `:495`, `:516` — under other
people's edits and my own, and the declaration matched every time. A line number
in a note is a claim with an expiry date nobody sets.

## 33. Re-read your own justifications; a wrong one that sounds principled survives longest

Two gates reported `no success path to break`, and §32 documented that as an
honest limit with a reason I had written:

> A ternary can yield 0 on one arm and a verdict on the other; forcing the whole
> line to 1 is the Silent operator's job seen backwards.

It is not. Silent forces the line to `0` — the gate never fails. Loud forces it
to `1` — the gate always fails. Two different mutants of one line.

The sentence had the shape of a careful distinction, which is exactly why it
lasted: it reads as someone having thought about it. **A justification that
sounds principled is harder to re-examine than an obvious mistake**, and it
printed a row that read as a clean score while being an absence of measurement.

### A predicate that scans for a digit is not a predicate for a verdict

Allowing ternaries found two survivors immediately and **both were false**:
`return out.splitlines()[0][:88] if out else "(nothing)"` and `return v == 0`.
Helper functions returning values. Both predicates scanned for a standalone
digit *anywhere* in the expression, so an index and a comparison read as
verdicts. The silent one had the same weakness: `return v == 1` would have been
a site.

Strict version: **a return is a verdict when the whole expression is a literal,
or a ternary whose two arms are literals.** Everything else is a value the
caller decides about.

### A denominator that grows can mean worse measurement

The counts fell — `check_elab_ratchet` 10 → 6 sites, `wp18_conformance_gate`
6 → 2. Those earlier kills were real: the controls did notice. They were
noticing **mutations of helper functions**. A higher denominator looked like a
more thorough measurement and was a measurement of the wrong thing.

When a fix makes your own numbers smaller, check which direction the truth moved
before deciding whether to be pleased.

### Three of my justifications have been wrong in this campaign

Two produced missing coverage. This one produced a row that **read as a score**,
which is worse: the first kind gets found by the next person who looks, and the
second gets quoted.

## 34. Audit your own skill the way you audit a gate

Thirty-three sections were written over six days, each in the moment, none
re-read with the question that catches a stale claim. So I ran the audit on the
file itself: every named file, every behavioural assertion, every number that can
be checked today.

**Ten file references, four behavioural claims, five numeric pairs. One was
stale, and one contradiction was live.**

- **The contradiction is the finding.** §32 stated, in the present tense, the
  justification that §33 retracts — and §32 comes first. A reader stopping there
  gets the wrong rule with a confident explanation attached. It now carries
  **RETRACTED, see §33** at the top of the paragraph, not a note appended after
  it.

- **The stale number.** "34 of 42 sites" was true when written and false the next
  day, because §33 tightened the predicate and the denominator fell to 36.
  Dated in place rather than updated: the point of the sentence is what the
  scanner missed at that moment, and a number that keeps being silently corrected
  teaches nothing about how it drifted.

- **A near-miss worth recording.** "30 of 109 catalog rows share a citation" read
  false at first — I counted 45. Both readings had to be computed before saying
  anything: 45 rows sit in shared groups, and **30 are duplicates beyond the
  first**, which is exactly what a uniqueness check would flag. The claim holds.
  Checking both readings is the difference between a correction and a false
  accusation, and this is the second time in three days that habit saved one.

### Prose you wrote is a claim like any other

Every rule this skill has about gate output applies to the skill: dated
measurements presented as standing facts, a label trusted over a property, a
justification that sounds principled. **The difference is that nothing runs
prose, so nothing goes red when it rots.** The only mechanism is re-reading it
against the tree, on purpose, with the same presumption of being wrong.

### A contradiction resolved in place, never appended

Repository doctrine here is to fix a contradiction where it lives rather than
adding a second truth next to the first. That is easy to honour in code and easy
to forget in a document, where appending is always the cheaper edit — and where
the reader who most needs the correction is the one who stops before reaching it.

## 35. The third axis, and the value of an instrument that finds nothing

Two operators asked whether a gate can still reach its verdicts. `--invert`
asks whether it reaches the **right** one: `if C:` becomes `if not (C):` on
conditions whose body carries a verdict. A gate that fires FAIL on a healthy
tree and OK on a broken one satisfies both return operators and neither is
looking at that.

Thirteen gates, **one survivor** — the same branch both other operators leave,
declared uncovered with its reason.

**That is the first new instrument here to find nothing, and it is worth as much
as the ones that found plenty.** Controls written over a week against two
questions held against a third they were not written for. A tool that comes back
empty on a corpus that has been failing every new question is evidence; a tool
that comes back empty on its first ever run is untested.

### Scope a mutation operator by what its kills would MEAN

Inverting any condition is easy and useless. Inverting a loop guard or a
plumbing check makes the gate **crash**, and a control that reds on a traceback
scores that as a kill — for the wrong reason, which is the mistake the command
exists to catch. Only conditions whose body holds a verdict return, a
`SystemExit` or a FAIL print are sites.

**Then measure that the scoping worked**, rather than asserting it: 19 mutations
across four gates, 19 killed by message, **zero tracebacks**.

### The probe that checks a tool gets less care than the tool

Mine was broken twice in one run: it mutated inside the control functions, which
the real implementation excludes and the probe did not; and its verdict label
conflated *not killed* with *killed by a crash*, so it printed `killed by crash`
for a mutant that had survived.

Its first output read as a finding about the tool. **A throwaway script written
to verify a careful one inherits none of its care**, and it is trusted anyway
because it is short. Seven instruments of my own have now been broken in this
campaign; the throwaway ones account for most of them.

## 36. A flag can be a banner

`tri gates mutate --invert` shipped, merged, and published a number. It never
ran the invert operator. `Direction::Invert` was declared and documented,
`invert_sites()` was written and unit-tested, and nothing joined them —
`mutate()` picked its direction with `if loud { Loud } else { Silent }`, so the
flag printed an invert header over a silent run.

Ten unit tests passed with the bug in place. Every one of them exercised
`invert_sites()` **or** `sites_in_direction()`; none crossed between them. That
is precisely the defect this whole command exists to find — the checking
function is covered, the wiring from it to the answer is not — arriving one
level up, in the auditor.

**Two things made it durable.**

The output was *plausible*: one survivor, the declared `UNCOVERED` branch, the
same one the other operators leave. Every part of that sentence was true, and
"the same branch as the other operators" was true **by construction** — it *was*
the other operator. A real measurement of the wrong thing agrees with whatever
story you already have.

And the operators were read one at a time. Three commands answering one question
cannot be cross-checked; three columns can. `--all` now prints them together,
and the first honest run shows them disagreeing exactly where they must:
`check_json_parses` has one silent site and no invertible condition,
`check_vector_data` five inverts against four silents.

**The check.** For any new mode, flag or direction, write the test that asserts
it produces something the *other* modes do not, on one fixture where all of them
have a site. Equality across modes is not a reassuring symmetry — it is the
signature of a fall-through.

**And the count.** First real measurement: 33 invert mutants across 13 gates,
33 killed, no survivors. The suite's one remaining survivor is
`check_elab_ratchet.py`'s no-baseline branch under the silent operator, declared
`UNCOVERED` by message.

**A sweep that could not find its own founding case.** After fixing `--invert`
I swept all 55 boolean CLI flags for the same shape, scoring "declared but
barely used" as suspicious at two or fewer references. Seven flags surfaced;
every one was a correct forward — declaration plus the call it feeds, which is
what two references *means*. And `--invert` with the bug in place had **three**:
signature, `if invert`, destructure. The threshold would have cleared it.

Static reference-counting cannot express the defect. The flag was used, in a
`println!`. Only a test that demands the mode produce a *different result* can
tell a working mode from a decorated one — which is why §36's check is a
behavioural assertion and not a lint.

## 37. The last survivor was a frame, not a limit

The suite's final surviving mutant was `check_elab_ratchet.py`'s "no baseline"
branch, declared `UNCOVERED` with a reason that had stood for a week:

> Reaching it needs `t27c fpga-build --smoke` to SUCCEED and then find no
> baseline, and a smoke build that succeeds needs the real spec tree — not
> something an empty directory can be given.

**Every clause is true. The conclusion does not follow.** A control does not
have to use an empty directory. The note reasoned entirely inside the frame of
`run_on_empty_tree()` — the helper that file happens to be built around — and
never asked whether a stage could keep the *real corpus* and empty only the
thing under test.

Splitting the two does it: `cwd=ROOT` so the smoke build finds real specs and
succeeds, `T27_ELAB_ROOT` pointed at a planted tree holding a built compiler, an
empty `generated/` and no baseline. The branch is reached in under a second, and
the real tree is only ever read — `git status` clean before and after.

This is §36's check paying out immediately. The justification sounded mechanical
and no measurement had produced it; one command falsified it. **When a
limitation is stated in terms of the helper you already have, the limit is
probably the helper.**

**End state, and its exact boundary.** 90 mutants across 13 gates — 36 silent,
21 loud, 33 invert — every one killed by the gate's own control. What that does
*not* say: that the gates are correct, or that their checks are the right
checks. It says no mutant in these three families survives, and the families are
narrow by construction (verdict literals, and conditions whose body carries a
verdict). A fourth operator would be a fourth question, and the honest prior
after this campaign is that a new question finds something — twice now the
instrument itself was what it found.

## 38. The fourth operator, and the count that would have been wrong

Yesterday's §37 ended with a prediction: *a fourth question finds something, and
twice it found the instrument.* Both halves held.

`--boundary` moves a comparison one place: `>` ↔ `>=`, `<` ↔ `<=`. The three
earlier operators ask whether a gate reaches a verdict and whether it reaches
the right one; this asks whether it reaches it at the right **place**. Ratchets,
floors and tolerances live on a boundary, and a control that tests *clearly
worse* and *clearly better* never tests **equal**.

**First run: 8 of 13 gates had survivors. That number was wrong.**

The scanner tracked quote state per LINE, so every `>` inside a multi-line
docstring became a site — prose about ratchets and usage on lines 10, 43, 136
and 230 of four different gates, reported as surviving mutants. Real count,
wrong meaning: §8.5 again, in the instrument, on its first run. Fixing the
scanner to carry triple-quote state across lines took it to **5 gates, 21
mutants, 9 killed, 12 survived**.

**And the survivor count still overstates the gap.** Classified by hand, because
a boundary mutant that lives can be a theorem rather than a hole:

| class | n | example |
|---|---|---|
| real threshold, closeable | 2 | `if prior and len(bad) > prior:` — a ledger ratchet whose `>=` form refuses on *no change* |
| real semantic, medium | 4 | `v[0] > 0` over non-negative counts: `>= 0` is always true, so the mutant changes what is counted |
| candidate theorem | 2 | `math.isinf(dec) and (inp > 0) == (dec > 0)` — an infinity is never zero, so the two forms agree *if* no zero input decodes to infinity |
| cosmetic | 2 | `if len(fixed) > 5:` guarding a "(+N more)" line — at the boundary it prints "+0 more" |
| plumbing | 1 | `len(parts) > 1` — `>= 1` would raise only on a line that splits into exactly one part, which no input produces |

Publishing "12 uncovered boundaries" would have been the same mistake as
"26,546 tokens never reach codegen": every word measured, the sentence false.

**The rule.** A new operator's first number is a claim about the operator, not
about the code. Read the surviving *sites*, not the count, before believing
either — and classify before publishing, because a boundary that survives is
sometimes a proof that the boundary does not matter there.

One closed immediately: the catalog floor. Every case in its control tested 0
against a floor of 109 — clearly under — so `n_ssot < MIN_ROWS` rewritten to
`<=` passed the entire control while failing every catalog sitting exactly *on*
the floor. A single case asserting that exactly `MIN_ROWS` is legal kills it.

## 39. Closing boundaries, and a classification that was wrong

Three of §38's twelve boundary survivors are closed, one is corrected, and the
correction is the useful part.

**Closed.** The catalog floor (every case tested 0 against a floor of 109, never
109 itself). The ledger ratchet — its control proved *refuses to grow* (1→2) and
*writes on shrink* (2→1) and never went through **equal**, which is where a
ratchet is defined. The re-derive tolerance: every case sat far from `1e-12`, so
nothing asked what happens *at* it. The arithmetic for that one is exact on
purpose — a row that decodes to its own input gives `rederived = 0.0`, and an
`abs_error` of exactly the tolerance makes the difference the tolerance itself
with no rounding anywhere.

**Closed rather than declared.** One survivor guarded a `"... and N more"`
display line and was classified cosmetic. Closed anyway, with a case planting
exactly ten departed entries: this campaign has twice found a written-down
limitation to be invented, and a declared exception costs a reader more than a
case costs to write.

**The classification that was wrong.** §38 called
`math.isinf(dec) and (inp > 0) == (dec > 0)` a *candidate* theorem "resting on a
property of the codec that has not been checked here". It rests on nothing of
the kind. The branch is guarded by `elif math.isinf(inp):` one line above, so
`inp` is infinite by construction and `dec` is infinite by the `and` that
short-circuits before the comparison. For a value in {+inf, −inf}, `x > 0` and
`x >= 0` agree — the only input separating them is zero, and neither can be
zero. **A proven equivalence, and I had written the caveat after reading the
comparison and not the `elif` above it.**

**So: `# mutant-equivalent: <why>`.** A survivor whose line carries the marker
prints its reason beside the row. It is **printed, never acted on** — the row
still reads SURVIVED and still counts. Suppressing a row on the strength of a
comment is exactly how a declared `UNCOVERED` stood for a week while being
false; the marker's only job is to stop the next reader re-deriving a proof
already written beside the code.

**And the marker's first implementation was a broken ruler.** It took
marker + 2 lines. The proof it was written for is a fifteen-line comment block,
so it named a line in the middle of its own explanation — and a one-line proof
would have passed the test. It now names the first line after the marker that is
neither comment nor blank, with a case for the multi-line block specifically.
Third time this campaign that a measuring device was calibrated against the one
example in front of it.

State: 13 gates, 3 with boundary survivors, 8 remaining — two of them proven
equivalences that will never go away and now say so.

## 40. A verification that checked the adjacent thing

A gate written yesterday for a question nobody was asking went red on real data
today, and what it found was eleven blog posts absent from a live site for two
days.

A deploy had replaced the site's application bundle with one built from a
different repository. Eleven posts existed **only** in the old bundle — they had
never been migrated into the shared source — so the swap removed them. They had
no static pages either, so nothing served them at all.

That deploy was verified before pushing. Its commit message says so:

> Verified before pushing: deck/, .claude/, blog/ (29) and ru/ (11) untouched

**Every word of that is true, and it is the wrong question.** It checked the
static trees. The loss was entirely inside the bundle. This is not carelessness —
it is a verification aimed one step to the left of the thing being changed, which
is much harder to notice than no verification at all, because the commit reads as
checked.

**The reusable rule: name what your change REPLACES, and verify that.** The
deploy replaced `assets/`; the check covered `blog/` and `ru/`. Nothing in the
sentence "blog/ untouched" is false and nothing in it is relevant.

**And a gate is not tested until it is red on data you did not plant.** The new
gate had four planted control cases, all passing, before it had ever seen a real
deploy. Replaying it over 160 commits of history found three drops — two healed
by the next deploy, one not. That replay is now a mode of the tool (`--history`)
rather than a script I ran once.

**Which needed its own two directions.** On its first real run `--history`
printed "still missing today: 0" for all three drops — correct, because the loss
had just been repaired, and a mode whose only observed output is green has not
been shown to go red. It now has three planted cases in a real git repository:
an unhealed drop reds, a healed drop stays green and says why, a clean history
does not invent one. The middle one is load-bearing: a drop a later deploy healed
is history, not damage, and a gate that reddens forever over a fixed incident
gets run once.

## 41. Sweeping for the shape, and what a guard cannot ask

§40 named a shape — a verification aimed one step to the left of what the change
replaces. A shape is worth sweeping for, so every deploy path in reach got the
same two questions: **what does it REPLACE, and what does it verify?**

Four paths, one hit.

**The hit was the worst one available.** `publish-website.yml` runs on a
15-minute cron, unattended, and replaces the site's `assets/` with a build from
another repository. It called the blog regenerator and the drift checker and
nothing that asks whether the incoming build still carries the posts the site is
serving — the exact gap that cost eleven posts two days of downtime, sitting in
the one path that fires ninety-six times a day without a human present.

**And the drift checker structurally cannot ask it.** It compares the shipped
bundle against the static tree and fails on disagreement. That is the right
check. It is also blind to a slug leaving **both sides at once**, which is what
happened: the posts lived only in the bundle, the bundle was replaced, and
nothing disagreed with anything. *A checker of agreement between two artefacts
says nothing about preservation across time.* Those are different questions and
one instrument cannot hold both.

**The clean paths were clean for a reason worth copying.** The CNAME guard reads
the CNAME file; the Pages guard reads the Pages API *and* curls production. Each
verifies the artefact it names, and the second one crosses an independent
channel to do it.

**Then the sweep turned on the instrument.** The new gate counted slugs by
PRESENCE on disk. `rsync` is additive, so yesterday's chunk sits beside today's:
a slug dropped from the reachable chunk is still on disk in the unreachable one,
so a disk-wide read calls it live while the site 404s it — and the orphan prune
that follows a deploy then deletes it for good. **That is the 2026-08-21
mechanism exactly, and the instrument built to catch it shared its blind spot.**
Today the apex carries 16 chunks, one reachable, and the reachable one happens
to hold all 48 slugs, so the old reading gave the right answer **by luck**.

Reachability from `index.html`, not presence on disk. Eighth control case,
verified RED under the old reading. The planted trees now carry a real
`index.html` and entry chunk so the cases run the walk rather than the no-index
fallback — planting only the chunk would test the fallback and call it the gate.

Verified in CI, not only locally: the cron fired on the commit that added the
gate and printed `no post is lost — live serves 48, this build carries 48`. And
on real data the other way: the pre-restoration build exits 1 and names all
eleven.

## 42. The mutation line closed, and what the boundaries had in common

All twelve boundary survivors are gone. The suite now stands at **111 mutants
across 13 gates — 36 silent, 21 loud, 33 invert, 21 boundary — with two
survivors, both proven equivalences that say so in the row.**

Six closed in one pass, and they rhymed. **Every one needed an input no case had
a reason to build**, and in each the missing input was the degenerate one:

- a file with **zero** cases (the prose census counted `> 0`; nothing planted an
  empty file, so `>= 0` was free)
- a **new** file with zero cases (announced as prose-only under `>=`, on a tree
  with nothing wrong)
- a file that was **already empty when the ledger was written** (an emptying is
  a *transition*; under `>=` a file that never made one is announced as EMPTIED
  on every run forever)
- a ledger line with **no separator at all** (`parts[1] if len(parts) > 1`; under
  `>=` the index raises and the gate dies on a healthy tree)
- **exactly five** repaired seals (the `"(+N more)"` continuation boundary)
- **exactly `MIN_ROWS`** in the catalog floor, closed earlier

Nothing here is exotic. A control author plants the fault they are testing for,
which is by construction a *non-degenerate* example: a file with the wrong
contents, not a file with no contents. The boundary operator's whole yield is
the empty case, the equal case, and the one-off-the-edge case — **the inputs
that are nobody's example.**

**One planted tree can close two boundaries when they are the same question on
either side of a ledger.** The prose count appears in `--update-baseline`'s
header and again in the verify path's census line; a single fixture carrying a
zero-case file at record time pins both.

**And "cosmetic" was closed rather than declared, twice.** The `"(+N more)"`
guards change a message and nothing else. Both are shut with a case, because
this campaign has twice found a written-down limitation to be invented, and a
declared exception costs every future reader more than a case costs to write
once.

## 43. Testing a published prediction, and what the attempt found instead

The twelfth post claims that controls miss *degenerate* inputs — the empty case,
the equal case, the one off the edge — because a control author plants examples
**of** a fault, and degenerate inputs are examples of nothing. It says outright
that this predicts a hole in other suites and does not measure one.

So: measure one. `tri gates mutate` learned `--dir`, and the campaign's own
question was pointed outside its repository for the first time.

**The prediction could not be tested, and the reason is a finding.** The second
repository has 54 Python tools and none of them declares a `--self-check` flag.

**RETRACTED — see §45.** That sentence is literally true and its meaning is
false. A control does not have to be a flag inside the script: this very file
already knows that (`EXTERNAL_CONTROL` names five gates whose control lives in a
different file), and I searched for the one mechanism I had built rather than for
the thing itself. Measured properly, the denominator is 6 and not 54, and three
of those six are well controlled — one of them by a test whose docstring names
this exact failure.

The only other controlled gate in reach is the one I wrote yesterday, *from
these lessons*. Its boundary column reads `0/0` — the file contains no
comparison at all, so the prediction is untestable there too, honestly and for a
boring reason.

**The run found something else.** That gate — nine control cases, written with
more discipline than anything else this week — had a surviving silent mutant:
`main()`'s **no-argument** branch. Every case passes a build path, so nothing
ran it the way a broken *caller* would.

It matters because of where the gate is wired. The unattended publisher calls it
inside an `if !`, so a lost argument reaching `return 0` reads as a passing check
and the cron publishes having compared nothing. **The vacuous pass, inside the
guard written against the loss it exists to prevent.**

And beside it: the usage line was `__doc__.splitlines()[-4]`, which printed the
`--history` line rather than the usage line. A message whose content is an index
into the prose above it is wrong the moment the prose is edited — and it already
was.

**Two rulers widened to make the run possible, both worth keeping.** The file
filter matched `check_` and not `check-`, so aimed elsewhere it found nothing
and printed an empty table, which reads exactly like a clean suite. And `code()`
hard-coded `tools/`, so with `--dir` it would have run a *different file of the
same name* or none. Both are the shape this campaign keeps meeting: an
instrument that works because everything it has ever seen was arranged one way.

**The rule.** A gate you wrote from the lessons is still a gate. Point the
auditor at it, especially then — and when a prediction cannot be tested, say
which precondition failed rather than letting the attempt read as a confirmation.

## 44. Feeding the degenerate inputs to my own new flag

§42 said the boundary operator's yield is the empty case, the equal case, and
the one off the edge — the inputs nobody plants. §43 said a tool you wrote from
the lessons is still a tool. This is what happened when the two were applied to
`--dir`, one iteration after shipping it.

Four degenerate inputs. **Two of them exited 0.**

| input | before | after |
|---|---|---|
| a directory that does not exist | refused, names the flag | unchanged |
| a **file** instead of a directory | died with `git status failed` | refused as *not a directory* |
| an **empty** directory | header, no rows, **exit 0** | refused: nothing measured is not nothing wrong |
| a directory **outside any git work tree** | ran, **exit 0** | refused |

**The empty directory is the vacuous pass, in the command whose subject is
vacuous passes.** A table with a header and no rows reads exactly like a clean
suite. I had widened the file filter the day before *because* a wrong filter
produced this table — and never made the empty result say so, so the hazard
survived the fix that was aimed at it.

**The one outside a work tree has teeth.** This command rewrites each gate file
in place and restores it afterwards, and that restore is only a promise because
`git checkout` can undo an interrupted run. Outside a repository there is no
undo — and the dirty-tree guard that exists for exactly this passed silently,
because `git status` *fails* there and its empty stdout reads as clean. **A guard
whose failure mode is indistinguishable from its success.**

**And then I wrote the bug into its own fix.** The first version of the
empty-directory refusal printed `FAIL: no gate scripts under …` and returned
`Ok(())` — announcing that nothing had been measured, and exiting 0. Measured,
not noticed by reading: `exit 0` before, `exit 1` after. A message is not a
verdict, and this is the third time this campaign that the two got confused.

**The rule.** After adding an option, spend one iteration giving it the inputs
you had no reason to try: nothing, empty, wrong type, wrong place. The author of
an option builds the case that motivated it — which is, by construction, not any
of those.

## 45. The retraction: a control is not a flag

§43 reported "54 Python tools and not one declared negative control" in a second
repository. Every word measured. **The sentence is false**, and the way it is
false is the one this campaign has now met six times: a true number whose meaning
does not survive contact with the thing it describes.

**The denominator was wrong.** 54 is every Python file under `tools/` and
`scripts/`. Of those, **7** are mentioned in any workflow, and **6** both run in
CI and carry a path to a non-zero exit. A script nobody invokes is not a gate
without a control; it is not a gate.

**The numerator was wrong too, and worse.** Of those six, three have real
negative controls:

- `check_build_paths.py` — clean and broken fixtures, and the workflow asserts
  *exactly one dangling path and one LIKELY*, not merely that it went red
- `conformance_check.py` — good/wrong RTL fixtures with `expect_mismatches: 0`
  and `expect_mismatches: 6`, a planted defect caught **with the right count**
- `signal_health.py` — a value-asserting test whose docstring says *"Values, not
  verdicts. The structural check's flop counter returned zero for every design
  on earth and stayed green for weeks because the only thing its self-test
  asserted was pass-or-fail."*

That last one is this campaign's own lesson, written down in the repository I
had just declared control-less, before I got there.

The remaining three — `signal_health_report.py`, `check_status_report.py`,
`fetch_run_report.py` — are data-refresh scripts that commit a JSON file. Their
non-zero exit fails a workflow, which is why they matched a mechanical search
for "carries a verdict", but nobody would call them gates.

**Why I got it wrong.** I searched for `--self-check` / `--selftest`, then
widened to any self-check-shaped flag, and concluded absence. But a control can
be a workflow job with fixtures, or a test file — and **this file already knew
that**: `EXTERNAL_CONTROL` exists precisely because five gates here have their
control in a different file. I looked for the mechanism I had built rather than
for the property I cared about.

**The check.** Before reporting that something has no control, enumerate the
*forms* a control can take in that repository — flag, sibling script, workflow
job, fixture pair, test file — and search for each. An absence proved by one
mechanism is a statement about the mechanism.

And the timing is the part worth keeping: this was published one iteration after
a post about right numbers with wrong meanings, in the sentence that post's own
verification produced.

## 46. Putting §45 into the tool, and two more assumptions it was hiding

§45 said: before reporting that something has no control, enumerate the *forms*
a control takes and search for each. A rule only I know is not a rule, so it
went into `tri gates sweep`.

**It now searches four forms** — a flag in the script, the `EXTERNAL_CONTROL`
table, `tests/test_<name>.py`, and a workflow that names the gate *beside*
planted-fault vocabulary (`fixture`, `expect_`, `planted`, `broken`, `must`).
The verdict column gained `OTHER`, distinct from `NONE`: **"I cannot run it" and
"it does not exist" are different findings and were conflated once already.**

Workflow evidence is labelled a **candidate**, never proof. A heuristic that
upgrades "no control" to "controlled" is the one error direction that hurts:
an uncontrolled gate reading as controlled is the false green this command
exists to find. A test asserts the other direction too — a workflow that merely
*runs* a gate is not evidence of controlling it.

**And the output now prints what it searched.** Both the forms and the file
filter, on every run, found or not. That is the actual repair for §45's mistake:
a reader cannot weigh a `NONE` without seeing the search behind it.

Pointing it at the second repository immediately produced two more assumptions
the old output was hiding.

**The file filter is narrower than the control search.** `conformance_check.py`
and `signal_health.py` are gates there and match neither `check_*` nor `gate`.
The command never saw them, and a table that does not list them reads as a
repository that does not have them. So the header now says
`Files considered: 3 of 22` — and in this repository, `13 of 28`. **Fifteen
Python files under `tools/` here are invisible to a command whose output looks
exhaustive.**

**And the vacuous pass survived its own repair, in the sibling.** `mutate` grew
a refusal for an empty gate set one iteration ago. `sweep` did not, so aimed at
a directory with no gates it printed `0 gate(s); 0 with no control` and exited
0 — a sentence in which every number is zero and which reads as a clean sweep.
Fixing a class in one command and not in the one beside it is how the class
survives.

## 47. Selecting gates by property, and a heuristic that lied on first use

§46 disclosed that the file filter was narrower than the control search. Measuring
the hidden files turned that disclosure into three findings, two of them about
this command.

**A correction first: the disclosure line was itself off by two.** It printed
`13 of 28` where 15 files match by name — `rows.len()` is the count *after* the
two control files are excluded. The line added for honesty understated the match
by exactly the number of controls, and I repeated its number in a report. Now it
says `17 gate(s) from 28 *.py … control files excluded`.

**Naming was never the property.** It failed as `check-` vs `check_`, and again
as `verify_*` / `run_*`. The property that matters is measurable: **a workflow
invokes it and it can exit non-zero.** Anything that can turn a pipeline red is a
gate whether or not its name says so. Selection is now by-name **or**
by-property, and the count moved 13 → 17 here.

The three that surfaced — `fuzz_trainer.py`, `run_conformance_vvp.py`,
`verify_multitarget.py` — run in CI, carry verdicts, and have **no control in any
form**. Invisible for the whole campaign to a command whose output looks
exhaustive.

**And the workflow heuristic lied on its first real use.** §46 added it with the
warning that upgrading "no control" to "controlled" is the one error direction
that hurts. It then did exactly that, for all three, on the strength of the word
`must` sitting in a prose comment **760 lines** from the call.

Tightened two ways: only vocabulary somebody chooses on purpose — `fixture`,
`expect_`, `planted`, dropping `must` and `broken`, which are ordinary English in
a comment — and **within 30 lines of an invocation**. Both directions tested: a
fixture path beside the call is found, prose 60 lines away is not.

**The cross-check worth keeping.** Pointed at the second repository, the tightened
version independently reproduced the by-hand measurement from §45: the same three
gates controlled, by the same evidence. A heuristic agreeing with a careful manual
pass on a corpus it was not tuned against is the closest thing to a control this
kind of search can have.

## 48. `sys.exit` is a verdict, and two columns of zeros said otherwise

Giving `verify_multitarget.py` its first negative control turned into a finding
about the operators, and then into a survivor inside a gate that had been
reported clean for the whole campaign.

**The control first.** This gate's skip pair is its most rottable branch: without
`--require` a missing prerequisite is a SKIP and exit 0, with it the same state
is a FAILURE. Three lines apart, same message, and only the exit code differs —
so each case names the other's marker as forbidden. `SKIP` reaching exit 1 and
`FAIL` reaching exit 0 are both silent successes.

**The plant was wrong on the first attempt, and said so.** I ran the gate from a
temp working directory, reasoning that `target/debug/t27c` would be missing
there. It resolves against `ROOT`, not cwd, so the gate found the real binary and
ran the whole check successfully — and the control reported CONTROL FAILED rather
than passing. A control that fails on a bad plant is doing its job; the fix is
the copy-into-an-empty-tree pattern every other gate here uses.

**Then two columns of zeros.** `silent 0/0, loud 0/0` for a file that is nothing
but verdicts. The operators understood `return N` and `raise SystemExit(N)` and
**not `sys.exit(N)`** — the spelling half this repository uses. Two empty columns
read as *nothing here to break*, which is the same sentence a clean gate prints.

**And the fix immediately found a survivor in a gate scored 3/3 for weeks.**
`check_catalog_count.py` has a `sys.exit(2)` for *the codegen subprocess itself
failing*, and every case in its control plants SSOT **content** — so the codegen
always ran and always succeeded, and that branch was unreachable from its own
control. Closed with a planted codegen that dies, distinguished from the other
exit-2 branch by message since both leave the same code.

**The rule.** An operator that recognises one spelling of a verdict is a
coverage measurement of that spelling. When a column reads `0/0`, ask whether
the gate has no such site or the scanner has no such pattern — those print
identically and mean opposite things.

Left open and named: `verify_multitarget.py`'s cross-target MISMATCH verdict
(three invert survivors at the comparison, and the final `sys.exit(0 if ok else
1)`). Reaching it needs a fake compiler emitting deliberately wrong C and Rust.
The tool's survivors match that written declaration exactly, which is the first
time this campaign a declared gap and a measured one agreed line for line.

## 49. A control with resolution and no wiring, and the product defect behind it

`verify_exhaustive.py` scored **0/2, 0/1, 0/9, 0/3** — a control that exists,
passes, and kills nothing. Worse than no control: the gate read as covered.

**Its control was good at the wrong thing.** It perturbs one input and proves the
digest changes — the comparison has *resolution*. Excellent, and it never leaves
the function. Every verdict lives in `main()`, which the control never ran. The
campaign's oldest defect, in the gate that measures bit-exactness across four
backends.

**Reaching main() found a product defect.** With no C compiler on PATH the gate
raised `FileNotFoundError` — exit 1 and not one word of verdict. A traceback is
not a verdict: CI sees red, the reader learns nothing, and "the tool is missing"
is indistinguishable from "the arithmetic is wrong."

**Guarding the crash exposed a worse one.** Once the absence was caught, the gate
announced `FAIL: 1 of 1 targets DISAGREED` — about arithmetic it had never
performed. `check()` returned `False` (ran and disagreed) where it meant `UNRUN`
(could not run).

And main()'s own comment, twenty lines below, says:

> check() already distinguishes them -- None is "could not run", False is "ran
> and disagreed" -- and the tally threw that away.

**That comment was true of the Verilog arm and false of the C/Rust arm, in the
same function.** A distinction documented as implemented, implemented once, and
believed twice. A third path returned `None`, which is in neither tally — so it
exited 1 having printed nothing at all.

**Three end-to-end cases now, one per reachable verdict**: an empty selection
fails, a missing compiler is UNRUN and never DISAGREED, and a clean target exits
0 *saying what it proved*. The last is the mirror the other two need — both demand
exit 1, so a gate rewritten to fail unconditionally satisfied every case in the
file, and `--loud` showed exactly that.

Result: **2/2, 1/1, 7/9, 0/3**. Named and left: the disagreement verdicts need a
backend that genuinely differs, which is a fixture worth building and not built
here.

**The rule.** A control that never leaves the function it tests measures the
function. Ask of every control: *which process exit does this case observe?* If
the answer is none, it is a property test wearing a gate's clothes — and the two
are told apart by exactly one thing, whether a mutant of the verdict survives it.

## 50. Sweeping for the shape: how many "missing tool" crashes are there?

§49 found a gate that raised `FileNotFoundError` when a compiler was absent —
exit 1 with no verdict, so *"the tool is missing"* and *"the arithmetic is wrong"*
left the same colour and the same silence. That is a shape, so it got swept for.

**Mechanically: 17 invocations of an external tool by bare name across the gate
directory, 11 with no `try/except OSError` around them.** And 11 is a raw number
whose meaning is not yet established — the same trap as *54 tools*.

**Five of the eleven are guarded by a `shutil.which` precondition**, which is a
better design than catching the exception: the gate says the tool is missing
before it does any work. Counting those as defects would have been the
false-accusation direction.

**Six have neither.** Three are `git` — nearly universal, low value, and named
rather than fixed. The other three matter: one `cc`, and an `iverilog`/`vvp` pair
inside a gate that also has **no control at all**, so two findings converge on
one file.

**Measured rather than assumed, and the measurement disagreed with the count.**
Running each with the tool stripped from PATH: `check_duplicate_agreement.py`
crashes as predicted; `check_elab_ratchet.py` reports the absence correctly (its
`which` precondition does dominate the call, which the static read could not
show); and the third exits 2 on usage before reaching the tool, so the probe
never tested what it meant to. **Eleven candidates, one confirmed crash.**

Fixed with the same shape as §49: catch, name the absence, and return the value
the caller already treats as *uncompared* rather than as *disagreed*. The gate
now reaches its own "the extraction is broken, not the tree" verdict — the right
class, which is the whole point.

**The control case reuses the AGREEING fixture**, so the only thing wrong with
that world is the missing tool, and `DIFFERENT behaviours` is named absent:
reporting an absence as a disagreement is exactly what the branch exists to
prevent.

**The rule.** A static count of unguarded calls is a list of candidates, not
findings. Run each one in the world it describes — a `which` above the call, an
argument check before it, or a wrapper you did not read all make the same static
pattern harmless, and only running tells you which.

## 51. The last file where both findings met

`run_conformance_vvp.py` carried two open findings at once: **no negative control
in any form**, and two unguarded external calls. It was invisible to the sweep
for the whole campaign because it is named `run_*` rather than `check_*` — the
naming proxy again, in the file where it cost the most.

**Four verdicts covered end to end**, one per branch reachable without a
simulator: no arguments is usage and not a pass; a module outside the registry
is refused; unbuildable RTL is a build failure; and a **missing simulator is
named**, which before this commit was a `FileNotFoundError` — red, with nothing
said about whether the RTL was wrong or the tool was absent.

Three of its exits are `2` and two are `1`, so the code alone cannot say which
branch spoke. Every case asserts the message and names its siblings as forbidden.

**The build-failure case needs the simulator PRESENT**, and says so: without that
guard it would silently become a second copy of the missing-simulator case, and
two cases measuring one branch read as two branches covered. When the tool is
absent the case reports **UNRUN and fails the control** rather than skipping —
a case that could not run proved nothing, and this file's own subject is a gate
that reported a vacuous pass.

**And the surviving mutant matched the written declaration for the third
consecutive time.** One survivor, at the `NOTHING WAS EXECUTED` branch — named
in the docstring as needing a planted corpus. Three gates in a row where what I
wrote down as uncovered and what the tool measured as uncovered were the same
line. That agreement is worth more than either alone: the declaration is checked
by something that cannot read it.

Gates with no control in any form: **1** — down from 3 when property-based
selection first exposed them, and from 4 of 12 when the campaign began.

## 52. Closing the last one revealed the count was wrong

`fuzz_trainer.py` was the last gate with no control. Closing it produced three
findings, and the third makes the first two look small.

**Two defects in two lines of a shared helper.** `skip()` hard-coded its own
name, so `fuzz_trainer.py` announced *"SKIP verify_trainer_c"* while running
something else — anyone reading a CI log to see which check declined got the
wrong answer. And it had no `--require` at all, while its sibling
`verify_multitarget.py` has one with a comment explaining why. **Two of the three
trainer checks in one workflow job could silently pass on a missing compiler
while the third refused** — same runner, same environment, opposite rules.

**A crash at import.** `ROUNDS = int(sys.argv[1])` read position, not meaning, so
`fuzz_trainer.py --require` — the spelling this commit adds to CI — died with a
`ValueError` before `main()` and before any verdict.

**A case that passed for the wrong reason.** The planted divergence used a bare
`str.replace` on the whole file and hit `run_model`'s return, three functions
before `run_c`. The counterexample case failed loudly; the **length** case
passed, because a shortened model is also a length mismatch. Satisfied by a
divergence planted somewhere it was never meant to be. Scoping the edit to the
text after `def run_c(` is what makes the case measure the arm it names.

### And then the count moved 18 → 21

Closing the last gate made the tool report a *new* uncovered one, and chasing
that produced a claim of my own: *"verify_trainer_c.py has no non-zero exit at
all — a CI step named 'Prove the WHOLE trainer bit-exact' that cannot fail."*

**That claim is false.** Its last line is `sys.exit(0 if ok else 1)`. My grep
looked for `sys.exit(` followed by a digit and could not see a ternary — and
**`is_gate_by_property` had the identical blind spot**, so it classified three
more CI gates as not-gates.

The campaign wrote `verdict_literals()` for exactly this, months ago, because the
mutation scanner was blind to ternaries. The selector reintroduced the blindness
as a substring shortcut, and I reintroduced it a third time in a one-off grep
while investigating.

Fixed by routing the property check through `verdict_literals` — the function
that already knew. Real state: **21 gates, 4 with no control in any form**, not
18 and 1. `gft_backprop_microcode.py`, `verify_emit_bitexact.py`,
`verify_igla_race.py` and `verify_trainer_c.py` were invisible for the whole
campaign because their only failure path is a ternary.

**The rule.** When you write a quick grep to check a property the codebase
already has a parser for, you are choosing the version with the known bug. Ask
what the existing checker would say, and if you cannot run it, at least give the
shortcut the same cases the real one has.

## 53. Three copies of one helper, one rule applied once

Waiting for a CI job before merging turned out to be the productive part of the
iteration, twice over.

**The wait was right.** #2518 tightened two CI steps with `--require`, and the
job that runs them takes twenty minutes. Merging on "the other checks are green"
would have been the exact error this campaign keeps correcting: the checks that
were green were not the ones the change affects. It completed **success**, and
only then did it merge.

**And the wait surfaced a stale red.** That workflow last ran on the default
branch on 2026-08-20 and **failed** — at a step my branch passes. Three days of
red that is not a live finding, just a run nobody has repeated. `run` on a branch
and `run` on the default branch answer different questions, and the older one
answers about a tree that no longer exists.

**The parallel work found the third copy.** `skip()` exists **four** times in
the trainer/verifier family — the count in this paragraph said three, and was
itself off by one until §54 recounted it. `verify_multitarget.py` has had `--require` from the
start with a comment explaining why. `verify_trainer_c.py` did not, until
yesterday. `verify_igla_race.py` did not either. **One rule, written down once,
applied in one of three places** — and the two without it are CI steps that could
exit 0 having compared nothing.

That is not a bug in any of the three. It is what happens when a rule lives in a
comment inside one file: the next author copies the code and not the reasoning.

**The fourth consecutive agreement.** `verify_igla_race.py` now scores 1/2, 1/1,
8/9 — and its single silent survivor is the final `sys.exit(0 if ok else 1)`,
which the new control's docstring names as uncovered. Four gates in a row where
the declared gap and the measured gap are the same line.

**The rule.** When you fix a defect in a helper, grep for the helper's other
copies before writing it up. A shared idea with three implementations has three
chances to be right and usually takes one.

## 54. The gate that exited 0 when the compiler refused to emit

Counting the copies of `skip()` corrected §53 (**four**, not three) and found the
one that mattered.

`verify_emit_bitexact.py` — the gate whose entire job is *"prove the generated
RTL equals the model bit-exactly"* — called `skip()` when `t27c gen-verilog`
returned **non-zero**. A code-generation failure in the exact thing being
verified made this check **exit 0**, in CI and locally, with or without any flag.

That is not a missing prerequisite. iverilog absent is an incomplete
environment; the compiler refusing to emit is the product being broken, and it
is the loudest thing this gate could possibly find. It was the quietest.

**Two different words for two different states.** `skip()` keeps its meaning —
the environment is incomplete, tolerated locally, fatal under `--require`. A new
`broken()` says the product failed, and is fatal always, with a line that names
which of the two happened so a reader never has to guess.

**The fourth copy also lacked `--require`**, so it was the last of the four that
could silently pass on a missing simulator. All four now agree.

**The control's third case is the one that would have caught it**: a planted
`t27c` that refuses to emit, asserting exit 1 and naming `SKIP` as forbidden —
because reporting a broken compiler as a missing tool is exactly the defect.

**The rule.** Every `skip` is a claim that the thing missing is *not the subject
of the test*. Read them as that sentence and the wrong ones become obvious: "we
skipped because the compiler under test would not compile" does not survive
being said out loud.

## 55. Reading every `skip` as the sentence it makes

§54 gave a mechanical test: **every `skip` claims that the thing missing is not
the subject of this check.** Fourteen calls in the tree; read each as that
sentence and two do not survive.

Both are a **spec file tracked in git**. `verify_emit_bitexact` skipped when
`gft_smul.t27` / `gft_sadd.t27` were absent; `verify_igla_race` when
`ternary_mac.t27` was. Measured, not argued: renaming the file aside makes both
exit **0** without a flag. A deleted spec is not a bare machine — it is the
repository missing the thing the gate exists to verify, and the sentence
"the missing spec is not the subject of this check" is simply false.

Both now call `broken()`: fatal with or without `--require`. `--require` should
not be what saves you from a deleted source file.

### And the extraction, at the one safe moment

Four hand-copied `skip()`s had just been made to behave identically, which is
the only time a deduplication is a pure deduplication. `tools/_prereq.py` now
holds both words, with the rule that separates them written **where the code
is** rather than in a comment inside one of four copies — which is exactly how
the four came to disagree.

**The extraction broke a control on its first run, and the control said so.**
`verify_multitarget`'s planted tree carried the script and not the module it now
imports, so the child died at import with empty stdout — and the case refused to
read that as a skip. A plant must carry everything the thing under test needs;
adding a shared dependency changes what "everything" means, and the only cheap
way to learn that is a control that fails loudly.

Two assertions also had to move: the messages now name the script from `argv`
rather than a constant, so `SKIP verify_multitarget` became
`SKIP verify_multitarget.py`. A control asserting exact text is a control that
notices a message changing — which is the point, even when the change is mine.

## 56. A null result, and a selector that was right by accident

Two measurements this iteration, and the honest one is the boring one.

**The assert blind spot costs nothing here.** `is_gate_by_property` recognises
`return`, `sys.exit` and `raise SystemExit`; a tool that fails only through an
uncaught `AssertionError` is invisible to it. I expected that to be costing
coverage. Measured across every CI-invoked tool: **zero** fail only that way —
every one has an explicit or ternary exit as well. The hole is real and empty,
and saying so is worth more than the fix I was about to write.

**But the selector has a false positive, and it fired.**
`gft_backprop_microcode.py` was classified as a gate on the strength of

    return 0 if ys[0] >= ys[1] else 1

which is a **class label from a classifier**, not a verdict. `verdict_literals`
was built to recognise return statements that carry verdicts; using it to answer
*"can this file exit non-zero"* conflates a returned value with an exit code.
The file is a gate anyway — sixteen `assert`s — so the classification is right
**by accident**, which is the least useful way to be right.

**The last of the four verifiers now has a control.** `verify_trainer_c.py` —
the one whose `skip()` every other copied — is 1/1, 1/1, 3/3, with a planted
divergence in its C arm and the clean direction asserted beside it. The plant
scopes its edit to the text after `def run_c(`, because the same plant in
`fuzz_trainer.py` hit `run_model` three functions earlier and a case passed on a
divergence planted where it was never meant to be.

Gates with no control in any form: **1**, and it is the file the selector was
right about by accident.

**The rule.** A predicate reused outside the question it was written for will be
right often enough to look correct. `verdict_literals` answers *"is this return a
verdict?"*; it does not answer *"can this program fail?"*, and the difference
only shows up on a classifier that returns 1 for a class.

## 57. Zero, and a null result that answered the easier question

**21 gates, 0 with no control in any form.** The count that opened this campaign
at 4 of 12 is at zero, across a set two-thirds larger and selected by property
rather than by name.

The last one, `gft_backprop_microcode.py`, carried sixteen `assert`s — XOR trains
to 4/4, held-out clears 90%, the emitted Verilog carries the ports it claims —
and nothing showed that any of them could go red. Now three planted cases do: a
sign flip in the shared multiplier stops XOR converging, a renamed port trips the
emitter assertion, and the clean tree stays green.

**And building it corrected §56's null result.** That entry measured *"do any
CI-invoked tools fail ONLY through assert?"* — answer zero, no gate is invisible
to the selector — and I reported it as if it settled the matter. It does not.

The mutation operators score this gate **0/0 silent, 0/0 loud, 0/0 invert**.
Its verdicts are asserts, which no operator recognises, so **every verdict in
this gate is invisible to three of the four questions**. The selector sees it
(via a ternary that returns a class label, §56's accident); the operators do not
see a single one of its sixteen assertions.

Two different questions — *"is this file classified as a gate?"* and *"can its
verdicts be broken and noticed?"* — and I answered the easier one, found nothing,
and moved on.

**And the documented trap, reproduced.** The port-rename plant spelled its needle
literally, so the first occurrence of that string in the file became **the
control's own source line**, and `str.replace(.., 1)` edited the harness instead
of the target. The case then reported the gate as blind when nothing had been
planted at all. `check_duplicate_agreement.py` carries a comment warning about
exactly this, written after it happened there. I had read that comment. The fix
is to assemble the needle — `"input [31:0] x" + "0i"` — so it does not exist as a
literal anywhere.

**Named and left:** the boundary column read 5/31 *(erratum: that denominator
came from the scanner bug §121 fixed -- the file has 62 boundary sites, not 31.
The killed count is unaffected; every number of the form `k/31` in this file
should be read as `k/62`.)* Those sites are arithmetic
internals — encodings, magnitude comparisons — where moving a comparison is a
numerical change rather than a verdict change. A different kind of surface,
larger than anything else outstanding, and not this campaign's question.

## 58. The fifth operator, and the control scope that leaked

§57 named a measurable hole: a gate whose verdicts are `assert`s scores 0/0 in
every column, which prints exactly like a gate with nothing to break. `--assert`
closes it — `assert C, "msg"` becomes `assert True, "msg"`, the silent operator
spelled the way a test-shaped gate spells it. The message is kept deliberately:
a mutant that also dropped the text would be killed by a control asserting that
text, and the kill would be for the wrong reason.

**Its first run found one site in a file with eighteen assertions.** Not a
scoping choice — a bug, and one all three scanners shared.

`in_control` was set by a top-level `def` and cleared only by the **next**
top-level `def`. So everything after the last function in a file inherits that
function's status, and when the last function is a `self_check`, the whole
`if __name__ == "__main__":` block below it is scored as control code. Sixteen
assertions live in exactly that block. The operator reported **0/1**.

A function ends at the next top-level *statement*, not at the next `def`. Three
scanners fixed; the silent, loud and invert operators had the same leak and never
showed it, because module-level verdicts are rare and asserts are where they
live.

**And then the honest number.** 16 sites, **2 killed**. The control I wrote for
that gate one iteration ago — three planted cases, all passing — covers **two of
its sixteen verdicts**. "Has a control" became a measurement, which is the entire
point of the operator.

**A cost, stated.** The full five-operator run now exceeds ten minutes; the
assert column alone spawns sixteen ten-second runs for one gate. The suite has
outgrown a single foreground command, and that is a real consequence of the
fifth question rather than a reason not to ask it.

### And a rule I wrote, then broke, in the same session

§53 said: wait for the job that runs what you changed, not for the checks beside
it. Two iterations later I read `in_progress`, and merged anyway. The branch run
completed **success** afterwards — so the outcome was fine and the method was
not, and the difference between those is the whole subject of this document.

Mitigating and worth saying precisely: that change added a `--self-check` branch
and touched no CI invocation, so the job could not have been affected by it.
That is an argument I could have made *before* merging, and did not — I simply
did not look.

## 59. A mutant escaped into a commit, and `git add -A` is how

The five-operator run exceeds ten minutes, so I backgrounded it. A timeout killed
an earlier one. The loop writes a mutant, runs the control, restores — and a kill
lands between the first and the third.

**A boundary mutant stayed in `gft_backprop_microcode.py`. `git add -A` staged
it. It went into a commit, a push, and an open pull request** — a deliberately
broken line, in the file whose control I had just written, in a repository whose
whole subject is gates that cannot fail.

The command's docstring already said the restore is recoverable with
`git checkout tools/`. True, and useless: **you have to know an interrupt
happened.** The dirty-tree guard could not help — it refuses to *start* dirty,
and by then the mutant was already staged.

**Two failures, and the second is the one that shipped it.** Staging everything
and trusting that nothing else moved. During a mutation run the tree is
*transiently* dirty by design, so `git add -A` in that window commits whatever
the loop is holding at that instant.

**Fixed both ways.** A marker under `target/` (already ignored, so it can never
be the dirt it warns about) is written before each gate and removed on success;
a later run refuses to start and prints the recovery commands. And the habit:
during any mutation work, stage named files, never `-A`.

**The demonstration failed on its first attempt, correctly.** With the background
run still holding a file mutated, the dirty-tree guard fired before the marker
check — the older guard doing its job, and proof that the two cover different
moments rather than the same one.

## 60. The second mutant, and why file-by-file recovery missed it

§59 caught one escaped mutant. **There were two.**

`check_specs_generate.py` carried `return 1` -> `return 0` — a silent mutant, in
a commit, in the open pull request, for two iterations. It survived the cleanup
because I recovered **the file named in the PR diff** instead of checking the
directory.

The command's own recovery instruction is `git checkout tools/` — the whole
directory. I quoted it in §59 while doing something narrower, and then asserted
the tree was clean on the strength of one file matching.

**What found it.** Not vigilance: the background run was still going, and two
files showed dirty at once. One mutated file is the loop working; two is either a
bug or residue. Chasing which produced the answer — and the honest note is that
without that anomaly I would not have looked, because the PR diff had stopped
mentioning it.

**The recovery that works is a directory comparison**, both directions:

    git diff origin/master HEAD --stat -- tools/     # nothing committed
    git status --porcelain -- tools/                 # nothing pending

Two empty outputs, not one file inspected.

**And the marker now proves both directions.** Present, it refuses and prints the
recovery commands, naming the gate the interrupted run was on; absent, the run
proceeds and clears it on success. The command it prints is
`git checkout -- tools/` — the directory, which is exactly the instruction I had
and did not follow.

**The rule.** After any interrupted tool that edits files in place, compare the
whole directory it edits against its baseline, in both the committed and the
working direction. A diff that names one file is a report about that file, not
about the tree.

## 61. A run nobody can finish is not a measurement

Five operators over 21 gates passed twenty minutes and kept growing —
`gft_backprop_microcode.py` alone has 47 sites, each a ten-second subprocess. The
last two attempts were killed by timeouts, and one of those kills is what leaked
two mutants into a branch. **The cost had stopped being an inconvenience and
started being a correctness problem**: the full picture was the entire point of
`--all`, and nobody could reach it.

**Cached by what the answer depends on**: the gate's bytes and the bytes of
whatever control judges it. Both hashed, both must match, and the cache is
written **after every gate** rather than at the end — so an interrupted run keeps
what it measured and the next one resumes. Cold 2.6s → warm 0.5s on one gate.

**Every reused row says `[cached]`.** A cached green that read like a fresh one
would be precisely the lie this command exists to find, and the summary names the
split: *N measured, M reused*.

**I put that marker into one of two print paths, then two of three.** The
multi-column branch got it first; the single-operator branch printed cached rows
identically to fresh ones; and the zero-site branch — *"no failure path to
break"* — printed a third way with no marker at all. Three printers, one
property, and it took two corrections to reach all three. The same shape as
`skip()` in four copies, inside one function.

**Invalidation is verified, not assumed.** On a planted repository: measure,
reuse, then append one comment to the gate and watch the third run measure again.
A cache that never invalidates is worse than no cache, and that direction is the
one worth testing.

**The stale case, stated rather than hidden.** A fixture changing underneath a
gate and its control leaves both hashes intact and the recorded row wrong. That
is why the marker exists instead of silence: a reader who sees `[cached]` knows
which question to ask, and `--fresh` answers it.

## 62. The marker fired on real data, and a control that kills nothing

**The interrupt marker caught a real one.** A run from the previous iteration had
been orphaned and left `gft_backprop_microcode.py` mutated. The marker named the
gate, printed the recovery commands, and I ran exactly what it printed —
`git checkout -- tools/` on the directory, which is the instruction §60 says I
should have followed the first time.

**And it spoke second, which is the wrong order.** The dirty-tree guard fires
first, and after an interrupt the tree *is* dirty — so the informative message
existed and was never the one shown. Marker now checked first. Found by hitting a
real interrupt and reading the wrong error.

**An orphaned run was still alive.** Thirteen minutes in, mutating files, from an
iteration I had already reported as finished. Third time mutants got loose, and
every time the cause was mine: starting a long background job and losing track of
it. The guard did refuse the concurrent run I started on top of it — an unplanned
safety property, since two mutation loops would each restore the other's
mutations and produce numbers that describe nothing.

### Seventeen of twenty-one rows, and one is stark

    verify_emit_bitexact.py   0/1   0/1   0/11   0/4   0/0

**Its control kills nothing.** Not one of seventeen mutants, across four
operators. I gave that gate a control three iterations ago — three planted cases,
all passing, covering the skip pair and the codegen-failure branch — and **none
of the sites the operators can reach are among them.**

That is the same class as `verify_exhaustive.py` before §49: a control that
exists, passes, and is not connected to the verdicts anyone would break. It is
worse here, because I wrote this one *knowing* that class, and the number that
exposes it could not be produced until the run learned to be interruptible.

**The table is 17 of 21 and says so.** The remaining four are measuring; the
cache means the next run finishes them rather than starting over. A partial table
reported as partial is a measurement; the same table reported as complete is the
thing this campaign is about.

## 63. Why that control killed nothing, and what fixing it cost

`verify_emit_bitexact.py` scored **0 killed of 17**. The reason, once the survivor
lines were read rather than counted:

- `sys.exit(0 if ok else 1)` survived **both** return operators
- every FAIL branch of the comparison — timeout, step count, mismatch, resource
  count, synth error — survived inversion

**All three of my cases leave through `skip()` or `broken()`.** The gate's own
verdict was never observed at all. A control that covers only preconditions is a
control *for* preconditions, and I wrote those three knowing that class.

**The plant that reaches the verdict moves one arm only.** The Python side comes
from the interpreter `g.run()`; the Verilog is emitted from the microcode
`steps`, not from `run()`. Perturbing the interpreter makes the model disagree
with an RTL that is unchanged — which is what a real bit-exactness failure looks
like. Perturbing shared arithmetic instead would move both arms together and
plant nothing.

Two cases now: a clean tree exits 0 saying `RTL == model BIT-EXACT`, and a
perturbed model exits 1 naming the disagreeing step. Five cases in the file, both
directions of the verdict.

### And the cost is real, and worth stating plainly

The mutation loop runs a gate's **whole control per mutant**. This control now
spawns two ~45-second whole-program runs, so seventeen sites cost roughly half an
hour — the measurement timed out at ten minutes and the marker caught the
interrupted tree, cleanly, with nothing leaked.

**That is not a reason to make the control cheaper.** A control that exercises
only what is fast to exercise is how this gate got to 0/17 in the first place.
The tension is inherent: *a control worth having is expensive, and the mutation
loop pays that cost once per mutant.* The cache is the answer — measure once,
reuse until the gate or its control changes — and `[cached]` is what keeps a
reused row honest.

**The guard chain worked end to end for the first time.** Timeout → marker names
the gate → `git checkout -- tools/` → tree clean → committed diff contains only
the intended change. Three iterations ago the same sequence leaked two mutants
into a branch.

## 64. Two gates, one mistake, made twice by the same author

`verify_multitarget.py` scored **0 killed of 7** — and the survivor lines are
identical in shape to `verify_emit_bitexact.py`'s: the gate's own
`sys.exit(0 if ok else 1)` under both return operators, and all three comparison
FAIL branches under inversion.

Both controls cover the skip pair. **Both leave through `skip()` and never reach
`main()`.** Two gates, one mistake, and I made it twice — the second time three
iterations after writing down the first.

That is what a *class* looks like when you have not internalised it: the rule was
recorded, the next control was written the same way, and only a measurement
caught it.

**Both plants move one arm.** In `verify_multitarget`, `py_ref` reads the Python
model while C and Rust come from `t27c`; perturbing `py_ref` makes the model
disagree with backends that are unchanged. Perturbing the spec or the emitter
would move every arm together and plant nothing — the same distinction that made
the `verify_emit_bitexact` plant work, and the reason both are one-line edits in
a specific place rather than "break something".

    verify_emit_bitexact.py   0/17  ->  13/17
    verify_multitarget.py      0/7  ->   5/7

Only boundary survivors remain in both.

### And the honesty mechanism needed its own correction

Re-measuring produced a row with **two columns measured and three reused**,
labelled `[cached]` wholesale. Under-claiming rather than over-claiming — the
safe direction, and still wrong, because the entire point of the marker is that
a reader can tell which they are looking at.

Three states now: no marker when fresh, `[cached]` when every column is reused,
`[3 cached, 2 fresh]` when mixed. The mixed case had to be reached by marking
cache entries stale by hand, which is worth noting: **a state that cannot occur
naturally during testing is a state nobody has seen your code produce.**

## 65. Triaging fifty boundary survivors, and a pull request that got 7 of 35 checks

The boundary column carried **50 survivors** — the largest and last unexamined
number in the table. Read rather than counted, they sort into four kinds, and
only one is a verdict:

| kind | n | example |
|---|---|---|
| **proven equivalence** | 6 | `sig = … if r.returncode < 0 else ""`, reached only after a returncode check |
| **fixture generation** | 8 | `cls = int((xs[0] > 0) != (xs[-1] > 0))`, `if p < 0.15:`, `while len(v) < N:` |
| **display truncation** | 4 | `if len(out) > 6:` guarding a "… N more" line |
| **possibly real thresholds** | 3 | `if total < 200:`, `if not full and space > budget:`, `if b < 0:` |

Plus 26 in one file's arithmetic internals, a separate surface.

**The operator has no scope discipline, and this is where that shows.** `invert`
restricts itself to conditions whose body carries a verdict. `boundary` takes
every comparison — and in *verifier*-style gates most comparisons are in test-data
generation and reporting, not in verdicts. On checker-style gates the same
operator found six real thresholds. **The same operator is sharp on one shape of
gate and noisy on another**, which is a fact about the operator worth knowing
before quoting its count.

**Five of the six equivalences are one line, copied five times.** The `signal`
message appears in five verifiers, each reached only after a returncode check —
three via `if returncode == 0: return`, two via `if returncode != 0:` — so the
value cannot be zero there and `< 0` ≡ `<= 0`. Proven from the line above, not
assumed from the shape. Fifth duplication family this campaign has found; all
five are now marked, and the rows still read SURVIVED and still count.

### And a pull request that silently received 7 checks of 35

PR #2541 changed `tools/verify_multitarget.py` — a path explicitly listed in
`emit-bitexact-gate.yml`'s `pull_request` paths — and **that gate did not run**.
Seven checks total, where a sibling opened minutes earlier got thirty-five.

Measured rather than assumed:

- an empty commit did **not** re-trigger it — so not a transient miss
- a second branch touching the **same files** got a normal check list — so not
  the paths
- the repository had no queue backlog — so not contention

**Not diagnosed.** The cause is inside GitHub's trigger evaluation for that
pull request, and nothing I can read from here explains it. What matters is the
response: **re-opened as a fresh pull request rather than merged without checks.**
The new one has 25 checks and `emit-bitexact` running.

**The rule.** A green check list is evidence; a *short* check list is a finding.
Count the checks against a sibling before reading the colours — a gate that never
ran is invisible in exactly the way a gate that passed is.

## 66. A conflicting pull request loses its path-filtered CI — and my first rule for it was wrong

§65 left one pull request's missing CI undiagnosed. Checking every open pull
request found the mechanism, and then the mechanism corrected itself.

**Three more pull requests have the same shape.** Two of them change
`bootstrap/src/compiler.rs` and got **three checks** — `check-linked-issue`,
GitGuardian, NotebookLM. Every one path-less.

That file's gate carries this comment, written to prevent exactly this:

> Without `bootstrap/**` here, a PR that rewrites the C emitter merges with the
> cross-target proof never running.

The path was added. **The gate is defeated by a merge conflict instead.**

**The mechanism.** A pull request that is CONFLICTING when an event fires cannot
have its merge diff computed, so `paths:` filters cannot be evaluated and only
path-less workflows run. The checks that remain are green — because they are the
ones that never look at the diff — which reads exactly like a passing pull
request.

### And the rule I first wrote was wrong

The correlation looked exact: four conflicting pull requests with 3, 3, 9 and 7
checks; everything else 21 to 35. I wrote *"a CONFLICTING pull request loses most
of its checks"* into the tool's own documentation.

**An hour later two of those four reported 21 and 26.** They had been mergeable
when their events fired, kept those results, and only conflicted afterwards.
**A conflict does not retract past runs.**

So the detectable shape is not the state. It is *conflicting **and** a check list
far below its siblings'*, and `tri gates prs` now computes a reference from the
non-conflicting pull requests rather than asserting from the state alone. Three
flagged, two correctly excluded.

**Caught by re-running the command on fresher data**, not by thinking harder —
which is the argument for putting a finding into a tool rather than a document.
A sentence cannot disagree with tomorrow's data; a command can.

**And it crashed on its first run**, slicing a title by byte index in the middle
of an em dash — in a pull request this campaign had opened. Truncate by chars.

## 67. The blocking check was right, and its ledger held 58 paid debts

`coverage` — the required check that blocked a pull request — is
`check_seal_coverage.py`, one of this campaign's own gates. **It is not a broken
instrument.** Its negative control passes, and the finding is real: **131 seals
are stale**, meaning the spec changed after sealing so the recorded hashes
describe something it no longer produces.

**What was mine to do, and what was not.** Re-sealing 131 specs is blessing the
drift the ratchet exists to prevent, and fixing them is 131 separate judgements.
Filed as an owner decision.

**But the gate also asked for something free.** It prints, every run:

> NOTE 56 baselined seal(s) now hold. Drop their lines so the gate holds them.
> DEPARTED … baselined as broken, and the seal FILE is gone.

Fifty-six debts paid and never collected, plus two lines naming files that no
longer exist. **209 → 151 lines**, and the 131 untouched. That *tightens* the
ratchet: fifty-six seals that were excused are now held.

**Removed by hand, line by line — not with `--update-baseline`.** That command
rewrites the whole ledger from today's state, which would bless all 131 in the
same stroke. **Tightening a ratchet and blessing drift use the same file and must
not use the same command**, and the only thing separating them here is which one
you reach for.

**And the state worth naming.** `coverage` is required, so this blocks merges —
and every recent merge went in with it red anyway. A required check that is
always red costs the friction of a gate without the protection of one, which is
the same condition this campaign opened on, arrived at from the opposite
direction: not a gate that cannot fail, but a gate that cannot pass.

## 68. The second blocking check was also right, and also unread

`Corpus ratchet`, the other permanently-red required check, had been printing a
precise verdict for its whole history and nobody had acted on it:

    UNEXPECTED PASSES  : 3
    UNEXPECTED FAILURES: 2

**The same two specs appear in both lists.** They used to fail `parse` outright;
they now parse and fail the narrower `parse-no-discard`. The third simply passes.

So the fix is three removals and two re-labels, and the distinction matters:
**the excuse for those two specs went from "does not parse at all" to "parses
but discards tokens", and the third lost its excuse entirely.** Ledger 221 → 220
against a cap of 221 — strictly tighter, and `RATCHET: CLEAN`.

**Both permanently-red required checks turned out to be right.** Neither was a
broken instrument; both were reporting real, small, actionable findings that had
gone unread long enough to become scenery. `coverage` asked for 58 lines to be
dropped; `Corpus ratchet` asked for 3 removals and 2 re-labels. Between them:
**five minutes of work each, blocking every merge in the repository.**

That is the failure mode neither §1 nor §57 covers. A gate that cannot fail reads
as coverage. A gate that cannot pass reads as noise. **A gate that is right, and
whose verdict is a short actionable list, becomes furniture if nobody reads it —
and the longer it stays red the more certain everyone is that it means nothing.**

**The check that separates the two cases takes one command.** Run the gate
locally and read what it says. Both of these named their own remedy in the
output, on every run, for days.

## 69. A detector keyed on a value that is recomputed while you look at it

`tri gates prs` flagged three pull requests one iteration ago. Run again today, it
flagged **one** — and nothing about the other two had changed.

`mergeable` is computed on demand. Between two runs, two pull requests moved from
CONFLICTING to UNKNOWN and back, and the detector — which tested
`m == "CONFLICTING"` — lost them and found them again. **The alarm was
intermittent for a condition that was not.**

**The observable is the short check list.** Three checks against a median of
twenty-one is the finding, whatever GitHub currently believes about
mergeability. The state is the *explanation*, and it belongs in the row rather
than in the test.

The reference had the same defect one layer down: computed as the median of the
*non-conflicting* rows, it read 21 on one run and 35 on the next, because two
rows crossed the filter in between. **A median over every row is unmoved by a
few short lists and does not depend on a value that changes while you watch.**
Two consecutive runs now agree.

**And `UNKNOWN` is not `fine`.** It means GitHub has not finished computing.
A short list with UNKNOWN beside it is the same finding as one with CONFLICTING
beside it, seen a moment earlier — which the output now says.

**The rule.** Before keying an alarm on a field, ask whether the field is
*measured* or *computed on demand*. A derived, cached, or lazily-evaluated value
makes a detector that reports the weather rather than the climate — and the
first symptom is a finding that comes and goes without anything changing.

## 70. A finding recorded as a line number expires on the next edit

Three boundary survivors were carried forward from §65's triage as "possibly real
thresholds", identified by file and line. Reading those lines today gave three
completely unrelated statements — a `subprocess.run` argument, a `None` guard,
and a random-vector append.

**The files had been edited in between.** Five equivalence markers went in, two
controls grew whole-program cases, and every line below each insertion moved. The
triage note was measurably false about its own repository within a day of being
written.

This campaign already knows the fix in one place: `check_gate_preconditions.py`
names its uncovered branch **by message**, with a comment explaining that an
earlier version said `:346` and `:390` and was wrong before it was ever pushed.
The same discipline did not reach my own notes.

**The rule.** A survivor list is a snapshot of one run against one tree. Carry
forward the *file and the expression*, or re-run and re-read — never the line
number alone. And when a triage spans iterations that edit the files, the
re-run is the cheap half.

**And I caught myself with a truncated view first.** Reading the fresh table
through `awk` that printed six columns, I concluded the cache was not working —
the marker lives in the seventh. The conclusion was drawn from a ruler I had cut
short myself, which is the third time this campaign that a display choice became
a finding.

## 71. The cache failed silently, three ways, in six lines

A full re-measurement re-measured gates whose hashes **matched entries already in
the cache**, and the entry count climbed 30 → 40 → 80 *during* that run. It was
rebuilding a cache it should have loaded.

Reading the six lines that load and save it found three silent failure paths:

    read_to_string(..).ok()          unreadable file  -> empty map
    serde_json::from_str(..).ok()    corrupt file     -> empty map
    let _ = fs::write(..)            failed write     -> next run starts empty

**Each degrades to "no data", which is indistinguishable from "nothing measured
yet".** A corrupt cache looked exactly like a first run — the same shape as a
gate that cannot fail, one layer down in the tooling that measures gates.

**The cause was mine, repeatedly.** `fs::write` truncates in place, and I killed
`--all` runs on a ten-minute timeout three times. Each kill landed some chance
between the truncate and the write and left half a JSON document, which the next
run swallowed.

Fixed both halves. **Write-then-rename** — atomic on one filesystem, so the file
is either the old complete document or the new one, never half of either. And
every path now says what happened: a missing file is a first run and silent, an
unreadable or unparseable one prints what it found and why it is re-measuring.

Verified on a planted repository in three states: fresh measures, a repeat says
`[cached]`, and a **deliberately truncated** cache prints the warning and
re-measures. The truncation was done by cutting the file to 40 bytes, which is
what a killed run does by accident.

**The rule.** `.ok()` on a read and `let _ =` on a write are the two commonest
ways a tool loses data without saying so. Neither is wrong where the absence is
expected — a missing cache file genuinely is a first run — but the *present and
broken* case has to be told apart from the *absent* one, and only the code that
opens the file can tell them apart.


## 72. A verification recipe is scoped to the repository that taught it

I keep a note that says: reproduce CI locally by exit code, `cargo fmt --all
--check` then `cargo clippy --all-targets -D warnings`. It is a good note. It
was learned in a different repository.

Run in this one, `cargo fmt --all` rewrote **150 files**, including
`bootstrap/src/compiler.rs` — which is frozen, with `bootstrap/stage0/FROZEN_HASH`
holding its sha256. The freeze gate went red. I broke a real gate with a check
that this repository does not run: `grep -c "cargo fmt" .github/workflows/*.yml`
returns nothing, and `cargo fmt -p tri -- --check` was already exit 1 on clean
master, untouched, for however long.

So the check I ran was not a gate here, and the gate it broke was.

Before running a "standard" check, ask the repository which checks it actually
has. The workflows are the answer, and they are two greps away:

```
grep -rhoE "cargo (fmt|clippy|test|build)[^|&;]*" .github/workflows/*.yml | sort -u
```

The tell that a check is not a gate: **it is already failing on untouched
master.** A check nobody runs drifts red and nobody notices — the same
condition as a gate that cannot fail (§3), one layer out. Two consequences,
and they point opposite ways:

* Do not "fix" that drift as a side effect of unrelated work. The diff buries
  your actual change, and here it also broke the freeze.
* Do not trust it as verification either. It says nothing about your change.

Recovery, when the formatter has already run: `git checkout -- .`, then restore
only your file from a copy made **before** the formatter. Keep that copy — I
had `/tmp/gates_fixed.rs` from the negative-control step by luck, not by plan,
and it is the only reason the recovery was one command. Then re-verify the
freeze explicitly rather than assuming the checkout covered it:

```
shasum -a 256 bootstrap/src/compiler.rs | cut -c1-16
cat bootstrap/stage0/FROZEN_HASH | tr -d '[:space:]' | cut -c1-16
```

## 73. A cache key that maps two different inputs to one key

`.ok()` and `let _ =` lose data loudly enough once you look for them (§71).
The quieter relative is a key function that **collides**. Both directions are
possible and only one hurts:

* same input, different key — the cache misses. Wasteful, self-announcing.
* **different input, same key — the cache serves a row measured against
  something else.** Silent, and it corrupts the measurement.

Four lines of `sha_of` had two collisions of the second kind:

```rust
for p in paths {
    h.update(std::fs::read(p).unwrap_or_default());  // gone == empty
}                                                     // and "ab"+"c" == "a"+"bc"
```

The first is §71 wearing a different hat: an unreadable file hashes as the
empty string, so *missing* and *empty* share a key, and any two unreadable
paths share a key with each other. The second is structural — concatenating
without separators lets the boundary move. It needs a list to be reachable,
and the control list is a list, so any gate declaring more than one control
could hit it.

Both close the same way: **one length-prefixed record per element**, covering
the path, the read outcome, and the bytes. Length prefixes pin the boundaries;
a status byte keeps an absence from impersonating an emptiness.

Test it by asserting the collisions are gone, then **plant the old function and
watch the test go red**. Mine printed `ba7816bf8f01cfea` on both sides — the
sha256 of `"abc"`, arriving from two different inputs. Without that step the
test only proves the new code is self-consistent.

## 74. The section that was overwritten by the section after it

This file is append-only by intent. It still lost a section.

`## 63` was written in `d3005bda6` and gone by `456c6b08f` — a commit about a
different gate entirely, whose diff on this file reads `38 insertions, 42
deletions`. Not an append. Section 64 landed *on top of* 63 instead of after
it, and the numbering hid it: sections still ran 62, 64, 65, and nothing about
that looks wrong at a glance. It survived several sessions.

I found it by accident, checking the numbering after an unrelated append.

**A gap in a numbered sequence is a question, and the history answers it.**
The two possible answers look identical in the file and completely different
in the log:

```
git log --all --oneline -S'## 63. ' -- path/to/file.md
```

* no commits — the number was never used. Nothing is missing.
* two commits — one that added it, one that removed it. Something is missing,
  and the second commit tells you which one to blame. The parent of the
  removing commit still has the text: `git show <sha>^:path | ...`
* **one commit, on a branch that is not merged** — the section exists and is
  waiting in a pull request.

**`--all` is not optional, and leaving it out is how I got this wrong.** The
first version of this section said the gap at `## 24` was a number never used,
citing an empty `git log -S` as proof. I ran it twice, on master, and read
"nothing here" as "nothing anywhere". Section 24 was sitting in PR #2487,
authored days earlier and unmerged.

`git log -S` searches the history of the **current branch**. A section that
exists only on an unmerged branch is byte-for-byte indistinguishable from one
that never existed — same empty output, opposite meaning. The scope of the
search is part of the claim, and I omitted it from the claim while omitting it
from the command.

Widen it further when the answer still looks empty: `git log --reflog`,
`git fsck --lost-found` for a section dropped by an abandoned rebase, and — for
a repository with more than one worktree, which this one has — the branches
those worktrees hold.

Recovery cost nothing because git had it. The cost was the sessions in
between, where the experience was simply absent and I could have re-learned it
the expensive way.

Cheap standing check, worth running whenever this file is touched:

```
python3 -c "
import re,pathlib
n=[int(m) for m in re.findall(r'^## (\d+)\.', pathlib.Path('SKILL.md').read_text(), re.M)]
print('ascending:', n==sorted(n), '| gaps:', [i for i in range(1,max(n)+1) if i not in n])"
```

Every gap it prints needs an answer from the log, once, recorded here — not a
shrug. A file whose whole purpose is to carry experience forward is exactly
the file where a silent deletion costs the most.

## 75. The gate told me how to fix it, and the command did not exist

`now-sync-gate-diff.sh` fails a PR that adds no `docs/now/` entry, and prints
the cure:

```
    ./scripts/tri now add "<title>" --bullet "<what changed>" --closes <N>
```

Running it printed `error: unrecognized subcommand 'now'`.

The command was not missing. `tri now` is implemented, tested, and documented
— in `cli/tri`, a Rust binary. `./scripts/tri` is a **different** program, a
bash front door that dispatches its own helpers and forwards everything else
to `t27c`, a **third** binary. Three things named `tri`-ish, and the
documented path went to the one that didn't have it. Seventeen subcommands
were in this state; `now` is simply the one a gate names out loud.

This is §3 one layer out. A gate that cannot fail is useless; a gate that
fails correctly and hands you a cure that doesn't run **costs more than
silence**, because the failure looks handled. I hand-wrote `docs/now/` entries
twice and got them wrong twice — bulletless, rejected — while a command that
makes a bulletless entry impossible sat one broken route away.

Two checks, both cheap:

* **Run the remediation text.** Any gate that prints a command should have
  that command executed once, from a clean checkout, by someone who did not
  write it. Copy-paste is the test.
* **Ask what is reachable, not what exists.** For a multi-binary front door,
  the reachable set is what the dispatcher routes, not what any binary
  implements:

```
comm -23 <(BIN --help | sed -n 's/^  \([a-z][a-z0-9-]*\).*/\1/p' | sort -u) \
         <(FRONT_DOOR_TARGET --help | sed -n 's/^  \([a-z][a-z0-9-]*\).*/\1/p' | sort -u)
```

Everything it prints exists and cannot be run by name.

When fixing the routing, reroute **only** what the fallback target does not
implement, and prove the untouched routes byte-for-byte:

```
for c in $SHARED; do front $c --help > after-$c; done
git stash -- scripts/tri; for c in $SHARED; do front $c --help > before-$c; done; git stash pop
for c in $SHARED; do cmp -s before-$c after-$c || echo "MOVED: $c"; done
```

Eight shared names here, eight byte-identical. "I don't think that changed" is
not a result; `cmp` is.

## 76. The build flag decided which bug you get, and CI picked the quiet one

Running the corpus ratchet locally panicked: `attempt to subtract with
overflow`, in `range_decl(width: u32)`, which formats `[width - 1 : 0]` and
guards only `width == 1`. On master. On a gate that is green in CI.

Both halves of that are true, and the reason is a build flag:

| build | overflow-checks | outcome |
|---|---|---|
| debug | on | panic, loud, stops everything |
| release | **off** | `function [4294967295:0] cover_point;` — **exit 0, stderr empty** |

CI builds `--release`. So the gate has been green over Verilog no synthesiser
can accept, and **the loud version of the bug is the one CI structurally
cannot see.** Ten specs, exit 0 on every one.

The general shape, which is not specific to Rust: **a defect can have two
faces, and the build you verify under decides which face you meet.** Debug
turns silent corruption into a crash. Release turns a crash into silent
corruption. Verifying only under one of them is half a measurement, and the
half CI runs is usually the quiet one, because release is what ships.

Three consequences worth carrying:

* **Run the corpus under both profiles at least once.** In Rust, one flag does
  it without a full rebuild of a different profile — one variable, changed
  alone: `CARGO_PROFILE_DEV_OVERFLOW_CHECKS=false cargo build`.
* **A checker for this must treat both faces as the same finding.** Mine scans
  emitted text for absurd widths *and* treats a compiler that panicked as a
  hit. Otherwise it reports CLEAN against a debug compiler — which emitted
  nothing to scan, because it died. **Absence of the string is not absence of
  the defect.**
* **Do not key the detector on the value you happened to see.** I searched for
  `4294967295` by hand and found eight specs. A check keyed on the *property*
  — no real bus is a million bits wide — found **ten**, and the two extra were
  a different mechanism each: `18446744073709551615` (the same underflow on a
  u64 path) and `4198431` (a `Map` flattened, apparently not an underflow at
  all). One literal, one mechanism; one property, all three.

## 77. What to do when the fix is behind a seal -- and how to check it is

`compiler.rs` is sealed by `stage0/FROZEN_HASH`. I read `FROZEN.md` -- "until
maintainers **intentionally** re-run the freeze ceremony (M5)" -- and withheld
a four-line fix, filing the design question instead and writing this section to
justify it.

Then I looked at what the repository actually does:

```
git log origin/master --since=30.days --oneline -- bootstrap/src/compiler.rs      # 184
```

**184 commits in 30 days, 178 of them moving `FROZEN_HASH` in the same
commit.** Six a day. Re-sealing is not a ceremony here; it is the ordinary
companion of any compiler change, and `build.rs` refuses to build until you do
it. The seal is a **drift detector** -- it catches an edit that forgot to
re-seal -- not an approval gate. I turned a document's register into a
prohibition the practice does not contain, and the cost was a real bug left in
place for a day.

**Before treating a document's caution as a rule, count how often the
repository breaks it.** Frequency in the log beats tone in the prose. A
sentence saying "maintainers intentionally" alongside 178 automated precedents
describes a workflow, not a gate.

The distinction that survives, and it is narrower than what I wrote:

* **Changing the sealed file and re-sealing in the same commit** -- routine.
  178 precedents. `build.rs` enforces the pairing, which is the whole point.
* **A change whose CORRECTNESS is a policy question** -- still not yours.
  Here: *should a type with no bit representation be lowerable to Verilog at
  all?* Refusing outright would fail ten specs; that is a ratchet decision with
  a cost, and costs are the owner's.
* **Repository settings** -- branch protection, required checks, rulesets --
  genuinely not yours, because no commit can express them and no gate catches
  the change.

What made the fix safe to make was not permission. It was that the file said
what to do. `field_type_width` opens with "0 is a POISON value, not a width"
and prescribes the repair -- the lowerability predicate must refuse any struct
it cannot size -- twenty lines above its own `return 0` for a string field. The
rule was written, the violation was in the same function, and nobody had
connected them.

**When a fix is behind a seal, deliver in this order:** the measurement first
(it is useful even if you stop there), then the check that keeps it from
getting worse, then -- if the log says the seal moves routinely and the file
itself tells you what correct looks like -- the fix, with the blast radius
measured rather than argued.

**And measure the fix you are shipping, not the one you meant to ship.** My
first version followed the file's own prescription and made a zero-width struct
non-lowerable. It was the principled repair and it cost **17 new elaboration
errors** in one module, because the non-lowerable path declares per-field
registers at some sites and not at function locals. The narrow version --
clamp the width, change nothing about lowerability -- costs none. Both numbers
came from `check_elab_ratchet` on the same iverilog: 176 before, 193 for the
principled fix, 176 for the narrow one, and 650 specs regenerated showing
exactly the 8 intended files differ.

The prescription in the comment was right about the *diagnosis* and wrong
about the *dose*. A repair that follows a file's stated intent is still a
change with a blast radius, and the intent was written before the fallback
path it routes into was known to be incomplete.

## 78. The one conflict you must not resolve by choosing a side

`bootstrap/stage0/FROZEN_HASH` holds `sha256(bootstrap/src/compiler.rs)`. When
two branches both change the compiler, git conflicts the seal and offers two
hashes. **Both are wrong.** Each describes its own side's file; the merged file
is a third thing. Measured on a real conflict:

```
ours   8e62cacb81c6e84d
theirs 4f003654a44a4348
merged 6e2bad56817414a6
```

Every reflex a conflict trains — take ours, take theirs, read both and pick the
newer — produces a seal for a file that does not exist. The right move is to
ignore both candidates and **recompute from the merged bytes**.

`build.rs` verifies the seal on every build, so a picked side fails loudly
rather than shipping. The cost is confusion, not corruption. But the class is
worth naming, because it generalises: **a derived value in conflict has no
correct side.** Lockfiles, checksums, generated indices, line counts in a
header — anything whose content is a function of other content in the same
merge. Resolve those by re-deriving, never by selecting.

The tell is one question: *is this file's content computed from something else
in the repository?* If yes, both sides are stale the moment the merge exists.

`tri reseal write` does it, and `tri reseal check` reports the three states
separately — matching, mismatched, and conflicted — because a conflicted seal
trimmed into a comparison reports a plain mismatch and sends the reader to fix
the wrong thing. The first line of a conflicted file is `<<<<<<< HEAD`, which
compares against a hash exactly as unhelpfully as any other wrong string.

## 79. A ratchet that fails because you fixed something

The corpus ratchet went red on a branch that **repairs** seventeen specs. Not a
regression: the ledger still listed them as broken, and a ledger that overstates
the damage is itself a defect (§—the same rule the widths ledger states in its
own header).

What the run actually said, once read rather than counted:

* **17 unexpected passes** — specs that now parse. Remove them.
* **5 unexpected failures** — and every one of the five was *in that list of
  seventeen*, now failing `parse-no-discard` instead of `parse`.

That second group looks like new damage in the summary line and is not. The
same specs moved **one stage deeper**: they used to fail before the parser
finished; now they parse and the next check catches what the parser discarded.
Twelve cleared entirely, five relocated.

**Read a ratchet's two lists against each other before treating either as a
verdict.** A path in both is a spec that advanced, and recording it as a fresh
failure — or refusing to add it, and reverting the fix — both get the sign
wrong. Net here: 220 entries to 208, and the five carry a reason saying which
stage they came from, so the next reader is not told they are new.

## 80. The tool truncated its own list, and I read the printout as the set

The corpus ratchet said **27 unexpected passes**, then printed a list. I
removed every name in the list, re-ran, and it still failed — with two more
names, from the same 27. The summary count and the printed list disagreed, and
I had trusted the list.

This is the third time this shape has cost me a wrong conclusion. Once it was
an `awk` view six columns wide hiding a marker in column seven; once a `head`
on a diff; now a tool that caps its own output. **The count and the listing are
two different things, and only one of them is complete.**

Two habits, both cheap:

* **Compare the summary number to the number of lines you extracted.** If the
  tool says 27 and your regex found 25, the gap is the finding — not a regex
  bug to shrug at. Print both.
* **Loop until the verdict is clean, do not act once and assume.** A single
  pass over a truncated list leaves a tail, every time:

```
for round in 1..N:
    run the gate
    if exit 0: done
    extract the removable names
    if none extractable: stop and read it by hand   # not an infinite loop
    if any NEW failure appeared: stop               # never auto-bless damage
    apply, and go round again
```

The two guards are what make the loop safe to leave running. Without the first
it spins on a failure it cannot parse; without the second it will happily
ratchet a real regression into a ledger while you sleep. Mine converged in two
rounds and printed each one.

The same caution applies to reading a gate's output through `head`, `tail`,
`grep -m`, or a truncating pipe of your own: you chose the truncation, so the
missing part is on you and not on the tool.

## 81. A flag's value became the thing the tool measured

`tri damage --json /tmp/out.json` scanned `/tmp/out.json` as the corpus. The
argument parser was one line:

```python
args = [a for a in argv if not a.startswith("--")]
corpus = args[0] if args else "specs"
```

It strips the **flag** and leaves the flag's **value**, so every valued flag
donates its argument to the positional list. `--out`, `--json`,
`--emit-fixtures`, `--class` — five flags across two tools, each of which
silently redirected the corpus to a path with no specs in it.

Alone that is a bug. What made it invisible is what it produced:

```
damaged lines: 0 in 0 files, 0 distinct shapes
```

**The same line the tool prints over a clean 650-spec corpus.** The "0 files"
counts files *with* damage, so a scan of nothing and a scan that found nothing
were byte-identical. Two defects, and each one hid the other: the parser sent
the tool somewhere empty, and the report could not say it had been nowhere.

**Print the denominator.** Any tool that reports "N found" must also report how
many things it looked at, and say so loudly when that is zero:

```
files scanned: 0
NOTHING WAS SCANNED. A zero below is the absence of a corpus,
not the absence of damage. Check the path.
```

This is §3 for reports rather than gates. A gate that cannot fail is useless; a
report that cannot distinguish *empty* from *clean* is worse, because it reads
as good news. The failure mode is not silence — it is **a reassuring number**.

The two-line test, on any tool that counts things: run it against a path that
does not exist, and against the real corpus. If the output is the same, the
tool cannot be trusted in either direction.

## 82. My own negative control was tautological

I salvaged six fixtures that say "this is not damage, report zero here", wired
a gate, and ran three controls on it. Two of the three found defects **in my
gate**:

* **Planted damage went undetected** — because I invented a damaged-looking
  shape instead of reading what the detector actually matches. `f : [u8,`
  contains nothing it looks for. The real signals are a doubled bracket `[[]`
  and an odd
  quote count; planting those, the gate failed as it should.
* **Removing a fixture still passed** — because I computed the expected count
  *from the directory being checked*. Delete a fixture and both sides fall to
  five and agree. That is §69 in a new costume: **an expectation derived from
  the thing under test cannot detect a change in it.** Pinned to a literal `6`,
  the loss fails.

Both mistakes have the same root: I wrote the control from what I imagined the
subject does, instead of from what it does. **Read the detector's source before
writing its negative control**, and pin every expectation to something outside
the subject.

The third control — remove the whole directory — worked first time, and that
is not reassuring. It is the easy one. The controls that pass immediately are
the ones least likely to be testing anything.

## 83. Sweeping for the empty-corpus zero, and what the sweep got wrong first

§81 gives a two-line test: run a counting tool against a path that does not
exist and against the real corpus, and compare. I ran it across the ten loop
tools. **The first sweep measured nothing**, and the way it failed is worth
more than the table it produced.

Five tools "did not distinguish" — identical output both ways. They also
returned **exit 2 on the real corpus**, which I printed in the same table and
did not read. They were rejecting the argument shape entirely: `corpus-parse`
wants `<binary> <dir> --out`, `diffbin` wants two binaries, `damage-repair`
demands `--snapshot` and says so at length. Identical output, because both runs
hit the same usage error. **A uniform invocation across tools with different
signatures compares error messages, not behaviour.**

The tell was in my own output. When a sweep reports "no difference" *and* an
error code on the input that should work, the sweep is broken, not the subject.

With valid invocations per tool, six take a corpus and the result was:

| tool | on a path that does not exist, before |
|---|---|
| `cost` | `no .t27 files matched`, exit 2 — correct already |
| `diffbin` | `no .t27 files under …`, exit 2 — correct already |
| `damage` | said `files scanned: 0` (from §81) but **exited 0** |
| `damage-freeze` | said nothing, exited 0, wrote a snapshot |
| `corpus-parse` | `total 0`, exited 0, wrote an artifact |
| `diffmodes` | printed `joined: 0` **and** "loss on the clean corpus: 0", exit 0 |

`diffmodes` is the sharpest and the most instructive. Every number it printed
was honest — `diffbin rows: 0`, `joined: 0`, `0 file(s)` — and it closes with a
careful paragraph about the limits of what it claims: *"a scoped claim, not a
clearance."* It hedges about **what** it asserts and not about **how much it
looked at**, and the exit code carried only the second. Green step, zeros on
the screen, scope of nothing.

All six now exit 2 on an empty corpus, with the reason in words. Three
consequences worth keeping:

* **The honest number and the honest verdict are separate things.** Printing
  `0 files` and returning 0 is still a green CI step, and the step is what gets
  read.
* **Put the denominator in the artifact, not only on the terminal.**
  `damage-freeze` now records `files_scanned` in the snapshot, so a reader
  months later can tell a clean corpus from an absent one without re-running
  anything. The terminal output is gone by then.
* **Write the file, change the verdict.** Refusing to write on empty would hide
  *which* path was empty from whoever reads the artifact afterwards. The
  artifact is evidence; the exit code is the claim.

And one on process: I introduced two guards that could never fire while writing
this — `if joined == 0` on a list, and an `empty` I referenced before defining.
Both were caught in seconds because I ran three states after each change
(empty, clean, damaged) instead of one. **A guard is code, and untested code in
a guard is worse than no guard: it looks like coverage.**

## 84. Ten gates had never been run in an empty tree, and one of them passed

`check_gate_preconditions.py` runs gates in a planted empty tree and asserts
each one refuses loudly. It ends with:

```
OK: 10 precondition(s) across 6 gates fail loudly; 0 known-uncovered
```

**`0 known-uncovered` counts rows this file chose to write, not gates.** Six of
the sixteen gate scripts in `tools/` are exercised; ten had never been run in
an empty tree at all, and the line reads like none were missing.

Run them:

| outcome in an empty tree | gates | reading |
|---|---|---|
| `skip()`, and `FAIL` under `--require` | 2 | by design |
| traceback, exit 1 | 5 | loud, though a crash is not a verdict |
| clear refusal, exit 2 or 3 | 2 | correct |
| **`OK: 0 tracked JSON files, none newly unparseable`, exit 0** | **1** | a pass over nothing |

`check_json_parses` printed its denominator — `0 tracked JSON files` — and
returned 0 anyway. §83's lesson again, now inside a gate rather than a report:
**the honest number and the honest verdict are separate things.** Zero tracked
JSON is not a clean repository; it is not this repository, and that is
`broken()` rather than `skip()` — the environment can be bare, but a repository
that tracks nothing it tracks is the repository being wrong.

The meta-gate now prints `coverage: 6 of 16` and names the ten. **Reported, not
enforced**: making an uncovered gate a failure turns a green file red today
over gates whose empty-tree behaviour is already loud, and a gate that is red
on the day it lands teaches people to ignore red. The number is there so the
gap is a fact rather than an implication.

**Whenever a file reports "0 missing", ask what the 0 is over.** A count of
rows, of files, of cases you happened to write — each is defensible, and none
of them is the count the reader assumes.

## 85. A self-check that plants the gate but not what the gate imports

Adding `from _prereq import broken` to a gate broke four of its own end-to-end
controls at once. They all reported the same way:

```
end-to-end empty file      CONTROL FAILED
     exit 1 (want 1)
     expected text absent: [...]
     stdout ''
```

`exit 1 (want 1)` — the code was right. `stdout ''` is the whole story: the
planted copy died on `ImportError` before printing anything, so every expected
string was "absent" and the failure read as a broken gate rather than a broken
plant.

The planting copied `__file__` into a temporary tree and nothing else. **A
plant that copies the subject but not its dependencies runs a different
program**, and the difference only shows the day a dependency is added — long
after the plant was written and trusted.

Two habits:

* **Copy what the subject imports**, and say so in the plant. One loop over a
  named list beats discovering the omission through four simultaneous
  failures.
* **Read `stdout ''` as a signal, not as detail.** A control that expected text
  and got *nothing at all* did not observe a wrong answer; it observed no
  answer. Those are different bugs, and the second one is usually in the
  harness.

## 86. Eleven controls would have gone silent, and none of them would have said why

§85 found one gate whose plant copied the script and not its imports. I checked
whether that was one file's oversight by injecting a single unused sibling
import into each self-planting gate and re-running its own self-check in the
real tree:

| gate | controls broken by one import |
|---|---|
| `check_specs_parse` | 5 |
| `check_catalog_integrity` | 4 |
| `check_gate_preconditions` | 2 |

**Eleven, and not one of them would have said "your plant is incomplete."**
They all report the same way — `exit 1 (want 1)`, then `stdout ''`, then every
expected string listed as absent. That reads as a gate that stopped working, on
a day when only the harness did.

The fix is one shared helper, `plant(script, dest)` in `_prereq.py`: copy the
script, then copy every sibling module it imports, transitively. It copies
`_prereq` itself, so the dependency it introduces is one it satisfies.

**Three things the sweep taught that the fix did not.**

* **A detector keyed on a literal finds the instances that share the literal.**
  I grepped `shutil.copy(__file__` and found four gates. The guard I then wrote
  keyed on the *destination* being a planted `tools/` directory and found
  **six** — two more used `shutil.copy(me, ...)` with `me` a variable. Same
  mistake as the widths sweep (§76), one iteration later.
* **A control that does not run is not a control that passed.** One gate came
  back "unaffected" by the injection. Its planted case had simply not executed
  in this environment — it is gated behind a tool that is absent — so the sweep
  recorded a clean where it had measured nothing. `grep` the output for the
  case's own label before believing a null result.
* **A shared helper can collide with a local name.** `check_seal_coverage`
  already had its own `plant(td, seals, ledger)` that builds a whole world.
  Importing the shared one under the same name shadowed it, and the planted
  world became the script's own path — `NotADirectoryError` on
  `<script>.py/.trinity/seals`. Imported under an alias. **Before adding an
  import, grep the file for the name you are about to bind.**

The guard now runs inside `check_gate_preconditions` and reports `BARE` with
the file and line. Negative control: reintroduce one bare copy, and it goes
red naming it.

## 87. A crash is loud, and it is still not a verdict

Five gates crashed rather than passed when run in an empty tree, which I filed
as survivable and came back for. `check_gate_preconditions` already has the
right word for it — **WRONG**: *"it goes red, but not through the branch that
explains why."*

Three raised `FileNotFoundError` on a file the repository tracks. That is
`broken()` by the §—`skip` vs `broken` rule: a missing **tool** is the
environment, a missing **tracked file** is the repository. One line each, and
the reader learns which file and that it is tracked, instead of a stack.

Two were self-tests of another gate. They build their own corpora, so passing
in an empty tree is correct — but with the gate under test absent they raised
`ModuleNotFoundError`. Same conversion: say that the subject is missing and
nothing was proved falsifiable.

**The measurement I trusted yesterday was wrong, by the defect I was fixing.**
Yesterday's sweep planted each gate alone and recorded five tracebacks. Today,
planting the whole `tools/` directory, two of those five **pass** — they had
been dying on an import, not on the repository. §86 fixed incomplete planting
in the gates; my own probe had it too, and it silently changed a table I then
reasoned from.

## 88. Widening a detector traded a miss for a false positive

The `BARE` guard from §86 keyed on the destination being a planted `tools/`
directory, spelled `/ "tools"`. It missed
`t / "tools/check_withdrawn_live.py"`, where the directory lives inside one
string — **the third time this campaign that a detector keyed on a shape found
only the instances sharing that shape.**

I found it the hard way: I added an import to that gate, four of its own
controls went red, and the guard written the previous day to catch exactly this
had stayed silent.

So I widened it to match `tools/` anywhere. It then flagged
`shutil.copy(REGISTRY, t / "tools/withdrawn.txt")` — a **data** file, which
needs no imports and is not a plant.

**Both errors come from keying on one end of the operation.** A plant is a copy
*of a script* *into a planted tools/*. The guard now requires both:

```python
dest_planted  = 'tools/' in t or '/ "tools"' in t or "/ 'tools'" in t
copies_script = "__file__" in t or '.py"' in t or ".py'" in t
```

Zero hits today; one hit when a bare copy is reintroduced in either spelling.

The general rule: **when a widened pattern starts firing on new things, do not
narrow it back — add the second condition that distinguishes them.** Narrowing
returns you to the miss you just fixed. And when a guard you wrote yesterday
stays silent on exactly what it was written for, the guard is the thing to
re-measure, not the code it cleared.

## 89. A probe retyped each time is a new instrument with new defects

I ran the same measurement three iterations running — every gate in a planted
empty tree — and wrote it by hand each time. The third copy planted each gate
ALONE, so two gates died on an ImportError and were recorded as crashing on the
repository. That table then justified a day of work.

Re-run with the whole `tools/` directory planted:

| | ad-hoc probe (planting one file) | complete plant |
|---|---|---|
| PASS | 3 | 4 |
| VERDICT | 2 | 13 |
| CRASH | 5 | **0** |

The crash column was entirely an artefact of my own harness — the same
incomplete-planting defect I had spent the previous iteration fixing *in the
gates*.

**A measurement you repeat is a tool, and it should live in the tree.** Not
because typing it again is slow, but because each retype is a fresh
implementation with fresh mistakes, and nothing compares it against the last
one. `tri gate-sweep` is now that measurement, with the planting done once and
the classification named:

* **PASS** — exit 0 over nothing. Legitimate for a self-test that builds its own
  corpus, and for a `skip()` that `--require` turns fatal. Anything else here
  is a gate that cannot fail.
* **VERDICT** — non-zero, with a sentence naming the missing input.
* **CRASH** — non-zero through a traceback. Loud, and still reporting the
  harness rather than the subject.

It reports and does not gate: which PASS is legitimate is a judgement, and
encoding it would be a second list to drift against `GATES`.

The three habits this cost me, in order of how expensive each was:

1. **Re-measure a conclusion when you fix the instrument that produced it.**
   §86 fixed incomplete planting in the gates. It should have been obvious that
   my probe planted the same way; it was not, until a number moved.
2. **Put the probe in the tree the second time you write it**, not the fourth.
3. **A tool that is not committed does not exist.** `loop-tools-tracked.sh`
   refused the new command until it was `git add`ed, naming the state that
   "already destroyed two of these scripts and every number they produced".

## 90. The sweep selected by name, so it measured the naming convention

`tri gate-sweep` picked files matching `check_*` or `*gate*`. Seventeen files.
There are thirty checkers in `tools/`; the other thirteen — every `verify_*`,
the fuzzer, the demos, the generators — had never been swept at all.

**Selecting by name measures the naming convention.** The sweep now takes every
`.py` in `tools/` that is not a private module and lets the reader judge what
belongs, which is the same call §—made for gate selection: choose by
**property**, not by what someone happened to call the file.

What the thirteen contained:

* **Five correct skips.** `verify_emit_bitexact`, `verify_multitarget`,
  `verify_igla_race`, `verify_trainer_c`, `fuzz_trainer` — all `SKIP … t27c not
  found`, all fatal under `--require`. Checked, because a skip that is not
  fatal under `--require` is a pass wearing a different word.
* **One traceback**, in a generator reading the catalog it generates from. Now
  a verdict naming the file.
* **One claim worth more than the rest of the sweep.**

## 91. The step name is the claim, and it was bigger than the file

`emit-bitexact-gate.yml` ran a step called:

> **Prove the trainer LEARNS (XOR 4/4 + nonlinear held-out >=90%, incl. deep 3-layer)**

The tool it runs is **pure Python**. No `t27c`, no `.t27` spec, no `iverilog`.
It generates backprop microcode, runs it on a bit-faithful GF-T *interpreter*,
and asserts on the *text* of the Verilog it emits. It passes unchanged in a
directory containing nothing but itself — which is exactly how the scope was
noticed, and could not have been noticed by reading the step.

The file's own docstring was **honest and precise** about all of this. The
overclaim lived entirely in the step name, and:

**The step name is the claim most people read.** Nobody opens the tool. They
read a green check and a sentence, and the sentence said the trainer learns —
which a reader takes for the compiler, the RTL, or the board. It is none of
those; it is a Python interpreter of generated microcode, which is a real
result and worth gating on, and is not that result.

Renamed to what it proves, with the reason in a comment above it, and a "what
this does not establish" block added to the tool listing the three claims it is
NOT: that the compiler produces this microcode, that the emitted Verilog
simulates, that any of it works on silicon.

**Two habits.** When a tool passes in an empty tree, read its CI step name
before deciding the pass is fine — an honest tool under an overclaiming name is
the most durable kind of wrong number, because every individual artefact is
accurate. And when writing a step name, write what would still be true if the
environment were bare.

## 92. "Every X" over a ratchet is a wrong number with no wrong number in it

Swept all 191 named CI steps for names carrying a completeness or proof word.
Fifteen. One stated what it does not check. Then I read the five that say
"Every X" against what their tool prints:

| step name | the tool's own output |
|---|---|
| Every spec still generates | `595 of 718 generate, 123 known-broken` |
| Every tracked JSON parses | `2078 tracked, none NEWLY unparseable (6 known)` |
| Every gate must fail loudly with nothing to check | `coverage: 6 of 16` |
| Every seal still describes its spec | red on master, 148 seals |
| Every catalog row still has its spec on disk | `SSOT == fresh regen == 109` ✓ |

Four of five name a **proof** over a tool that implements a **ratchet**: a
ledger of exceptions, failing only when the set grows. Both are worth having
and they are different claims — and every artefact underneath is accurate, so
there is no wrong number anywhere to find. The wrongness lives only in the
sentence a reader of a green check actually reads.

Renamed to what a ratchet proves: *no X newly fails*.

**Then I nearly made it worse.** My first rename put the count in the name —
"(123 known-broken, ledgered)". A name carrying a number becomes a **wrong
number** the day the ledger moves, and a precise falsehood is worse than the
vague truth it replaced. The count belongs in a comment, which git dates, and
in the tool's output, which is recomputed. The name carries the *shape* of the
claim and no figures.

`tri claims` is the sweep, kept in the tree per §89: it lists every step whose
name carries a strong word and whether the step says what it does not check.
It reports and does not gate, and it says so: **a flagged name is not a wrong
name.** "Every tracked JSON parses" over a gate that reads every tracked JSON
is exactly right, and no pattern can tell that from a ratchet wearing the same
sentence — only reading the tool can. What the sweep buys is a short list out
of 191.

It also reports the workflows it could not parse, separately. A file this
cannot read is not a file without claims.

## 93. Matching a tool name in a `run:` block matched a comment

My first pass at "which CI steps invoke a tool that passes over nothing"
searched the whole `run:` text for the tool's filename. Thirteen hits. One was
`fpga-build.yml`'s "Yosys parse + hierarchy" step, which looked like a serious
mismatch — until the match turned out to be this, inside the step body:

```
# convention (suite.rs:859, verify_emit_bitexact.py:184) reads generated
```

A **comment**. The step never runs that tool, and it is in fact the best-behaved
step in the repository: a deliberately narrow name, and a `DOES NOT CHECK`
block echoed into the job summary with counts and issue links.

Stripping comment and `echo` lines before matching, and requiring the name at a
path boundary, gives twelve real invocations and no false one.

**A `run:` block is prose and code in one string.** Anything that greps it is
reading documentation as if it were behaviour — the same error as grepping a
source file for a call and finding it in a docstring. Strip what is not a
command first, and match at a boundary rather than as a substring.

## 94. "Prove" over a fixed sample, and the distinction lived in the tools

The remaining strong-named steps, read against what their tools actually do:

| step name | evidence |
|---|---|
| Prove generated RTL == GF-T model (bit-exact) | 14 topologies from a **list**, one seeded training run |
| Prove GF-T primitives bit-exact across C + Rust + model | `N = 600` pairs drawn from **48** values |
| Prove the WHOLE trainer bit-exact in C | `STEPS = 80`, seeded |
| Exhaustive, every arm, whole domain | genuinely enumerates — **"wherever the space is small"** |

Three name a proof over a **fixed sample**. The fourth is the real thing, and
its first line says exactly where the real thing stops.

**The distinction was already in the repository — in the tools, in careful
prose, by an author who clearly understood it. It had simply never reached the
step names**, and the step name is what a green check shows.

I checked the sample first for the failure I expected: an unseeded random
sample, which would make a "proof" draw different inputs every run. **It is
seeded** — `random.seed(202)`, `random.seed(101)` — and two consecutive runs
produce byte-identical output. That is a strength, and it has a shadow worth
naming: *the sample never moves*, so a disagreement outside those 600 pairs
cannot be found here however many times CI runs.

Renamed to name the evidence, with `DOES NOT CHECK` blocks saying which inputs
are outside.

## 95. My limits statement was invisible because I wrote it in my own words

After adding a scope note to three steps, `tri claims` still reported them as
stating none. The note said "FIXED, SEEDED sample … and not that". The
detector's vocabulary is the repository's: `does not check`, `not claimed`,
`says nothing`, `scoped claim`, `is a ratchet`.

**A limit stated in different words reads as no limit at all** — to the
detector, and to a reader scanning for the familiar phrase. Rewritten as
`DOES NOT CHECK: …`, the count moved.

The tempting fix was to widen the detector's vocabulary to include my phrasing.
That is backwards: **an established vocabulary is worth more than any single
statement made in it**, and widening it once per author ends with a regex that
matches everything and means nothing.

The same sweep has a blind spot I could not fix the same way, and named
instead: **it reads workflows, so a limit stated in the tool is invisible to
it.** `verify_exhaustive.py` opens with as careful a scope statement as exists
in this tree and is reported as having none. A flagged name may be the most
honest thing in the repository — the flag means *worth reading*, and reading
means opening the tool.

## 96. Three absurd widths, three different mechanisms, and only two were bugs

The widths ledger opened with ten specs carrying a Verilog range no hardware
can hold. It closes with one, and the interesting part is that the three
mechanisms behind them were unrelated:

| width | mechanism |
|---|---|
| `4294967295` (u32::MAX) | `range_decl(width - 1)` with no case for 0; a struct of only `&str` sums to zero |
| `18446744073709551615` (usize::MAX) | `total_width - 1` where `total_width = dims[0] * elem_w` and `dims[0]` is **0** — `var xs : [0]T`, a legal empty list |
| `4198431` | **not an underflow at all** |

The third is arithmetic that is correct:

```
Str { data: [4096]u8; len: u32 }                    = 4096*8 + 32     =    32,800
Map { keys: [64]Str; values: [64]Str; count: u32 }  = 2*64*32800 + 32 = 4,198,432
```

513 KiB in one packed register. The backend computed it faithfully; the spec
asked for something with no hardware form. **A gate that catches a class will
catch things outside the class it was written for**, and the discipline is to
find the mechanism for each hit rather than assume the one you already fixed.

Two habits this cost:

* **The detector keyed on a property found all three; a search for the literal
  found one.** Keyed on `4294967295` by hand: 8 specs, one mechanism. Keyed on
  "no real bus is a million bits wide": 10 specs, three mechanisms.
* **Write the mechanism into the ledger next to the entry.** The remaining line
  now carries the arithmetic, so the next reader does not spend an evening
  looking for a subtraction that is not there. A ledger entry without its
  mechanism is an invitation to re-derive it, wrongly.

## 97. Ninety-six sites format a width inline, and one of them was the bug

Searching for the second underflow turned up **96** places in the compiler that
format `[{}:0]` with a bare `- 1`, bypassing the helper that clamps. Eight use
`saturating_sub`.

That is a **candidate list, not a finding**. Exactly one of the 96 produced a
bad width over the 650-spec corpus, because most operate on widths that cannot
be zero — bus widths from a config, element widths already `.max(1)`'d.

The temptation is to fix all 96. Don't:

* The change is enormous, and its blast radius is the whole backend.
* 95 of the edits would be unverifiable — there is no input that reaches them
  with zero, so no test can distinguish the fixed version from the broken one.
* An unverifiable edit in a compiler is a change you cannot defend later.

**Fix the site the corpus reaches. Record the other 95 as a candidate list with
the count, not as a claim that they are broken.** The measured result is what
belongs in the commit: 650 specs regenerated, **1 file differs**, and it is the
target.

The general form, and it is the same rule as §50 one layer down: *a static
search over an emitter tells you where a bug COULD be. Only running the corpus
tells you where one IS.*

## 98. The full table, and two of five operators were measuring the wrong sites

Recomputed all five operators over the tree, with the repaired cache:

| operator | killed | sites | rate |
|---|---|---|---|
| silent | 47 | 52 | 90% |
| loud | 28 | 32 | 88% |
| invert | 72 | 78 | 92% |
| **boundary** | 26 | 77 | **34%** |
| **assert** | 2 | 16 | **12%** |
| **total** | **175** | **255** | **69%** — 80 survivors |

Previously 157/244 with 87 survivors. The verdict-reaching operators sit near
90%; the whole weakness is two columns, and **40 of the 80 survivors are in one
file**.

Reading the survivors instead of counting them changes what they mean:

* **The boundary operator has no notion of a verdict.** `invert_sites` keeps
  only conditions whose body carries one — hence 92%. `boundary_sites` mutates
  *every* comparison, and the survivors it reports are `while len(v) < N`,
  `if len(out) > 6`, `while j < len(src)`: **loop bounds and display cutoffs**.
  Moving those cannot make a gate stop failing, so they were never the question.
  A 34% rate over sites that do not reach a verdict is not a finding about
  gates; it is a finding about the operator.
* **One survivor was already labelled equivalent by the tool itself**:
  `if r.returncode < 0` where a guard above forces `!= 0`, so `<` and `<=` are
  the same predicate. The machinery to say this exists; it just runs over the
  wrong population.
* **The assert survivors are thresholds with margin.** `te >= int(0.9 * 60)` is
  a floor of 54, and the measured values are 58, 56, 59, 59, 55, 55. Two sit
  **one point** above the line — so the threshold is doing real work, and the
  boundary operator survives because 55 satisfies both `>= 54` and `> 54`.
  Slack is not the problem; the operator is asking a question the data cannot
  answer either way.

So the honest correction to this campaign's own framing: **"survivor" means
"defect" for the return operators and does not for the other two**, and I had
been reading one number across all five.

Restricting `boundary_sites` to verdict-bearing comparisons is the fix, and it
is a change to the instrument that deserves its own measurement rather than
being folded into a tick that discovered it.

## 99. The weakest operator was the one with no flag

`tri gates mutate` had `--loud`, `--invert`, `--assert` — and boundary
reachable **only through `--all`**, which runs all five over every gate. The
one operator you cannot iterate on alone was the one with the worst rate.

Four selectable and the weakest not is exactly backwards, and no measurement
would have found it: the gap was in the **argument parser**, where none of this
campaign's instruments look. It surfaced because I tried to run a single column
and the CLI refused.

Added, with a test asserting every operator has a flag and that a misspelling
is refused.

**The negative control earned its place immediately.** My first version of the
test built the wrong argv — `["tri", "gates", "mutate", flag]` against a parser
that wraps `GatesCmd` directly — and *every* flag came back unaccepted,
including the three that already worked. `--loud` failing was the tell that the
harness was wrong rather than the parser. A test whose failure names something
you know to be true is reporting on itself.

## 100. The tool for this existed, and I typed my own loop instead

Yesterday I merged a pull request with two checks still running, and wrote it up
as a discipline failure. It is worse than that.

`tri pr ready <N>` already:

* counts checks that have not completed and prints
  `VERDICT: WAIT — N check(s) still running, the list is incomplete`;
* classifies each failure against the default branch, so a pre-existing red
  does not read as a new one;
* carries a `--merge` flag whose own help says why:

> *"The verdict cannot gate anything if the caller puts `gh pr merge` in the
> same batch as this command: it prints WAIT, the merge runs anyway, and nobody
> reads the line. **That happened four times in one session.** Handing the merge
> to the command makes the two inseparable."*

An earlier me built the defence, measured the failure mode, and wrote it into
the flag's documentation. I then hand-rolled a bash polling loop every tick and
reproduced the failure the flag exists to prevent.

**§89 said a probe retyped each time is a new instrument with new defects. This
is the sharper case: the instrument was already in the tree, correct, and
documented — and I typed past it.** Writing a tool does not make you use it;
nothing in the repository could have stopped a bare `gh pr merge`.

## 101. And the verdict never reached the exit code

Reading it to use it turned up a real defect: `ready` ends in `Ok(())`. **WAIT,
CANNOT TELL, DO NOT MERGE and "safe to merge" all exited 0.**

So `tri pr ready N && gh pr merge N` merges on WAIT. The flag's help says the
verdict "cannot gate anything" when the merge is a separate command — and part
of why it cannot is that the exit code carried nothing. An honest line under a
zero exit is §83 again, now in the tool that decides whether to merge the
gates.

```
0  safe        every failure is failing elsewhere too
1  DO NOT      a failure appears only here
2  WAIT        the list is incomplete
3  CANNOT TELL a failure has no baseline to compare against
```

With a test that the four are **distinct** — two verdicts sharing a code is a
caller that cannot tell them apart — and that **pending outranks everything**,
including a clean failure list. An incomplete list must win over anything
computed from it.

The general rule, and this campaign keeps arriving at it from new directions:
**a verdict that lives only in stdout gates nothing except a human reading
carefully.** Print the sentence *and* return the code.

## 102. The live control caught me measuring with the wrong binary, again

Having fixed `tri pr ready` to return its verdict, I ran it against the very
pull request carrying the fix, while 19 checks were still running. It printed
`VERDICT: WAIT` and **exited 0**.

The fix was correct. `./scripts/tri` dispatched to `target/release/tri`, seven
hours old, because the front door tried release first and `cargo build -p tri`
writes debug. Running the debug binary directly gave the expected `2`.

**Third time this campaign that a measurement ran against a binary that did not
contain the change** — and the first time the front door itself was the cause
rather than my command line.

Two fixes, both in the wrapper:

* **Newest wins, not release-first.** Preferring a profile means preferring
  whichever one you did not just build.
* **Say so when the binary predates its source.** `find cli/tri/src -name '*.rs'
  -newer "$BIN" -print -quit` is one call; silence there is indistinguishable
  from "your edit did nothing".

The habit worth keeping is smaller than either: **run the new behaviour against
something real before believing it.** A unit test on the verdict table passed
the whole time. What caught this was pointing the command at a live pull
request with checks in flight — the one situation the change exists for.

## 103. The unused instrument was red from the day it landed

Yesterday's lesson was that a correct tool sat in the tree while I typed my own
loop. So I ran the front door's inventory and picked the command I had never
run: `tri audit` — *"every pre-wave invariant in one exit code."*

It fails on master:

```
FAIL  lessons   0 lessons, highest 0, no anomalies
```

Read that line. **Zero lessons, no anomalies, FAIL.** Three claims, and they
cannot all be about the same thing.

Two defects behind it, and the second is the interesting one.

**The feature is not this repository's.** `git log --all -S'**1. '` over the
file the counter reads returns nothing: the `**N. Title.**` lesson format has
*never existed there*. This repository keeps **theorems** — 914 headings,
highest T727b, counted correctly by the same command. The lesson ledger belongs
to a wave loop in another repository, and the command arrived carrying it.
So zero is the right answer, and calling it a failure is the bug.

**And the verdict was never the checker's to give.** `scripts/tri` runs under
`set -euo pipefail`, and the counter is

```sh
grep -oE '^\*\*[0-9]+\.[[:space:]]' "$FILE" | tr -dc '0-9\n'
```

An empty `grep` exits 1, `pipefail` hands that to the pipeline, and the awk
block's own `exit 0` never reaches the caller. **The exit code belonged to the
search, not to the answer** — so even after teaching the checker that absence
is not an anomaly, it still returned failure. `|| true` on the search, and the
verdict is the checker's again.

Three things worth keeping:

* **Absence and anomaly must not share a verdict.** "Nothing here" and "something
  is wrong here" are different findings; a line that says both is reporting
  neither.
* **`grep | ... ` under `pipefail` returns the grep.** This campaign has the
  mirror of it recorded — `rc=$?` after a pipeline reading `tail`'s status —
  and this is the same defect from the other end of the pipe.
* **An instrument nobody runs is not neutral; it rots red.** This one landed
  four days ago and has been failing since, which is survivable only because
  nothing depends on it. The moment something had, the first thing it would
  have learned is a falsehood.

Verified in both directions: with two lessons planted at 1 and 3 the checker
prints `gap: 1 -> 3` and exits 1; restored, it exits 0.

## 104. The rules file was sealed, and two of its clauses are observed by nobody

Continuing yesterday's sweep of commands I had never run: `tri loop-rules`
checks `docs/loop/LOOP-RULES.md` against a sealed digest and passes. It closes
with its own disclaimer:

> *"This certifies identity only. It does not certify that the rules are
> correct, nor that the tick obeyed them."*

That is the file's own **R6** — *the seal certifies identity, not correctness* —
stated about the tool that enforces it. Nothing measured the second half, so I
did. Over the last 30 merged pull requests, **every author**:

| clause (R11) | observed |
|---|---|
| branch prefix `w699-<topic>` | **0 of 30** |
| provenance tag on a number | **0 of 25** stating one |
| no `Closes #N` autoclose | 30 of 30 ✓ |

The tag convention is alive elsewhere — `[measured]` appears **211** times in
the theory document. It has simply never reached a pull request.

**Two clauses at zero across every author is not one operator drifting.** The
file and the practice have parted, and **the seal made the file look governed
while nothing looked at the practice.** A checksum over rules is exactly as
strong as a green check over a gate that cannot fail.

There is a third clause I am knowingly not observing — *"do not merge; merging
is a human act"* — because the repository owner authorised it outside this
file. I did not edit the rule to match what I do. **A rules file edited by the
thing it governs is worth nothing**; the conflict is filed where a human sees
it.

`tri rule-observance` measures the three mechanical clauses and **reports
without gating**, for two reasons worth separating:

* **Which way a drift resolves is not the measurer's call.** Practice to rule,
  rule to practice, or rule plus a gate — each is defensible, and encoding a
  preference makes the decision by default.
* **Most of the file is not mechanically checkable.** *"Your own instrument is
  the first suspect"* and *"a differential claim names the class of loss it
  checked"* are judgements about a tick's content. A regex pretending to score
  them would be worse than the silence it replaced — it would make the
  unmeasured look measured, which is the whole subject of this file.

And the smaller finding under it: **R2 is "your own instrument is the first
suspect."** I have re-derived that rule from scratch five times in this campaign
— the broken ruler, the stale binary twice, the incomplete plant, the retyped
probe — while it sat written down, sealed, in a file I had never opened.

## 105. Every rule with a program is observed; every rule without one is at zero

Read `docs/loop/LOOP-RULES.md` properly instead of the three clauses a regex
could reach, and checked each one that can be checked at all:

| clause | enforced by | observed |
|---|---|---|
| R1 — `cost` output format | the tool itself | **yes** — n, med, p95, min/max, CV, `alpha` suppressed at n<8, KB range, build profile |
| R5 — a tool whose number reaches a report is committed | `loop-tools-tracked.sh` | **yes** (it refused my new command until `git add`) |
| R6 — the seal certifies identity, not correctness | `tri loop-rules` | **yes**, and it says so in its own output |
| R10 — an absent check is not a passing check | `check_pr_branch_filters.py` | **yes** |
| R15 — six categories, `unchanged` ≠ "could not compare" | `diffbin`, **at runtime** | **yes** |
| R0 — one outcome per tick, ledger at `cron_tracking/` | nothing | the directory has **never existed** anywhere |
| R11 — branch prefix `w699-<topic>` | nothing | **0 of 30** |
| R11 — provenance tag on a number | nothing | **1 of 25** (the one is mine, yesterday) |

**The split is total.** Not "mostly" — every clause with a program behind it is
followed, and every clause without one is at zero.

R15 makes the point itself, about its own enforcement:

> *"The set must be **asserted** to partition the corpus at runtime; claiming it
> in a docstring is not a check."*

And `diffbin.py:458` does exactly that — sums the six categories, compares
against the file total, errors on mismatch. The rule anticipated that a rule
about categories, kept only in prose, would drift.

So the finding is not that anyone is careless. **A rule kept in a document is a
wish; a rule kept in a program is a rule** — and this file holds both kinds
under one heading, indistinguishable to a reader, all equally sealed by a
checksum that certifies neither.

Two consequences for how to write one:

* **When you write a rule, ask what would notice it being broken.** If the
  answer is "a careful person", the rule is a wish. That is not always wrong —
  some things cannot be checked — but it should be *said*, so a reader knows
  which kind they are holding.
* **A checksum over rules measures the text, not the practice.** Yesterday's
  §104 found the seal made the file look governed. Today's measurement says
  what governs instead: the five clauses that compiled themselves into tools.

I have re-derived R2 five times this campaign while it sat sealed in a file I
never opened. R2 has no program either.

## 106. The filter was wrong, and the killed set is what proved it

`boundary` kills 26 of 77 while `invert` kills 72 of 78, and the difference is
that `invert_sites` keeps only conditions whose body carries a verdict. So I
gave `boundary` the same filter: comparisons on an `assert`, or on an
`if`/`elif`/`while` whose body returns a verdict literal, raises `SystemExit`,
or says `FAIL`.

Sites went 77 → 5. And among the 72 removed were these:

| gate | before | after |
|---|---|---|
| `check_vector_data` | **6/6** | 0/0 |
| `check_seal_coverage` | **3/3** | 0/0 |
| `check_catalog_integrity` | **1/1** | 0/0 |

Those were **killed** mutants. Moving those comparisons demonstrably made the
control fail — which is proof that they reach a verdict. **My filter removed
sites the measurement had already proven verdict-bearing**, so the filter is
wrong, and reverted.

Why it cannot be fixed by widening: the sites it misses look like

```python
if x > threshold:
    problems.append(...)     # no return, no FAIL, no SystemExit
...
if problems:
    return 1                 # the verdict, several statements away
```

**Verdict-reachability is a dataflow property.** A line-local pattern cannot
decide it, and widening the pattern until the number looks agreeable is exactly
the move R2 forbids — the same one that made `tri damage` report 429.

So the 34% is not a defect to fix. **A boundary survivor means "moving this
comparison did not change the verdict", which for a loop bound is the correct
answer.** The population is mixed on purpose, and the *kills* are the ones that
identify the verdict-bearing half — after the fact, which is the only way it can
be known.

The negative result is the deliverable. What would improve this operator is
reporting the two populations separately, not filtering one away.

## 107. The cache key covered the subject and not the instrument

The filter above appeared to do **nothing**: 26/77 before, 26/77 after. Two
runs, one number, an edit that looked inert.

The mutation cache keys on the gate's bytes and its control's bytes. **Changing
how sites are selected changes neither.** So a rebuilt `tri` served 24 rows
measured by the version before the change, and the table did not move because
it was the same table.

`--fresh` gave 3/5, and only then was the filter's real effect visible — which
is also how the filter's wrongness became visible.

The key now includes `sha256(current_exe())`. Verified in three states:

* same binary twice → 2 rows cached;
* **rebuild the tool → 0 cached**, every row re-measured;
* same binary again → 2 cached.

**A cache that cannot see its own instrument change is the instrument lying
about itself** — R2 one level down. And the failure mode is the worst kind: not
a wrong number, but an *unchanged* number, which reads as "your change had no
effect" and invites you to conclude something about the subject.

The general form: **a cache key must cover everything that can change the
answer**, and the code computing the answer is part of that. Subject bytes are
the obvious half; the tool's own bytes are the half nobody writes down.

## 108. I ran the command twice and blamed the first run for the second's answer

Chasing §107 I saw `VERDICT: WAIT` and `rc=0` and concluded the exit-code fix
was not working. It was working. I had run the command **twice** -- once to
capture the output, once to capture `$?` -- and in the second between them the
last check completed. The WAIT belonged to the first invocation and the zero to
the second, and neither was wrong.

```
./tri pr ready N 2>&1 | tail -3     # says WAIT
./tri pr ready N >/dev/null; echo $?  # says 0
```

Two invocations of a command that reads a **live** external system are two
measurements of two different states. Written on adjacent lines they read as
one.

```
out=$(./tri pr ready N 2>&1); rc=$?     # one invocation, both facts
```

This is the whole campaign's subject arriving in the shell one-liner used to
investigate it. Before suspecting the subject, look at how many times the
question was asked -- and against a moving system, asking twice is already a
different question.

## 109. When you cannot separate two populations, show them

§106 established that the boundary column's denominator holds two populations —
comparisons that decide a verdict, and loop bounds and display cutoffs that
cannot — and that no line-local filter can separate them, because a kill is the
only proof of verdict-reachability and the filter removed proven kills.

That leaves the column unreadable rather than wrong. `SURVIVED at boundary
lines 45, 91, 214, 223, 226, 409` tells a reader nothing they can act on.

Two changes, neither of which touches the measurement:

**Print the source beside the line number**, for this operator only:

```
91   `if len(out) > 6:`          <- a display cutoff
214  `while len(v) < N:`         <- a loop bound
226  `while j < len(src):`       <- a scan bound
223  `if b < 0:`                 <- the only one worth reading
```

Five of six classify themselves at a glance. The reader does the separation the
tool cannot, and does it in seconds instead of opening the file.

**Say what the denominator is**, in the summary, only when this operator ran:

> *"its denominator counts EVERY comparison, including loop bounds and display
> cutoffs, where a survivor is the right answer. The killed count is a lower
> bound on the comparisons that reach a verdict — proven by the kill itself. Do
> not read killed/total there as a rate; the denominator is two populations."*

The general rule, and it is the honest alternative to a filter that lies:
**when a metric mixes populations you cannot separate, do not report a ratio —
report the members.** A rate over a mixed denominator invites exactly the
conclusion this whole file exists to prevent, and it invites it most from the
person who computed it.

The killed count keeps its meaning either way: it is a **lower bound**,
established after the fact, on how many comparisons reach a verdict. That is a
smaller claim than a percentage and it is true.

## 110. The first survivor I read was real, and it was in the encoder

Printing the source beside the line number (§109) paid for itself on the first
survivor that did not classify itself: `if off < 0: return 0`, in the GF-T
encoder every trained weight passes through.

`off = e + 40`, so **`off == 0` is the smallest normal binade, `[2^-40, 2^-39)`**.
The mutant `off <= 0` encodes every value there as `0` — which is the **zero
sentinel**. A non-zero magnitude, silently reported as zero.

**The obvious witness does not work, and that is why the mutant lived.** At
exactly `2^-40` the mantissa is 0, so the value *already* encodes as 0 and the
mutation changes nothing. A witness needs a non-zero mantissa in that binade:
`1.5 * 2^-40` encodes `256` and would become `0`.

Anyone testing "the boundary" by reaching for the boundary value would have
concluded the mutant was equivalent. **The boundary value was the one point in
the binade where it is.**

Three assertions pin it: the magnitude, the sign, and the value one binade
lower which *is* legitimately zero. Negative control: with `<=` planted the tool
exits 1 naming the new assertion; restored, 0. The column moved **5 → 6 killed**
*(published as 5/31 → 6/31; the denominator was half the real one -- see §121)*.

## 111. "Boundary survivors are loop bounds" was too broad, measured

§106 concluded that boundary survivors are loop bounds and display cutoffs where
a survivor is the right answer. That was drawn from one gate's six survivors.
Classifying all four `if x < 0:` sites across the tools:

| site | source of the value | `<=` equivalent? |
|---|---|---|
| `verify_igla_race:223` | `src.find("{", start)` | **yes** — `find` returns −1 or ≥ start, and every caller passes a match start whose first character is a keyword, so 0 is unreachable |
| `gft_backprop_microcode:50` | `off = e + 40` | **no** — 0 is the smallest normal binade |
| `gft_backprop_microcode:203` | `d = ho - lo_o` | **no** — 0 is equal exponents, a normal case |
| `diffbin:148` | `rest.rfind('"')` | **no** — 0 is an empty quoted string, `""` |

**One of four.** Three are real boundaries where zero is a valid value, and the
`rfind` one is real *despite* being the same idiom as the equivalent one — the
difference is whether index 0 is reachable in that string, not what function
produced it.

So the earlier generalization was a sample of one gate promoted to a rule about
an operator. The correction is not "survivors are real" either: it is that
**each survivor is a separate question, and the source line is what makes
answering it a minute's work instead of an afternoon's.** That is the whole
value of §109, and it showed up on the first reading.

## 112. An outcome test cannot see an arithmetic defect that outcomes tolerate

`if t > hf: mant += 1` survived. Reading it: this is the round-half-to-even
decision in the GF-T adder.

```python
if t > hf: mant += 1                  # strictly above half
elif t == hf and (s & 1): mant += 1   # exactly half -> to even
```

With `>=` the pair becomes **round-half-up** and the parity test on the next
line is dead code. Measured, not predicted:

```
_magadd(25600, 20480)   clean 25600    mutated 25601
_magadd(20992, 20481)   clean 21248    mutated 21249
```

**256 such witnesses exist, and the self-tests notice none of them.** They train
a network and assert on **accuracy**. An optimiser absorbs a last-bit error in
every addition without changing whether XOR reaches 4/4 — planted, the mutant
passes every training assertion in the file.

That is the shape worth carrying: **a test that checks an outcome cannot see a
defect the outcome tolerates.** Accuracy, throughput, "it still converges" —
each is a real property and each is a filter that removes exactly the errors an
adaptive process routes around. Arithmetic needs assertions on arithmetic.

Three now pin the decision, one per branch. Negative controls: weakening `>` to
`>=` fails the even-tie assertion; disabling the parity test fails the odd-tie
one. Each control hits the assertion aimed at it, which is what makes them three
tests rather than one repeated.

## 113. I planned a tick around a line number I never checked

This tick's plan was "close `microcode:203` and `diffbin:148`", carried from
yesterday's classification table. `203` is **not a survivor** — with `<=` the
XOR self-test reports 1/4 and the mutant dies loudly.

I had listed it among four `if x < 0:` sites found by grep, classified it as a
real boundary, and then silently promoted "real boundary" to "surviving
boundary". They are different claims: one is about the code, the other about
what the tests reach.

**§70 is mine: a finding recorded as a line number expires on the next edit.**
This is worse — it never was a finding, and the line number gave it enough
shape to look like one. A grep hit is a candidate; a survivor is a measurement;
and the plan for the next tick should be built from the second.

The tick recovered because the first thing I did was reproduce the survivor
rather than fix it. **Reproduce before repairing** costs one command and is the
only thing standing between a plan item and an afternoon spent on a defect that
is not there.

## 114. The same defect, one function over

`_magadd`'s tie rounding was fixed last tick. `_magmul` carries the **identical
pair** — over a product instead of a sum — and its `if r > half` survived for
the identical reason.

```python
if r > half: mant += 1
elif r == half and (q & 1): mant += 1
```

Two paths, carry and no-carry, with different `half`, so **four** cases are
needed rather than three:

| case | clean | `r >= half` | parity off |
|---|---|---|---|
| carry=0, q even | 20610 | **20611** | 20610 |
| carry=0, q odd | 20738 | 20738 | **20737** |
| carry=1, q even | 20998 | **20999** | 20998 |
| carry=1, q odd | 21016 | 21016 | **21015** |

Only the even-q rows distinguish `>=`. Only the odd-q rows distinguish a dead
parity branch. A test set that covered one parity would have looked thorough
and caught one mutant of two.

**When a fix lands, grep the file for the shape rather than the line.** The
adder and the multiplier are different functions with different variables over
different arithmetic, and the defect is the same five tokens. The survivor list
had already named it; I only had to read the next row.

## 115. A negative control that silently did not apply

My first control reported the `>=` mutant as **surviving** the new assertions —
which would have meant the assertions were useless.

Nothing had been planted. The replacement string was built in a shell loop and
lost the leading `if `, so `str.replace` matched nothing and rewrote the file
unchanged. The "mutated" run was the clean run.

**A no-op substitution reads as the strongest possible evidence for the wrong
conclusion.** Not silence — a confident *"your assertion does not catch this"*,
in the voice of a measurement. It nearly cost the four assertions that had just
been shown, by direct measurement, to work.

The fix is one line in every plant:

```python
assert old in s, "anchor not found -- the replacement would be a no-op"
```

The general rule this campaign keeps circling: **a control must fail when it
cannot do its job.** A plant that cannot find its anchor has not tested a
weaker version of the subject; it has tested nothing, and must say so rather
than return a verdict. §85 found the same thing in a planted *tree* dying on an
import; this is the same defect in a planted *edit*.

## 116. Three sweeps, three ways of never reaching the branch

Four sites carry the same renormalisation line — `enc`, `_magmul`, `_magadd`,
`_magsub`, each ending in `if mant >= 512: mant = 0; <exponent> += 1` — and all
four survived `>=` → `>`. Three probes in a row reported **no difference**, and
every one of them was wrong about the probe, not about the subject.

| probe | why it read clean |
|---|---|
| swept `1.0 + k/4096` | every value in one binade, so `off = 40` throughout — **even** |
| swept `(512+k)·2^(e−9)` | exactly representable, and the branch is reached **only** by rounding up from 511 |
| four separate builds | one character changed, so file **size** was unchanged and Python reused the first build's `__pycache__` |

The mechanism the first two miss: the mutant leaves `mant == 512`, and
`(off << 9) | 512` is **identical** to `((off + 1) << 9)` whenever `off` is
**even**, because `512 == 1 << 9` is the low bit of the exponent field. The
stuck mantissa renormalises the value *by accident* on half the exponent space.
So the defect is not rare — it is **invisible on half the inputs**, and a sweep
that holds the exponent fixed lands entirely inside the invisible half.

**A sweep that varies one coordinate finely proves nothing about the coordinate
it holds fixed.** Fine resolution inside one binade reads as thoroughness and
is not: 4096 samples of a single `off` answer one question 4096 times. What was
needed was the exponent's **parity** — a property the sweep never varied because
it never occurred to me that the exponent was a coordinate at all.

### The stale-bytecode run is the one worth keeping

It reported all four sites with the **same** difference count at the **same**
position — inside a function three of them do not touch. That is impossible,
and impossible is cheap to notice. Had the cache produced a merely *plausible*
table, it would have shipped.

`-B` (or clearing `__pycache__`) is the fix, but the transferable part is the
precondition: **mutation harnesses generate same-size files by construction.**
`>=` → `>`, `<` → `<=`, `return 1` → `return 0` all preserve or barely move the
length, and same-size plus coarse mtime is exactly the case Python's bytecode
cache resolves in favour of the stale copy. A harness that rebuilds a module
in place is the one place this hits hardest, and it hits silently.

### What actually established the harness worked

A **positive control on the harness**: plant a mutant already proven live
(§114's `r > half`) and require differences. It reported 66 — so the harness was
sound while its answers were still wrong, which localised the fault to the input
space rather than the machinery. Without it I would have concluded the sites
were unreachable and closed them as equivalent mutants.

### One site was structurally different and the count said so

In `_magmul` the carry path cannot reach `mant == 512` **in principle**: the
largest product is `1023 * 1023 == 1046529`, giving `q == 1022`, `mant == 510`.
Only the no-carry path distinguishes that site. My hand-derivation said
`q == 1021`. The computation said `1022`. Both support the same conclusion, and
that is the point — **the conclusion surviving does not make the derivation
right**, and only the one I ran was checked. This is the second time this
campaign a hand-derived intermediate was off while its conclusion held.

### The four assertions are load-bearing, and that was measured too

Each holds clean, fails when its own site is mutated, and **catches no other
site's mutant**. Without the third check, four assertions could have been one
real one and three duplicates wearing different arguments — indistinguishable
from the outside, and a maintainer deleting "the redundant ones" would have been
right three times and catastrophically wrong once.

Boundary column for the file, measured: **killed 8 → 12** *(published as 8/31 →
12/31. The denominator was wrong: the scanner stopped at `def self_check()` and
never resumed, so it saw 31 of the file's 62 comparisons. §121. Measured again
after the fix: 14/62 -- the killed count never moved, the denominator doubled,
and every extra site is in the `__main__` block where surviving is correct.)*

## 117. A dead branch and a dead signal are different defects

`_magsub` resolves a rounding tie three ways, and only two of the arms run:

```python
elif rem == half:
    if sticky: mant += 1      # never taken
    elif q & 1: mant += 1
```

Deleting the `sticky` arm changes **0** of 525,918 outputs. A mutation tool
that only mutates *branches* reports it equivalent and moves on.

Mutating the **producer** instead tells a different story:

| change | differences |
|---|---|
| force `sticky = 0` at both producers | 0 |
| delete the `if sticky: mant += 1` arm | 0 |
| **force the sticky detector to always fire** | **9851** |

The third row is the finding. The arm is *wired* — waking the signal wakes the
arm. What never happens is the **value**: an exhaustive search over every
`(hm, lm, d)` finds no input where `rem == half` and `sticky == 1` hold
together. So this is not unreachable code; it is **live code behind a signal
that never asserts**, and the two look identical from the branch.

**The mutation that distinguishes them is upstream of the branch.** Branch-level
mutation can only ask "does anything notice if this arm stops running?" Both
defects answer *no*. Only mutating the producer asks "does anything notice if
this arm *starts* running?", and there the answers diverge.

### The stakes are not cosmetic

The arm is the **addition** rule sitting in the **subtraction** path. Discarded
bits of the subtrahend make the true difference *smaller*, so a tie with lost
bits sits strictly below half and must round **down** — measured with exact
rationals: the code's `diff` is `13303794` against a true `13303793.734375`.
The arm rounds up. In `_magadd` the mirrored rule is correct, because there the
discarded bits make the true sum *larger*. **The sign flips with the operation
and the copy did not.**

So the function is correct *because* a branch never runs. Nothing in the tree
records that, and any future change to the alignment or the normalisation loop
that lets `sticky` reach a tie turns a dormant copy-paste into a live rounding
error with no test in front of it. Reported as #2652 rather than fixed. **Measured the next tick:** the
counterpart is `specs/ternary/gft_sadd.t27`, not `board/bpseq.v` — that path
has never existed in this repository. The spec carries the rule line for line,
30 specs carry a copy, and the arm is dead there too. See §120.

### Equivalent mutants are worth proving, once

`if d >= 26` → `if d > 26` is a permanent survivor: at `d == 26` the else branch
computes `la = ls >> 26 == 0` (`ls` is at most `1023 << 14`, below `2**26`) and
`sticky = 1` — exactly what the taken branch assigns. 0 differences over the
same 525,918 points.

**An equivalent mutant proven equivalent is closed work; an equivalent mutant
assumed equivalent is an excuse.** The difference is one measurement, and
writing the proof down is what stops the next tick from re-chasing it. Three
ticks of this campaign each re-examined a survivor an earlier tick had already
looked at, because looking left no trace.

### Redundancy is a claim about operators, not about assertions

Three assertions went in, and the third caught none of the five mutants the
other two were written against. That looked like a redundant assertion until it
was tested against a sixth: **deleting the primary rounding branch**, which the
boundary operator cannot express and which the first two assertions do not
notice.

§116 checked that four assertions catch different *mutants*. That is the weaker
question. The right one is whether they cover different **operator classes** —
here boundary, dead-branch, and deletion — because an assertion set can be
non-redundant against every mutant one tool generates and still be blind to the
whole class that tool does not emit.
## 118. Six claims that nothing had ever tried to refute

`# mutant-equivalent: <why>` marks a survivor as unkillable by construction,
and `tri gates mutate` prints it as *"claims equivalent: …"*. The word `claims`
was doing real work: **nothing had ever checked one.** Six sat in `tools/`.

A claim is a statement about the code. It ages with the code. And the run best
placed to notice it has gone stale is the mutation run itself — it already
built the mutant and already knows the verdict. The missing step was one
comparison.

Now a claimed line whose mutant **dies** is reported as contradicted. Measured
against the six that already existed: five checked, **none contradicted** —
which is worth exactly as much as the run that could have refuted them and did
not, and nothing more.

**An unfalsifiable claim is prose wearing the costume of an analysis.** It reads
like settled work to everyone who comes after, and the cost of leaving it
unchecked is not that it is wrong today — it is that nothing will say so on the
day it becomes wrong.

### The counting detail that matters

A line can hold more than one mutable site: `if a < 1 or b < 1:` holds two. So
the check compares **counts**, not membership — a claimed line with two sites of
which one survived has been contradicted **once**. Keying on *"did this line
vanish from the survivor list"* would call that claim intact, and a half-true
claim is the hardest kind to catch by eye.

### Writing about the marker created a claim

The extractor matched `mutant-equivalent:` **anywhere inside** any comment. So
the sentence *"that reasoning now sits on the line as a `# mutant-equivalent:`
claim"* — prose describing the mechanism — registered as a claim of its own,
bound to whatever code line happened to follow it.

Caught because the count printed **2** where I had written **1**. Nothing else
would have caught it: a claim nobody made, attached to a line it says nothing
about, silently waiting to be reported as *contradicted* the day that unrelated
line's mutant died. **A false positive in the extractor becomes a false
refutation in the checker** — the new check gave the old bug a way to lie.

The marker must now OPEN the comment. Every real claim in the tree already did.

The general shape: **a scanner that matches a marker anywhere cannot tell a use
from a mention**, and documentation is exactly where mentions live. Any tool
that greps the tree for its own vocabulary will eventually read its own
documentation as data — and the count is the cheapest place to notice, which is
an argument for printing counts nobody asked for.

### The claim marker names no operator, and that is a real gap

Every claim in the tree argues about a comparison (*"so `>=` is `>`"*), so they
are all about `boundary` — but the marker cannot say so. A line legitimately
equivalent under `boundary` may well die under `invert`, and the check would
report that as a contradiction. It names the direction rather than pretending to
judge. **The ambiguity is in the marker's design, not in the check**, and
reporting it is what makes it visible enough to fix.

### The positive control, and how it nearly did not run

Planting a false claim over a line whose mutant is known to die must produce a
contradiction. My first attempt printed **nothing** — and I read that as "the
check does not fire".

It fired at nothing. Planting the claim made `tools/` dirty, `mutate` refuses to
start on a dirty tree, and I was reading its output through
`grep -i "CONTRADICTED"` — a filter that cannot show a refusal. **The verdict
and the error went to the same stream and I had filtered out the half that was
speaking.** Same shape as §85 and §115: the control could not do its job and
said so, into a channel I had closed.

The fix was to satisfy the guard rather than bypass it — commit the plant on a
throwaway branch. Then it fired, exactly once, naming the line and the claim.

## 119. Two commits I destroyed with git, in one session

Neither was a merge conflict or a lost stash. Both were a command that means
something adjacent to what I wanted.

**`git checkout master -- <file>`** — I wanted to *branch* from master while
keeping an edit. That form restores master's copy of the file **over** the edit.
It came from a different recipe in this same campaign (measuring what master's
version does), where it is exactly right.

**`git checkout -b tmp` → commit → `git branch -D tmp`** — `checkout -b` carries
uncommitted work onto the new branch. I committed there to satisfy a clean-tree
guard, then deleted the branch, and my *unrelated* uncommitted edits to
`cli/tri/src/gates.rs` went with it.

Both were recoverable — the second from `git reflog`, which still held the
commit. Neither was noticed by a test, because in both cases the tree was
*consistent*, just missing work.

**The pattern is a command borrowed from a neighbouring recipe, where the verb
matches and the object does not.** `cargo fmt --all` (§ earlier) was the same
mistake with a different tool: right command, wrong repository.

The cheap guard is to name what you expect to still be there, and check:

```
grep -c "<the thing I just wrote>" <the file I wrote it in>
```

after any git command that moves branches or paths. Two seconds, and it turns a
silent loss into an immediate one. A destructive git operation deserves the same
suspicion as a destructive shell command — and the doctrine already says
*destructive tools last, never on the last working copy*; I had been reading
that as being about hardware.

## 120. The reference I cited was to a file that never existed

Closing §117 I wrote, in a merged pull request, that the sticky arm was
*"reported, not changed — this mirrors `board/bpseq.v`, and I have not measured
the RTL."*

`git log --all -- board/bpseq.v` is **empty**. The path has never existed in
this repository. I had repeated a pointer out of the file's own docstring
without checking it resolved, and then used it as the reason **not** to fix a
rule I had just shown to be wrong.

**A wrong pointer is worse than a missing one.** A missing one sends a reader
looking; a wrong one makes them stop — and it stopped me at exactly the moment I
was deciding whether a defect was real.

The actual counterpart is `specs/ternary/gft_sadd.t27`. Measured this tick:

| question | answer |
|---|---|
| does the spec carry the same rule? | **yes, line for line** |
| how many specs carry a copy? | **30** under `specs/ternary/` |
| is the arm dead there too? | **yes** — exhaustive over every `(hm, lm, d)` |
| do the two normalisations agree? | **yes** — bit-for-bit over 2,193,075 points |

The spec's barrel shift (`8/4/2/1` capped) and the Python's 12-step loop are
different code reaching identical results. So the generated Verilog, C and Rust
all carry the same dormant wrong rule.

### The bit-exactness suite cannot see this, by construction

`verify_multitarget` and `verify_emit_bitexact` prove Verilog, C and Rust agree
with the Python model. **They prove the compiler faithful to the spec; they can
never say the spec is right.** A defect written in the spec propagates to every
target and the suite stays green — that is the suite working, not failing.

`ALL TARGETS BIT-EXACT` is a true sentence that a reader takes for "the
arithmetic is verified". It is worth exactly what it says.

### One narrowness that WAS fixable

The suite's operands come from `uniform(-4, 4)` — measured, that is `off` 35..41,
**six of the format's 81 exponents**. A divergence that only shows when operands
are decades apart had never been in front of it. Widened to 16 of 81 across the
full range, appending pairs rather than reseeding so the existing 600 stay
byte-identical: **1744 pairs, still green**, and the negative control confirms it
still reddens (12 mismatches on a mutated model).

I had also guessed the suite was blind to rounding **ties**. Measured: 20 of 256
pairs land on `rem == half`. **The guess was wrong and the measurement was
cheap** — the blind spot was one dimension over from where I expected it.

### `tri pointers`, and how it caught its own docstring

The obvious version — check every path-shaped string — was measured first: 873
mentions, **409 unresolved**, dominated by paths a program *creates* and
fixture names inside unit tests. A report that is 95% noise gets switched off,
and the 5% goes with it. Narrowing to a prose pointer (`see X`, `cf. X`,
`documented in X`) gives 193 mentions and **16** unresolved — a list a person
can read, each row written on purpose.

**The narrowing was chosen by measuring both, not by taste.**

Then the first run reported the new tool's own docstring, which quoted
the dead path after a cue word, while explaining the bug. **Second time in two days that
a checker read its own documentation as data** (§118 was the first). A special
case for the tool's own file would have hidden the same thing in the next
document that discusses pointers, so the fix went where it belongs: the
docstring no longer spells the path after a cue word, and it says why.

Two instances in two days is not a coincidence — it is what happens when tools
scan prose for their own vocabulary. **Expect it, and expect the count, not the
list, to be what tells you.** Both times the tell was a number that was one
higher than it should have been.

### Third instance: this section

The paragraph above originally quoted the dead path after the word "see" — and
`tri pointers` flagged **this file**, the moment §120 landed on master. The
count went 16 → 17 and the row named the skill.

So: the section that names the pattern, warns you to expect it, and prescribes
fixing the writer rather than the tool — **committed the pattern in the act of
describing it**, and was caught by the check it was documenting.

Which is the honest form of the lesson. It is not a mistake you make once and
then know better; **the trigger lives in the vocabulary, so every document that
teaches the vocabulary carries it.** The only durable defence is the one that
caught all three: run the checker after writing about the checker, and read the
count.

## 121. The count was right for the wrong reason, and that is why I believed it

`tri gates mutate --boundary` reported **31 sites** for
`tools/gft_backprop_microcode.py`. An independent tokeniser counted **31**
comparisons before the `if __name__ == "__main__":` block. Two numbers agreed,
and the story wrote itself: the operator sensibly declines to mutate a file's
own self-test.

Both numbers were real. **The agreement was a coincidence.** A direct probe of
the site list showed it ending at line **371** — nine lines before `__main__`,
at `def self_check():`.

`is_control_fn` sets `in_control` on a control-named function, and
`boundary_sites` **never resets it**. So everything from line 380 to the end of
the file — including the `__main__` block, where the accuracy thresholds live —
had silently never been mutated. After the fix: **31 → 62 sites**, which is
exactly the tokeniser's count for the whole file.

Three of the four site finders already carried this fix, in these words:

> a function ends at the next TOP-LEVEL statement, not only at the next `def`

`assert_sites`, `mutable_sites` and `invert_sites` are **line-oriented** and got
it. `boundary_sites` is **byte-oriented** — it tracks quotes and comments
character by character — so when `--boundary` was added the fix had no line to
attach to and was not ported. **A bug fixed three times in one file can still be
live in the fourth place, when the fourth place is shaped differently.**

### The lesson is about the agreement, not the bug

A cross-check between two instruments is supposed to be the strong move. Here
both instruments were correct and the conclusion was still wrong, because
**they were answering different questions and I read the matching numbers as
agreement on mine.** The tokeniser answered *"how many comparisons precede
`__main__`"*; the tool answered *"how many sites did I find before I stopped"*.
Nothing connected the two but a number.

What broke the tie was not a third opinion — it was asking the tool for the
**line numbers** rather than the count. A total can coincide; a range cannot
lie about where it stopped.

**When two measurements agree, check that they agree about a shared object, not
a shared integer.** Prefer the answer with more structure: a list over a count,
a range over a total, a witness over a verdict.

### And the fix needed its own test, because the file could not exercise it

After the change the file yields 62 sites and **none inside `self_check`'s
body** — correct. But `self_check` in that file contains **zero comparisons**,
so the exclusion half of the logic was never exercised by the very file that
motivated the fix. A synthetic three-region source (helper / control / `__main__`)
covers all three answers, and reverting the fix makes it fail with `[2]` —
proving the test is load-bearing rather than decorative.

**A regression test written from the motivating file tests only the half that
was broken.**

## 122. I committed a live mutant, and the merge carried it to master

`tri gates mutate` rewrites files in `tools/` in place and restores them after
each site. It refuses to START on a dirty tree, and it drops a marker so an
interrupted run is recoverable. Both guards work. Neither protects **me**.

I launched a full boundary sweep in the background, kept working, and later ran
`git add -A && git commit --amend`. At that instant the sweep was holding
`tools/gft_backprop_microcode.py` mutated. The mutant went into the commit, into
the pull request, through 30 green checks, and onto master:

```
-    if d >= 26: la = 0; sticky = 1
+    if d > 26:  la = 0; sticky = 1
```

Nothing broke — that is the mutant I had *proved equivalent* two ticks earlier,
0 differences over 525,918 points, and the line even carries a
`# mutant-equivalent:` comment saying so. **The code stopped matching the
comment directly above it and every test stayed green**, which is exactly the
condition under which a wrong line survives indefinitely.

### The guard protects the run, not the operator

`mutate`'s dirty-tree refusal answers *"is the tree clean before I start?"*. The
question nobody was asking is *"is a sweep running while I stage?"* — and
`git add -A` cannot tell a mutant from an edit. **A background process that
mutates the working tree turns every `git add -A` into a lottery**, and the
odds scale with how many files the sweep touches and how long it runs.

Three cheap defences, in order of how much they cost:

1. **Stage paths, not `-A`,** while any sweep is running. `git add <the files I
   actually changed>` cannot pick up a file I did not touch.
2. **Check the marker before staging** — `test -f target/.tri-mutating` already
   answers "is a sweep in flight", and it exists precisely because an
   interrupted sweep is otherwise invisible. It was sitting right there.
3. **Diff the staged set against what you meant to change.** One line: the
   commit said `20 insertions, 1 deletion` and I had written only insertions.
   **A deletion I did not intend was in the summary git printed me**, and I read
   past it.

### Third self-inflicted git loss this session

§119 catalogued two: `git checkout master -- <file>` restoring over an edit, and
`checkout -b` + `branch -D` carrying unrelated work away. This is the third, and
it is the only one that reached **master** — the other two were caught locally.

The shape is the same each time: **a command whose blast radius is the whole
tree, run while my attention was on one file.** `-A`, `--all`, `.` — the
arguments that mean *everything* are the ones to distrust when anything else is
writing to the tree.

And the detector, when it finally came, was not a test. It was `mutate`
refusing to start on a dirty tree the next time I ran it — the guard catching
the consequence of its own bypass, one tick late.

## 123. The weakest operator was measuring the control, not the gate

`--assert` scores **2 of 34** on `gft_backprop_microcode.py` — by far the worst
of the five operators, deferred three iterations running as "the column to fix".

It is not a column about assertions. It is arithmetic on the number of planted
faults.

`self_check()` spawns exactly **three** whole-program runs: one clean, two with
a fault planted. A neutered assertion changes nothing on the clean run, so it
can only be noticed on a run where **that specific assertion would have fired** —
and only if it is the **first** to fire, because Python stops at the first
failing assert.

Measured, one variable at a time:

- plant 1 (a sign flip in `smul`) falsifies **both** line 632 and line 660;
- the program dies at 632, so neutering 660 alone is **invisible**;
- neuter 632 and the same plant now dies at 660 — *"real-task held-out too low:
  32/60"*.

**One kill per plant, and the kill set is exactly `{632, 668}`** — precisely the
two lines missing from the survivor list. Two plants, two kills, ceiling reached.

Three things follow, and none of them is about the survivors' quality:

1. **The survivors are not unreachable.** All 34 sites execute on a clean run
   (34/34), and replacing one with `assert False` fires it. They are invisible,
   not dead.
2. **Iterating on the operator cannot move the number.** Only adding a
   `spawned()` case can — one per *assertion*, not per family, because fail-fast
   shadowing means a plant that falsifies a whole family still surfaces its
   first member only.
3. **This file is the only gate in `tools/` with any assert site**, so the
   repo-wide assert column *is* this one file's row. "2 of 34 across the tree"
   and "2 of 34 in one file" are the same sentence wearing different clothes.

### My published explanation was wrong for at least 26 of the 32

`docs/now/2026-08-24-the-weakest-mutation-operator-was-the-one-with-no-flag.md`
accounts for the six held-out-threshold assertions — *"thresholds with margin:
floor 54, measured 58 56 59 59 55 55"* — and offers a boundary-operator
rationale for the column. That explanation is true of six rows and silent about
the other twenty-six, and it was published as an account of the column.

**A partial explanation presented as a complete one is the most expensive kind
of wrong**, because it closes the question. Three iterations then deferred the
column as "known and understood" rather than "unexplained".

### The second plant does not test what it says

The control's second case is *"a renamed port is caught by the emitter check"*.
The string `input [31:0] x0i` appears literally **only inside the assertion** —
the emitter builds ports at line 336 as `f"input [31:0] x{k}i"`. So the plant's
`str.replace` rewrites **the assertion's own expected string** and the emitted
Verilog is untouched. The case passes for a reason unrelated to its name.

This is the self-referential-plant class the comment at lines 447-455 was
written about — **fixed one line short.** A plant must edit the thing under
test; when the literal it targets exists only in the check, it edits the check.

### What made this findable

Not a better operator. A question about the **shape of the control** rather than
the contents of the column. The number had been read four times as a statement
about 34 assertions; it was a statement about 2 plants, and nothing in the
output distinguished the two readings.

**When a score is stuck at a small integer, count the things that can produce a
kill before you look at the things being killed.**

## 124. Raising a ceiling you have finally understood

§123 established that `--assert`'s 2 of 34 was the *number of planted faults*,
not a verdict on 32 assertions. This tick tested that model the only way a model
can be tested: **predict the number before measuring it.**

| plants | predicted | measured |
|---|---|---|
| 2 | 2 | 2/34 |
| +7 | 9 | **9/34** |
| +7 more | 16 | **16/34** |

Three for three. A model that survives being used to predict is worth more than
one that survives being argued for.

### Cost was never the obstacle, and nobody had checked

Each new plant fires in the arithmetic block at the top of `__main__`, long
before any training runs: **0.06s apiece**. The whole control went 11.8s → 12.5s
for **eight times** the coverage.

Three iterations deferred this column partly on an unexamined assumption that
more plants meant more whole-program runs. The two existing plants *are*
full runs, so the assumption generalised from a sample of two. **A cost you have
not measured is a reason you have not checked.**

### Within a family, the fault has to be surgical

Adjacent assertions test adjacent cases of the same code, so the obvious fault
breaks them all and only the first is ever seen. To surface the second member,
the fault must falsify it while leaving the first **true**:

```python
elif t == hf and (s & 1): mant += 1       # clean
elif t == hf and (s & 1) and False: ...   # kills the ODD tie only
```

The even tie still correctly declines to round up, passes, and lets the odd one
speak. Seven of eight designed this way hit their target exactly.

### The eighth is the lesson arriving inside the lesson

Disabling `_magsub`'s `if rem > half` to surface *"strictly above half"* also
breaks the renormalisation-carry case — which is checked **earlier**. The plant
fires that one instead.

It is left alone on purpose. A plant narrow enough to separate them would have
to encode the exact remainders, and then **the control becomes a second copy of
the thing it checks**, which fails for its own reasons and agrees with the
subject about all of them.

### The guard that mechanises two earlier lessons

T124 caught a plant whose needle's first occurrence was the control's own
source. T211 caught one whose needle's only occurrence was the **assertion**
checking the result. Both went green; both had names that were lies.

Every piece of subject code in that file sits **above** the control, and every
assertion under test sits **below** it. So one comparison decides it:

```python
cut = first byte where the plant's output differs from the input
assert cut < src.index("def self_check(")
```

Positive control: a plant that edits an assertion's text is now refused with
*"the plant edited the control or an assertion (byte 28218), not the subject
(which ends at 20179)"*. **Neither of the two earlier cases was caught by
reading; both would have been caught by this line.**

### A smaller one worth keeping

Listing the file's assertions with a throwaway `line.strip().startswith("assert ")`
scored **prose inside the control's docstring** as an assertion — the exact bug
fixed in `assert_sites` one tick earlier. The tool had learned; my one-off script
had not.

**A fix that lives only in the tool does not protect the scratch commands you
reach for while using it**, and those are where a wrong list quietly becomes a
wrong plan. The tell was the classification coming out nonsensical — zero
training assertions in a file that is mostly training assertions.

## 125. "The most recent N" cannot answer "has X happened since T"

I opened an issue asserting a repository-wide CI outage: *required checks have
not fired since 2026-08-24 11:06; every PR since is permanently BLOCKED.* It had
a measurement, a five-row table of refuted hypotheses, and a sharp closing
observation. **It was wrong.** The checks had not started yet. Twenty minutes
later both branches showed 30+ checks.

Two instrument errors compounded, and neither was visible from inside the
conclusion.

### `gh run list --limit N` is a recency window

The last 10, then 60, runs were dominated by two workflows that fire on every
push. I read *"the only workflows running are NotebookLM"* off a list that had
simply not reached back far enough. The API's own filter says otherwise:

```
gh api "repos/…/actions/runs?created=>2026-08-25" → 103 runs
   pull_request: 39   push: 31   schedule: 14   issues: 10
```

**A query that returns the most recent N cannot answer a question about a time
range.** It answers "what is newest", and if something noisy is newest,
everything else is invisible at any N you are willing to read.

### I queried the wrong workflow and believed the answer

`--workflow=now-sync-gate.yml` returned runs ending 08-24, which I took as *"the
required check stopped firing"*. The required context `check-now-freshness` is
produced by **two** files, and the run that satisfies it displays as
**"Check Now Freshness"** — a name I never searched for, because I had gone
looking by filename.

One file answered honestly. The check came from somewhere else.

### The shape of the error

Every row of my "what is NOT the cause" table was **correct**. Actions enabled,
workflows active, files present, no branch filter, no path filter, identical
trigger on a workflow that did run. I eliminated hypotheses carefully and
thoroughly — around a premise I never tested.

**A well-run elimination over a false premise produces more confidence than a
sloppy one.** The table was what made the issue persuasive, including to me.

### What would have caught it, and it is embarrassing

Waiting. The difference between "has not started" and "will never start" is
time, and nothing else. I had already written §116's lesson — *prefer the answer
with more structure* — and the structured answer here was available the whole
time: `created=>` returns a range, `--limit` returns a window.

The check that costs nothing: **before reporting an absence, ask the same
question with a different instrument.** Not a second opinion on the conclusion —
a second instrument for the observation.

### One thing in it was true and worth keeping

`gh pr checks` on a PR whose checks have not started shows a short **green**
list: two successes, zero failures, indistinguishable at a glance from a PR that
passed thirty-three gates. Both of the workflows I wrongly accused carry a
comment saying exactly that — *"An absent check is not a passing check"* — which
is why the outage reading was plausible enough to write down.

The hazard is real. The outage was not. **A true observation is not evidence for
the theory it made you think of.**

### Withdrawal is cheap; a standing wrong issue is not

Closed within the hour with the correction as the closing comment, because the
issue named an owner action that did not exist. The campaign rule stands: a
finding that survives a genuine attempt to kill it is worth acting on, and I
never attempted to kill this one — I only attacked its alternatives.

## 127. A check that aborts on the first failure reports one defect, not the count

`corpus_classifier_matches_lean_completeness` asserted Rust/Lean agreement one
spec at a time inside a loop. It had been reporting **one** disagreement for as
long as it has existed. There were **73**, and the other 72 had never been
printed by anything.

The shape to look for is an `assert_eq!` inside a `for` over a corpus. It answers
"is there a defect" and reads like it answers "how many". Collect into a map,
compare against an identity-keyed ledger that moves down only, and print the
count every run:

```
Rust/Lean completeness: 225 theorems compared, 73 disagree (ledger holds 73)
```

Both directions of the ledger must be load-bearing, and both must be checked by
breaking them: a name that appears and is not in the ledger fails as a
regression, and a name in the ledger that has started agreeing must be **removed**
or the stale entry fails. Without the second direction, a fixed entry leaves
slack for the next real regression to hide in.

### Removing an early abort exposes what was standing behind it

Two more guards in the same test had been unreachable: `specs/scratch` envs
(untracked since #2283, so absent on any fresh checkout) were counted as
Lean-only witnesses, and a `>= 245` floor could be walked under by any
deliberate skip. Budget for this. Fixing the first defect in a function is how
you find the second.

## 128. Forty of the 73 were theorems about an empty module

`native_decide` proved `Module.isLowerable env module = true` for a `Module` with
no functions, no globals and no tests. True, and true of nothing in the spec it
is named after. A proof over a hand-written model is only as good as the
transcription, and `Completeness.lean` has **250** models with no generator in
the tree — nothing can re-derive them from the specs.

Check the model is non-empty before believing the theorem. It is the proof-layer
form of the vacuous-invariant phase the suite already runs on specs.

## 129. Check whether the note's MECHANISM still exists, not just the verdict

Three stale rows this session, and each described a cause that was no longer
there:

- `stray_closing_brace` demanded `Rejected` because `Rejected` was the only way
  the table could call an input unclean. The parser had since stopped ending the
  file at a stray `}` and started counting it as a discarded token. Fix: give the
  table the vocabulary it lacked (a `discards` field), not the verdict it could
  express.
- Two lexer rows said `#` is "an unrecognised character DISCARDED with no
  diagnostic". The unknown-character path does skip, continue and record — and it
  never sees `#`, which opens a comment to end of line by an explicit, measured
  decision.
- A spec comment claimed "Verified end to end: `on_comb` ... takes [511:0]". The
  emitter does not emit `on_comb` at all.

A row whose *note* is wrong is worse than a row that is merely failing: it sends
the next reader after a mechanism that does not exist.

## 130. Rulers that break: substring counts and pinned shapes

- `v.matches("module ").count() == 1` found two, because the emitter's own
  comment says "this **module** cannot move a value across its boundary". Count
  declarations: a line whose first token is `module`.
- A test pinned `reg [15:0] \buf [0:3];`. The emitter now packs the array into
  one vector. The escaping the test exists to protect was working the whole
  time. Assert the invariant — *no mention of the name appears unescaped,
  anywhere* — which is stricter than the two literals it replaced and survives
  the next change of shape.
- A test pinned `if (!(` for an assert; the emitter writes `if (((x) != (y)))`
  now. Assert what a real check must DO: branch on the expected value **and**
  emit the failure path. A comparison that cannot fail is not a check.
- Two tests asked `gen-verilog` to lower a test block. That backend deliberately
  does not — `gen-verilog-for-simulation` does. A test asking the wrong component
  can only ever fail.

## 131. The suite was green while six specs regressed

A parser fix rewound to re-read a condition when what followed the closing paren
was not a body. It was right for `if`/`while` and wrong for the `if` EXPRESSION,
where `if (c) a else b` is legitimate: `base/ops`, `base/ternary_add`,
`base/types`, `numeric/gf16`, `numeric/gfternary`, `numeric/tf3` all died on
`Unexpected token in expression: KwElse`. **Every test passed.**

The only control that catches this is parsing the whole corpus with the binary
from before the change and the binary after, and diffing **per spec**:

```
558 -> 553   # the broken version, all tests green
558 -> 559   # the fixed version, one spec moved, and it is the one the fix was for
```

An aggregate count would have shown 553 vs 558 and told you nothing about which.
Build the baseline binary from `HEAD~1` — commit your work first, so a checkout
of the previous file cannot take uncommitted edits with it.

### Rebuild the baseline ledger after fixing a regression

The 73-entry ledger was generated while the six-spec regression was live. A
baseline baked from a broken state enshrines your own bug as known debt.
Regenerate and diff the two: here it was 73 either way, no entry added or
removed, so the regression had not polluted it — but that was measured, not
assumed.

## 132. `$?` after a pipeline is the last command's status

Third time this session. `python3 tools/check_seal_coverage.py | tail -12` then
`echo $?` prints `tail`'s zero while the gate exited 1. Redirect to a file, then
read `$?`:

```bash
python3 tools/check_seal_coverage.py > /tmp/seal.out 2>&1
echo $?
```

## 133. Improving a broken thing is not the same as fixing it

A partial fix for #2743 cut iverilog's errors from 7 to 2 and left `layer2`
emitting a call to a function that had been refused — a module that looks
complete and is not. Corpus acceptance was identical before and after: Zig 217,
cc 157, Zig-AND-Verilog 194.

Reverted, and filed with the measurement. A loud refusal beats a plausible
half-lowered artefact, and "no measured improvement" is a reason to stop, not a
reason to ship the diff you already wrote.

## 134. `cargo test` stops at the first failing binary

Every test total in this loop before the last hour — 1629/6, 1635/0, 1786/1 —
came from the binaries that ran before the stop. None was the repository total.

```
cargo test -p t27c                    ->  1786 passed / 1 failed
cargo test -p t27c --no-fail-fast     ->  2419 passed / 5 failed
```

Four of those five failures had never been printed by anything I ran, and they
were the *interesting* ones: four generate Verilog, compile it with iverilog,
**run** it and check numbers against a reference model.

Use `--no-fail-fast` for any number you intend to report. A partial count that
reads like a total is the same defect as an assert inside a loop (§127), one
level up.

## 135. Three refusals, one answer, and the smaller fix was the right one

`gen-verilog` refused a whole function when an array parameter had no call
site, or disagreeing call sites, or a non-identifier argument. Refusing is what
made bitnet_layer print *"function on_comb has array parameter(s) but no call
site"* four lines above `assign result = on_comb(...)`.

I built the elaborate fix first: monomorphise the conflicting function into
`neuronN__acts_w0` / `neuronN__acts_w1`, bind the entry point's parameters to
their own ports, teach the index path that a bound name can be packed. It
worked, and it closed **one** of five failures.

The answer was to **delete** the three refusals. An array parameter that cannot
be bound is passed BY VALUE — a real `input [W-1:0]`, indexed by element slice,
which #1745 already implemented and which every one of those testbenches was
already written against. 18 insertions, 22 deletions, five failures closed, no
test file touched.

When a fix needs three new mechanisms to make one check pass, the model is
wrong. Ask what the callers already assume.

## 136. A gate that only checks half of what it is named for

`check_seal_coverage.py` runs as *"Every seal still describes its spec"* and its
docstring says a seal breaks when *"gen_hashes no longer describe what it
produces"*. It compared `spec_hash` and stopped.

```
spec_hash    := zeros   ->  exit 1, stale
gen_hash_zig := zeros   ->  exit 0, SILENT
```

| | seals |
|---|---|
| the gate called broken | 418 |
| actually not describing their output | **1,078** |
| **only the output drifted — invisible** | **612** |

Two controls, one per field, found it in a minute. Run one control per THING A
GATE CLAIMS, not one per gate: this one passed the single control anyone had
tried.

### Re-seal first, then tighten

All 1,078 were re-sealed from the compiler's current output, so tightening the
gate landed GREEN rather than red-on-arrival. A stricter gate is only landable
at the moment the tree satisfies it — otherwise it is a red gate nobody can
merge past, and it gets reverted or baselined into silence.

### The new check gave the gate a new way to lie

With no seals at all it answered *"the compiler is not built"* instead of *"the
path is wrong"*. `check_gate_preconditions.py` — which hands every gate an empty
tree and asks what it says — caught it one commit later. **Order the diagnoses:
nothing-to-check comes before tool-is-missing.**

## 137. A proof nobody compiles

`proofs/lean4/` has a `lakefile.lean` and 250 theorems. Across **45 workflow
files**: no `lake build`, no `elan`, no mention of `proofs/lean4`. Every
`by native_decide` there is a claim no instrument has checked.

Before trusting any proof layer, grep the workflows for the thing that would
build it. The absence is quick to establish and it reframes everything
downstream — "40 vacuous theorems" is a detail when *none* of the 250 is
checked.

What is holdable without the toolchain is the **shape of the model**: a module
with no functions, globals or tests makes its theorem vacuous, and that is
readable from the source. Ratchet what you can read; file what you cannot run,
with the job written out, rather than landing a gate you could not execute once.

## 138. `-o /dev/null` is not a free way to discard output

`t27c corpus` measured the Rust backend with

    rustc --emit=metadata -A warnings -o /dev/null <file>

and rustc writes metadata through a temp file **next to** the output path. It
therefore tried to create `/dev/rmeta<random>` and died with `couldn't create a
temp dir: Operation not permitted` — on every input, valid or not.

The column read **0 of 559** for as long as it existed, and that zero was
reported as a finding, published in a release note, and quoted to the owner.
Fixing the flag alone: **0 → 144**. Fixing the largest real defect after it (an
unconditional `serde` derive on every generated struct, compiled standalone with
no `--extern`): **144 → 173**.

The tell was available the whole time: the other three backends wrote to real
files and had non-zero numbers. **A column at exactly zero while its siblings
are not is a claim about the instrument, not the subject.** Reproduce it on a
trivially valid input before believing it — twenty-three characters of Rust was
enough.

### The control that made the fix safe

Zig 217 and cc 157 were unchanged across both fixes. A change to the Rust
invocation must move the Rust number and nothing else; when it does, the two
numbers you did not touch are the evidence.

## 139. Dispatch every gate once; the readings are the payload

`workflow_dispatch` was added to twenty workflows in one commit — a manual
trigger causes no runs by itself, it only makes the question askable. Firing all
27 unmeasured ones at the default branch took minutes and returned:

- **12 refused**: their file is not in the tree. 13 of 59 workflows GitHub calls
  *active* are deleted-but-registered ghosts. They can never run, so they can
  never be measured, and they inflate the count of gates someone might fix.
  Separate them in any tool that reports "unmeasured".
- **8 ran and failed.** A Rust pin two editions stale, a workflow that pushes to
  a branch its own ruleset forbids, a missing secret, a failed docker pull, and
  three bare exit codes. Not one was new; every one had been true since June.
- **2 failed correctly** — worth checking before filing. A PR-scoped gate has no
  pull request on a dispatch, and the release pipeline refused an empty tag and
  published nothing.

### NOT APPLICABLE is a third answer

A gate whose subject is absent should say so and exit 0 — not fail, and not
pass silently. `check_now_entry_shape.py` reads the entry a *pull request* adds;
on a dispatch there is no pull request, and failing there is failing at
something outside its subject. Distinguish three states, not two: *checked and
passed*, *checked and failed*, *nothing here to check, and here is why*.

## 140. A gate catching its author is the gate working

Two of this session's own changes were caught by gates added hours earlier:

- The `serde` fix altered generated Rust while touching no spec. `spec_hash` was
  unchanged, so before the `gen-drift` category existed it would have passed
  without a word. It caught 668 seals.
- Dispatching the release pipeline by accident hit the preflight that refuses an
  empty tag — every publishing job skipped, no registry written.

When you add a check, the first thing it catches will probably be yours. That is
the strongest evidence it works, and it is worth saying out loud in the commit
rather than quietly re-sealing.

## 141. Read the verdict, not the list underneath it

`corpus-ratchet` went red on `master` and stayed red for twelve runs and a full
day. The cause was my own commit: it fixed a spec's parse and left that spec's
entry in the expectation ledger. The gate said so on every single run —

    UNEXPECTED PASSES  : 1
      - specs/ar/asp_solver.t27 [parse] (fixed -- remove from the ledger)

— and I had run the suite that day, seen `ACCEPTABLE: no`, and gone to read the
list of ninety-seven parse failures instead of the three lines above it.

The failure list is long and looks like work. The verdict is one line and IS the
work. When a gate prints both, read the verdict first, and treat a red gate as
unfinished business belonging to whoever last touched the thing it names —
usually you.

### The ratchet was enforcing what I had spent the day preaching

An entry that starts passing must be REMOVED, or the slack is where the next
regression hides. I wrote that into three other gates in the same session and
broke it in a fourth without noticing. Being the author of a discipline is not
the same as following it.

### Slack accumulates silently

`max_entries` had drifted to 219 against 179 entries — forty slots — so the
"raising the cap is a reviewable event" arm had been inert since it decoupled.
A cap is only a cap at zero slack. Set it to `len(entries)` in the same commit
that blesses the ledger, and have the blesser write it rather than leaving the
old value.

## 142. A file rustc cannot PARSE has no error code, so no histogram sees it

`gen-rust` emitted `Vec<const u8>` for the Zig slice spelling `[]const u8` --
the `const` there qualifies the pointee and Rust has no such qualifier. 84 of
559 generated files contained one.

They were invisible to every first-error census, because those group by error
CODE and a file that fails to parse never reaches one: 278 of 386 failures had
no code at all. Fixing it moved rustc acceptance **173 → 214**.

When ranking causes by first error, count the files with NO diagnosable error
separately and look at them first. They are not a long tail; they are a
different failure mode wearing the same column.

## 143. Distrust a small sample when the population is generated

Checking the `<const ` claim, I generated Rust for 120 specs, found ONE, and
nearly dismissed a finding that was true of 84 of 559. Generated corpora cluster
by construct, not uniformly: whether a spec has a slice-typed field depends on
what it models, and the first 120 by path happened to be light on them.

Sample the whole population, or sample randomly. `git ls-files | head -120` is
neither.

## 144. A number that goes DOWN when you fix a silent drop was flattered before

Two parser fixes recovered 1,049 tokens of invariant bodies that were being
discarded at the top level. The acceptance columns then fell:

    Zig accepts it   217 -> 215
    cc accepts it    157 -> 156

That is not a regression to be undone. Those specs were accepted while their
assertions were silently vanishing — the backend compiled less code than the
spec contains, and called it a pass. Recovering the content revealed that the
recovered content does not compile.

**When a fix removes a silent drop and a quality number falls, the number was
measuring the drop.** Say so in the commit and keep going; reverting to restore
the pretty figure is the actual regression.

## 145. Bisect to a file you can hold in your head

`specs/vsa/sdk.t27` discarded 682 tokens across 86 lines. Reading it taught me
nothing. `parse-complete --bisect` named the top-level item worth removing, and
copying THAT item into a nine-line file gave a reproduction I could vary one
token at a time:

    invariant has_comment          ->  0 discarded without the comment line
        // a comment                   11 discarded with it
        const a = f(1);

The second cause needed the same treatment and a further narrowing: `then
<expr>` was clean, `then for (...) { }` was not, so the trigger is `then` plus a
*statement*, not `then` at all.

Do not debug in the 600-line spec. Bisect, extract, shrink until each variable
can be flipped alone.

### And use the corpus's shape, not your invention

My first minimal case wrote `then assert x == y;` and failed to reproduce,
because `assert` is itself a clause keyword — I had invented a shape the corpus
does not contain and drawn a conclusion from it. Copy the real lines first;
invent only after the real ones reproduce.

## 146. Never delete lines by matching a debug word

Removing my probes with "delete every line containing PROBE" also deleted a
pre-existing `format!` argument line for a variable named `probe_idx`, breaking
the build in a place I was not working. Save the file before instrumenting and
restore it after, or revert and re-apply the intended edit — a text filter over
a 38,000-line file will find your word somewhere you did not put it.
