# NOW — putting the rule into the tool (2026-08-23)

Yesterday's §45 said: before reporting that something has no control, enumerate the *forms* a control takes and search for each. A rule only I know is not a rule, so it went into `tri gates sweep`.

- **Four forms searched**: a flag in the script, the `EXTERNAL_CONTROL` table, `tests/test_<name>.py`, and a workflow naming the gate *beside* planted-fault vocabulary. The verdict column gained `OTHER`, distinct from `NONE` — "I cannot run it" and "it does not exist" are different findings, and were conflated once already.
- **Workflow evidence is a candidate, never proof.** A heuristic that upgrades "no control" to "controlled" is the one error direction that hurts. A test asserts the other direction: a workflow that merely *runs* a gate is not evidence of controlling it.
- **The output now prints what it searched** — forms and file filter, on every run, found or not. That is the actual repair: a reader cannot weigh a `NONE` without seeing the search behind it.

Pointing it at the second repository produced two more assumptions the old output was hiding.

- **The file filter is narrower than the control search.** `conformance_check.py` and `signal_health.py` are gates there and match neither `check_*` nor `gate`. The header now says `Files considered: 3 of 22` — and here, `13 of 28`. Fifteen Python files under `tools/` in this repository are invisible to a command whose output looks exhaustive.
- **The vacuous pass survived its own repair, in the sibling.** `mutate` grew a refusal for an empty gate set one iteration ago; `sweep` did not, so aimed at a directory with no gates it printed `0 gate(s); 0 with no control` and exited 0. Fixing a class in one command and not the one beside it is how the class survives.

Verified: `check_build_paths.py` in the second repository now surfaces as `OTHER` with its test file and three workflow candidates — the control I declared nonexistent yesterday. Default output for this repository is unchanged apart from the new disclosure lines; 15 unit tests green.
