# NOW -- I probed in bash and CI runs dash (2026-09-04)

## The gate is red, and my first diagnosis of why was wrong (Refs #2994)

- `coq-kernel.yml` has failed on every run since the previous pass's repair landed. I said the cause was `set -e` on `HITS=$(grep ...)`: probed it in bash, watched a clean tree exit 1, shipped a fix
- **the log said otherwise.** The job runs in `coqorg/coq:8.19-ocaml-4.14-flambda`, and there the step shell is **`sh -e`** -- dash. The step died on its FIRST line: `/__w/_temp/….sh: 1: set: Illegal option -o pipefail`. `set -uo pipefail`, added by the repair, is not valid in dash, and **nothing in the step ever ran**
- every sibling step in the same job uses `set -eux`; this was the only `-o pipefail`, and the step contains no pipeline for pipefail to guard
- **a local `sh` probe would not have caught it either**: on this machine `/bin/sh` is bash in POSIX mode and accepts `-o pipefail`. The probe world was not the world twice over -- the wrong shell, then a stand-in that behaves like the wrong one
- **the `set -e` reasoning was still needed.** Measured under real dash: a plain assignment on a clean tree exits 1 and never reaches `rc=`, while the `if` form reaches `rc=1` and exits 0. The repair takes both
- verified under `sh -e` in three planted worlds: clean **0**, a real `Admitted.` **1**, a deleted operand **2** naming the file
- **the lesson is the shell, not grep.** The interpreter is named in the run log and nowhere else -- not in the workflow, not in the step, not in the container image's name

## The conflict resolver lost the section it was carrying (Refs #2994)

- the merge poller resolves skill-file conflicts by extracting its own sections with an `awk` anchored on `## 490.` and re-appending them above master's highest
- the first resolution renumbered the section to 492. **The second looked for `## 490.` again, matched nothing, and wrote an empty carry** -- the section was gone from the branch and nothing failed
- recovered from `git show 729a2837f:` and re-appended. The anchor is a number the resolver itself changes, which is the same defect as a pointer at a renumbered section: **anchor on the title, which does not move**
