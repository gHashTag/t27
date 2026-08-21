# Current Issue — Wave Loop 899

**Issue:** #1901 — feat(igla): Wave Loop 899 — module-scope [617][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes

**Branch:** `wave-loop-899` (TBD)

**PR:** TBD

**Spec:** `specs/scratch/w899_bench_module_617x2p6_aos_var_call_write.t27`

**Outer dimension:** 617 (non-power-of-two)
**Inner struct:** `[2]^6 Pt` (2 fields × 6 trits × 32 bits = 384 bits per element)
**Total elements:** 617 × 64 = 39,488 structs
**Packed vector width:** 39,488 × 32 = 1,263,616 bits (~1.206 MiBit)

**Pattern:** module-scope array-of-struct variable initialized from a function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block.

**Previous:** Wave Loop 898 (#1859, PR #1900, branch `wave-loop-898`)

---

## Acceptance

- [ ] `t27c parse` PASS
- [ ] `t27c icarus-lowerable` → `lowerable`
- [ ] `t27c icarus-simulate` → `PASSED`
- [ ] `t27c icarus-cocotb` → reference-model OK
- [ ] `t27c seal --save` saved and `seal --verify` MATCH
- [ ] `cargo test --release --test icarus_lowerable accepts_w899_bench_module_617x2p6_aos_var_call_write` PASS
- [ ] `FROZEN_HASH` unchanged
- [ ] PR opened with `Closes #1901`

phi^2 + 1/phi^2 = 3 | TRINITY
