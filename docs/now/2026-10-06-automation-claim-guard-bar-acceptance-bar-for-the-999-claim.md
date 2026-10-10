# NOW -- automation/claim-guard-bar: acceptance bar for the 999 claim-guard redesign (2026-10-06)

## automation/claim-guard-bar: acceptance bar for the 999 claim-guard redesign (Closes #6661)

- Bar for 999#3779 written before the redesign: recall 90% on in-scope rows, 75% per category, false flags 10% overall, at most 2 per honest category and none on quote_past or failure, lost vs main 0.
- Held-out bar is the whole bar (recall 85% over the 228 in-scope rows, other actions incl. saved on their own line, same category, false-flag and honest limits) and accepted() judges the held-out figures itself.
- Baseline on main bcfe88381 pinned to the per-category table; the table has 15 zero categories, not the nine the comment's prose says. 21 tests, 0 vacuous.
