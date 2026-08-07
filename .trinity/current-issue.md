# Current Issue — Wave Loop 892

**Issue:** #1845 — feat(igla): Wave Loop 892 — module-scope [603][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes

**Branch:** `wave-loop-892` (TBD)

**PR:** TBD

**Spec:** `specs/scratch/w892_bench_module_603x2p6_aos_var_call_write.t27`

**Outer dimension:** 603 (non-power-of-two)
**Inner struct:** `[2]^6 Pt` (2 fields × 6 trits × 32 bits = 384 bits per element)
**Total elements:** 603 × 64 = 38,592 structs
**Packed vector width:** 38,592 × 32 = 1,234,944 bits (~1.178 MiBit)

**Pattern:** module-scope array-of-struct variable initialized from a function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block.

**Next issue:** TBD (Wave Loop 893)

---

## Acceptance

- [ ] `t27c parse` PASS
- [ ] `t27c icarus-lowerable` → `lowerable`
- [ ] `t27c icarus-simulate` → `PASSED`
- [ ] `t27c icarus-cocotb` → reference-model OK
- [ ] `t27c seal --save` saved and `seal --verify` MATCH
- [ ] `cargo test --release --test icarus_lowerable accepts_w892_bench_module_603x2p6_aos_var_call_write` PASS
- [ ] `FROZEN_HASH` unchanged
- [ ] PR opened with `Closes #1845`

phi^2 + 1/phi^2 = 3 | TRINITY
