# NOW -- insertion_sort repaired, 9 stale sort/wp18 seals regenerated (2026-10-06)

## Seal Coverage back to green (Refs #6609)

- `specs/tri/sort/insertion_sort.t27` failed both of its own tests. The body was wrong: `var j = i - 1` with `while (j >= 0 && ...)` lowers to an unsigned `usize` index, so `j >= 0` is always true and `j = j - 1` underflows as soon as a key has to move to slot 0. The body now tracks the free slot `j = i` and stops at `j > 0`, comparing `values[j - 1]`. The expectation in `sort_simple_case` (`{5,2,8,1,9}` -> `{1,2,5,8,9}`) was already correct and is unchanged.
- `sort_basic_case` was a given/when/then placeholder (`sort(undefined)`, compared to a void result) that could never compile. It is now a real test: duplicates, a negative minimum that moves to slot 0, and an already-sorted tail. `t27c test-report`: 2/2 pass.
- Resealed on the Railway t27c lab with master's compiler (13e212a5a) via `tri seals drift --fix`: TriCountingSort, TriHeapSort, TriInsertionSort, TriTimSort, their `sort_*` twins, and tools_Wp18GateSelfConsistentSelfTest. `tools/check_seal_coverage.py`: OK, 1454 seals, nothing new in the baseline.
- The GateScarab corpus-ratchet half of #6609 is #6612. This PR does not touch it.
