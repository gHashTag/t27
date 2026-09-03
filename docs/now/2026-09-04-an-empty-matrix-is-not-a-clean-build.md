# NOW -- An empty matrix is not a clean build (2026-09-04)

## A crate rename made rings-rust compile nothing and report success

- `rings_matrix.py` drops a directory unless the name starts `ring-`, ends
  `-rust`, and holds a `Cargo.toml`. Any of the three is one rename away.
- With the matrix empty the build job is skipped -- `if: needs.discover.outputs
  .count != '0'` -- and **a skipped job is green**. `discover` printed
  `Discovered 0 ring-*-rust crate(s).` and succeeded.
- The trigger is the same commit: `paths: rings/ring-*-rust/**` means the rename
  matches the filter, runs the workflow, and collects a green tick for it.
- The script now refuses an empty population and exits **2**, naming what it
  looked for and where. Verified with GitHub's own shell flags that
  `MATRIX="$(python3 …)"` aborts the step, so `discover` goes red instead.
- Today the population is 17, all carrying `Cargo.toml`; the control asserts a
  real tree still emits its matrix and reports its count.
- The workflow's own header records the previous visit: seven master runs with
  all 17 crate jobs failing, every one concluding `success`. That was fixed per
  job. This is the same door one step earlier -- with zero jobs there are no
  verdict rows to write.
- The control caught a false pass. Emptying `$GITHUB_OUTPUT` in the harness made
  all three defect arms fail on line 2's redirect rather than on the script's
  exit 2, and every one of them "passed". Fixed, and each arm now also asserts
  that the refusal is what stopped it.
- Prior art, checked: pytest reserves exit code **5** for "No tests were
  collected" as a public-API outcome. This repository's 2-for-everything is
  coarser than the field's by one distinction.
