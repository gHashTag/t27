# NOW — the second mutant (2026-08-23)

Yesterday's entry caught one escaped mutant. **There were two.**

- `check_specs_generate.py` carried `return 1` → `return 0` — a silent mutant, in a commit, in the open pull request, for two iterations. It survived the cleanup because I recovered **the file named in the PR diff** instead of checking the directory.
- The command's own recovery instruction is `git checkout tools/` — the whole directory. I quoted it while doing something narrower, then asserted the tree was clean on the strength of one file matching.
- **What found it was an anomaly, not vigilance:** the background run showed *two* files dirty at once. One mutated file is the loop working; two is either a bug or residue. Without that I would not have looked — the PR diff had stopped mentioning it.

**The recovery that works is a directory comparison, both directions:** `git diff origin/master HEAD --stat -- tools/` and `git status --porcelain -- tools/`. Two empty outputs, not one file inspected.

**The marker now proves both directions.** Present, it refuses and prints the recovery commands, naming the gate the interrupted run was on; absent, the run proceeds and clears it on success. The command it prints is `git checkout -- tools/` — the directory, which is exactly the instruction I had and did not follow.

**The rule:** after any interrupted tool that edits files in place, compare the whole directory against its baseline, in both the committed and the working direction. A diff that names one file is a report about that file, not about the tree.
