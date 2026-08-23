# NOW — selecting gates by property (2026-08-23)

Measuring the files hidden from `tri gates sweep` produced three findings, two of them about the command.

- **The disclosure line was itself off by two.** It printed `13 of 28` where 15 match by name — `rows.len()` counts rows *after* the two control files are excluded. The line added for honesty understated the match by exactly the number of controls, and I repeated its number in a report.
- **Naming was never the property.** It failed as `check-` vs `check_`, and again as `verify_*` / `run_*`. The measurable property is: **a workflow invokes it and it can exit non-zero.** Anything that can turn a pipeline red is a gate whether or not its name says so. Selection is now by-name *or* by-property; the count moved 13 → 17.
- The three that surfaced — `fuzz_trainer.py`, `run_conformance_vvp.py`, `verify_multitarget.py` — run in CI, carry verdicts, and have **no control in any form**. Invisible for the whole campaign.

**The workflow heuristic lied on its first real use.** It was added yesterday with the warning that upgrading "no control" to "controlled" is the one error direction that hurts — and it then did exactly that, for all three, on the strength of the word `must` in a prose comment **760 lines** from the call.

Tightened two ways: only vocabulary chosen on purpose (`fixture`, `expect_`, `planted` — dropping `must` and `broken`, ordinary English in a comment), and **within 30 lines of an invocation**. Both directions tested.

**The cross-check:** pointed at the second repository, the tightened version independently reproduced the by-hand measurement from two iterations ago — the same three gates controlled, by the same evidence. A heuristic agreeing with a careful manual pass on a corpus it was not tuned against is the closest thing to a control this kind of search can have.
