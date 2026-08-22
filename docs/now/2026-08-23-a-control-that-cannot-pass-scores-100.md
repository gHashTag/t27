# NOW — a control that cannot pass scores 100% (2026-08-23)

Fourth defect in `tri gates mutate` in three days, found by an adversarial reviewer checking the command's own output rather than the code it audits. It is the worst of the four.

- **The command took no baseline.** A mutant is scored *killed* when the control exits non-zero. A control that is red **before** any mutation is therefore red after every one of them, and the command printed a perfect score against it.

- **Reproduced, one variable changed**, with a `return 1` planted at the top of `check_json_parses.py`'s `self_check`:

  ```
  old tri:  check_json_parses.py    1/1  all killed
  new tri:  check_json_parses.py      -  CONTROL ALREADY RED -- scored nothing
  ```

- **It is the exact inverse of the defect the command was written to find.** There, a control that could not FAIL scored everything as covered. Here, a control that cannot PASS does the same. Both replace a measurement with a constant, and both print a number that reads like evidence. Every `all killed` this command has printed carried an unstated precondition — that the control was green to begin with — and nothing checked it.

- **The gate is named in the survivor list, not silently credited.** "Nothing was measured here" is a finding. A row that quietly vanished from the report would repeat the mistake one level up.

- **What I could not reproduce, said plainly.** The reviewer reached this through `check_withdrawn_live.py` with a one-row registry, where a control refuses to run a case and exits 1. That branch arrives with that agent's own patch, which has not landed. The defect in the tool is real and reproduced independently, as above; the specific trigger they described is not yet in the repository.

- **Two operational lessons from losing the fix and getting it back.** `git stash` is repository-global, not worktree-local: with nine agents working in worktrees of this repo, a `stash`/`pop` pair popped an agent's stash instead of mine and my edit vanished. And a `git commit -m tmp` wrapping a demonstration swept the real work in with the throwaway, so the `git reset --hard` that cleaned up the demonstration destroyed the fix too. Commit the work first, plant the demonstration second, and never `stash` in a shared repository.

Refs #2468
