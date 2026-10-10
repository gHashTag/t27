# NOW -- the t27b ledger steward: master's run writes the ledger rows, spec PRs stop touching them (2026-10-10)

## specs/tri/t27b/ledger_steward.t27, specs/jobs/t27b_ledger_steward.t27 (Closes #8654; see #8637 item 1, #8321 option 3)

- **The rules.** `ledger_steward.t27` decides what the steward does with each ratchet finding on a master run. It applies UNLISTED (a non-pass row gets reason `unimplemented`), UNEXPECTED PASS (a gain) and STALE (the row is dropped). The cap may rise by at most the new non-pass rows (steward.t27 `cap_rise_is_new`).
  - It refuses UNEXPECTED FAILURE, MOVED, BAD REASON, OVER CAP, a defect where the reference passes, a cap rise larger than the new rows, and more than 16 STALE rows in one run. A refused run gets one issue and no PR.
  - The tests use the three hand batches as vectors. #8289 and #8646 are applied, with caps 18 -> 20 and 25 -> 26. #8548 is refused for its two failures; without them it is applied, with cap 23 -> 25.
- **The job card.** `t27b_ledger_steward.t27` is a jobs.t27 card with eight steps:
  - CHECK the run is a counted master run on master's ledger;
  - EFFECT the refusal issue;
  - CHECK every finding is appliable;
  - EFFECT the ledger issue, the composed ledger, the PR, auto-merge;
  - WAIT merged.
- **Measured on the t27c lab:**
  - `t27c test-report`: 10 tests with 0 vacuous, and 2 with 0 vacuous;
  - `t27b corpus --blockers --reference` under qemu: both pass, with 108 and 26 runtime asserts and 0 reference disagreements.
- **Runtime.** The Queen's job runner (gHashTag/trios `queen-jobs.ts`) runs only the release card today. The missing loader, trigger, executors, journal kinds and credential are listed in gHashTag/trios#1751. Until it runs, spec PRs keep adding their own rows.
