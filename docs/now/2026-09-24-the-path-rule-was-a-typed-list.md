# NOW -- the path rule was a typed list, and the list had drifted (2026-09-24)

## `fpga`, `tests`, `gen`, `cli` and `.github` were never in it (Closes #4740)

- `propose_boundary.py` decided whether a token was a path by matching a hardcoded `ROOTS = "specs|tools|scripts|src|bootstrap|conformance|docs|bindings|apps"`. Somebody typed that, and it was wrong in the direction that **hides work**: the tree holds `fpga` (150 files), `tests` (162), `contrib` (195), `research` (160), `.github` (83), `cli` (69), `gen` (67), `proofs`, `experiments` -- none of them listed.
- So "Add a regression test suite for `gen/zig/measure.py`" read as naming nothing at all and sat in the pile that looks unreachable. A list nobody re-reads drifts from the repository it describes; `git ls-tree -d` cannot.
- Derived from the tree it found 18 more boundaries the old rule could not see, including `.claude/skills/ci-gates/SKILL.md` -- a path whose whole directory was invisible.
- The typed list stays as the fallback for a run with no checkout, which is what `--self-test` uses, and both branches are pinned there.
- Board after: **611 open issues, 296 without a boundary (was 565), 315 with one; 108 claimed, 12 bees running, `refusal` None.** The 296 that remain mostly name no file at all -- there is nothing to draft from, and a guess would be the wrong boundary this tool exists to avoid.
