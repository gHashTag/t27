# NOW -- Reseal the seals the corpus and the compiler moved (2026-10-02)

## Seals describe what the tree generates again (Refs #5497)

- `check_seal_currency.py` reported 544 stale generated-code hashes on master db870cdf and the restored seal-coverage gate (#5454) 604 seals that no longer hold. Nothing in the merge path reseals, and both gates were invisible: one behind the first red step of Spec Guards, the other emptied until #5454.
- Why they moved, measured: 90 specs never changed and their output did -- bisected to the compiler changes of 2026-09-17 (#3962 "and the seal moves for it", #3973, #4114) and the 2026-09-08 gen-c/gen-rust wave, plus the S07-S10 seals made with a pinned binary on 2026-10-01. 245 specs changed after sealing: 154 last edited by bee commits that implement bodies and never reseal, 91 by repairs on master.
- Reseal 336 specs with the t27c built from master (the compiler sources are unchanged since 756bcffc): the 307 currency-stale ones and 29 the coverage gate flags, 34 of them hollow seals (`gen_hash=none`) whose specs now generate. `specs/ar/ternary_logic.t27` still generates nowhere (specs_generate_baseline.txt) and is resealed with `--force`, on the record as hollow. `tri seals sync-twins`: all twins consistent.
- After this: currency stale 544 -> 14 and coverage 604 -> 14 -- the 14 are the seven specs/ml/ seals #5575 reseals; hollow 65 -> 31. A reseal records what the compiler produces, not that it is correct, as in #3440 and #2909.
