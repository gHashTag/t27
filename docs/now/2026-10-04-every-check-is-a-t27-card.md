# NOW -- every check is a .t27 card, P1 (2026-10-04)

## What was read

- The last 300 runs of `conflict-markers.yml`, `withdrawn-live-gate.yml` and `damage-negatives.yml` (2026-10-02T18:02Z..2026-10-04T07:48Z): 295 / 296 / 295 completed, 0 red, cancellations only; medians 0.83 / 0.68 / 0.55 min.
- The master ruleset at 2026-10-04T07:55Z: required contexts `validate`, `check-linked-issue`, `parse-ratchet`. None of the three pilots is required.
- `t27c test-report` passes a card that lost its `module` line (`tests 1, pass 1`), and `parse --json` names that module `''` with exit 0. A lone `;` in prose is a `BLOCKED` parse error, not a failure.

## What changed

- `specs/ci/schema.t27`: the fields a card declares, the laws a card must pass, and tests of the laws themselves.
- `tools/ci/gen_workflows.py`: test-report, `parse --json`, laws spliced in as `test law_<name>`, render, `yaml.safe_load` round-trip, write. Modes: write, `--check`, `--self-check`. What it refuses and why is in its docstring.
- Three pilots moved to cards under `specs/ci/gates/`; each regenerated workflow is field-for-field what it was. Their comments moved into the cards.
- `.github/workflows/ci-cards.yml`, generated from its own card: `--self-check`, then `--check`.
- Census: shell files +1, jobs +1, `run:` steps +8, runner +8; quiet files +1. All from `ci-cards.yml`; the pilots move nothing. The ledgers carry the absolute numbers; master moved them twice under this PR.

## Measured

- Negative controls, each exit 1 with the reason named: a hand-edited YAML (`DIFFERS`), an emptied concurrency group (`FAIL law_pr_has_group`), a test calling an undefined law (`BLOCKED does not compile`), a lone `;` (`BLOCKED codegen failed`), a lost `module` line (`module is ''`).

## Not established

- The `cargo build` and zig download path of `ci-cards.yml` was first exercised by this PR's CI (run 37188991035, green); there is no longer history than that.
- `ci-cards.yml` cannot catch an edit that deletes its own check step.

Closes #5954
Refs #5933
