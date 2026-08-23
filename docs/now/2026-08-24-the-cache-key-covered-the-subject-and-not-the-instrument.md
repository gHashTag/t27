# NOW -- The cache key covered the subject and not the instrument (2026-08-24)

## The cache key covered the subject and not the instrument (Closes #2161)

- Gave the boundary operator invert's verdict filter. Sites went 77 to 5, and among the 72 removed were 6/6 in check_vector_data, 3/3 in check_seal_coverage and 1/1 in check_catalog_integrity — all KILLED mutants, which is proof they reach a verdict. The filter removed sites the measurement had already proven verdict-bearing, so it is wrong and reverted [measured].
- It cannot be fixed by widening: the missed sites append to a list that a later statement turns into a verdict. Verdict-reachability is a dataflow property and a line-local pattern cannot decide it; widening until the number looks agreeable is the move R2 forbids.
- The filter first appeared to do nothing — 26/77 before and after. The mutation cache keys on the gate's bytes and its control's bytes, and changing how sites are SELECTED changes neither, so a rebuilt tri served 24 stale rows. Two runs, one number, an edit that looked inert.
- The key now includes sha256 of the running binary. Verified in three states: same binary twice caches, a rebuild misses every row, the same binary again caches. A cache that cannot see its own instrument change is the instrument lying about itself [measured].
