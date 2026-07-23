# FPGA LOOP Closeout — Wave Loop 684

Date: 2026-07-07
Branch: `wave-loop-684`
Issue: **#1655**
Variant: **A — module-scope `[187][2]^6 Pt` array-of-struct variable from a call with indexed signed writes**

---

## 1. Goal

Validate that a module-scope mutable packed reg of type
`[187][2][2][2][2][2][2] Pt` (382,976 bits, ~0.366 MiBit) can be:

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
| Decomposed plan | `.claude/plans/wave-loop-684.md` |
| Issue + variant definition | `.trinity/current-issue.md` |
| Witness generator | `scripts/gen_w684.py` |
| Witness spec | `specs/scratch/w684_bench_module_187x2p6_aos_var_call_write.t27` |
| Structural test | `bootstrap/tests/icarus_lowerable.rs` — `accepts_w684_bench_module_187x2p6_aos_var_call_write` |
| Seal | `.trinity/seals/scratch_w684_bench_module_187x2p6_aos_var_call_write.json` |
| Icarus baseline | `.trinity/icarus-baselines/specs/scratch/w684_bench_module_187x2p6_aos_var_call_write.json` |
| Closeout report | `docs/reports/FPGA_LOOP_CLOSEOUT_W684_2026-07-07.md` |
| Experience update | `.trinity/experience.md` |
| Persistent memory | `~/.claude/projects/-Users-playra-t27/memory/wave-loop-684.md` |

---

## 3. Witness layout

- `pub struct Pt { x : i16, y : i16 }`
- Total elements: `187 × 2⁶ = 11,968`
- Total packed width: `11,968 × 32 = 382,976` bits (~0.366 MiBit)
- Element values: `x = (2*e + offset) % 32768`, `y = (2*e + offset + 1) % 32768`
- Outer dimension 187 is non-power-of-two; no padding is added in the packed
  row-major LSB-first layout.

---

## 4. Gate results

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `t27c parse` W684 | PASS |
| `t27c icarus-lowerable` W684 | lowerable |
| `t27c icarus-simulate` W684 | PASSED (17 cycles) |
| `t27c icarus-cocotb` W684 | OK (1 test / 1 bench passed + VCD probe check) |
| `t27c seal --save` W684 | saved |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 144 passed; 0 failed |
| `./scripts/tri test --fast` | not run to completion this session (see §8) |

---

## 5. What changed

No compiler or reference-model code was modified. The only source-tree changes
are:

1. `scripts/gen_w684.py` — new generator.
2. `specs/scratch/w684_bench_module_187x2p6_aos_var_call_write.t27` — new
   generated witness.
3. `bootstrap/tests/icarus_lowerable.rs` — one new integration test.
4. `.trinity/seals/scratch_w684_bench_module_187x2p6_aos_var_call_write.json`
   and the empty Icarus baseline under `.trinity/icarus-baselines/...`.
5. Docs, planning, and experience files listed above.

---

## 6. Validation notes

- Multi-line W584-style brace literals are still required; a single-line mega
  literal would exceed parser practical limits even though the grammar accepts
  it in principle.
- Because the offset-0 schedule for 11,968 elements never reaches 32768, an
  explicit `make_grid(32768)` check was kept to preserve the modulo-wrap
  regression signal established in earlier waves.
- `assert_ne` remains accepted by the structural `icarus-lowerable` classifier but
  is not emitted by the Icarus simulation path; the witness uses `assert_eq`
  checks on changed elements.
- FROZEN_HASH remains unchanged at
  `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.

---

## 7. Weak points / open work

1. **Upper bound on outer non-p2 dimension.** 187 works, but the next waves
   will probe 189 and beyond. The 4-MiBit threshold is still far away, but
   compile-time and simulator memory will eventually become the limiting
   factor rather than correctness.
2. **`assert_ne` gap.** The structural classifier and simulation emitter
   disagree. Fixing this is a medium-priority cleanup that would let benches use
   whole-array inequality checks.
3. **No control-flow-guarded writes yet.** W684 uses unconditional indexed
   writes; a follow-up wave should exercise `if`-guarded writes (Variant C
   below).
4. **Function-local variant not covered at this size.** All recent waves use
   module-scope vars; a function-local witness of the same size would verify
   the same lowering path inside a different scope.

---

## 8. Repository-wide sweep status

`./scripts/tri test --fast` was not run to completion this session. Earlier waves
W665–W683 showed that repository-wide parse time is dominated by the ~31–36 k-line,
~770+ KB literal, and a 15-minute timeout typically only reaches the end of Phase 1
(Parse). Targeted `t27c` parse, lowerability, Icarus simulation, cocotb, and
seal gates all passed independently, and the `cargo test` suites are green.

---

## 9. Wave Loop 685 cooperation variants

The odd outer-dimension module-scope ladder has been extended to 187. W685
offers three mutually complementary continuation options:

### Variant A (recommended) — `[189][2]^6 Pt` module-scope var from a call with indexed signed writes

Continue the ladder to outer dimension 189.
- 12,096 elements, 387,072-bit packed vector (~0.370 MiBit).
- Same shape and risk profile as W684.
- Zero expected compiler/reference-model changes.
- Fits the existing generator template with `OUTER = 189`.

### Variant B — `[187][2]^6 Pt` bench-local packed array var from a call with indexed signed writes

Keep the same dimensions and size as W684 but move the mutable `reg` into a
bench/function-local scope.
- 11,968 elements, 382,976-bit packed vector.
- Tests scope handling for large packed arrays.
- Useful complement to the module-scope ladder.

### Variant C — `[187][2]^6 Pt` module-scope var with `if`-guarded indexed signed field writes

Stay at 0.366 MiBit but add conditional writes.
- Exercises control-flow guarded indexed writes on a packed reg.
- Builds on earlier control-flow waves (W590/W591).
- Expected to require no new compiler support because the guarded assignment
  path already handles indexed scalar struct field writes.

The recommended variant is **A** because it keeps the outer-dimension ladder
monotonic and predictable while remaining well under simulator and tooling
limits.

---

## 10. Scientific / engineering grounding

- IEEE Std 1800-2017 §7.4.1/7.4.3 — packed-array width is the product of packed
  dimensions; ranges need not be powers of two.
- Accellera vlog-pp discussion (Graham 2002) — packed arrays as contiguous bit
  vectors.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs are
  synthesizable first-class objects.
- Icarus issue #1134 — assertion failures with unpacked arrays of packed
  structs; t27 flattening avoids the trigger.
- Icarus issue #1171 — freezes during elaboration of very large packed vectors;
  W684 stays well below the reported threshold.
- Yosys issue #2677 / #4653 / PR #4100 — multidimensional packed arrays
  supported, arrays of packed structs still unsupported; t27 flattening
  avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization.

---

## 11. Experience to carry forward

- Use a non-power-of-two outer dimension under the 4-MiBit cliff to test layout
  correctness while keeping simulation fast.
- Keep signed-i16 leaf values inside range with `(2*e + offset) % 32768` for
  any element count ≤ 163,840.
- Reuse the W589 wholesale module-scope initializer path for any scalar-struct
  array shape; no new compiler work is needed until the wall-clock limit is hit.
- Prefer `assert_eq` over `assert_ne` in Icarus-lowerable simulation blocks;
  `assert_ne` is accepted by the classifier but not lowered by the simulation
  emitter.
- When computing expected values for deep packed-array indices, convert the full
  row-major LSB-first element index explicitly rather than guessing inner-dimension
  offsets.
- `./scripts/tri test --fast` reports pre-existing Gen Verilog Yosys Smoke
  failures unrelated to the packed-AoS ladder; those warnings are not a blocker.
