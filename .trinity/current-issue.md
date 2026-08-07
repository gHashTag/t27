# Current Issue — Wave Loop 887

**Issue:** #1834 — feat(igla): Wave Loop 887 — module-scope [593][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes

**Branch:** `wave-loop-887`

**PR:** TBD

**Spec:** `specs/scratch/w887_bench_module_593x2p6_aos_var_call_write.t27`

**Outer dimension:** 593 (non-power-of-two)
**Inner struct:** `[2]^6 Pt` (2 fields × 6 trits × 32 bits = 384 bits per element)
**Total elements:** 593 × 2 = 1,186 structs → 37,952 field slots
**Packed vector width:** 37,952 × 32 = 1,214,464 bits (~1.159 MiBit)

**Pattern:** module-scope array-of-struct variable initialized from a function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block.

**Next issue:** TBD (Wave Loop 888)
