# Wave Loop 475 — Decomposed Plan (2026-07-07)

**Issue:** (to be opened)  
**Branch:** `wave-loop-475`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

## Variant selection

**Selected: Variant B** — continue compiler-backend aggregate hardening. The physical bench is still blocked (DLC10 cable / unwired P12 relay), so hardware Variant A is impossible. Variant B has a small, reviewable tail that directly extends the W455–W474 struct-array line.

## Goals

1. Array-of-struct equality for structs whose element struct has array-typed fields (e.g., `[2]Shape` where `Shape { pts : [3]Pt }`).
2. Whole-struct equality for nested structs with array-typed fields (e.g., `a == b` where `a : Shape`).
3. Function-local arrays of structs passed as array parameters, including memory-mode arrays with array-typed fields.
4. Adversarial yosys-elaboration witness that combines the new W475 paths.
5. Keep the suite green: ≥637/637 non-smoke PASS, zero yosys smoke failures, zero seal mismatches.

## Weak-spot analysis

- **Physical evidence gap.** The strongest differentiator (live cold-POR CCLK sweep → Lean theorem) remains blocked. This is a project-level risk, not a code risk.
- **No formal proof of per-field memory lowering.** The compiler emits SoA-style per-field memories, but there is no Lean theorem that the source read/write semantics are preserved. This is the largest scientific gap.
- **Equality for arrays with array-typed fields is not yet lowered.** The W474 equality path explicitly falls back for this case; extending it requires teaching the packer to read multi-dimensional field memories.
- **Master-merge debt.** An independent fix set lives on `master` (`701d79b3b`). It is rejected as a single-wave merge because it is insufficient for earlier baselines and risky relative to the wave-line sub-fixes. The wave line now has a zero-failure baseline, so a dedicated merge wave should be planned separately.
- **Literature context.** High-level synthesis (Vitis HLS, Intel HLS) routinely uses AoS/SoA/AoSoA layout transformations; Sparkle HDL (Lean 4) and CktFormalizer are the closest formal-HDL competitors. t27's unique intersection remains spec-first ternary compute + sealed numeric conformance + physical boot-evidence instrumentation.

## Decomposition

### Task 1 — Equality for arrays with array-typed fields
- Add a scratch spec `w475_aos_nested_equality.t27` with `==`/`!=` on `[2]Shape` where `Shape { pts : [3]Pt }`.
- Extend the W474 `ExprBinary` `==`/`!=` path so that `array_of_struct_has_array_field` no longer blocks lowering.
- Generalize `gen_verilog_pack_array_of_struct_expr` to read multi-dimensional field memories (`field[outer][inner]`) for array-typed fields, producing a packed vector that includes every inner element.
- Verify with `./scripts/tri test --fast`.

### Task 2 — Whole-struct equality for nested structs with array-typed fields
- Add a scratch spec `w475_struct_nested_equality.t27` with `==`/`!=` on scalar `Shape` values, including struct literals, variables, and function-return calls.
- Extend `gen_verilog_pack_scalar_struct_expr` (or add a dedicated path) to pack array-typed struct fields by reading their field memories or local per-field arrays.
- Verify with `./scripts/tri test --fast`.

### Task 3 — Function-local AOS passed as array parameters
- Add a scratch spec `w475_local_aos_param.t27` where a function accepts a `[N]Shape` parameter and is called with a local memory-mode array.
- Extend the array-parameter binding pass to handle local array identifiers as actuals, or lower the call by passing the per-field memory names to a clone.
- Ensure memory-mode local arrays with array-typed fields can be passed without flattening into scalar registers.
- Verify with `./scripts/tri test --fast`.

### Task 4 — Adversarial yosys-elaboration witness
- Add a regression spec `w475_adversarial_aos_param.t27` that combines local AOS parameter passing, nested-array-field equality, and module-level AOS return writeback.
- The yosys smoke gate is the witness; if the generated Verilog elaborates and synthesizes, the integration path is clean.
- Verify with `./scripts/tri test --fast`.

### Task 5 — Reseal and full verification
- Run `cargo build --release`, `cargo test -p t27c`.
- Run `./scripts/tri test --fast` and `./scripts/tri test`.
- Reseal all affected specs whose `gen_hash_verilog` changed legitimately.
- Refreeze `bootstrap/stage0/FROZEN_HASH`.

### Task 6 — Close-out and next-wave cooperation variants
- Write `docs/reports/WAVE_LOOP_475_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W476_2026-07-08.md` with Variants A/B/C.
- Update `.trinity/experience.md`, `.trinity/ring-475.md`, `docs/NOW.md`, `.trinity/current-issue.md`.
- Update `~/.claude/projects/-Users-playra-t27/memory/` and `MEMORY.md`.
- Create `wave-loop-476` branch.

## Acceptance criteria
- `./scripts/tri test --fast`: ≥637/637 non-smoke PASS, 117/117 or more yosys smoke PASS, 0 seal mismatches, ACCEPTABLE: yes.
- Full `./scripts/tri test`: same green result plus FPGA standalone lake build OK.
- New scratch specs pass yosys smoke and have sealed JSON files.
- Close-out artifacts committed and branch `wave-loop-476` exists.

## Scientific / engineering references
- CompCert memory-model / verified-compilation line (Leroy & Blazy; Besson et al.; Monniaux) — semantic preservation during lowering is the formal target t27 has not yet reached.
- Intel / AMD Vitis HLS memory-layout docs — AoS/SoA/AoSoA transformations are standard in HLS; t27's per-field memory model matches SoA at the leaf level.
- Sparkle HDL / Verilean and CktFormalizer — the closest Lean-native formal-HDL competitors; no balanced-ternary / sealed-conformance competitor exists.
- Froleyks, Yu, Biere — ternary simulation as abstract interpretation (0/1/X), not ternary-weight compute.

---

*φ² + φ⁻² = 3 | TRINITY*
