# Current Issue — Wave Loop 888

**Issue:** #1836 — feat(igla): Wave Loop 888 — module-scope [595][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes

**Branch:** `wave-loop-888`

**PR:** TBD

**Spec:** `specs/scratch/w888_bench_module_595x2p6_aos_var_call_write.t27`

**Outer dimension:** 595 (non-power-of-two)
**Inner struct:** `[2]^6 Pt` (2 fields × 6 trits × 32 bits = 384 bits per element)
**Total elements:** 595 × 64 = 38,080 structs
**Packed vector width:** 38,080 × 32 = 1,218,560 bits (~1.162 MiBit)

**Pattern:** module-scope array-of-struct variable initialized from a function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block.

**Next issue:** TBD (Wave Loop 889)
