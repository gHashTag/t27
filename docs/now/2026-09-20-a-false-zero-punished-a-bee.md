# NOW -- A false zero punished a bee for a generator defect (2026-09-20)

## `grep -c` prints zero for an empty input, and that became a criterion (Closes #4467)

- Issue #4446 went out carrying `t27c test-report specs/fpga/linker.t27 2>&1 | grep -c BLOCKED` prints `0` - **this spec compiles today and must still compile (today: 0)**. It has never compiled. Its generated Zig writes `.align = 4`, and `align` is a reserved word in Zig, so the file fails with `expected field initializer` at `spec.zig:26:84`.
- The bee added a correct four-line test, met **four criteria of five**, and was sent back for a defect it did not cause and could not fix inside its boundary. Checked by running every criterion against the bee's own branch: 1 ok, 2 ok, 3 ok, 4 ok, 5 fails - and 5 fails identically on master.
- The cause is the same shape as this morning's: `grep -c` prints `0` for an empty input, so a `t27c` that did not run at all produces the same `0` as a spec that compiles. The feeder now requires the report to SAY it ran - the words `test report:` - before believing its count.
- Repaired in the open backlog, with the distinction that matters: **5 issues** whose spec exists and does not compile lost the criterion (it asks for what the boundary cannot deliver), and **49 port issues** whose spec does not exist YET kept it - a file a bee is about to create has no excuse for not compiling. The first pass got that wrong and removed all 54; the correction reads the filesystem rather than the exit code.
- The generator defect is filed on its own: a struct field named `align` is emitted unescaped.
