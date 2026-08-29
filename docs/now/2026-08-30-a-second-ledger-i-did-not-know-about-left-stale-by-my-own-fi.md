# NOW -- A second ledger I did not know about, left stale by my own fix (2026-08-30)

## A second ledger I did not know about, left stale by my own fix (Refs #2864)

- The corpus ratchet has been RED on master for at least three runs, and the reason is mine: #2897 made specs/pins/emitter_xdc.t27 typecheck, I updated tools/specs_generate_baseline.txt, and docs/reports/suite_expectations.json still listed it as expected-to-fail. UNEXPECTED PASS is a ratchet failure by design.
- Entry removed and max_entries lowered 151 -> 150, which is the direction the cap is allowed to move. RATCHET: CLEAN.
- Control both ways: putting the entry back reproduces 'UNEXPECTED PASSES: 1' and exit 1; removing it gives exit 0.
- This is the same lesson as #2892 -- a fix that does not travel -- one level up: I checked the sibling FUNCTION that day and not the sibling LEDGER. When a repair makes a spec pass, grep every file that names it, not just the one you know.
