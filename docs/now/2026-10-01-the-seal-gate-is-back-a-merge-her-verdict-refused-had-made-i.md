# NOW -- The seal gate is back: a merge her verdict refused had made it one comment line (2026-10-01)

## The seal gate is back: a merge her verdict refused had made it one comment line (Closes #5453)

- tools/check_seal_coverage.py is restored byte-for-byte from bcb32d723^ (688 lines). #5183 (queen-5158, her verdict escalate) had replaced it with one comment, and an empty file exits 0, so coverage and its negative control reported success while checking nothing.
- Measured at master with t27c built from master's bootstrap/: self-check passes; the real run reports 649 seals that do not hold (540 stale, 109 gen-drift), up from 131 when the gate was removed.
- Not done here: re-sealing. #5158 named it the owner's decision; this restores the instrument, not the verdict on the 649.
