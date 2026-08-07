# Current Issue — Wave Loop 886

**Issue:** #1832 — feat(igla): Wave Loop 886 — module-scope [591][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes

**Branch:** `wave-loop-886`

**PR:** TBD

**Spec:** `specs/scratch/w886_bench_module_591x2p6_aos_var_call_write.t27`

**Outer dimension:** 591 (non-power-of-two)
**Inner struct:** `[2]^6 Pt` (2 fields × 6 trits × 32 bits = 384 bits per element)
**Total elements:** 591 × 2 = 1,182 structs → 37,824 field slots
**Packed vector width:** 37,824 × 32 = 1,210,368 bits (~1.155 MiBit)

**Pattern:** module-scope array-of-struct variable initialized from a function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block.

**Next issue:** TBD (Wave Loop 887)
