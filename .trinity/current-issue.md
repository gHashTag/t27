# Current Issue — Wave Loop 895

**Issue:** #1853 — feat(igla): Wave Loop 895 — module-scope [609][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes

**Branch:** `wave-loop-895` (TBD)

**PR:** TBD

**Spec:** `specs/scratch/w895_bench_module_609x2p6_aos_var_call_write.t27`

**Outer dimension:** 609 (non-power-of-two)
**Inner struct:** `[2]^6 Pt` (2 fields × 6 trits × 32 bits = 384 bits per element)
**Total elements:** 609 × 64 = 38,976 structs
**Packed vector width:** 38,976 × 32 = 1,247,232 bits (~1.190 MiBit)

**Pattern:** module-scope array-of-struct variable initialized from a function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block.

**Previous:** Wave Loop 894 (#1851, PR #1852, branch `wave-loop-894`)

---

## Acceptance

- [ ] `t27c parse` PASS
- [ ] `t27c icarus-lowerable` → `lowerable`
- [ ] `t27c icarus-simulate` → `PASSED`
- [ ] `t27c icarus-cocotb` → reference-model OK
- [ ] `t27c seal --save` saved and `seal --verify` MATCH
- [ ] `cargo test --release --test icarus_lowerable accepts_w895_bench_module_609x2p6_aos_var_call_write` PASS
- [ ] `FROZEN_HASH` unchanged
- [ ] PR opened with `Closes #1853`

phi^2 + 1/phi^2 = 3 | TRINITY
