# Current Issue — Wave Loop 897

**Issue:** #1857 — feat(igla): Wave Loop 897 — module-scope [613][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes

**Branch:** `wave-loop-897` (TBD)

**PR:** TBD

**Spec:** `specs/scratch/w897_bench_module_613x2p6_aos_var_call_write.t27`

**Outer dimension:** 613 (non-power-of-two)
**Inner struct:** `[2]^6 Pt` (2 fields × 6 trits × 32 bits = 384 bits per element)
**Total elements:** 613 × 64 = 39,232 structs
**Packed vector width:** 39,232 × 32 = 1,255,424 bits (~1.198 MiBit)

**Pattern:** module-scope array-of-struct variable initialized from a function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block.

**Previous:** Wave Loop 896 (#1855, PR #1856, branch `wave-loop-896`)

---

## Acceptance

- [ ] `t27c parse` PASS
- [ ] `t27c icarus-lowerable` → `lowerable`
- [ ] `t27c icarus-simulate` → `PASSED`
- [ ] `t27c icarus-cocotb` → reference-model OK
- [ ] `t27c seal --save` saved and `seal --verify` MATCH
- [ ] `cargo test --release --test icarus_lowerable accepts_w897_bench_module_613x2p6_aos_var_call_write` PASS
- [ ] `FROZEN_HASH` unchanged
- [ ] PR opened with `Closes #1857`

phi^2 + 1/phi^2 = 3 | TRINITY
