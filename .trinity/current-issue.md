# Current Issue — Wave Loop 896

**Issue:** #1855 — feat(igla): Wave Loop 896 — module-scope [611][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes

**Branch:** `wave-loop-896` (TBD)

**PR:** TBD

**Spec:** `specs/scratch/w896_bench_module_611x2p6_aos_var_call_write.t27`

**Outer dimension:** 611 (non-power-of-two)
**Inner struct:** `[2]^6 Pt` (2 fields × 6 trits × 32 bits = 384 bits per element)
**Total elements:** 611 × 64 = 39,104 structs
**Packed vector width:** 39,104 × 32 = 1,251,328 bits (~1.194 MiBit)

**Pattern:** module-scope array-of-struct variable initialized from a function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block.

**Previous:** Wave Loop 895 (#1853, PR #1854, branch `wave-loop-895`)

---

## Acceptance

- [ ] `t27c parse` PASS
- [ ] `t27c icarus-lowerable` → `lowerable`
- [ ] `t27c icarus-simulate` → `PASSED`
- [ ] `t27c icarus-cocotb` → reference-model OK
- [ ] `t27c seal --save` saved and `seal --verify` MATCH
- [ ] `cargo test --release --test icarus_lowerable accepts_w896_bench_module_611x2p6_aos_var_call_write` PASS
- [ ] `FROZEN_HASH` unchanged
- [ ] PR opened with `Closes #1855`

phi^2 + 1/phi^2 = 3 | TRINITY
