# NOW -- Twenty steps run under a shell nobody named (2026-09-04)

## `tri gates shell` (Refs #2994)

- `coq-kernel.yml` cost hours because its container's shell is dash and a repair added `set -uo pipefail`. This command asks the question that would have caught it, of every step: **71 jobs, 227 `run:` steps -- 207 named by the runner, 0 by a `shell:` key, 20 by NOBODY**
- the repository has exactly one `shell:` key and no `defaults: run: shell:`, so for every step inside a container the interpreter is whatever the image carries. GitHub uses bash when the image HAS bash and `sh -e` otherwise, **and the image's contents are not visible from the workflow** -- an Unknown step is not wrong, it is unnamed, and the only direct evidence is a run log
- **the payload is the syntax scan, and only inside those twenty.** Split by consequence: `pipefail`, `[[ `, `<<<`, `${var,,}` are **fatal** (the step ends and nothing in it runs); `echo -e` and `source` are **quiet** and mean something else
- **validated against the failure it was written for**: run against the commit carrying `set -uo pipefail`, it prints `FATAL coq-kernel.yml:121 pipefail`. On master today it prints one hazard, and that one is quiet

## The needle is `pipefail`, not `-o pipefail` (Refs #2994)

- found by a test, not by reading. The line that broke the gate is `set -uo pipefail` -- **the flags are joined, so `-o pipefail` is not a substring of it**. A rule written for the textbook spelling would have missed the only instance this repository has ever had
- **a hand count disagreed, five against one**, and every one of the four extra was a case the narrower population is right to exclude: three lines of prose explaining this very defect, and one `<<<` in a job with no container, which therefore runs under bash
- the disagreement was the tool being correct and the grep being loose. Worth writing down because it usually runs the other way
- six clauses mutated, six killed. Two survived until the fixtures carried a two-space key BEFORE `jobs:` and an upper-case `${var^^}` expansion
