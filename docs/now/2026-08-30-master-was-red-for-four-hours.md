# NOW -- master was red for four hours (2026-08-30)

## master was red for four hours (Closes #2908)

- `seal-coverage` failed on 29 consecutive master runs; the first red is #2866, mine, and the check was ALREADY failing on that PR when I merged it
- standing authorisation to merge my own PRs is not authorisation to merge red ones, and "unrelated to my change" was false -- it was caused by it
- 926 seals drift, all `gen_hash_rust` plus 62 verilog / 33 zig / 31 c; cumulative over 29 commits, not #2866 alone
- read the acceptance columns before re-sealing, as the gate instructs: `UNEXPECTED PASSES 1`, `DISCARD WORSENED 0`, `GATE FAILURES 0` -- the corpus ratchet is red on an IMPROVEMENT
- so re-sealing records output whose acceptance is better than the ledger, not worse; the ledger entry itself belongs to the other session and is left alone
- `tri seals drift --fix`, 926 files, coverage back to 1224 of 1318 holding
