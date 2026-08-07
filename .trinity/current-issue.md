# Current Issue — Wave Loop 893

**Issue:** #1848 — feat(igla): Wave Loop 893 — module-scope [605][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes

**Branch:** `wave-loop-893` (TBD)

**PR:** TBD

**Spec:** `specs/scratch/w893_bench_module_605x2p6_aos_var_call_write.t27`

**Outer dimension:** 605 (non-power-of-two)
**Inner struct:** `[2]^6 Pt` (2 fields × 6 trits × 32 bits = 384 bits per element)
**Total elements:** 605 × 64 = 38,720 structs
**Packed vector width:** 38,720 × 32 = 1,239,040 bits (~1.182 MiBit)

**Pattern:** module-scope array-of-struct variable initialized from a function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block.

**Next issue:** TBD (Wave Loop 894)

---

## Acceptance

- [ ] `t27c parse` PASS
- [ ] `t27c icarus-lowerable` → `lowerable`
- [ ] `t27c icarus-simulate` → `PASSED`
- [ ] `t27c icarus-cocotb` → reference-model OK
- [ ] `t27c seal --save` saved and `seal --verify` MATCH
- [ ] `cargo test --release --test icarus_lowerable accepts_w893_bench_module_605x2p6_aos_var_call_write` PASS
- [ ] `FROZEN_HASH` unchanged
- [ ] PR opened with `Closes #1848`

phi^2 + 1/phi^2 = 3 | TRINITY
