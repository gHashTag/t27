# NOW — the note said what to build (2026-08-24)

`check_gate_preconditions.py` carried a note naming two branches it did not cover, and the reason: *"Covering them needs a stage that requires iverilog and skips loudly without it; that is a bigger change than this file is, and it is filed rather than faked."*

That sentence was the design. Building it took one stage.

- **Surviving mutants across all thirteen gates: 2 → 1.** `check_elab_ratchet.py` goes 8/10 to 9/10; the branch that reds when `t27c fpga-build --smoke` fails is now covered.

- **The stage needs a tool that cannot be planted.** `iverilog` lives on PATH, not in the tree. So it reports **UNRUN** when absent rather than passing or guessing — the same choice the `WITH_T27C` rows already make, for the same reason: a row that cannot run proved nothing, and reporting it as absent would be the vacuous pass this file exists to catch, one level up.

  ```
  with iverilog:      the gate reaches "t27c fpga-build --smoke failed", exit 1
  without iverilog:   UNRUN  check_elab_ratchet.py [t27c+iverilog] needs iverilog on PATH
  branch neutered:    VACUOUS check_elab_ratchet.py [t27c+iverilog] exits 0 with nothing to check
  ```

  All three measured, not reasoned.

- **The CI job that runs this installs iverilog** — `fpga-conformance`, verified by locating the job's line range rather than grepping near the step. My first check was a substring test over a slice that did not span the job, and it answered "no". The right answer is yes.

- **The one that stays uncovered, and why it is not a stage away.** `"no baseline; run --update-baseline once"` needs `t27c fpga-build --smoke` to *succeed* and then find no baseline. A smoke build that succeeds needs the real spec tree, which an empty directory cannot be given. **Planting a fake success would test the plant, not the gate.** `UNCOVERED` is now 1, and the note says this rather than leaving a reader to infer completeness from a number.

- **A note that names what it does not cover is a work item.** This one specified the mechanism, the tool it needs, and the failure mode to avoid, then sat unread for a day because it was written as an admission rather than as a task. The admission was the more useful half — it survived other people's edits moving its line numbers, because it names branches by message.

Refs #2472
