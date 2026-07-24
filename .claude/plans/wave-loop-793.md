# Wave Loop 793 — Plan

**Date:** 2026-07-24
**Current branch:** `wave-loop-792` → target branch `wave-loop-793`
**Parent:** `wave-loop-792` HEAD at closeout
**Issue:** TBD after W792 PR opened

---

## 1. Context

Wave Loop 792 closed the next rung of the module-scope packed-array-of-struct
ladder: `[403][2]^6 Pt`, 25,792 elements, 825,344 bits, ~0.787 MiBit. All
validation gates passed with zero compiler, reference-model, or `FROZEN_HASH`
changes. The recommended next wave continues the odd outer-dimension ladder at
`[405][2]^6 Pt` (25,920 elements, 829,440 bits, ~0.791 MiBit).

---

## 2. Cooperation variants

### Variant A — `[405][2]^6 Pt` module-scope var from call (recommended)

Continue the established mechanical generator discipline:

1. Create `wave-loop-793` from `wave-loop-792` HEAD.
2. Copy `scripts/gen_w792.py` → `scripts/gen_w793.py`.
3. Update generator:
   - `OUTER = 405`
   - `MID_IDX = 202`
   - module prefix `w793_bench_module_405x2p6_aos_var_call_write`
   - fix f-string header / docstring after copy (the wave prefix is hardcoded).
4. Generate `specs/scratch/w793_bench_module_405x2p6_aos_var_call_write.t27`.
5. Add integration test `accepts_w793_bench_module_405x2p6_aos_var_call_write`
   in `bootstrap/tests/icarus_lowerable.rs` after the W792 test.
6. Run gates: parse, icarus-lowerable, icarus-simulate, icarus-cocotb, seal.
7. Write closeout report and W794 variants.

**Why recommended:** minimal surprise, tests non-power-of-two stride 405, stays
well under 4-MiBit packed-vector cliff.

### Variant B — `[403][2]^6 Pt` bench/function-scope packed var from call

Keep W792 width but move the mutable `dst` declaration into a `bench` or
function scope to test local-variable packed-vector lowering and lifetime:

1. Generator based on W792 with `OUTER = 403`, `MID_IDX = 201`.
2. Emit `dst` as a local var, not module-level.
3. Same mid-index / frame-condition element as W792.

**Trade-off:** exercises a different compiler code path (local arrays) but does
not advance the width ladder.

### Variant C — `[403][2]^6 Pt` module-scope var with `if`-guarded writes

Stay at W792 width and add conditional indexed signed field writes:

1. Generator based on W792 with `OUTER = 403`, `MID_IDX = 201`.
2. Wrap some indexed writes in `if (index % 2 == 0) { ... }`.
3. Verify Icarus path emits correct conditional write logic for packed reg.

**Trade-off:** tests control-flow emission but does not advance the width ladder.

---

## 3. Estimated impact

- No compiler changes expected.
- No reference-model changes expected.
- `FROZEN_HASH` expected unchanged.
- `icarus_lowerable` test count will increase from 252 to 253.
- Expected element count: `405 × 64 = 25,920`.
- Expected packed vector width: `25,920 × 32 = 829,440` bits (~0.791 MiBit).

---

## 4. Risk register

| Risk | Mitigation |
|------|------------|
| Copy hazard in generator header/docstring | Manual review of `w793_bench_module_405x2p6_aos_var_call_write` everywhere |
| `assert_ne` still not emitted by Icarus | Continue using `assert_eq` on changed elements |
| Earlier waves (W774-W792) PRs open | Branch from `wave-loop-792` HEAD; do not wait for merge |
| Pre-existing `verilog_array_literal_expr` regression | Out of scope; reference separate issue |
| 4-MiBit packed-vector cliff | Width stays ~0.791 MiBit, comfortable margin |

---

## 5. Completion criteria

- [ ] `wave-loop-793` branch created from `wave-loop-792` HEAD.
- [ ] `scripts/gen_w793.py` committed and executable.
- [ ] Witness `specs/scratch/w793_bench_module_405x2p6_aos_var_call_write.t27` generated.
- [ ] Integration test added and passes.
- [ ] `t27c parse` / `icarus-lowerable` / `icarus-simulate` / `icarus-cocotb` / `seal --save` pass.
- [ ] `cargo test -p t27c --test icarus_lowerable` passes (253/0 expected).
- [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W793_2026-07-24.md` written.
- [ ] Plan `.claude/plans/wave-loop-794.md` with three variants.
- [ ] `.trinity/experience.md`, `.trinity/current-issue.md`, `docs/NOW.md`, `.claude/skills/t27-wave-loop.md` updated.
- [ ] Memory `wave-loop-792.md` saved and `MEMORY.md` index updated.
- [ ] Commit with `Closes #N`, push, open PR.

---

φ² + 1/φ² = 3 | TRINITY
