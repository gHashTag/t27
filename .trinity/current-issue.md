# Current Issue — Wave Loop 885

**Issue:** #1830 — feat(igla): Wave Loop 885 — module-scope [589][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes

**Branch:** `wave-loop-885`

**PR:** TBD

**Spec:** `specs/scratch/w885_bench_module_589x2p6_aos_var_call_write.t27`

**Outer dimension:** 589 (non-power-of-two)
**Inner struct:** `[2]^6 Pt` (2 fields × 6 trits × 32 bits = 384 bits per element)
**Total elements:** 589 × 2 = 1,178 structs → 37,696 field slots
**Packed vector width:** 37,696 × 32 = 1,206,272 bits (~1.151 MiBit)

**Pattern:** module-scope array-of-struct variable initialized from a function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block.

**Next issue:** TBD (Wave Loop 886)
