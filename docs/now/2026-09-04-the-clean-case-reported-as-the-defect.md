# NOW -- The clean case, reported as the duplicate case (2026-09-04)

## `tri skill check` exits 1, and always has (Refs #2994)

- I wrote that the checker detects a duplicate section number, prints `PROBLEMS`, **and exits 0**. Measured today by planting a duplicate: it prints `PROBLEMS` and exits **1**. `skillnum.rs` has carried `std::process::exit(1)` since #2789
- the error was mine and it came with a number attached: **I read the exit code of a run made after I had already removed the duplicate**. The clean case, reported as the duplicate case
- **when an exit code is the finding, the run that produced it must be the run that contained the defect.** The cheapest proof is to plant the defect deliberately and watch it fail
- the other half was true: `grep -rn "skill check"` across `.github/workflows/`, `scripts/` and `tools/` returned **nothing**. A checker that exits 1 correctly and is called by nobody fails as silently as one that exits 0
- a neighbouring session wired it into `cli-tri.yml` (#3165) after checking the half I got wrong. Two failures, not three, and the surviving one was enough on its own
