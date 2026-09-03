# NOW -- The hooks invented an issue number (2026-09-04)

## 1048 of 1294 branches got a number they do not carry

- `grep -oE '(issue-|#)?[0-9]+'` -- the prefix OPTIONAL -- turned any digits in a
  branch name into an issue number. Measured over every branch, local and
  remote: **1294 examined, 1048 given a number, 0 branches actually using the
  `issue-N` or `#N` form the comment documents**, 140 using `wNN-`.
- `w42-status-ruler`, `w42-tri-vsim`, `w42-verilog-break`, `w42-vsim-unknown`
  all answer **#42**, which is a wave number. The documented population is
  empty, and the optional prefix turned that emptiness into 1048 confident
  wrong answers.
- Both `.githooks/post-merge` and `.githooks/pre-commit` carry the line; the
  pre-commit one has run on every commit this session.
- Nothing visibly broke because the number feeds a `sync.py` call that cannot
  run: three call sites hard-code `python3.10`, absent on this host while
  `python3` is 3.14.3.
- Each of those failed into an `|| echo` blaming CONFIGURATION, and the output
  contradicted itself: `Could not update metadata` and `Metadata updated`
  printed one line apart, followed by `Post-merge complete`.
- Now: the prefix is required, the interpreter is resolved once and named when
  absent, the metadata lines are exclusive, and a run that skipped the sync
  finishes with `ℹ️` rather than `✅`. The hook still exits 0 -- a post-merge
  hook must not fail a merge that already happened.
- Three mutants, three kills: restoring the optional prefix in either hook, or
  breaking the matcher entirely, each fails three assertions. The parser is
  extracted from the hooks rather than restated, and the extraction is asserted.
