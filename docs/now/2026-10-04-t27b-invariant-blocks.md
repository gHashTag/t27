# NOW -- t27b runs invariant blocks (2026-10-04)

## t27b: invariant blocks run natively, reported apart from tests (Refs #5977)

- `invariant` blocks run through the same machinery as tests and are reported apart from them (`INVARIANT PASS` / `INVARIANT FAIL`); an invariant whose body the front-end discarded is reported as NOT CHECKED, never as held.
- Corpus (1184 specs): 48 supported (47 pass, 1 real spec failure), up from 37; 0 JIT/interpreter mismatches.
- Differential test: 0 mismatches on 2500 random programs per overflow mode. Part of epic #5905.
