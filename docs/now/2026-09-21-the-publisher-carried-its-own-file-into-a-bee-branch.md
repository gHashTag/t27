# NOW -- The publisher carried its own file into a bee's branch (2026-09-21)

## Commits now happen in a worktree of their own (Closes #4499)

- #4338 passed every required check and still could not merge: `CONFLICT (add/add): Merge conflict in tools/queen/publish.py`. The issue's boundary named one spec. The publisher had run from a checkout that carried its own copy of `publish.py`, `git checkout -B` switched branches **in place**, and the file travelled into the bee's branch. Taken back out; the pull request is armed.
- `publish.py` now cuts a fresh worktree from the bee's own branch, writes the coordination entry there, commits and pushes from there, and removes it. A worktree holds exactly what its branch holds. Verified on the next real publish, #4498: its diff is the bee's one file and the entry, and the checkout the publisher ran from was untouched.
- **And that very publish found the next trap.** The bee on #4489 had been asked to write `.github/workflows/wasm-explorer.yml`, and a new workflow moves two ledgers no bee can know about - the census and the gate-topology classification - each of which has turned master red before (#4303, #4319). Auto-merge on #4498 is disarmed with that explained, and the publisher now refuses such a branch and says why, rather than publishing it into a guaranteed red.
