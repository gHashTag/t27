---
id: analog-emulation
name: analog-emulation
description: Drive epic #10486, t27 analog emulation v0.1 with Symbiotic EDA -- an analog netlist with IHP SG13G2 models becomes one .t27 spec that is the reference model and the FPGA design, run per corner and judged by ngspice and Symbiotic EDA's evaluator. Use when working a milestone (M1-M6), when updating the status dashboard, or when writing a progress report to the Symbiotic EDA contact on X.
argument-hint: "[milestone, e.g. M1] | report | dashboard"
---

# /analog-emulation -- netlist to t27 spec to FPGA, per corner

<!-- only-t27-rule -->
> **Only t27: everything is written in `.t27`.** (Owner hard rule 2026-10-05; budget #7371; restated in every skill per #7588.)
> - New logic starts as a `.t27` spec with `test` / `invariant` blocks. It reaches Rust, C, Zig, JS or Python only through `t27c gen-*`, and generated files are never hand-edited (L2).
> - Hand-written code in any other language is denied by the hooks and by CI: `.rs .py .ts .js .sh .zig .c .go .v .yml .toml`, Dockerfile, Makefile; the full list is in `specs/policy/own_language.t27`.
> - Existing foreign code is debt that only shrinks. Delete it, or replace it with a spec and its generated output.
> - Thin hand-written glue is capped at 40 added lines per file and 80 per PR. No label lifts the cap. Never split a PR to slip under it.
> - What gen cannot express yet is a compiler defect. File it on #5980 with a minimal `.t27` repro instead of writing the code by hand.
> - This rule overrides any step below that says to write a script or code in another language. The full text is in `AGENTS.md`, "Only t27".

## Where the truth lives

| What | Where |
|---|---|
| Plan, milestones, acceptance, needs | epic gHashTag/t27#10486 (edit the epic, not a copy) |
| Current milestone | its sub-issue, opened when the milestone starts (M1 is #10487) |
| M0, the pattern every cell follows | `specs/fpga/rc_filter_ax7203.t27`, #10276, PR #10278 |
| Status dashboard | https://claude.ai/artifact/75G2ZjZ5CihcWPNLgFDxwD (private artifact; the owner shares it) |
| External references | mirekez/cpphdl `analog/` (examples, IHP setup in `analog/pdk`), ckdur/asicone_sg13g2, regymm/pcie_7x |
| The conversation | the owner's X chat with the Symbiotic EDA contact; context in the owner's memory, never in this repo |

## The milestone loop

Each milestone follows M0's shape. Do not mark one done until all five hold:

1. **Spec.** One `.t27` module that is both the reference and the hardware. The header comment states the circuit, the step, the number format, the run, the receipt line, *what a PASS says* and *what it does not say*.
2. **Tests on the lab.** Error bound against a float64 run of the same method, named in the spec. 0 vacuous. ngspice + IHP runs (M3 on) happen on the Railway lab, never on the Mac.
3. **Negative control.** A mutant (truncation for rounding, a mis-stamped element, the wrong corner's table) that fails the tests, is refused by `t27c seal`, and on the board prints the predicted FAIL line.
4. **Board receipt.** Bitstream from `t27b verilog` (no hand HDL, no XDC), run on the AX7203 bench (`board-eye`, `bench-agent` skills). Record LUT, FF, DSP, BRAM, Fmax and the UART line.
5. **Oracle.** From M3 on, ngspice with the named PDK revision and corner. From M5 on, Symbiotic EDA's self-checking evaluator decides.

Then: close the sub-issue with `Closes #N`, tick it in the epic, open the next milestone's sub-issue in the same Goal / Boundary / Scenarios / Requirements / Success Criteria shape (the issue-shape bot closes an issue without `## Success Criteria`), update the dashboard, and draft a report.

### Technical choices that are already decided

- **Fixed step, compile-time matrices, delta form.** Linear MNA `(G + C/dt) x' = (C/dt) x + B u` folds to `x' = x + rnd(D x + N u)` with `D = M - I` and `N` quantized to Q.24, the sum accumulated exactly and rounded once, half away from zero. M0 is the 1x1 case (`D = -798915`, `N = 798915`), so M0 is reproduced bit for bit by construction. No solve and no division in fabric.
- **Stability is proved:** `||M_q^256||_inf < 1` in the tests (bounds the spectral radius); `||M_q||_inf < 1` alone fails for an underdamped RLC.
- **Unsigned in hardware.** `t27b verilog` refuses signed types, so the step keeps a sign flag per term with unsigned magnitudes, as M0 does. f64 is fine inside tests.
- **Nonlinear devices as per-corner tables first.** `Id(Vgs, Vds)` from the PDK's PSP103 in ngspice, one table per corner (tt ff ss fs sf), one bitstream per corner. Compact-model (Verilog-A) ingest is post-v0.1 research; never promise it for v0.1.
- **Designers keep their netlist; the reader is a t27o front end.** The SPICE subset reader is `specs/tools/t27o/spice_convert.t27` beside `rust_convert.t27`, run as `tri t27o <deck>`, so the project has one converter, not two (DRY). Unsupported input is refused with the line and the reason, never approximated silently. Real IHP decks need `.subckt`, instance parameters (`w`, `l`, `ng`) and `.lib` corner selection early, not late.

## Toolchain for an analog milestone

| Step | Command | Note |
|---|---|---|
| syntax | `./target/release/t27c check <spec>` or MCP `t27_check` | parse + typecheck only |
| inner loop | `./target/release/t27b test <spec>` | native arm64 JIT, compiles in milliseconds; exit 4 = JIT and interpreter disagree, 2 = unsupported construct |
| unchanged specs | `./target/release/t27c frontier --only <specs> --list` | ~2 s; REUSE = seal, closure and toolchain still match and the sealed test verdict stands -- do not re-run its tests |
| tests (final gate) | `./target/release/t27c test-report <spec>` and `tri test <spec>` | per-test assert counts; `vacuous passes` must be 0; `tri test` fails on vacuous passes since #10507 |
| negative control | copy to `specs/analog/` (a `use` does not resolve from `$TMPDIR`), break one constant, `test-report`, delete the copy | must FAIL before the PR |
| seal | `t27c seal <spec> --save`, then `--verify` | commit `.trinity/seals/<dir>_<Module>.json`; the seal is also the cache key for reuse |
| board top | `t27b verilog <top spec>` | pin attributes in the spec, no XDC; no signed types; slice parameters and array indexing in the clocked path exit 2 (tables become a ROM) |
| board run | local openXC7 flow, see below | `t27c fpga-build/fpga-flash` and `t27c silicon` assume the Wukong fbg676 (#10498) |
| bitstream reuse | `tri hot caches`, `tri hot prebuild <spec> --top <v>` | seal-keyed bitstream cache; check it targets fbg484 before trusting it for the AX7203 |
| ngspice reference | `ngspice -b deck.cir` (ngspice-47, Homebrew) | fixed-step backward Euler on our grid: `.options method=gear maxord=1 trtol=1e7`, drive with a B source (PWL corners restart the step ramp), circuit at rest before the source switches on; matches the f64 recurrence to 1e-15 V (#10505) |
| converter | `tri t27o <file> [--check]` | Rust only today; SPICE front end is M2a (#10509) |
| who else | `tri topic analog spice <keyword>` | before opening an issue or a lane |
| PR verdict | `tri pr ready <N> --required-only` | one VERDICT line |

**The board flow is local; the Railway lab is not needed.** `O="$(git rev-parse --git-common-dir)/../build/fpga/openxc7"` (the main checkout's `build/`, from any worktree) holds `np-main/build/nextpnr-xilinx`, the AX7203 chipdb `xc7a200tfbg484-2.bin`, `prjxray/utils/fasm2frames.py` (run with `$O/venv/bin/python3`) and `px-main/build/tools/xc7frames2bit`; yosys and openFPGALoader come from Homebrew. nextpnr on this Mac takes about 22 s and 2.4 GB: check `memory_pressure` (>= 25 % free) and `df -h /` (>= 8 GB) first and never run two at once. Read the memory notes before declaring a blocker: the recipe was there.

Sandboxed sessions: `export ZIG_GLOBAL_CACHE_DIR=$TMPDIR/zig-cache ZIG_LOCAL_CACHE_DIR=$TMPDIR/zig-local`; commit with `git -c core.hooksPath=.githooks commit`; `gh` fails TLS, use curl with `$(gh auth token)`. The t27 trainer is t27t: `specs/t27t/t27t.t27` (`tri t27t`); its runner is gHashTag/igla-coder-gpu.

## The overnight loop tick

A `/loop` tick on this epic follows this order, so a new tick never undoes or duplicates the last one:

1. **Read state.** The ledger is a comment on #10486 (the one headed "Loop ledger"). Read it, `CronList` (delete duplicate crons after a restart), and list running subagents. A lane the ledger says is running is not started again.
2. **Look for anomalies before new work.** Open PRs of this epic that are red, conflicting or stale (`tri pr ready`); a subagent that reported done but whose PR does not exist (`git ls-remote`); a merged PR whose spec is missing on master (`tri pr landed`); a seal that no longer verifies (`tri seals drift`); a ledger row older than 45 minutes still "running". Fix or file each one first (self-repair beats new work).
3. **Sub-tasks start in a free T27 OS pane** (`specs/claw/model_route.t27` TIER_PANE, owner 2026-10-20, #11317; how: skills `model-routing` and `t27os-orchestra`): a pane on the free key tier, a Sonnet subagent after 2 failures, Opus after 4 or on the owner's word; the orchestrator stays on Sonnet. Until #11316 lands a pane cannot run unattended (no permission mode on its command line), so a lane falls back to a Sonnet subagent. 3 to 5 lanes (panes or subagents) on the 8 GB Mac while `memory_pressure` stays above 25 % free; each lane in its own worktree and branch; the orchestrator reads every diff before auto-merge is enabled. Fast review: check out the PR head, `t27c frontier --only <its specs> --list` and `t27c seal --verify`; re-run `test-report` only when frontier says rebuild or the diff looks wrong; regenerate any `gen/` file the PR touches and compare it byte for byte; run `python3 -I tools/dupe_scan.py`. To change a PR that has auto-merge on, disable auto-merge first, push, then enable it again (a second push races the merge).
4. **Merge only green.** Auto-merge after the required checks pass. A red is inherited only if it is a subset of master's failures; `claw-sarif` failing on "permissions requested are not granted to this installation" is the known App-permission red (owner steps in `docs/claw/README.md`).
5. **Write state.** Edit the ledger row, tick the epic's Progress table, republish the dashboard, and add a learning to this skill when a trap cost more than one try. Renew the keep-awake hold at the start of each tick (it lapses after about 5 idle minutes). Then run `/self-cleanup` (`.claude/skills/self-cleanup/SKILL.md`): remove the worktrees and Zig caches of lanes whose PRs merged, and put the freed space in the row.
6. **Requests out, never questions.** Owner, 2026-10-19: "do not ask, request; everything not forbidden is allowed". A blocked item is not parked as "waiting for the owner": name who holds what it needs, send that party a request with the options, a default and a time, and continue on the default. End every tick with `Requests out:` -- who, what, sent when, default, due. An unanswered request is chased once, then the default is taken and written in the ledger row. The owner is asked only for a password, a sign-in, a payment, a legal commitment or a choice between two businesses (rule: `CLAUDE.md` "Proactivity: request, do not ask").
7. **Reports** to the partner: the owner gave standing permission on 2026-10-18 to message the Symbiotic EDA contact directly about anything the work needs and at every milestone. Read the chat for answers first in every tick.

## Claim guard

Every number that leaves the repo -- in a report, the dashboard, a post or a partner summary -- comes from a test or a receipt in gHashTag/t27. Before sending, check each sentence against the receipt:

- Never claim power (nothing measures it), accuracy outside measured circuits, real-time or "faster than SPICE" unless measured on the same circuit, or PCIe throughput before it runs.
- A speed claim needs `may_claim_speedup` in `specs/analog/bench.t27` to say yes: a measured board time including readout, a measured bitstream build time (so the break-even run count exists), and a wall ratio >= 1. Today it says no for every circuit; ngspice wall time is mostly process start-up and the board's is mostly the UART line.
- Report three errors separately: arithmetic (fixed point vs f64 of the same scheme), discretization (scheme vs analytic or dt/2), model (vs ngspice with the PDK).
- Say "table model of PSP103", not "the IHP transistor model", for M3-M4.
- If a previous message overstated something, the next report corrects it explicitly.

## Progress reports to the Symbiotic EDA contact

The owner wants a report on every breakthrough and every milestone. Since 2026-10-18 the owner's standing word is to message the contact directly about anything the work needs; keep every message within the claim guard.

- **When:** a milestone closes; a board receipt or oracle result lands for the first time; a blocker appears that needs something from them; a claim made earlier turns out wrong.
- **Language and form:** English, plain, written. Lead with what was measured, then what it does not show, then the next step and what is needed from them. Link the PR or issue. No more than about 12 lines.
- **Template:**
  ```
  <Milestone>: <one-line result>.

  What we measured:
  - <number> (<source: test / receipt>)
  - ...
  Not covered yet: <boundary>.
  Next: <next milestone> by <date>. From you we would need: <item, if any>.
  <link to PR or issue>
  ```
- **Privacy:** the chat is private. Nothing from it (names, budgets, proposals, payments, accounts) goes into the public repo, issues or PRs. The epic says "Symbiotic EDA" and nothing more personal.
- **Sending on X:** in BrowserOS neo, open the chat in your own tab. `act type` and `insertText` drop characters in X's composer; use ONE `act fill` with the whole text (newlines included), compare the textarea's value with the intended text, then press the Send button by its ref. Read the chat first: X keeps message text in a shadow root and a virtualised list, so read `[role=log]` through the shadow roots and scroll it with mouse-wheel events.
- After sending, add a line to the dashboard's report log and republish it.

## Updating the dashboard

The page is a single HTML file whose data sits in four arrays at the top of its script: `MS` (milestones, status `done|active|planned|blocked`), `PATH_STATE` (the seven stages of the signal path), `NEEDS`, and `LOG`, plus `UPDATED`. Read the live artifact, change only those values, and republish to the same URL. A status changes only when the epic or a receipt says so.

## Traps

- `gh` fails TLS inside the Claude sandbox (x509 OSStatus -26276); `curl` with `$(gh auth token)` against api.github.com works.
- The asicone sub-blocks (`ro_reliability`, `saradc_auto`) are submodules on gitlab.symbioticeda.com; a shallow clone of asicone_sg13g2 leaves them empty. Ask for the netlist instead of guessing it.
- `t27b` bitstream specs are skipped by the fpga-build job and have no gen-verilog rows (#10300); follow M0's ledger handling.
- A PR that removes a dead `tri` mention must lower `MAX_DEAD_TRI` in `tools/check_documented_commands_exist.py` in the same commit (an equality ratchet; #10500 forgot, #10514 fixed master).
- `t27b-native-ratchet` red with only UNLISTED rows for new specs is expected: spec PRs do not edit the ledger; the steward lists them from master runs (#8656).
- uberWAVE reads ngspice RAW (binary or ASCII). Its `wp2-parser/golden/top.spice` (SAR ADC, xschem, IHP SG13G2) is GPL-2.0: test against it at run time, never vendor it.
- A module-level one-element array literal `[798915]` lowers to a Zig array type (#10512); write `[798915] ** 1` until it is fixed.
- Seal only with `ZIG_GLOBAL_CACHE_DIR` set in a sandboxed session: otherwise the seal's test run cannot create the zig cache and the seal records a false "blocked" verdict plus a home path, which turns `scan` red (#10521). Never write a home path into a committed file; derive it from `git rev-parse`.
- 2026-10-19 Closing a Terminal tab kills the script running in it (a remote-job poller and a ledger patch were lost this way); read the tab or wait for the script's marker file before stop_terminal_tab with close. Remote containers started with docker run -d survive and can be fetched again. (#11063)
- 2026-10-19 In zsh an unquoted variable holding several words is ONE argument; pass a list through 'sh -c' or use ${=var} (masterchecks.sh was handed eight shas as one and every row was empty). (#11063)
- 2026-10-19 VACASK: the PyPI wheel vacask-bin (0.3.4.dev3) reads main-branch syntax in a container with python3 and no install rights; the 0.3.4 release tarball needs libopenblas; the IHP PDK converter comes from a main-branch clone (12 s, no hand edits); the hb analysis of a demo newer than the wheel stops with an unknown parameter. (#10911)
- 2026-10-19 VACASK decks: a top-level var is a syntax error (use it in the control block); integer sweep values divide as integers (write the divisor as a real); uic mode starts an unlisted node at 0 V even if a source drives it; a module_path_prefix in the toml replaces the PDK osdi directory, so list both; one failing sweep point aborts the whole sweep. (#10911)
- 2026-10-19 Comparing two simulators: take crossings from the clock edge on, not from ngspice meas (it misses the first-step jump to about 1.2 V that both show, and prints 6 digits); ngspice print times are the adaptive grid, so take the last point inside the interval. (#10911)
- 2026-10-19 t27b can pass while t27c test-report (Zig) panics, here on a 39-bit shift of a u64 value: run test-report as well as t27b before landing; t27b does not take f64 as u64, and a failing assert_eq on an f64 prints its raw bit pattern. (#10881)
- 2026-10-19 The test runner folds an assert on constant array words into no runtime assert (the test reads VACUOUS): read test data through variables; ngspice cross-plot references (dc1.l0n[k]) work only while the plots are named dc1, dc2, so the deck must say so. (#10881)
- 2026-10-24 Before claiming a spec task, list open PRs that touch the same files (gh pr list --state open --json number,files) and read the peer sessions. Tick 5 (2026-10-24) designed a corner register that PR 12031 already did differently and had to withdraw it. Measure on the committed spec instead: only BOARD_CORNER changes per corner.
- 2026-10-24 Board capture: the UART reader starts before the load, so the first lines are the previous bitstream's tail and one cut line. Judge from the first well-formed N=000 L=0000 line of the new design, with grep -a after tr -d CR and NUL. Check the previous corner's pinned S is absent from the counted lines.
- 2026-10-24 Build B fs corner: seed 2 gave 96.42 MHz and seed 4 93.55 (FAIL at 100), seed 3 gave 104.18. A re-route reuses the same yosys json, so a seed rerun is nextpnr, fasm2frames, xc7frames2bit only. ss and ff pass at the default seed, sf at seed 2.
- 2026-10-24 A bare seal command runs no tests and exits 0 on a faulty spec; only seal --save (test-gated, same report as test-report) and seal --verify (hash compare) refuse. Plant ONE fault at a time: specs import analog::linear, so all-at-once inflates FAIL counts and can break compile. Proof: docs/analog/seal-refusal-2026-10-25.md. (#12065)
- 2026-10-24 On this Mac the openXC7 tools are not under the doc's O dir: yosys /opt/homebrew/bin, nextpnr-xilinx and xc7frames2bit in ~/.local/bin, chipdb ~/openxc7-src/chipdb/xc7a200tfbg484-2.bin, prjxray and venv in ~/openxc7-src, db root ~/openxc7-src/prjxray-db-full/artix7. (#12050)
- 2026-10-24 A +-1 step on a 4-5 digit load constant of sl_arc (integer ps, floor) is below the ps resolution: tri mutate plant SURVIVES and no honest test kills it. Probe weighted edge sums first (CK rises weights 1..11, 21 calibration chains); if equal, document an equivalent mutant (seal-refusal note section 7), never copy the constant into an assert.
- 2026-10-24 tri mutate plant in a lane with no target/release/t27c fails with 'cannot start t27c test-report': pass --t27c <master-built t27c> (the repair-v7 checkout). One plant at a time; the tool restores the file and prints the hash before and after.
- 2026-10-24 2026-10-25 gen_hash_rust MISMATCH on 14 of 30 analog specs while spec_hash and the zig/verilog/c hashes MATCH is a stale t27c, not drift: #12042 changed the Rust emitter, #12062 re-saved seals with it. Before filing, run git merge-base --is-ancestor 55040898f9 <binary source commit>, or use a t27c from the master tip (warm rebuild takes 42 s in a shell cell).
- 2026-10-24 2026-10-25 the issue-shape gate closes an issue as NOT_PLANNED within 2 s unless Success Criteria holds a number with a unit (ms s mb gb % file(s) line(s) test(s) attempt(s)), a backticked file ending swift ts rs md json zig v, 'exit 0' / 'exits non-zero', or a backticked make/swift/bun/cargo/git command. A .t27 file and tri or t27c commands do not count.
- 2026-10-24 2026-10-25 a seal re-saved by a stale t27c is green locally and red in CI (spec-guards and coverage: gen-drift, gen_hash_rust): PR 12068 carried built_by t27c-bootstrap@0.5.2+63040712e. Re-save only with the t27c of the current master tip, and read built_by in the JSON diff before pushing: it must name master's short sha.
- 2026-10-24 Tick 9: a compiler older than the last Rust emitter change prints ONLY gen_hash_rust MISMATCH (spec/zig/verilog/c MATCH) = stale compiler, not drift; tri coverage now ends such a row STALE-T27C? (PR 12072, measured on a real stale binary in 12073). A tracked gen/rust twin must be regenerated (gen-rust prints to stdout) or seal verify says DRIFT. coverage.t27 probe_of arg 2 = REPORT exit.
- 2026-10-24 Tick 10: tri loop claim must run INSIDE a git checkout (from /tmp it exits 2: could not build the claim commit). set_auto_merge refuses a PR already CLEAN: merge it with gh pr merge --squash. To measure a tri change end to end, rebuild tri in a shell cell (38 s warm), run it with the stale and the master compiler, quote the rows.
- 2026-10-24 tri mutate spec: --timeout 30 is enough (300 is slow); 169 mutants take ~7 min at --jobs 3. --accepted FILE: one 'path:line [kind] note' covers every mutant of that kind on the line; exit 2 on an unaccepted survivor AND on an accepted line now killed. Append new tests at the END of the spec: the file keys mutants by line, a test inserted above un-accepts them.
- 2026-10-24 Seal only with the master-built t27c: T27C=<master t27c> tri seals changed (no flags; step 1 exits 1 when the spec changed, then save, recheck, importers pass). Classify a survivor before a witness: flip-cmp on an equality branch that assigns the same value is equivalent; a branch tied to a terminal (PMOS bulk = its high end) is dead.
- 2026-10-24 A loop bound off by one step is invisible when the extra step moves the state but no RETURNED field: probe with exact ==/!= before building a witness. t27 tests have no print: localize a failing assert by splitting the test or threshold probe tests. swap-arith changes the LAST operator of an expression; ret-default of a void fn is unviable.
- 2026-10-24 Accepted-mutant notes: write "equivalent" only when the arithmetic shows both branches give the same result. A flip-cmp that differs only at an exact f64 equality no test builds is "not constructed", not "equivalent in practice" (a review bot flagged it on #12085; fixed in #12103). A state change no returned field shows is "unobservable".
