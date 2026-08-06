# Wave Loop 788 — Plan

**Date:** 2026-07-24
**Current branch:** `wave-loop-787` → target branch `wave-loop-788`
**Parent:** `wave-loop-787` HEAD at closeout
**Issue:** TBD after W787 PR opened

---

## 1. Context

Wave Loop 787 closed the next rung of the module-scope packed-array-of-struct
ladder: `[393][2]^6 Pt`, 25,152 elements, 804,864 bits, ~0.767 MiBit. All
validation gates passed with zero compiler, reference-model, or `FROZEN_HASH`
changes. The recommended next wave continues the odd outer-dimension ladder at
`[395][2]^6 Pt` (25,280 elements, 808,960 bits, ~0.771 MiBit).

---

## 2. Cooperation variants

### Variant A — `[395][2]^6 Pt` module-scope var from call (recommended)

Continue the established mechanical generator discipline:

1. Create `wave-loop-788` from `wave-loop-787` HEAD.
2. Copy `scripts/gen_w787.py` → `scripts/gen_w788.py`.
3. Update generator:
   - `OUTER = 395`
   - `MID_IDX = 197`
   - module prefix `w788_bench_module_395x2p6_aos_var_call_write`
   - fix f-string header / docstring after copy.
4. Generate `specs/scratch/w788_bench_module_395x2p6_aos_var_call_write.t27`.
5. Add integration test `accepts_w788_bench_module_395x2p6_aos_var_call_write`
   in `bootstrap/tests/icarus_lowerable.rs` after the W787 test.
6. Run gates: parse, icarus-lowerable, icarus-simulate, icarus-cocotb, seal.
7. Write closeout report and W789 variants.

**Why recommended:** minimal surprise, tests non-power-of-two stride 395, stays
well under 4-MiBit packed-vector cliff.

### Variant B — `[393][2]^6 Pt` bench/function-scope packed var from call

Keep W787 width but move the mutable `dst` declaration into a `bench` or
function scope to test local-variable packed-vector lowering and lifetime:

1. Generator based on W787 with `OUTER = 393`, `MID_IDX = 196`.
2. Emit `dst` as a local var, not module-level.
3. Same mid-index / frame-condition element as W787.

**Trade-off:** exercises a different compiler code path (local arrays) but does
not advance the width ladder.

### Variant C — `[393][2]^6 Pt` module-scope var with `if`-guarded writes

Stay at W787 width and add conditional indexed signed field writes:

1. Generator based on W787 with `OUTER = 393`, `MID_IDX = 196`.
2. Wrap some indexed writes in `if (index % 2 == 0) { ... }`.
3. Verify Icarus path emits correct conditional write logic for packed reg.

**Trade-off:** tests control-flow emission but does not advance the width ladder.

---

## 3. Estimated impact

- No compiler changes expected.
- No reference-model changes expected.
- `FROZEN_HASH` expected unchanged.
- `icarus_lowerable` test count will increase from 247 to 248.
- Expected element count: 395 × 64 = 25,280.
- Expected packed vector width: 25,280 × 32 = 808,960 bits (~0.771 MiBit).

---

## 4. Risk register

| Risk | Mitigation |
|------|------------|
| Copy hazard in generator header/docstring | Manual review of `w788_bench_module_395x2p6_aos_var_call_write` everywhere |
| `assert_ne` still not emitted by Icarus | Continue using `assert_eq` on changed elements |
| Earlier waves (W774-W787) PRs open | Branch from `wave-loop-787` HEAD; do not wait for merge |
| Pre-existing `verilog_array_literal_expr` regression | Out of scope; reference separate issue |
| 4-MiBit packed-vector cliff | Width stays ~0.771 MiBit, comfortable margin |

---

## 5. Completion criteria

- [ ] `wave-loop-788` branch created from `wave-loop-787` HEAD.
- [ ] `scripts/gen_w788.py` committed and executable.
- [ ] Witness `specs/scratch/w788_bench_module_395x2p6_aos_var_call_write.t27` generated.
- [ ] Integration test added and passes.
- [ ] `t27c parse` / `icarus-lowerable` / `icarus-simulate` / `icarus-cocotb` / `seal --save` pass.
- [ ] `cargo test -p t27c --test icarus_lowerable` passes (248/0 expected).
- [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W788_2026-07-24.md` written.
- [ ] Plan `.claude/plans/wave-loop-789.md` with three variants.
- [ ] `.trinity/experience.md`, `.trinity/current-issue.md`, `docs/NOW.md`, `.claude/skills/t27-wave-loop.md` updated.
- [ ] Memory `wave-loop-787.md` saved and `MEMORY.md` index updated.
- [ ] Commit with `Closes #N`, push, open PR.

---

φ² + 1/φ² = 3 | TRINITY
