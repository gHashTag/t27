# NOW -- specs/hosting/sybil.t27: a sybil earns nothing for wrong answers (2026-10-11)

## specs/hosting/sybil.t27 (Closes #8743; gHashTag/trios#1761)

- Five rules on top of slice 1's k-of-n agreement: an anchored quorum (an owner or trusted replica, or a known answer, in every public job; a public majority against it is never agreed), canaries (100 permille of public leases carry a known answer; a wrong one is slashed and struck), probation (0 mTRI until 10 agreeing votes), a cap per key per epoch (24 / 96 attested / 384 trusted mTRI), and the attestation flag that raises it.
- The seeded simulation (M attacker keys on M networks answering one wrong hash, 8 honest and 2 anchor hosts, 4 epochs x 60 jobs, 8 seeds): with all five rules the attacker earns 0 mTRI and 0 wrong results are agreed for M = 1, 2, 5, 20; with the anchor and canaries off it earns 2081 mTRI at M = 20 (0 below that, held by probation and strikes, while 13 and 64 wrong results are still agreed at M = 2 and 5); slice 1 as merged pays it from two keys up.
- `t27c test-report`: 13 tests, 0 FAIL, 0 vacuous, 99 runtime asserts, 3 invariants. 11 negative controls: 10 FAIL by name, 1 passes and is explained in the PR (the verdict-level anchor is defence in depth behind the placement rule).
