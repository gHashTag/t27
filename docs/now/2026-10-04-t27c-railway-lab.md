# NOW -- t27c lab on Railway and the t27c steward (2026-10-04)

## The compiler's CI gates run on Railway for every watched head (Closes #6093)

- `infra/t27c-lab/lab.py` (stdlib only) polls `git ls-remote --heads` every 180 s and runs FROZEN_HASH, build, suite `--ratchet`, the Lean completeness test, seal currency, seal coverage, specs-still-parse and specs-generate on each new head of a watched branch, one commit at a time. Results are GET-only JSON: `/latest.json`, `/runs/<sha>.json`, `/runs/<sha>/<gate>.log`.
- No secret, no GitHub write, no request that changes anything; `refs/heads` only, so a fork's pull request is never built. A queued commit its branch moved past is dropped.
- Measured: the #5947 head f521a8a35, cold, on 24 vCPU: build 30 s, suite 95 s (RATCHET CLEAN, 112 / 112), all eight gates green in 3 min 29 s. The same build took 15 min 32 s on the workstation at load 780-840, and the same suite 1 h 47 min.

## A profile agent owns the loop (Closes #6094)

- `.claude/agents/t27c-steward.md`: the map, one round (claim, observe, advance, report, learn) and the never-list for epic #6092. It points at `lab.py` and `tools/queen/task_shape.py` instead of copying them.
