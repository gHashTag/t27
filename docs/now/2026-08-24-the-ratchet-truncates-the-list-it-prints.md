# NOW -- The ratchet truncates the list it prints (2026-08-24)

## The ratchet truncates the list it prints (Closes #2407)

- Merging master into this branch conflicted only on FROZEN_HASH, resolved with tri reseal write — neither side is right after a merge, since each hash describes its own side's compiler.rs.
- The corpus ratchet then failed for the good reason: 27 unexpected passes, 0 unexpected failures. This branch fixes 27 specs the ledger still called broken. Ledger 208 to 181; check_specs_generate 562 to 589.
- Two of the 27 needed a second round. The ratchet truncates its printed list, so one pass over the printed names removes most and leaves a tail. Iterated until exit 0, with a guard that stops rather than looping if an unexpected FAILURE ever appears.
