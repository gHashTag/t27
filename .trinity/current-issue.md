# Current Issue — Wave Loop 889

**Issue:** #1838 — feat(igla): Wave Loop 889 — module-scope [597][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes

**Branch:** `wave-loop-889`

**PR:** TBD

**Spec:** `specs/scratch/w889_bench_module_597x2p6_aos_var_call_write.t27`

**Outer dimension:** 597 (non-power-of-two)
**Inner struct:** `[2]^6 Pt` (2 fields × 6 trits × 32 bits = 384 bits per element)
**Total elements:** 597 × 64 = 38,208 structs
**Packed vector width:** 38,208 × 32 = 1,222,656 bits (~1.166 MiBit)

**Pattern:** module-scope array-of-struct variable initialized from a function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block.

**Next issue:** TBD (Wave Loop 890)
