# NOW — twenty surviving mutants down to two (2026-08-23)

Nine agents, one gate each, in separate worktrees. Every patch then went to a fresh agent whose brief was to **refute** it, default verdict "does not work". All nine survived refutation.

- **Survivor sites 20 → 2; gates with every site killed 4 → 12 of 13.** The two that remain are exactly the ones `tools/check_gate_preconditions.py` names as `UNCOVERED` — and because that note names them by **message rather than by line**, it still matched after the agents' additions moved them from `:359/:403` to `:451/:495`. The convention paid for itself inside a day.

- **The refuters found two things the patches did not.** Both are worth more than the patches.

- **`tri gates mutate` cleared no caches.** Python keys a `.pyc` on (source mtime in whole seconds, source size). `return 1` → `return 0` preserves the size, and the loop writes mutant / restore / next mutant well inside one second — so an **imported** gate can be served the previous state's bytecode. `tools/wp18_selftest_gate.py` does `import wp18_conformance_gate as G`, and that `.pyc` is on disk. The sibling command `tri mutate` had already solved this in `cli/tri/src/mutate.rs`, and `gates.rs` did not call it. Found by a reviewer who went looking in the neighbouring module rather than in the file under review. Cleared now in three places; `wp18` reports 6/6 deterministically across three runs.

- **`check_seal_coverage`'s four end-to-end cases never built the configuration the live gate is in every day** — a ledger present *and* something outside it newly broken. Proven to matter rather than assumed: with `new = [b for b in bad if not known]` planted, so that a ledger existing hides every new breakage, the four original cases all pass and only the fifth reds. That mutant would make the gate useless for its actual daily job, and nothing would have said so.

- **Applied from the refutations.** `_plant` renamed to `_self_check_plant` in two gates so the tool can never mutate the instrument by name rather than by the accident that the helper currently holds no `return 1..4`; a 300 s timeout on a spawned subprocess; `"  fpga_beta.json"` added to a `forbid` list, proven to matter by planting a refusal that names every file instead of only the unreadable one; `json.loads` guarded so garbage stdout records `FAIL` instead of aborting the control with a traceback.

- **Not done, and filed rather than left to be inferred from a green** (#2472): a whole-process case for wp18's drift verdict, which is the verdict CI depends on; `check_specs_parse`'s ratchet comparison exercised only at `was == 0`; two paths written into `check_seal_coverage` as uncovered; and the `json.loads` guard, which is correct by construction and **has not been seen to fire** — which by this repository's own rule is not the same as working.

- **Two of my own edits broke the files they were meant to fix.** A `split()` on a repeated name truncated one file by a hundred lines, and a regex put `timeout=` inside `str()`. Both were caught by running the gates rather than by reading the diff, and both were redone by hand. String surgery on a file you have not read is not a shortcut.

- **And a git lesson from losing a fix and getting it back.** `git stash` is repository-global, not worktree-local: with nine agent worktrees on this repo, a `stash`/`pop` pair popped an agent's stash instead of mine. Separately, a `git commit -m tmp` wrapping a demonstration swept real work in with the throwaway, so the `git reset --hard` that cleaned up the demonstration destroyed the fix too. Commit the work first, plant the demonstration second, and never stash in a shared repository.

Refs #2468, #2472
