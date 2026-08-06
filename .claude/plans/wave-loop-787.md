# Wave Loop 787 — Plan

**Date:** 2026-07-24
**Current branch:** `wave-loop-786` → target branch `wave-loop-787`
**Parent:** `wave-loop-786` HEAD at closeout
**Issue:** TBD after W786 PR opened

---

## 1. Context

Wave Loop 786 closed the next rung of the module-scope packed-array-of-struct
ladder: `[391][2]^6 Pt`, 25,024 elements, 800,768 bits, ~0.763 MiBit. All
validation gates passed with zero compiler, reference-model, or `FROZEN_HASH`
changes. The recommended next wave continues the odd outer-dimension ladder at
`[393][2]^6 Pt` (25,152 elements, 804,864 bits, ~0.767 MiBit).

---

## 2. Cooperation variants

### Variant A — `[393][2]^6 Pt` module-scope var from call (recommended)

Continue the established mechanical generator discipline:

1. Create `wave-loop-787` from `wave-loop-786` HEAD.
2. Copy `scripts/gen_w786.py` → `scripts/gen_w787.py`.
3. Update generator:
   - `OUTER = 393`
   - `MID_IDX = 196`
   - module prefix `w787_bench_module_393x2p6_aos_var_call_write`
   - fix f-string header / docstring after copy.
4. Generate `specs/scratch/w787_bench_module_393x2p6_aos_var_call_write.t27`.
5. Add integration test `accepts_w787_bench_module_393x2p6_aos_var_call_write`
   in `bootstrap/tests/icarus_lowerable.rs` after the W786 test.
6. Run gates: parse, icarus-lowerable, icarus-simulate, icarus-cocotb, seal.
7. Write closeout report and W788 variants.

**Why recommended:** minimal surprise, tests non-power-of-two stride 393, stays
well under 4-MiBit packed-vector cliff.

### Variant B — `[391][2]^6 Pt` bench/function-scope packed var from call

Keep W786 width but move the mutable `dst` declaration into a `bench` or
function scope to test local-variable packed-vector lowering and lifetime:

1. Generator based on W786 with `OUTER = 391`, `MID_IDX = 195`.
2. Emit `dst` as a local var, not module-level.
3. Same mid-index / frame-condition element as W786.

**Trade-off:** exercises a different compiler code path (local arrays) but does
not advance the width ladder.

### Variant C — `[391][2]^6 Pt` module-scope var with `if`-guarded writes

Stay at W786 width and add conditional indexed signed field writes:

1. Generator based on W786 with `OUTER = 391`, `MID_IDX = 195`.
2. Wrap some indexed writes in `if (index % 2 == 0) { ... }`.
3. Verify Icarus path emits correct conditional write logic for packed reg.

**Trade-off:** tests control-flow emission but does not advance the width ladder.

---

## 3. Estimated impact

- No compiler changes expected.
- No reference-model changes expected.
- `FROZEN_HASH` expected unchanged.
- `icarus_lowerable` test count will increase from 246 to 247.
- Expected element count: 393 × 64 = 25,152.
- Expected packed vector width: 25,152 × 32 = 804,864 bits (~0.767 MiBit).

---

## 4. Risk register

| Risk | Mitigation |
|------|------------|
| Copy hazard in generator header/docstring | Manual review of `w787_bench_module_393x2p6_aos_var_call_write` everywhere |
| `assert_ne` still not emitted by Icarus | Continue using `assert_eq` on changed elements |
| Earlier waves (W774-W786) PRs open | Branch from `wave-loop-786` HEAD; do not wait for merge |
| Pre-existing `verilog_array_literal_expr` regression | Out of scope; reference separate issue |
| 4-MiBit packed-vector cliff | Width stays ~0.767 MiBit, comfortable margin |

---

## 5. Completion criteria

- [ ] `wave-loop-787` branch created from `wave-loop-786` HEAD.
- [ ] `scripts/gen_w787.py` committed and executable.
- [ ] Witness `specs/scratch/w787_bench_module_393x2p6_aos_var_call_write.t27` generated.
- [ ] Integration test added and passes.
- [ ] `t27c parse` / `icarus-lowerable` / `icarus-simulate` / `icarus-cocotb` / `seal --save` pass.
- [ ] `cargo test -p t27c --test icarus_lowerable` passes (247/0 expected).
- [ ] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W787_2026-07-24.md` written.
- [ ] Plan `.claude/plans/wave-loop-788.md` with three variants.
- [ ] `.trinity/experience.md`, `.trinity/current-issue.md`, `docs/NOW.md`, `.claude/skills/t27-wave-loop.md` updated.
- [ ] Memory `wave-loop-787.md` saved and `MEMORY.md` index updated.
- [ ] Commit with `Closes #N`, push, open PR.

---

φ² + 1/φ² = 3 | TRINITY
