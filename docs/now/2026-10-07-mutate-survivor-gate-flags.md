# NOW -- `tri mutate spec --fail-on-survived` and `--accepted FILE`, as specs/tri/mutate/survivors.t27 states them (2026-10-07)

## cli/tri/src/mutate.rs (Refs #7303)

- Slice 2 of #7303. Before it, `tri mutate spec` exited 0 when mutants survived. A script could gate on survivors only by grepping the text `SURVIVED`. The rule came first, in t27 (`docs/now/2026-10-07-mutate-survivor-gate-contract.md`). This is the Rust.
- Two flags:
  - `--fail-on-survived`: exit 2 when a mutant survives or hangs.
  - `--accepted FILE`: lists the mutants the caller has judged equivalent, and turns the gate on by itself. Each line is `path:line [kind]`, as the tool prints a mutant under SURVIVED or HUNG, so a line is copied, not retyped. Text after `]` is a note, a `#` line is a comment, and blank lines are skipped. Any other line is an error that names the line number, and the run stops before the baseline.
- Without either flag the exit code is unchanged: 0 with survivors, and 1 for a red baseline or mutants not run.
- With the gate on, the run prints one summary line, the mutants the file does not name (`NOT ACCEPTED`), and the file's lines whose mutant ran and was killed (`ACCEPTED BUT KILLED`). It then exits 2, or 0 with `Gate passed.`. A listed line whose mutant did not run in this run (another `--fn`, a line that moved) is counted nowhere, as the spec says. An unviable mutant is counted nowhere too.
- The five gate functions in the Rust copy the spec's five. The test `the_gate_agrees_with_every_assert_row_of_its_spec` reads `specs/tri/mutate/survivors.t27` when it runs, checks the three exit constants, and evaluates each of its 38 assert rows (10 tests) against the Rust copy. It fails if it read fewer rows than the file has, so a row changed in the spec fails the Rust test until the copy follows.
- Results, on the Railway lab with `specs/tri/mutate/lab.t27` (50 of 54 killed, 4 survived, 0 hung, 0 unviable):

  | Run | Exit | What it printed |
  |---|---|---|
  | no flag | 0 | the 4 survivors, as before |
  | `--fail-on-survived` | 2 | 4 NOT ACCEPTED |
  | `--accepted` naming all 4 (with a comment, a note, a blank line, a `./` path) | 0 | `Gate passed.` |
  | `--accepted` naming 2 of them | 2 | 2 NOT ACCEPTED |
  | the 4 plus a killed `lab.t27:83 [drop-guard]` | 2 | 1 ACCEPTED BUT KILLED |
  | a file with the line `lab.t27 line 104` | 1 | `accepted file line 2: ... is not path:line [kind]` |
  | `--fail-on-survived` on `specs/tri/mutate/survivors.t27` itself | 0 | 20 of 20 killed, `Gate passed.` |

- `cargo test --release -p tri mutate::`: the 3 new tests and the 35 already there pass, 38 in all.
- Negative controls, each run with the Rust test:
  - Deleting the `accepted_killed` guard from the Rust `survivor_exit` fails `the_gate_agrees_with_every_assert_row_of_its_spec`.
  - Changing the spec row `unaccepted(4, 3) == 1` to `== 2` fails the same test.
- The whole `cargo test --release -p tri` on the lab fails 2 tests: `fpga::tests::test_smoke_gate_json_synthetic_verify_lean` and `test_smoke_gate_json_theorem_matrix_is_computed`. They fail the same way on the base commit without this change: the lab has no `yosys`, and `cli-tri.yml` installs it before these tests for that reason.
