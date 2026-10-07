# NOW -- `tri mutate census --dir D --lab` is one lab run (2026-10-07)

## cli/tri/src/mutate.rs (Closes #7471)

- Slice 2 of 2. Slice 1 (`2026-10-07-mutate-lab-files.md`) let a lab run carry any number of files. Now `mutate census --dir D --lab <args>` parses as `mutate spec --file D --lab <args>`, and `lab_request` reads a directory as a census:
  - every `.t27` directly in `D` goes up, sorted by name, each at `D/<name>`;
  - on the lab, `mutate census --dir D` runs where `mutate spec --file F` ran;
  - the lab's own `.t27` files in `D` are removed before the upload, so a spec deleted here is not judged there;
  - a directory with no `.t27` is exit 1 here, before anything reaches the lab, as `census_of(0, _)` is 1 in `specs/tri/mutate/census.t27`;
  - `--fn NAME` must name a function in one of the files.
- The lab job reads `mutate spec --help` and `mutate {cmd} --help` before it starts. A lab `tri` with no `--dir` exits 1 and names `T27C_LAB_TRI`; it never exits with the gate's 2.
- `git diff --numstat`: 40 lines added, 10 removed. That is the #7371 limit of 40 per file, so slice 2 is its own PR.
- New test `a_lab_directory_runs_a_census`, through the `T27C_LAB_LOCAL=1` fixture:
  - with a fixture `tri` whose help has `--dir`: exit 0, the fixture is called once as `mutate census --dir specs/y ...`, and the stale `specs/y/old.t27` is gone before it runs;
  - with one whose help has no `--dir`: exit 1, `has no --dir`.
- Results, on the Railway lab:
  - `cargo build --release -p tri` exits 0, with no warning from `mutate.rs`;
  - `cargo test --release -p tri mutate::`: 58 passed, 0 failed.
- End to end, this branch's `tri` on the lab's copy of master 05e633d03, with the master `t27c`, `--jobs 8 --zig-threads 6 --timeout 60`:

  | Run | Exit | Output |
  |---|---|---|
  | `mutate census --dir specs/tri/mutate`, local | 0 | `3 spec(s), exit 0: 153 of 159 killed, 96.2% of judged; 4 survived, 2 hung, 0 unviable.` in 72 s |
  | the same with `--lab`, through `T27C_LAB_LOCAL=1` | 0 | the same `census:` lines and mutant lists, in 77 s; three more lines name the launch pid, the specs' commit and the tool's build |
  | `--lab` with `T27C_LAB_TRI` set to a `tri` built before census | 1 | `... has no --dir (built 2026-10-07T08:56:08Z); set T27C_LAB_TRI to a newer build` |
  | `--lab` with `T27C_LAB_RAILWAY=/nonexistent/railway` | 3 | `cannot reach the lab ...` |
  | `--lab` on a directory with no `.t27` | 1 | `no .t27 file in specs/empty7471; a census of none is exit 1` |

  Afterwards the runs' directory was empty. The ps listing at the script's end held one `zig test --test-no-exec` compile, with a live parent rather than PID 1. Three minutes later both were gone, so it was not an orphan.
- Negative controls, one edit each to `mutate.rs`. A grep count of the edited line showed 1 before and 1 after, so each edit applied. The file was restored by copy and checked by sha256.

  | Control | Fails |
  |---|---|
  | the stale-file removal off (`if run.census {` -> `if false {`) | `a_lab_directory_runs_a_census` only (57 passed, 1 failed) |
  | the lab runs `mutate spec --file` for a directory | `a_lab_directory_runs_a_census` only (57 passed, 1 failed) |

- Not covered by a unit test: the job's help check for `mutate census --help`. The fixture `tri` gives one help text for every subcommand, so a job that reads only `mutate spec --help` passes the fixture. The end-to-end run covers it: this branch's `mutate spec --help` holds no `--dir` (grep count 0) and its `mutate census --help` does (3), so such a job would have made the `--lab` census exit 1, not 0.
