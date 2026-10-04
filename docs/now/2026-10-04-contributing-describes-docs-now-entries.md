# NOW -- CONTRIBUTING describes the docs/now entry the NOW gates check (2026-10-04)

## CONTRIBUTING describes the docs/now entry the NOW gates check (Closes #5950)

- `CONTRIBUTING.md` still told contributors to keep root `NOW.md` and `docs/NOW.md` aligned, to touch `docs/NOW.md` in every PR, and that the pre-commit hook reads the `Last updated` line of root `NOW.md`. None of that is what the code does: `docs/NOW.md` is a frozen archive, and no NOW gate reads either file.
- The section is now `## NOW entries (one file per PR)`: one added `docs/now/<YYYY-MM-DD>-<slug>.md` per PR; `now-sync-gate.yml` (job `check-now-freshness`, `scripts/ci/now-sync-gate-diff.sh`) checks that the range adds a fresh entry; `check-now-freshness.yml` (job `check`, `tools/check_now_entry_shape.py`) checks its shape; the two local commands that ask the same questions; what `.githooks/pre-commit` runs with and without a `tri` binary.
- It points at `docs/now/README.md` and the checker's docstring for the format instead of copying it, says `tri now add` needs `cargo build --release -p tri` and that a hand-written entry works, and drops two claims the code no longer backs: that `phi-loop-ci.yml` builds `t27c` and runs `check-now`, and that `gen*`/`compile*` run the gate first.
- Item 4 of "Before you change code or specs" pointed at the same stale rule; it now links to the rewritten section.
- Docs only: no gate, hook, workflow or tool changed.
