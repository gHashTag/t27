# NOW -- the vector gate caught one data-loss mode of four (2026-08-23)

Refs #2241, #2325. From the gate audit; each mode reproduced by hand on the
real corpus before the fix, and each control re-run after.

- The ledger recorded only the NAMES of prose-only files and asked one
  question: did a new name appear? Measured against four ways a file can lose
  its data: stripping every data field was caught; reducing 7 cases to 1,
  setting `cases: []`, and corrupting the JSON all exited 0. The `total > 0`
  guard made an emptied file invisible entirely, and a parse failure was
  swallowed as `(0, 0)` -- "nothing to check".
- Deleting a file printed **"FIXED ... now carries data"**. The FIXED branch
  asserted from set arithmetic alone: a name that left the bad set must have
  been repaired. It congratulated a deletion, a corruption and a renamed
  top-level key. The NEW branch re-read the file; the FIXED branch did not.
- The ledger is now a CENSUS -- `name | total | data` for every file. A count
  that falls is a failure whatever the cause, a vanished file is DEPARTED, an
  unreadable one is UNREADABLE (the convention `check_seal_coverage.py`
  already follows). Seven controls, seven distinct messages, all exit 1;
  the original class (a new prose-only file) is still caught.
- The executor had the matching hole: `CONFORMANCE OK: all executed cases
  passed / executed cases: 0`. Vacuously true and indistinguishable from a
  real run, so emptying a vector file made the whole job green end to end.
  Zero executed cases is now a failure.
