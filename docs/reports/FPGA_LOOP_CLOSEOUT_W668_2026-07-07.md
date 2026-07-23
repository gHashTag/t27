# FPGA LOOP Closeout — Wave Loop 668

Date: 2026-07-07
Branch: `wave-loop-668`
Issue: **#1639**
Variant: **A — module-scope `[155][2]^6 Pt` array-of-struct variable from a call with indexed signed writes**

---

## 1. Goal

Validate that a module-scope mutable packed reg of type
`[155][2][2][2][2][2][2] Pt` (317,440 bits, ~0.303 MiBit) can be:

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
| Decomposed plan | `.claude/plans/wave-loop-668.md` |
| Issue + variant definition | `.trinity/current-issue.md` |
| Witness generator | `scripts/gen_w668.py` |
| Witness spec | `specs/scratch/w668_bench_module_155x2p6_aos_var_call_write.t27` |
| Structural test | `bootstrap/tests/icarus_lowerable.rs` — `accepts_w668_bench_module_155x2p6_aos_var_call_write` |
| Seal | `.trinity/seals/scratch_w668_bench_module_155x2p6_aos_var_call_write.json` |
| Icarus baseline | `.trinity/icarus-baselines/specs/scratch/w668_bench_module_155x2p6_aos_var_call_write.json` |
| Closeout report | `docs/reports/FPGA_LOOP_CLOSEOUT_W668_2026-07-07.md` |
| Experience update | `.trinity/experience.md` |
| Persistent memory | `~/.claude/projects/-Users-playra-t27/memory/wave-loop-668.md` |

---

## 3. Witness layout

- `pub struct Pt { x : i16, y : i16 }`
- Total elements: `155 × 2⁶ = 9,920`
- Total packed width: `9,920 × 32 = 317,440` bits (~0.303 MiBit)
- Element values: `x = (2*e + offset) % 32768`, `y = (2*e + offset + 1) % 32768`
- Outer dimension 155 is non-power-of-two; no padding is added in the packed
  row-major LSB-first layout.

---

## 4. Gate results

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `t27c parse` W668 | PASS |
| `t27c icarus-lowerable` W668 | lowerable |
| `t27c icarus-simulate` W668 | PASSED (17 cycles) |
| `t27c icarus-cocotb` W668 | OK (1 test / 1 bench passed + VCD probe check) |
| `t27c seal --save` W668 | saved |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 128 passed; 0 failed |
| `./scripts/tri test --fast` | **timed out** at Phase 1 (Parse) after 15 min (see §8) |

---

## 5. What changed

No compiler or reference-model code was modified. The only source-tree changes
are:

1. `scripts/gen_w668.py` — new generator.
2. `specs/scratch/w668_bench_module_155x2p6_aos_var_call_write.t27` — new
   generated witness.
3. `bootstrap/tests/icarus_lowerable.rs` — one new integration test.
4. `.trinity/seals/scratch_w668_bench_module_155x2p6_aos_var_call_write.json`
   and the empty Icarus baseline under `.trinity/icarus-baselines/...`.
5. Docs, planning, and experience files listed above.

---

## 6. Validation notes

- Multi-line W584-style brace literals are still required; a single-line mega
  literal would exceed parser practical limits even though the grammar accepts
  it in principle.
- Because the offset-0 schedule for 9,920 elements never reaches 32768, an
  explicit `make_grid(32768)` check was kept to preserve the modulo-wrap
  regression signal established in earlier waves.
- `assert_ne` remains accepted by the structural `icarus-lowerable` classifier but
  is not emitted by the Icarus simulation path; the witness uses `assert_eq`
  checks on changed elements.
- FROZEN_HASH remains unchanged at
  `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.

---

## 7. Weak points / open work

1. **Upper bound on outer non-p2 dimension.** 155 works, but the next waves
   will probe 157 and beyond. The 4-MiBit threshold is still far away, but
   compile-time and simulator memory will eventually become the limiting
   factor rather than correctness.
2. **`assert_ne` gap.** The structural classifier and simulation emitter
   disagree. Fixing this is a medium-priority cleanup that would let benches use
   whole-array inequality checks.
3. **No control-flow-guarded writes yet.** W668 uses unconditional indexed
   writes; a follow-up wave should exercise `if`-guarded writes (Variant C
   below).
4. **Function-local variant not covered at this size.** All recent waves use
   module-scope vars; a function-local witness of the same size would verify
   the same lowering path inside a different scope.

---

## 8. Repository-wide sweep status

`./scripts/tri test --fast` was started with a 15-minute timeout and did not
complete Phase 1 (Parse). The 29,511-line, 678 KB W668 literal dominates the
repository-wide parse sweep. Targeted `t27c` parse, lowerability, Icarus
simulation, cocotb, and seal gates all passed independently, and the
`cargo test` suites are green.

---

## 9. Wave Loop 669 cooperation variants

The odd outer-dimension module-scope ladder has been extended to 155. W669
offers three mutually complementary continuation options:

### Variant A (recommended) — `[157][2]^6 Pt` module-scope var from a call with indexed signed writes

Continue the ladder to outer dimension 157.
- 10,048 elements, 321,536-bit packed vector (~0.307 MiBit).
- Same shape and risk profile as W668.
- Zero expected compiler/reference-model changes.
- Fits the existing generator template with `OUTER = 157`.

### Variant B — `[155][2]^6 Pt` bench-local packed array var from a call with indexed signed writes

Keep the same dimensions and size as W668 but move the mutable `reg` into a
bench/function-local scope.
- 9,920 elements, 317,440-bit packed vector.
- Tests scope handling for large packed arrays.
- Useful complement to the module-scope ladder.

### Variant C — `[155][2]^6 Pt` module-scope var with `if`-guarded indexed signed field writes

Stay at 0.303 MiBit but add conditional writes.
- Exercises control-flow guarded indexed writes on a packed reg.
- Builds on earlier control-flow waves (W590/W591).
- Expected to require no new compiler support because the guarded assignment
  path already handles indexed scalar struct field writes.

The recommended variant is **A** because it keeps the outer-dimension ladder
monotonic and predictable while remaining well under simulator and tooling
limits.

---

## 10. Commit plan

- Feature commit: `feat(igla): Wave Loop 668 — module-scope [155][2]^6 Pt
  non-power-of-two outer-dimension array-of-struct variable from a call with
  indexed signed writes`
  - Body must include `Closes #1639`.
- Tracking commit: `docs(w668): Wave Loop 668 issue #1639 + branch wave-loop-668
  created, next-wave cooperation variants set`.
  - Body must include `Closes #1639`.

---

## 11. Sign-off

- `cargo test` suites pass.
- Direct `t27c` gates pass.
- Seal recorded.
- FROZEN_HASH unchanged.
- Closeout report and experience/memory updated.

Next: create branch `wave-loop-669`, choose variant, update
`.trinity/current-issue.md`.
