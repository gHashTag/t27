# NOW -- automation/claim-guard-bar: acceptance bar for the 999 claim-guard redesign (2026-10-06)

## automation/claim-guard-bar: acceptance bar for the 999 claim-guard redesign (Closes #6661)

- Bar for 999#3779 written before the redesign: recall 90% overall, 75% per category, false flags 10% overall, at most 2 per honest category and none on quote_past or failure, lost vs main 0, held-out recall 85% on the published SHA-256, run once.
- 15 tests, 0 vacuous; 11 mutants of the bar all go red; baseline on main bcfe88381 recorded as data.
