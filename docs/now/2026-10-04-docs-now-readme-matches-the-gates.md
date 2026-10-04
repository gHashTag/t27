# NOW -- docs/now README matches the gates that read entries (2026-10-04)

## docs/now README matches the gates that read entries (Closes #5961)

- `docs/now/README.md` now names the second required gate, `check-now-freshness.yml` (job `check`, `tools/check_now_entry_shape.py`), points at the checker's docstring for the entry shape, and notes that its lowercase slug pattern is stricter than the sync gate's `[A-Za-z0-9._-]`.
- It no longer claims `.githooks/pre-commit`, `scripts/pre-commit`, `t27c check-now` and `tri hooks now-gate` enforce the CI conditions: they only check that the `docs/now/` directory holds a fresh entry, which any entry on master satisfies.
- It says `docs/NOW.md` edits are refused by `now-sync-gate-diff.sh` without an `Archive-Repair:` trailer, instead of "no longer read by any gate".
- It says `tri now add` needs `cargo build --release -p tri`, that a hand-written entry works, and names `tri now check` and `tri hooks pre-push`. Follows #5950 / #5952.
- Docs only: no gate, hook, workflow or tool changed.
