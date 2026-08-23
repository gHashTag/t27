# NOW — the fifth operator, and the control scope that leaked (2026-08-23)

Yesterday named a measurable hole: a gate whose verdicts are `assert`s scores 0/0 in every column, which prints exactly like a gate with nothing to break.

- **`--assert` closes it**: `assert C, "msg"` becomes `assert True, "msg"`. The message is kept deliberately — a mutant that also dropped the text would be killed by a control asserting that text, and the kill would be for the wrong reason.
- **Its first run found one site in a file with eighteen assertions.** Not a scoping choice — a bug all three scanners shared. `in_control` was set by a top-level `def` and cleared only by the **next** one, so everything after the last function inherits its status; when the last function is a `self_check`, the whole `if __name__ == "__main__":` block below is scored as control code. Sixteen assertions live there.
- A function ends at the next top-level **statement**, not the next `def`. Three scanners fixed. Silent, loud and invert had the same leak and never showed it, because module-level verdicts are rare and asserts are where they live.
- **The honest number: 16 sites, 2 killed.** The control written for that gate one iteration ago — three planted cases, all passing — covers two of its sixteen verdicts. "Has a control" became a measurement.
- **A cost, stated:** the full five-operator run now exceeds ten minutes. The suite has outgrown a single foreground command.

### And a rule I wrote, then broke

Two iterations ago: wait for the job that runs what you changed, not the checks beside it. Today I read `in_progress` and merged anyway. The branch run completed **success** afterwards — the outcome was fine and the method was not, and the difference between those is the whole subject here.

Worth saying precisely: that change added a `--self-check` branch and touched no CI invocation, so the job could not have been affected. That is an argument I could have made *before* merging, and did not — I simply did not look.
