# NOW -- tri mutate loads the survivor gate and the lab rules from t27 (2026-10-07)

## cli/tri/src/mutate.rs, gen/rust/tri/mutate/ (Closes #7472)

- `mutate.rs` held two hand-written Rust copies of rules that live in t27: the survivor gate (`gate_on`, `not_killed`, `unaccepted`, `survivor_exit`, `mutate_exit`, from `specs/tri/mutate/survivors.t27`) and `mod lab` (from `specs/tri/mutate/lab.t27`). Both are gone. In their place are two `#[path]` modules of `t27c gen-rust` output, `gen/rust/tri/mutate/survivors.rs` and `gen/rust/tri/mutate/lab.rs`, loaded the way `census.rs` is (#7433). The call sites did not change: `lab::` names the generated module, and the gate's names come in through one `use`.
- `git diff --numstat` on `mutate.rs`: 20 lines added, 198 removed.
- `specs/tri/mutate/lab.t27`: `var j` (`zig_j`) and `var fit` (`lab_jobs`) are never assigned again, so they are `const` now. `gen-rust` lowered them as `let mut`, which `rustc` warns about (`unused_mut`). `parse`, `typecheck`, `gen-rust`, `gen-verilog` and `gen-c` exit 0 on both specs (lab, master `t27c` of 05e633d03); `test-report`: 0 vacuous of 21 (lab) and of 10 (survivors); the generated files are byte-equal to `t27c gen-rust` of each spec (sha256).
- Changed from the issue's plan: the issue said the specs' own tests "compile into the generated module". They do not. The generator writes `NOT LOWERED BY THIS BACKEND: 10 test(s)` (survivors) and `21 test(s)` (lab) at the top of each file. So the two agreement tests stay: `the_gate_agrees_with_every_assert_row_of_its_spec` and `the_lab_rules_agree_with_every_assert_row_of_their_spec` now run every `assert` row of each spec against the generated Rust. Before, they compared the spec with a hand copy. Now they check that `gen-rust` lowers the bodies as the spec's own tests expect, and that is the only place the generated Rust runs a spec's tests.
- Results, on the Railway lab:
  - `cargo build --release -p tri` exits 0, with no warning from `mutate.rs` or `gen/rust/`. `cargo test --release -p tri mutate::`: 56 passed, 0 failed.
  - Same exit and same stdout, byte for byte (`cmp`), from the #7475 head's `tri` and this one. Both runs used `mutate spec --file priority.t27 --accepted F` on a copy of `specs/queen/priority.t27` (`--jobs 8 --timeout 60 --zig-threads 6`). With every accepted line, both exit 0. With the `:72 [flip-cmp]` line removed, both exit 2. `mutate census --dir` on that copy prints the same `census:` lines from both: exit 0, `27 of 31 killed, 87.0%`.
- Negative controls, one edit to a generated file each, restored by copy and checked by sha256 afterwards:

  | Control | Fails |
  |---|---|
  | `survivors.rs`: `accepted_killed > 0` -> `> 1` | `the_gate_agrees...`: `test an_accepted_line_now_killed_fails` |
  | `lab.rs`: `orphans > 0` -> `> 1`, both lines that hold it (`may_remove` and `lab_exit`) | `the_lab_rules_agree...`: `test a_failed_survivor_gate_passes_through_as_2` |

- `TOOL_RC_SURVIVED` was `super::EXIT_SURVIVED as u32` in the hand copy. In `lab.t27` it is the literal 2, so the lab test's last assert keeps it tied to the gate's own 2 in `survivors.t27`.
- `tri mutate census --dir specs/tri/mutate --jobs 8 --timeout 60 --zig-threads 6` with this branch's `tri`, on the lab: exit 0, 3 specs, 153 of 159 killed, 96.2% of judged; 4 survived, 2 hung, 0 unviable.
  - `survivors.t27`: 20 of 20 killed.
  - `census.t27`: 81 of 83 killed; the 2 hung mutants drop the step of a scan loop (`:101 end = end + 1`, `:108 k = k + 1`), so the loop never ends.
  - `lab.t27`: 52 of 56 killed. The 4 survivors are equivalent, each a flip at a boundary where both sides return the same value:
    - `:109 wait < POLL_CAP_SECONDS` -> `<=`: at `wait == CAP` the mutant doubles once more and `:113` cuts it back to CAP;
    - `:113 wait > POLL_CAP_SECONDS` -> `>=`: at `wait == CAP` both return CAP;
    - `:138 pids_used + PID_RESERVE >= pids_max` -> `>`: at equality `fit` is 0, so the mutant returns 0 too;
    - `:140 fit < requested` -> `<=`: at `fit == requested` both return the same number.
