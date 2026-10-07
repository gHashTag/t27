# NOW -- the rules of `tri mutate spec --lab` copied into Rust and checked against every row of their spec (2026-10-07)

## cli/tri/src/mutate.rs (Refs #7050)

- Slice 2 of #7050. Slice 1 (`docs/now/2026-10-07-mutate-lab-contract.md`) wrote the rules of a lab run in t27, in `specs/tri/mutate/lab.t27`. This slice copies them into Rust before any `railway ssh` code exists. The `--lab` flag and the plumbing that calls the copy are the next slice.
- `mod lab` in `cli/tri/src/mutate.rs` holds the spec's 15 constants and its 8 functions: `run_state`, `launch_allowed`, `may_remove`, `lab_exit`, `ssh_should_retry`, `poll_wait_seconds`, `zig_j` and `lab_jobs`. `TOOL_RC_SURVIVED` is the survivor gate's own `EXIT_SURVIVED`, not a second literal 2. Until the plumbing lands only the tests call the module, so it carries `#[cfg_attr(not(test), allow(dead_code))]`.
- The test `the_lab_rules_agree_with_every_assert_row_of_their_spec` reads `specs/tri/mutate/lab.t27` when it runs. It evaluates all 73 assert rows of the spec's 19 tests against the copy. It also checks that the spec's constants are exactly the copy's: same names, same values, none missing, none extra.
- The row reader that `the_gate_agrees_with_every_assert_row_of_its_spec` used for `specs/tri/mutate/survivors.t27` (#7303) is now one helper, `assert_every_row_of`. Each test passes it the spec's path, the Rust constants and the Rust functions. The survivors test now checks its constant set exactly too, where before it only checked that the three exit codes were there.
- Results, on the Railway lab:
  - `cargo test --release -p tri mutate::`: 39 passed, 0 failed. That is the 38 from #7303 plus the new test.
  - The build gives 0 warnings in `mutate.rs`.
- `lab.t27` after its header edit, on the Mac with the release `t27c`: parse, typecheck, gen-c, gen-rust and gen-verilog exit 0, and `zig test` on the `gen` output passes all 19 tests. `t27c test-report` counts 0 vacuous passes of 19.
- Negative controls on the lab, each with `cargo test --release -p tri mutate::tests::the_`:

  | Control | Result |
  |---|---|
  | none | both agreement tests pass |
  | delete the `orphans > 0` guard from the Rust `may_remove` | the lab test fails at `orphans_are_killed_before_the_directory_goes`: `may_remove(s, 1)` returned true |
  | change the spec row `zig_j(48, 7) == 6` to `== 7` | the lab test fails at that row: the copy returns 6 |
  | change the spec's `PID_RESERVE` from 104 to 105 | the lab test fails on the constant check, before any row |
  | change the survivors row `unaccepted(4, 3) == 1` to `== 2` | the survivors test fails at `one_unlisted_survivor_fails` |

  The three files were restored after the controls, and their sha256 matched the copies in this commit.
