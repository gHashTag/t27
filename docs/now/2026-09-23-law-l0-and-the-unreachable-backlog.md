# NOW -- L0 PURPOSE adopted, and the backlog no bee could take made visible (2026-09-23)

## The law the board cited before it existed (Closes #4605)

- `QueenRoadmap.tsx:2` and `roadmap-stack.mjs:5` both cited "trios CLAUDE.md, law L0" for the rewrite goal. `trios/CLAUDE.md` carries L1-L9 about shell scripts, `Closes #N`, clippy, tests, port 9005, a GB fallback, an experience log and push-first. There is no L0 in that file, in this repository, or anywhere else.
- The cause is a real gap, not a typo: L1-L7 govern how work ENTERS the repository and each has a live gate, but none of them says what the work is FOR. The `.t27` rewrite is the stated goal, measured on the ROADMAP view and named as the win condition in `onboarding.t27`, and it had no constitutional standing at all.
- **L0 PURPOSE** is therefore adopted as an ADDITION, not a rewrite: everything below the interface is written once in `.t27` and generated to its target, with two named exceptions (the seed `t27c` stays hand-written Rust; the interface stays Swift for trios and TSX for the web).
- It sits OUTSIDE the Asimov ordering on purpose and may never be cited to excuse breaking L1-L7: work that reaches the goal faster by merging untraced, hand-editing generated files or shipping an untested spec has not reached it.
- Enforcement is a measurement that already exists rather than a new one: the `.t27` share of the stack, computed by `apps/website/scripts/roadmap-stack.mjs` from the GitHub tree API and published at `/roadmap/stack.json`, must not fall. A law nobody can check is decoration.

## The backlog was unreachable, not empty

- Measured today: 563 of 654 open issues carry no `## Boundary` section, so the Queen can reserve no paths for them and skips every one; her tick reported `missingBoundary` for 565 candidates while the swarm sat at 2 of 20 lanes.
- `.github/ISSUE_TEMPLATE/swarm-task.md` carries the section, so the class stops growing, and links the tutorial and the board from the issue chooser.
- `tools/queen/needs_boundary.py` plus `.github/workflows/needs-boundary.yml` label the ones missing it: 553 to label, with the 10 roadmap goals and epics held back because a goal is not a task. Self-test passes and the dry run was taken against the live repository.
- Neither writes a boundary into an issue body. Which paths a task may touch is a judgement about the work, and a WRONG boundary is worse than a missing one because it reserves the wrong files.
