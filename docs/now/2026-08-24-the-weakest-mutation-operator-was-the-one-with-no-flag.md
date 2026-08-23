# NOW -- The weakest mutation operator was the one with no flag (2026-08-24)

## The weakest mutation operator was the one with no flag (Closes #2161)

- Recomputed all five operators: silent 47/52, loud 28/32, invert 72/78, boundary 26/77, assert 2/16 — 175 of 255, 80 survivors, and 40 of them in one file. Previously 157/244 with 87.
- Reading the survivors changes what they mean. boundary_sites mutates every comparison, while invert_sites keeps only conditions whose body carries a verdict — hence 92 percent against 34. The boundary survivors are loop bounds and display cutoffs; moving those cannot make a gate stop failing.
- The assert survivors are thresholds with margin: floor 54, measured 58 56 59 59 55 55. Two sit one point above the line, so the threshold works and the operator survives because 55 satisfies both comparisons. Survivor means defect for the return operators and does not for the other two.
- boundary had no flag of its own, reachable only through --all. The one operator you cannot run alone was the weakest, and no measurement would have found it: the gap was in the argument parser. Added, with a test; its negative control caught my own wrong argv immediately, because --loud came back unaccepted.
