# NOW -- The push gate refused a deletion, and the barrier was never priced (2026-09-05)

## The push gate refused a deletion (Refs #3308)

- Three gates were added to the barrier today and none of them was priced. Pricing them is
  what found the defect.
- `.githooks/pre-push` did not read stdin, so it could not tell a push from a deletion.
  `git push origin --delete <branch>` was refused with `SYNC REQUIRED`, and the branch
  survived on the remote. A deletion adds no entry and never will.
- An EMPTY range was refused too. The CI script answers `SYNC REQUIRED` on base == head,
  so the local gate was faithful -- but that case cannot arise in CI, where a pull request
  with no commits cannot be opened, and locally it arises constantly.
- Both now skip, each saying why. A non-empty range with no entry still refuses.
- The cost, best of three: the whole barrier is **329 ms**. `census pin --gate` 144,
  `fix-carries-source` 58, `now-gate` 45, `pre-push` 153, `commit-msg` 26.
- And the answer I did not expect: the conflict checker in its OLD whole-tree form costs
  **876 ms** by itself, against **93 ms** for the `--staged` form that replaced it. The
  barrier did not get more expensive today. It got roughly three times cheaper, and that
  one change more than pays for everything added.
- The lesson is not the number. It is that I introduced a cost, asserted nothing about it
  for a whole pass, and the measurement that settled it also found a defect no control had.
