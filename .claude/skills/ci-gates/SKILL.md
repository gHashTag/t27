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
