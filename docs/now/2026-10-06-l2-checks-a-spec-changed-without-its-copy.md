# NOW -- L2 fails a spec changed without its tracked copy; 65 stale tracked copies dropped (2026-10-06)

## tools/l2_regen_check.py, gen/ (Refs #6971; Closes #7103)

- The gap. L2 compared only the gen/ files a PR modifies. A PR that changed a spec and left its tracked copy behind passed, so tracked copies drifted from t27c output without a red check. #7091 did it to `gen/c/queen/priority.c` and `review_valve.c` until 5b338c74f.
- Now a tracked gen/ file is also checked when its spec, or any spec in its `use` closure, is added, modified or deleted in the PR. gen-c splices what a spec imports, so a changed import changes the copy: `DENY` 1 -> 9 in `specs/policy/own_language.t27` changes `gen/c/ci/affected.c`. A copy whose spec is gone fails with "no spec". `--list` prints these copies, so the workflow builds t27c for them; the workflow file is unchanged.
- Copies of specs a PR did not touch are not charged to it: a gen-c change re-stales every copy, and that is the compiler lane's regeneration.
- `--all` checks every tracked gen/ file.
- Controls, old script vs new, in a throwaway worktree:
  - a spec changed, no copy regenerated: old passes, new fails 2 STALE COPY (the spec's own copy and an importer's);
  - only the importer's copy left behind: new fails 1;
  - both regenerated: new passes;
  - a spec deleted, its copy kept: new fails "no spec".
- 65 tracked copies were not t27c output and nothing reads them: 31 gen/c, the same 31 gen/verilog, `gen/rust/memory/notebooklm.rs`, and `vsa/core.c` + `vsa/core.v`, whose spec does not exist. They are deleted, not regenerated: gen/ is in `.gitignore`, 29 of the 32 stale C copies did not compile before and 29 of 32 regenerated ones do not compile now (#5711, #5712), and a copy nobody reads goes stale again at the next gen-c change.
- Kept: 14 tracked copies. `--all` prints 12 of 14 as t27c output; the 2 that are not are `gen/c/numeric/gf16.c` and `gen/verilog/numeric/gf16.v`, left to #6996 item 5.
- Foreign Python: an edit to an existing tool, with an entry in `tools/policy/foreign-exceptions.txt` and the label owner-approved-foreign. The debt is moving the L2 rule into a spec, as `specs/policy/own_language.t27` did for the Only-t27 gate.
