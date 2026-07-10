# Wave Loop 474 — Decomposed Plan (2026-07-08)

**Issue:** (to be opened)  
**Branch:** `wave-loop-474`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

## Variant selection

**Selected: Variant B** — continue compiler-backend aggregate hardening. The physical bench is still blocked (DLC10 cable / unwired P12 relay), so hardware Variant A is impossible. Variant B has a small, reviewable tail that directly extends the W455–W473 struct-array line.

## Goals

1. Function-local arrays of structs with array-typed fields (`var tmp : [2][3]Shape`; `tmp[i].pts[j].x = v` and read-back).
2. Array-of-struct function returns assigned to module-level variables/const initializers (`var grid : [2][3]Shape = make_grid();`).
3. Whole-struct equality for scalar structs and small arrays of structs (`a == b`, `arr == other`).
4. Adversarial yosys-elaboration witness that scans generated Verilog for undeclared identifiers / width mismatches / illegal inline declarations.
5. Keep the suite green: ≥633/633 non-smoke PASS, zero yosys smoke failures, zero seal mismatches.

## Weak-spot analysis

- **Physical evidence gap.** The strongest differentiator (live cold-POR CCLK sweep → Lean theorem) remains blocked. This is a project-level risk, not a code risk.
- **No formal proof of per-field memory lowering.** The compiler emits SoA-style per-field memories, but there is no Lean theorem that the source read/write semantics are preserved. This is the largest scientific gap.
- **Adversarial witness is manual.** The smoke gate checks yosys acceptance and an allowed-warning list, but it does not systematically pre-check emitted Verilog for undeclared identifiers or illegal inline declarations.
- **Master-merge debt.** An independent fix set lives on `master` (`701d79b3b`). It is rejected as a single-wave merge because it is insufficient for earlier baselines and risky relative to the wave-line sub-fixes. The wave line now has a zero-failure baseline, so a dedicated merge wave should be planned separately.
- **Literature context.** High-level synthesis (Vitis HLS, Intel HLS) routinely uses AoS/SoA/AoSoA layout transformations; Sparkle HDL (Lean 4) and CktFormalizer are the closest formal-HDL competitors. t27's unique intersection remains spec-first ternary compute + sealed numeric conformance + physical boot-evidence instrumentation.

## Decomposition

### Task 1 — Function-local nested struct arrays
- Add a scratch spec `w474_local_nested_struct_array.t27` with `var tmp : [2][3]Shape`, literal and variable-index writes of nested fields (`tmp[i].pts[j].x = v`), and read-back assertions.
- Extend `gen_verilog_try_struct_array_assign` to handle `collect_field_index_path` for function-local multi-dimensional struct arrays. The local per-element per-field register layout is already `{base}_{i0}_{i1}_..._{fname}`; the write path must collect the outer indices, determine how many belong to the source array dimensions vs inner field dimensions, and emit the correct flattened register target.
- Reuse `local_array_dims` to split outer vs inner indices.
- Verify with `./scripts/tri test --fast`.

### Task 2 — Array-of-struct return writeback to module-level vars
- Add a scratch spec `w474_module_aos_return_assign.t27` where a function returns `[2][3]Shape` and the result is assigned to a module-level `var` or `const`.
- Extend `gen_verilog_var` / `gen_verilog_const` initializer handling: when the RHS is an `ExprCall` whose return type is an array of structs, emit a packed temporary and unpack it into the per-field memories exactly like `gen_verilog_unpack_array_of_struct_call` does for local arrays.
- Verify with `./scripts/tri test --fast`.

### Task 3 — Scalar-struct and small-array-of-struct equality
- Add a scratch spec `w474_struct_equality.t27` with `==`/`!=` on scalar structs and small arrays of structs.
- Extend `ExprBinary` `==`/`!=` path to detect array-of-struct operands (both identifiers and literals), pack both sides using `try_emit_array_of_struct_literal_packed` / `gen_verilog_pack_array_of_struct_literal`, and compare the resulting packed vectors.
- Verify with `./scripts/tri test --fast`.

### Task 4 — Adversarial yosys-elaboration witness
- Add a regression spec `w474_adversarial_verilog_witness.t27` whose generated Verilog is passed through a new lightweight scanner (could be a unit test in `bootstrap/src/compiler.rs` or a script step) that checks:
  - every assignment target identifier appears in a preceding `reg` / `wire` / `input` / `output` declaration;
  - no `reg` declaration appears after the first executable statement inside a function;
  - no width mismatch is emitted for concatenation operands (checked by yosys warnings already, but captured as a spec).
- The spec itself must contain `test`/`invariant`/`bench` per L4 TESTABILITY; the witness is the yosys smoke gate result.
- Verify with `./scripts/tri test --fast`.

### Task 5 — Reseal and full verification
- Run `cargo build --release`, `cargo test -p t27c`.
- Run `./scripts/tri test --fast` and `./scripts/tri test`.
- Reseal all affected specs whose `gen_hash_verilog` changed legitimately.
- Refreeze `bootstrap/stage0/FROZEN_HASH`.

### Task 6 — Close-out and next-wave cooperation variants
- Write `docs/reports/WAVE_LOOP_474_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W475_2026-07-08.md` with Variants A/B/C.
- Update `.trinity/experience.md`, `.trinity/ring-474.md`, `docs/NOW.md`, `.trinity/current-issue.md`.
- Update `~/.claude/projects/-Users-playra-t27/memory/` and `MEMORY.md`.
- Create `wave-loop-475` branch.

## Acceptance criteria
- `./scripts/tri test --fast`: ≥633/633 non-smoke PASS, 113/113 or more yosys smoke PASS, 0 seal mismatches, ACCEPTABLE: yes.
- Full `./scripts/tri test`: same green result plus FPGA standalone lake build OK.
- New scratch specs pass yosys smoke and have sealed JSON files.
- Close-out artifacts committed and branch `wave-loop-475` exists.

## Scientific / engineering references
- CompCert memory-model / verified-compilation line (Leroy & Blazy; Besson et al.; Monniaux) — semantic preservation during lowering is the formal target t27 has not yet reached.
- Intel / AMD Vitis HLS memory-layout docs — AoS/SoA/AoSoA transformations are standard in HLS; t27's per-field memory model matches SoA at the leaf level.
- Sparkle HDL / Verilean and CktFormalizer — the closest Lean-native formal-HDL competitors; no balanced-ternary / sealed-conformance competitor exists.
- Froleyks, Yu, Biere — ternary simulation as abstract interpretation (0/1/X), not ternary-weight compute.

---

*φ² + φ⁻² = 3 | TRINITY*
