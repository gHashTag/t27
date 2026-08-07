# Current Issue — Wave Loop 894

**Issue:** #1851 — feat(igla): Wave Loop 894 — module-scope [607][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes

**Branch:** `wave-loop-894` (TBD)

**PR:** TBD

**Spec:** `specs/scratch/w894_bench_module_607x2p6_aos_var_call_write.t27`

**Outer dimension:** 607 (non-power-of-two)
**Inner struct:** `[2]^6 Pt` (2 fields × 6 trits × 32 bits = 384 bits per element)
**Total elements:** 607 × 64 = 38,848 structs
**Packed vector width:** 38,848 × 32 = 1,243,136 bits (~1.186 MiBit)

**Pattern:** module-scope array-of-struct variable initialized from a function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block.

**Previous:** Wave Loop 893 (#1848, PR #1850, branch `wave-loop-893`)

---

## Acceptance

- [ ] `t27c parse` PASS
- [ ] `t27c icarus-lowerable` → `lowerable`
- [ ] `t27c icarus-simulate` → `PASSED`
- [ ] `t27c icarus-cocotb` → reference-model OK
- [ ] `t27c seal --save` saved and `seal --verify` MATCH
- [ ] `cargo test --release --test icarus_lowerable accepts_w894_bench_module_607x2p6_aos_var_call_write` PASS
- [ ] `FROZEN_HASH` unchanged
- [ ] PR opened with `Closes #1851`

phi^2 + 1/phi^2 = 3 | TRINITY
