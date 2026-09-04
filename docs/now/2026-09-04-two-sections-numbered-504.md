# NOW -- Two sections numbered 504, and nothing was going to say so (2026-09-04)

## The law with a checker that exits 0 and is never called (Refs #2994)

- on master today two sections were both **504** -- "The subject of a step is the step" and "An extension is not a language" -- landed 26 minutes apart by two sessions, each appending above the same highest number it had read
- `tri skill check` **finds it** and prints `PROBLEMS`, `section 504 appears 2 times`, `section 504 comes after 505 -- the file reads out of order`. **And then exits 0**
- worse: `grep -rn "skill check"` across `.github/workflows/`, `scripts/` and `tools/` returns **nothing**. No gate, hook or script calls it. The law is enforced by a command that detects the violation, reports it as text, exits successfully, and is never run
- three independent failures stacked, and any one alone would have caught it: a non-zero exit fails a PR; a wired check prints where someone reads; a checker run by hand shows it
- **the collision is not carelessness.** Both sessions did the correct thing. It is a figure over a sliding population in the one place this file legislates about: **"the highest number" is a query, not a fact**
- resolved by moving the LATER of the two; `grep` for references returns zero either way, so the tie was broken by merge time rather than by cost

## And the shell section's premise was wrong (Refs #2994)

- the first draft said the image decides -- bash if present, `sh -e` otherwise. **The log says otherwise.** In run 33840876340, inside `coqorg/coq`, **both shells appear in one job**: this repository's `run:` steps get `sh -e {0}` while a composite action's steps, which declare `shell: bash`, get `bash --noprofile --norc -e -o pipefail {0}` and succeed
- **bash is present in that image.** What selects `sh` is the CONTAINER, not bash's absence: on the runner host the default is bash, inside a container it is `sh`, and only a `shell:` key changes it
- the practical consequence inverts: `shell: bash` is not a gamble on the image, it is measured to work in the one image tested here. Corrected in place
