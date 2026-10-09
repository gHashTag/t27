# NOW -- t27b ledger rows carry the host-free key; bless rewrites only rows that moved (2026-10-09)

## step 3 of verdict reuse (Closes #8186, epic #8095)

- `specs/tri/t27b/verdict_key.t27` gains `ledger_key`: the verdict key of #8208 without the host
  part (spec bytes, generated output or a "not generated" marker, runner, zig). Its own
  `LEDGER_SCHEMA` keeps it from ever equalling a cache key. 10 tests.
- `specs/tri/t27b/steward.t27` gains the two bless rules, with 9 tests and an invariant:
  `bless_source_ok` refuses a run that does not carry a key on every row the reference passes,
  and `bless_writes` rewrites a row only when its key, verdict or blocker moved. A row where
  nothing moved stays byte for byte.
- Glue: `t27b keys <dir> --reference <t27c>` prints the ledger key per file; corpus records
  carry `"key"`; the lab publishes the key per row; `tri t27b ratchet --bless` keeps rows
  that did not move and refuses a keyless source.
- The first keyed bless rewrites every row once, to add the key. Until the lab redeploys,
  bless refuses: it says so instead of guessing.
