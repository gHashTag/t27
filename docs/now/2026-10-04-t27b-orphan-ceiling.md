# NOW -- the orphan-ceiling ledger names cli/t27b (2026-10-04)

## cli/t27b gets an orphan ceiling of 0 (Closes #5988)

- #5979 added `cli/t27b` to the workspace members but not to `docs/reports/orphan_modules.json`, so master failed "Every source file is reachable from its crate root" and `modreach::tests::the_ratchet_watches_every_member_and_no_ghost` in cli-tri.
- The gate's own count for the crate is 11 files under `src/`, 11 compiled, 0 orphaned (plus 2 reached via `#[path]`), so the ceiling is 0.
