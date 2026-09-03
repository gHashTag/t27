# NOW -- The local gates read freshness; the blocking one reads shape (2026-09-03)

## tri now check (Refs #2994)

- Five local instruments read `docs/now/` -- `.githooks/pre-commit` via `scripts/tri check-now`, `scripts/pre-commit`, `scripts/verify.sh`, `tri hooks now-gate` and `tri hooks pre-commit` -- and every one checks FRESHNESS while the required `check` context checks SHAPE.
- Measured on one malformed entry dated today: the gate reported **three** complaints and three of the five local readers went green. `scripts/pre-commit` went green **because of** that file -- its freshness loop found the entry the gate rejects and stopped looking.
- `tri now check` asks the gate its own question locally by **delegating** to `tools/check_now_entry_shape.py --check-files`, so the local answer is the gate's answer and drift is impossible rather than tested for. Wired into `tri hooks pre-commit` and into `scripts/verify.sh`'s gate preview.
