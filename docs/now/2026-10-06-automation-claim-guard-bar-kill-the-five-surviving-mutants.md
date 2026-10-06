# NOW -- automation/claim-guard-bar: kill the five surviving mutants (2026-10-06)

## automation/claim-guard-bar: kill the five surviving mutants (Closes #6679)

- New assertions in four existing tests pin what the round-2 review mutants left open: pct_ceil(1, 99) and pct_floor(2, 3) catch rounding off by one at remainder 1 and d-1; heldout_set_big_enough(200, 100) pins the exact minimum.
- A position-weighted checksum per language (EN 97, RU 147) catches a compensating swap inside one language; the control corpus 18 silent rows are pinned directly and in per mille (112).
- N16, N17, N19, N21 and N22 survived on master and are now killed; still 21 tests, 0 vacuous, resealed. The flat 2-per-honest-category limit is unchanged and stays open in the issue.
