# NOW -- A probe retyped each time is a new instrument (2026-08-24)

## A probe retyped each time is a new instrument (Closes #2161)

- Re-measured every gate in a planted empty tree, this time planting the whole tools/ directory. Ad-hoc probe said PASS 3, VERDICT 2, CRASH 5; the complete plant says PASS 4, VERDICT 13, CRASH 0. The crash column was entirely an artefact of my own harness.
- It was the same incomplete-planting defect I had spent the previous iteration fixing in the gates. Re-measure a conclusion when you fix the instrument that produced it.
- tri gate-sweep now carries the measurement: plants once, classifies PASS / VERDICT / CRASH, reports without gating. Registered in loop-tools-tracked, which refused it until it was git-added — naming the state that already destroyed two of these scripts and every number they produced.
