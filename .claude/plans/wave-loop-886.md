# Plan — Wave Loop 886

**Next issue:** TBD (create as #1832 or next available)  
**Next branch:** `wave-loop-886`  
**Base:** `wave-loop-885` HEAD (earlier wave PRs remain open)  

## Goal
Generate and validate the next mechanical packed-vector witness: module-scope `[591][2]^6 Pt` array-of-struct variable from call with indexed signed writes.

## Dimensions
- Outer dimension: 591 (non-power-of-two)
- Struct shape: `[2]^6 Pt` (2 fields × 6 trits × 32 bits = 384 bits per struct)
- Total field slots: 591 × 2 = 1,182 structs → 37,824 field slots
- Packed vector width: 37,824 × 32 = 1,210,368 bits (~1.155 MiBit)

## Steps
1. Create GitHub issue for W886 with body template.
2. Branch `wave-loop-886` from `wave-loop-885` HEAD.
3. Copy `scripts/gen_w885.py` → `scripts/gen_w886.py`.
4. Update generator copy-hazard checklist:
   - `DST` path → `w886_bench_module_591x2p6_aos_var_call_write.t27`
   - `OUTER = 591`
   - `MID_IDX = 591 // 2` → 295
   - module header → `w886_bench_module_...`
5. Run generator; verify `grep -E "w885|589|294"` is empty.
6. Run `t27c parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, `seal --save`, `seal --verify`.
7. Add integration test `accepts_w886_bench_module_591x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
8. Run targeted test and full `icarus_lowerable` suite; expect 345 passed, 1 pre-existing failure.
9. Commit with `Closes #1832`.
10. Push branch, open PR, enable auto-merge.
11. Update `.trinity/current-issue.md`, `docs/NOW.md`, `.trinity/experience.md`, skills, and persistent memory.

## Acceptance criteria
- [ ] New spec generated and sealed.
- [ ] All `t27c` commands pass.
- [ ] Targeted Rust test passes.
- [ ] Zero compiler / reference-model / `FROZEN_HASH` changes.
- [ ] PR open with auto-merge.

## Notes
- Continue the mechanical ladder pattern.
- Watch for generator copy-hazard stale references.
