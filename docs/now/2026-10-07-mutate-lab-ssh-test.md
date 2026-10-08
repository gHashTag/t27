# NOW -- a test for the `railway ssh` call of `tri mutate spec --lab` (2026-10-07)

## cli/tri/src/mutate.rs (Closes #7390, Refs #7050)

- `tri mutate spec --lab` (#7366, `docs/now/2026-10-07-mutate-lab-flag.md`) reaches the lab through `sh -c` when `T27C_LAB_LOCAL=1`, and through `railway ssh -p P -e ENV -s SVC <script>` otherwise. Every test drove the `sh -c` branch. The only test with `local: false` pointed at a missing `railway` and checked the error, so the real call's argument list had no test.
- The #7050 comment and the #7366 body said the `railway ssh` branch was covered by the scripted tests. It was not. Both texts now say so.
- New test `a_lab_run_goes_through_railway_ssh`. It gives the run a stand-in `railway` script that appends its first 7 arguments to a file and runs its last argument through `sh -c`. The whole run goes through it: probe, upload, launch, read-back and removal. The test checks:
  - exit 0 and the tool's output (`4 of 4 killed`) came back;
  - at least 3 calls were recorded, and every one reads `ssh -p p1 -e production -s t27c-lab`;
  - the tool started once, and the run directory was removed.
- Results, on the Railway lab, with `cli/tri/src/mutate.rs` at the same sha256 as this commit's:
  - `cargo test --release -p tri mutate::`: 56 passed, 0 failed (55 before), 0 warnings in `mutate.rs`.
  - Negative controls, one `cargo test` each, file sha256 the same before and after:

    | Control | Fails |
    |---|---|
    | drop `c.arg("ssh")` from `lab_call_once` | `a_lab_run_goes_through_railway_ssh`, at the argument check |
    | drop the `-p PROJECT` pair | `a_lab_run_goes_through_railway_ssh`, at the argument check |

- Still not done: no run has gone through the real `railway` CLI from a Mac, because tri is not built on the Mac. That stays open on #7050.
