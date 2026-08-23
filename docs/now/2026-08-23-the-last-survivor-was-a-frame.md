# NOW — the last survivor was a frame, not a limit (2026-08-23)

The suite's final surviving mutant is dead, and the reason it lived is the interesting part.

- `check_elab_ratchet.py`'s "no baseline" branch was declared `UNCOVERED` because *"a smoke build that succeeds needs the real spec tree — not something an empty directory can be given."*
- Every clause of that is true. The conclusion does not follow: a control does not have to use an empty directory. The note reasoned inside the frame of `run_on_empty_tree()`, the helper the file is built around.
- `REAL_CORPUS` keeps the real corpus and empties only the thing under test — `cwd=ROOT` so the smoke build succeeds, `T27_ELAB_ROOT` a planted tree with a built compiler, an empty `generated/` and no baseline. Reached in under a second; the real tree is only read, `git status` clean before and after.
- Two negative controls for the new runner (VACUOUS and WRONG), a real 1.2 MB corpus planted in both end-to-end trees, and an `UNRUN` verdict when no corpus is present — a stage that cannot run proved nothing.

**90 mutants across 13 gates — 36 silent, 21 loud, 33 invert — all killed.** `UNCOVERED` 1 → 0.

What that does not say: that the gates are correct, or that their checks are the right checks. It says no mutant in these three narrow families survives its control. A fourth operator is a fourth question, and the prior after this campaign is that a new question finds something — twice it found the instrument.
