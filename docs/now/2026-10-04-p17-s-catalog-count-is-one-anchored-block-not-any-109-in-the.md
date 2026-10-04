# NOW -- P17's catalog count is one anchored block, not any **109** in the doc (2026-10-04)

## P17's catalog count is one anchored block, not any **109** in the doc (Closes #5881)

- `test_catalog_table_matches_the_gate.py` passed if today's record count, in bold, appeared anywhere in the 27,000-line `IGLA-FORMAL-RESULTS.md`; T397's own **109** kept it green whatever P17's re-take said.
- P17's figure is now one generated block, `<!-- catalog-count anchor=15ac5b5b1... -->109<!-- /catalog-count -->`, checked against the `CATALOG:` count in the anchor's tree, its `RE-TAKEN AT` heading and the re-take's `mandatory-field` row; nothing outside the marker is read.
- The marker, fetch-by-SHA and `--write` moved from `test_retaken_propositions_still_match.py` (#5801) into `scripts/ci/anchored_count.py`, shared by both gates; that script's output is byte-identical.
- `--self-check` runs the whole script on four temp copies: correct block rc 0, wrong block rc 1, wrong block + right number planted elsewhere rc 1, marker removed rc 1. The old check returned 0 on all four.
