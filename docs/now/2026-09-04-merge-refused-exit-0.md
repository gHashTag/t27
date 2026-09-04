# NOW -- Merge refused, exit 0 (2026-09-04)

## Merge refused, exit 0 (Closes #3109)

- tri pr ready --merge printed 'Merge refused: the head branch is not up to date' and returned 0 -- observed on three pull requests in one batch
- The worse half was beside it: gh pr merge succeeding while the content is not on the branch prints 'Do NOT report this as merged' and also returned 0
- A fifth code, 4 NOT MERGED, decided by a pure merge_outcome(ran_ok, on_branch); only a landing the API confirms is 0
- Two mutations: always returning 0 kills two tests, ignoring the API confirmation kills one
