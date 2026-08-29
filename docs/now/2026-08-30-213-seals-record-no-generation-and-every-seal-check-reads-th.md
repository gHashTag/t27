# NOW -- 213 seals record no generation, and every seal check reads them as healthy (2026-08-30)

## 213 seals record no generation, and every seal check reads them as healthy (Refs #2864)

- tri seals hollow: 213 of 1311 spec seals carry gen_hash=none for all four backends -- a spec that does not parse, sealed. Freshness matches spec_hash, drift compares none against none and reports zero, coverage counts the file as covered.
- Reconciled against tools/specs_generate_baseline.txt rather than competing with it: 101 of the 104 specs are already recorded there as debt. What is new is that the same fact reads as health on the seal side.
- --why groups the compiler's error by kind with coordinates stripped: 39 kinds over 104 specs, largest covering 23. A dozen parser gaps, not 104 repairs.
- t27c validate printed VALIDATION: FAILED and exited 0; FAILED now exits 1, PASSED still exits 0.
