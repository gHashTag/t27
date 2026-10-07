# NOW -- the rules of `tri mutate spec --lab`, written in t27 before any Rust (2026-10-07)

## specs/tri/mutate/lab.t27 (Refs #7050)

- Slice 1 of #7050. `tri mutate spec --lab` will start a mutation run on the Railway lab and read it back over `railway ssh`. This spec states what the command decides. The Rust does not exist yet, and the header says so. Slice 2 adds the flag, plus a test in `cli/tri/src/mutate.rs` that asserts each scenario here row by row.
- The run is a nohup'd job in its own directory. A pid file, an exit file and the tool's output are its whole state, so a dropped ssh does not lose the run.
- 8 functions:
  - `run_state`: NONE, RUNNING, DONE or LOST, read from the directory. An exit file wins over a runner that is still closing.
  - `launch_allowed`: only where there is no run.
  - `may_remove`: only DONE or LOST, and only with no orphan left. A test binary still running in a deleted directory keeps spinning (trap T9).
  - `lab_exit`: 0 for OK, 1 when the tool failed, 2 for no result, 3 for orphans, 4 while still running. Orphans outrank a failed tool.
  - `ssh_should_retry`: a read retries, up to 3 tries in all, and only after a transient failure. A write is never resent blindly; whether the launch landed is read back through `run_state`.
  - `poll_wait_seconds`: 5, 10, 20, 40, then 60.
  - `zig_j`: max(1, nproc / jobs).
  - `lab_jobs`: the request, cut to the pids that fit after a reserve of 104. Zero means do not start.
- The reserve and `zig_j` come from a probe on the lab: 8 parallel `zig test` compiles of `specs/queen/actors.t27`, two runs at each setting. The lab has 48 cores, pids.max 1000, and pids.current was 713 before the probe. All 8 compiles passed in every run.

  | zig setting | pids added | time per run |
  |---|---|---|
  | default thread count | 104 and 101 | 4396 and 4341 ms |
  | `-j6` | 20 and 20 | 4371 and 4486 ms |
  | `-j1` | 10 and 10 | 8918 and 8706 ms |

  `-j6` is 48 / 8. It matched the default's time with about a fifth of the pids.
- Results:
  - 18 tests and 4 invariants.
  - `zig test` on the `gen` output passes all 18, and `t27c test-report` counts 0 vacuous passes of 18.
  - parse, typecheck, gen-c, gen-rust and gen-verilog exit 0. `cc -fsyntax-only` on the gen-c output gives 0 errors, the gen-rust output compiles with `rustc --test`, and `iverilog -g2012` accepts the gen-verilog output.
  - Negative control for the invariants: with `PID_RESERVE` set to 1000, `zig test` fails at compile time.
- Mutation testing:
  - `tri mutate spec` on the lab, whole file: 48 of 52 mutants killed, all by a failing test. The 4 survivors are equivalent:
    - `wait < POLL_CAP_SECONDS` becoming `<=`: 5 * 2^k never equals 60.
    - `wait > POLL_CAP_SECONDS` becoming `>=`: the function returns the cap either way.
    - In `lab_jobs`, the guard's `>=` becoming `>`, and `fit < requested` becoming `<=`: at equality the function returns the same value either way.
  - 38 mutants made by hand cover each constant, each returned code, the arithmetic and each dropped guard. All 38 are killed. The unmutated copy passes in the same harness.
  - One gap was found and closed. A first form of `lab_jobs` had no test with a free count strictly inside the reserve, so dropping its reserve guard survived. A new assert, `lab_jobs(8, 1000, 950, 1) == 0`, kills it: the guard's absence underflows. The two guards were then merged into one.
