# Wave Loop 791 — Plan

**Date:** 2026-07-24
**Current branch:** `wave-loop-790` → target branch `wave-loop-791`
**Parent:** `wave-loop-790` HEAD at closeout
**Issue:** TBD after W790 PR opened

---

## 1. Context

Wave Loop 790 closed the next rung of the module-scope packed-array-of-struct
ladder: `[399][2]^6 Pt`, 25,536 elements, 817,152 bits, ~0.779 MiBit. All
validation gates passed with zero compiler, reference-model, or `FROZEN_HASH`
changes. The recommended next wave continues the odd outer-dimension ladder at
`[401][2]^6 Pt` (25,664 elements, 821,248 bits, ~0.783 MiBit).

---

## 2. Cooperation variants

### Variant A — `[401][2]^6 Pt` module-scope var from call (recommended)

Continue the established mechanical generator discipline:

1. Create `wave-loop-791` from `wave-loop-790` HEAD.
2. Copy `scripts/gen_w790.py` → `scripts/gen_w791.py`.
3. Update generator:
   - `OUTER = 401`
   - `MID_IDX = 200`
   - module prefix `w791_bench_module_401x2p6_aos_var_call_write`
   - fix f-string header / docstring after copy (the wave prefix is hardcoded).
4. Generate `specs/scratch/w791_bench_module_401x2p6_aos_var_call_write.t27`.
5. Add integration test `accepts_w791_bench_module_401x2p6_aos_var_call_write`
   in `bootstrap/tests/icarus_lowerable.rs` after the W790 test.
6. Run gates: parse, icarus-lowerable, icarus-simulate, icarus-cocotb, seal.
7. Write closeout report and W792 variants.

**Why recommended:** minimal surprise, tests non-power-of-two stride 401, stays
well under 4-MiBit packed-vector cliff.

### Variant B — `[399][2]^6 Pt` bench/function-scope packed var from call

Keep W790 width but move the mutable `dst` declaration into a `bench` or
function scope to test local-variable packed-vector lowering and lifetime:

1. Generator based on W790 with `OUTER = 399`, `MID_IDX = 199`.
2. Emit `dst` as a local var, not module-level.
3. Same mid-index / frame-condition element as W790.

**Trade-off:** exercises a different compiler code path (local arrays) but does
not advance the width ladder.

### Variant C — `[399][2]^6 Pt` module-scope var with `if`-guarded writes

Stay at W790 width and add conditional indexed signed field writes:

1. Generator based on W790 with `OUTER = 399`, `MID_IDX = 199`.
2. Wrap some indexed writes in `if (index % 2 == 0) { ... }`.
3. Verify Icarus path emits correct conditional write logic for packed reg.

**Trade-off:** tests control-flow emission but does not advance the width ladder.

---

## 3. Estimated impact

- No compiler changes expected.
- No reference-model changes expected.
- `FROZEN_HASH` expected unchanged.
- `icarus_lowerable` test count will increase from 250 to 251.
- Expected element count: `401 × 64 = 25,664`.
- Expected packed vector width: `25,664 × 32 = 821,248` bits (~0.783 MiBit).

---

## 4. Risk register

| Risk | Mitigation |
|------|------------|
| Copy hazard in generator header/docstring | Manual review of `w791_bench_module_401x2p6_aos_var_call_write` everywhere |
| `assert_ne` still not emitted by Icarus | Continue using `assert_eq` on changed elements |
| Earlier waves (W774-W790) PRs open | Branch from `wave-loop-790` HEAD; do not wait for merge |
| Pre-existing `verilog_array_literal_expr` regression | Out of scope; reference separate issue |
| 4-MiBit packed-vector cliff | Width stays ~0.783 MiBit, comfortable margin |

---

## 5. Completion criteria

- [x] `wave-loop-791` branch created from `wave-loop-790` HEAD.
- [x] `scripts/gen_w791.py` committed and executable.
- [x] Witness `specs/scratch/w791_bench_module_401x2p6_aos_var_call_write.t27` generated.
- [x] Integration test added and passes.
- [x] `t27c parse` / `icarus-lowerable` / `icarus-simulate` / `icarus-cocotb` / `seal --save` pass.
- [x] `cargo test -p t27c --test icarus_lowerable` passes (251/0 expected).
- [x] Closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W791_2026-07-24.md` written.
- [x] Plan `.claude/plans/wave-loop-792.md` with three variants.
- [x] `.trinity/experience.md`, `.trinity/current-issue.md`, `docs/NOW.md`, `.claude/skills/t27-wave-loop.md` updated.
- [x] Memory `wave-loop-791.md` saved and `MEMORY.md` index updated.
- [x] Commit with `Closes #1511`, push, open PR.

---

φ² + 1/φ² = 3 | TRINITY
