# NOW -- `tri mutate spec --lab`: start a mutation run on the Railway lab and read it back (2026-10-07)

## cli/tri/src/mutate.rs, specs/tri/mutate/lab.t27 (Refs #7050)

- Slice 3 of #7050. Slice 1 (`docs/now/2026-10-07-mutate-lab-contract.md`) wrote the rules of a lab run in t27, and slice 2 (`docs/now/2026-10-07-mutate-lab-rules-in-rust.md`) copied them into `mod lab`. This slice adds the flag and the `railway ssh` plumbing that calls the copy, so `mod lab` no longer needs its `allow(dead_code)`.
- `tri mutate spec --file F [--fn NAME] --lab [--lab-wait SECS]`:
  - Uploads the spec (as base64, 64000 characters per ssh call) into a staging file on the lab, then makes the run directory in one write. Its name is `tri-mutate-` plus 16 hex digits of a sha256 over the request (spec bytes, path, `--fn`, `--max`, `--jobs`, `--timeout`, the gate flags, the accepted file, and the lab's `tri`, `t27c` and source paths). The same command names the same directory.
  - The run copies the lab's `specs/` (for the spec's `use` lines), writes the uploaded spec over its own path, and starts `tri mutate spec` under `nohup setsid`. A pid file, an exit file and the tool's output are its whole state, as slice 1 says.
  - Before it starts, `run.sh` looks every flag it passes up in the lab tri's `--help`. A tri that is too old to know one reads as a failed tool (1), not as the survivor gate's 2. A clap usage error also exits 2 (#7334).
  - It then polls with `poll_wait_seconds` until the exit file appears or `--lab-wait` (default 3600 s) runs out. If time runs out, it exits 5 and the run keeps going; the same command again reads it back and does not start a second one.
  - It prints the tool's output, lists any process still running in the run's directory (trap T9), and removes the directory only when `may_remove` allows. The exit code is `lab_exit`'s.
- Jobs and zig's threads:
  - `--jobs` defaults to 8 with `--lab`. The probe reads pids.max and pids.current from the lab's cgroup, and the lab's `nproc`.
  - `tri mutate spec` now passes `-jN` to every mutant's `zig test`, local or on the lab. N is `zig_j` (the cores shared by the jobs), or the new `--zig-threads N` (1 to 4096). Before this, `zig_j` was in the spec and in `mod lab` but nothing called it.
  - `lab_jobs` cuts the request to the pids the lab has free. Its per-job cost is the spec's new `job_pids(zig_threads) = zig_threads + JOB_OVERHEAD_PIDS`, where `JOB_OVERHEAD_PIDS = 3`. A cut batch keeps the request's -j, so its per-job cost does not change.
  - `PID_RESERVE` goes from 104 to 408: one batch of 8 jobs at zig's default thread count on 48 cores, `8 * (48 + 3)`. A new invariant pins that sum, in place of slice 1's `PIDS_PER_JOB * 8 == PID_RESERVE`.
- Where the lab is: `T27C_LAB_RAILWAY`, `T27C_LAB_PROJECT`, `T27C_LAB_ENV` and `T27C_LAB_SERVICE` choose the `railway ssh` target. `T27C_LAB_TRI`, `T27C_LAB_BIN` (t27c), `T27C_LAB_SRC`, `T27C_LAB_ZIG` and `T27C_LAB_RUNS` are paths on the lab. `T27C_LAB_LOCAL=1` runs the same scripts through `sh -c` on this machine, which is how the tests and the run below drive it on the lab itself.
- Retries follow `ssh_should_retry`: a read is tried up to 3 times, and a write is not resent. If the launch's answer is lost, the next probe reads whether it landed.
- `--t27c` is refused with `--lab` (set `T27C_LAB_BIN`), and `--lab-wait` is refused without it. The path must be relative to the repo root and ASCII, `--fn` must name a function of the spec, and the accepted file must parse, all checked before any ssh.
- Runs go to `/tmp` on the lab by default, not `/data`: `/data` had 1.4G free (97% used) when this was written, and `/tmp` 594G.
- The compile command moved into `zig_compile`, so a test can read its arguments: `each_compile_gets_its_zig_threads` checks the `-j6`.
- `lab.t27` now has 21 tests, 6 invariants, 16 constants and 9 functions; the Rust agreement test evaluates its 78 assert rows.
- The pid cost, measured again on the lab on 2026-10-07, after its redeploy with tini as PID 1 (48 cores, pids.max 1000). `tri mutate spec --jobs 8` over this spec's 54 mutants of the time, pids.current sampled every 5 ms, timed without `--lab` (its poll waits round the time to 5, 10 or 20 s):

  | zig -j | pids added | time |
  |---|---|---|
  | zig's default | 394 and 394 | 4.94 and 5.15 s |
  | 12 | 110 | 5.06 s |
  | 6 | 58 | 4.74 s |
  | 3 | 34 | 5.50 s |
  | 1 | 18 | 8.11 s |

  Every run killed the same 50 of 54. One job alone peaked at 9 tasks with -j6 and at 51 with -j48, which gives `JOB_OVERHEAD_PIDS = 3`. Slice 1's table (104, 20 and 10 pids) read pids.current too seldom and missed the peaks; zig starts a thread per -j, and a thread is a pid. Before the redeploy the lab's PID 1 reaped nothing (#7090): 766 zombies held pids at 07:38Z, and slice 1's probe started with 713 of 1000 in use.
- Results, on the Railway lab:
  - `cargo test --release -p tri mutate::`: 55 passed, 0 failed, 0 warnings in `mutate.rs`.
  - `T27C_LAB_LOCAL=1 tri mutate spec --file specs/queen/actors.t27 --fn start_answer --lab --lab-wait 0` exits 5 ("Starting it: 8 job(s) of 8 asked, zig -j6 each, on 48 cores"). The same command without `--lab-wait`, 20 s later, reads the run back: `4 of 4 killed`, exit 0, and the directory is removed.
  - `tri mutate spec --file specs/tri/mutate/lab.t27 --fail-on-survived --lab`: 52 of 56 killed, all by a failing test, and exit 2. The 4 survivors are the 4 equivalent ones slice 1 lists. The run added 60 pids at its peak, with 0 fork failures.
  - After both runs: 0 `tri-mutate-` directories in `/tmp`, 0 `zig test` processes, 0 zombies.
  - On the Mac with the release `t27c`, `lab.t27` passes parse, typecheck, gen-c, gen-rust and gen-verilog, and `zig test` on the `gen` output passes all 21 tests. `t27c test-report` counts 0 vacuous passes of 21. With `JOB_OVERHEAD_PIDS` set to 4, or `PID_RESERVE` to 409, `zig test` fails at compile time.
- Negative controls on the lab, one `cargo test` each:

  | Control | Fails |
  |---|---|
  | drop the `-j` argument from `zig_compile` | `each_compile_gets_its_zig_threads` |
  | Rust `job_pids` without `JOB_OVERHEAD_PIDS` | the agreement test, at `job_pids(zig_j(48, 8)) == 9` |
  | the spec's `JOB_OVERHEAD_PIDS` 3 -> 4 | the agreement test, on the constant check |
  | `--lab` prices a job at `job_pids(1)`, ignoring its -j | `the_labs_free_pids_cut_the_jobs` |
  | `run.sh` drops `--zig-threads` | `a_lab_run_reads_back_and_removes_its_directory` |

  Both files were restored after the controls, and their sha256 matched before and after.
