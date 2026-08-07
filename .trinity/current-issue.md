# Current Issue — Wave Loop 890

**Issue:** #1841 — feat(igla): Wave Loop 890 — module-scope [599][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes

**Branch:** `wave-loop-890` (TBD)

**PR:** TBD

**Spec:** `specs/scratch/w890_bench_module_599x2p6_aos_var_call_write.t27`

**Outer dimension:** 599 (non-power-of-two)
**Inner struct:** `[2]^6 Pt` (2 fields × 6 trits × 32 bits = 384 bits per element)
**Total elements:** 599 × 64 = 38,336 structs
**Packed vector width:** 38,336 × 32 = 1,226,752 bits (~1.170 MiBit)

**Pattern:** module-scope array-of-struct variable initialized from a function call, then mutated via signed-index field writes and read back with `assert_eq` inside a `bench` block.

**Next issue:** TBD (Wave Loop 891)

---

## Acceptance

- [ ] `t27c parse` PASS
- [ ] `t27c icarus-lowerable` → `lowerable`
- [ ] `t27c icarus-simulate` → `PASSED`
- [ ] `t27c icarus-cocotb` → reference-model OK
- [ ] `t27c seal --save` saved and `seal --verify` MATCH
- [ ] `cargo test --release --test icarus_lowerable accepts_w890_bench_module_599x2p6_aos_var_call_write` PASS
- [ ] `FROZEN_HASH` unchanged
- [ ] PR opened with `Closes #1841`

phi^2 + 1/phi^2 = 3 | TRINITY
