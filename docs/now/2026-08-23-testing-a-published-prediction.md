# NOW — testing a published prediction (2026-08-23)

The twelfth post claims controls miss *degenerate* inputs and says outright that this predicts a hole in other suites and does not measure one. So: measure one. `tri gates mutate` learned `--dir`.

- **The prediction could not be tested, and the reason is the finding.** The second repository has **54 Python tools and not one declared negative control** — no `--self-check`, no `--selftest`, under any spelling. Nothing to ask "did the control notice?" of. That is the state the first repository was in before this campaign.
- The only other controlled gate in reach is the one I wrote yesterday *from these lessons*. Its boundary column reads `0/0` — no comparison in the file — so the prediction is untestable there too, honestly and for a boring reason.

**The run found something else.** That gate — nine control cases, more discipline than anything else this week — had a surviving silent mutant: `main()`'s **no-argument** branch. Every case passes a build path, so nothing ran it the way a broken *caller* would.

It matters because of the wiring: the unattended publisher calls it inside an `if !`, so a lost argument reaching `return 0` reads as a passing check and the cron publishes having compared nothing. **The vacuous pass, inside the guard written against the loss it exists to prevent.**

Beside it: the usage line was `__doc__.splitlines()[-4]` and printed the `--history` line. A message that is an index into the prose above it is wrong the moment the prose is edited — and it already was.

**Two rulers widened to make the run possible.** The file filter matched `check_` and not `check-`, so aimed elsewhere it found nothing and printed an empty table — which reads exactly like a clean suite. And `code()` hard-coded `tools/`.

**The rule:** a gate you wrote from the lessons is still a gate — point the auditor at it, especially then. And when a prediction cannot be tested, name the precondition that failed rather than letting the attempt read as a confirmation.
