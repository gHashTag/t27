# NOW -- Three seals follow their specs (2026-10-05)

## Re-seal the three stale generated-code hashes (Refs #6092)

- spec-guards on master (run 37337729112 at efc7d3c8a) fails its seal-currency step on 3 seal files: `automation_automation::kanban_card_chat.json`, `runtime-process.json` and `runtime_runtime-process.json`. All 4 gen hashes are stale in each.
- None of them is compiler drift. Both specs changed after they were last sealed, and nobody re-sealed them: `specs/automation/kanban-card-chat.t27` (v7, 9a5b7f5e4) and `specs/runtime/process.t27` (`process_continue` test, 3d73d5135). The `spec_hash` changes in all three files.
- Re-sealed on the Railway t27c lab with `t27c seal <spec> --save` and t27c built from master (bootstrap/ is unchanged since 682564f48). No `--force` and no baseline edit. kanban-card-chat records 24/24 tests passing. runtime/process stays `blocked` on the same Zig error as before (`expected '}', found ']'`).
- After: `check_seal_currency.py` reports STALE generated-code hash 0 (was 3). `check_seal_coverage.py` reports OK with 1330 holding.
- Not done: the 30 `stale, already ledgered` seals in `tools/seal_baseline.txt` do not fail the gate and are untouched. Each needs its spec fixed first, and ledger moves are owner-only.
