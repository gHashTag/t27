# Current Issue — Wave Loop 891

**Issue:** #1843 — feat(igla): Wave Loop 891 — module-scope [601][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes

**Branch:** `wave-loop-891` (TBD)

**PR:** TBD

**Spec:** `specs/scratch/w891_bench_module_601x2p6_aos_var_call_write.t27`

**Outer dimension:** 601 (non-power-of-two)
**Inner struct:** `[2]^6 Pt` (2 fields × 6 trits × 32 bits = 384 bits per element)
**Total elements:** 601 × 64 = 38,464 structs
**Packed vector width:** 38,464 × 32 = 1,230,848 bits (~1.174 MiBit)

**Pattern:** module-scope array-of-struct variable initialized from a function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block.

**Next issue:** TBD (Wave Loop 892)

---

## Acceptance

- [ ] `t27c parse` PASS
- [ ] `t27c icarus-lowerable` → `lowerable`
- [ ] `t27c icarus-simulate` → `PASSED`
- [ ] `t27c icarus-cocotb` → reference-model OK
- [ ] `t27c seal --save` saved and `seal --verify` MATCH
- [ ] `cargo test --release --test icarus_lowerable accepts_w891_bench_module_601x2p6_aos_var_call_write` PASS
- [ ] `FROZEN_HASH` unchanged
- [ ] PR opened with `Closes #1843`

phi^2 + 1/phi^2 = 3 | TRINITY
