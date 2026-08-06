# FPGA LOOP Closeout — Wave Loop 679

Date: 2026-07-07
Branch: `wave-loop-679`
Issue: **#1650**
Variant: **A — module-scope `[177][2]^6 Pt` array-of-struct variable from a call with indexed signed writes**

---

## 1. Goal

Validate that a module-scope mutable packed reg of type
`[177][2][2][2][2][2][2] Pt` (362,496 bits, ~0.346 MiBit) can be:

- declared as a `pub var` and initialized from a function call returning a
  non-power-of-two 6-D packed literal,
- read with full-index paths including the outer non-p2 dimension,
- partially updated via signed-index field writes,
- read back and cross-checked against a `cocotb`/Python reference model, and
- sealed with deterministic spec/codegen hashes.

No compiler, reference-model, or language changes were required.

---

## 2. Deliverables

| Artifact | Path |
|----------|------|
| Decomposed plan | `.claude/plans/wave-loop-679.md` |
| Issue + variant definition | `.trinity/current-issue.md` |
| Witness generator | `scripts/gen_w679.py` |
| Witness spec | `specs/scratch/w679_bench_module_177x2p6_aos_var_call_write.t27` |
| Structural test | `bootstrap/tests/icarus_lowerable.rs` — `accepts_w679_bench_module_177x2p6_aos_var_call_write` |
| Seal | `.trinity/seals/scratch_w679_bench_module_177x2p6_aos_var_call_write.json` |
| Icarus baseline | `.trinity/icarus-baselines/specs/scratch/w679_bench_module_177x2p6_aos_var_call_write.json` |
| Closeout report | `docs/reports/FPGA_LOOP_CLOSEOUT_W679_2026-07-07.md` |
| Experience update | `.trinity/experience.md` |
| Persistent memory | `~/.claude/projects/-Users-playra-t27/memory/wave-loop-679.md` |

---

## 3. Witness layout

- `pub struct Pt { x : i16, y : i16 }`
- Total elements: `177 × 2⁶ = 11,328`
- Total packed width: `11,328 × 32 = 362,496` bits (~0.346 MiBit)
- Element values: `x = (2*e + offset) % 32768`, `y = (2*e + offset + 1) % 32768`
- Outer dimension 177 is non-power-of-two; no padding is added in the packed
  row-major LSB-first layout.

---

## 4. Gate results

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `t27c parse` W679 | PASS |
| `t27c icarus-lowerable` W679 | lowerable |
| `t27c icarus-simulate` W679 | PASSED (17 cycles) |
| `t27c icarus-cocotb` W679 | OK (1 test / 1 bench passed + VCD probe check) |
| `t27c seal --save` W679 | saved |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 139 passed; 0 failed |
| `./scripts/tri test --fast` | not run to completion this session (see §8) |

---

## 5. What changed

No compiler or reference-model code was modified. The only source-tree changes
are:

1. `scripts/gen_w679.py` — new generator.
2. `specs/scratch/w679_bench_module_177x2p6_aos_var_call_write.t27` — new
   generated witness.
3. `bootstrap/tests/icarus_lowerable.rs` — one new integration test.
4. `.trinity/seals/scratch_w679_bench_module_177x2p6_aos_var_call_write.json`
   and the empty Icarus baseline under `.trinity/icarus-baselines/...`.
5. Docs, planning, and experience files listed above.

---

## 6. Validation notes

- Multi-line W584-style brace literals are still required; a single-line mega
  literal would exceed parser practical limits even though the grammar accepts
  it in principle.
- Because the offset-0 schedule for 11,328 elements never reaches 32768, an
  explicit `make_grid(32768)` check was kept to preserve the modulo-wrap
  regression signal established in earlier waves.
- `assert_ne` remains accepted by the structural `icarus-lowerable` classifier but
  is not emitted by the Icarus simulation path; the witness uses `assert_eq`
  checks on changed elements.
- FROZEN_HASH remains unchanged at
  `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.

---

## 7. Weak points / open work

1. **Upper bound on outer non-p2 dimension.** 177 works, but the next waves
   will probe 179 and beyond. The 4-MiBit threshold is still far away, but
   compile-time and simulator memory will eventually become the limiting
   factor rather than correctness.
2. **`assert_ne` gap.** The structural classifier and simulation emitter
   disagree. Fixing this is a medium-priority cleanup that would let benches use
   whole-array inequality checks.
3. **No control-flow-guarded writes yet.** W679 uses unconditional indexed
   writes; a follow-up wave should exercise `if`-guarded writes (Variant C
   below).
4. **Function-local variant not covered at this size.** All recent waves use
   module-scope vars; a function-local witness of the same size would verify
   the same lowering path inside a different scope.

---

## 8. Repository-wide sweep status

`./scripts/tri test --fast` was not run to completion this session. Earlier waves
W665–W678 showed that repository-wide parse time is dominated by the ~31–34 k-line,
~750+ KB literal, and a 15-minute timeout typically only reaches the end of Phase 1
(Parse). Targeted `t27c` parse, lowerability, Icarus simulation, cocotb, and
seal gates all passed independently, and the `cargo test` suites are green.

---

## 9. Wave Loop 680 cooperation variants

The odd outer-dimension module-scope ladder has been extended to 177. W680
offers three mutually complementary continuation options:

### Variant A (recommended) — `[179][2]^6 Pt` module-scope var from a call with indexed signed writes

Continue the ladder to outer dimension 179.
- 11,456 elements, 366,592-bit packed vector (~0.350 MiBit).
- Same shape and risk profile as W679.
- Zero expected compiler/reference-model changes.
- Fits the existing generator template with `OUTER = 179`.

### Variant B — `[177][2]^6 Pt` bench-local packed array var from a call with indexed signed writes

Keep the same dimensions and size as W679 but move the mutable `reg` into a
bench/function-local scope.
- 11,328 elements, 362,496-bit packed vector.
- Tests scope handling for large packed arrays.
- Useful complement to the module-scope ladder.

### Variant C — `[177][2]^6 Pt` module-scope var with `if`-guarded indexed signed field writes

Stay at 0.346 MiBit but add conditional writes.
- Exercises control-flow guarded indexed writes on a packed reg.
- Builds on earlier control-flow waves (W590/W591).
- Expected to require no new compiler support because the guarded assignment
  path already handles indexed scalar struct field writes.

The recommended variant is **A** because it keeps the outer-dimension ladder
monotonic and predictable while remaining well under simulator and tooling
limits.

---

## 10. Commit plan

- Feature commit: `feat(igla): Wave Loop 679 — module-scope [177][2]^6 Pt
  non-power-of-two outer-dimension array-of-struct variable from a call with
  indexed signed writes`
  - Body must include `Closes #1650`.
- Tracking commit: `docs(w679): Wave Loop 679 issue #1650 + branch wave-loop-679
  created, next-wave cooperation variants set`.
  - Body must include `Closes #1650`.

---

## 11. Sign-off

- `cargo test` suites pass.
