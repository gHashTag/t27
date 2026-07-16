# Wave Loop 550 Closeout — 4-D primitive scalar array function returns

**Issue:** #1521  
**Branch:** `wave-loop-550`  
**Closeout date:** 2026-07-16  
**Source variant:** `docs/reports/FPGA_LOOP_CLOSEOUT_W549_2026-07-16.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 550 extended Wave Loops 545–549 to four-dimensional primitive scalar
array function returns (`[2][2][2][2]u8`).  The goal was to prove that the
row-major index linearization is truly rank-independent and to expose any latent
hard-coded 2-D or 3-D assumptions.  No such assumptions were found: the
classifier, compiler, reference model, and Lean formal model all handled rank 4
without code changes.  The loop added one positive scratch witness, one Rust
integration test, one Lean value-preservation theorem, an Icarus baseline, and a
seal.

---

## Weak points investigated

1. **Compiler rank independence.**  `try_emit_primitive_array_access` had only
   been proven for ranks 1–3.  Rank 4 exercises the same variable part-select
   with a deeper nested product: `(((a*4 + b)*2 + c)*2 + d) * 8`.
2. **Reference-model recursion.**  `_collect_index_chain` and `_eval_array_lit_bv`
   are recursive; rank 4 could have exposed a base-case bug or a Python
   recursion-depth issue.
3. **Lean formal model depth.**  The nested `.array` type and recursive
   `evalExpr .index` had only been evaluated to depth 3.  Rank 4 stresses the
   partial-evaluation fuel budget and the `widthOfType'` recursion.
4. **Classifier generalization.**  The structural `icarus-lowerable` classifier
   might normalize multi-dimensional types differently at rank 4.

## Scientific grounding

- **CIRCT `hw.array_create` / `hw.array_get`.**  Arbitrary-rank arrays lower to a
  flat bit-vector with a rank-independent linear address
  `flat = Σ idx[k] * Π dims[k+1:]`.  The generated Verilog for the 4-D witness
  confirms this formula.
  [CIRCT HW Dialect](https://circt.llvm.org/docs/Dialects/HW/)
- **Vitis HLS / Catapult C array flattening.**  4-D and higher arrays are
  commonly flattened in image/tensor processing kernels.  Row-major is the
  default layout absent partitioning pragmas.
  [Vitis HLS](https://docs.xilinx.com/r/en-US/ug1399-vitis-hls/Arrays-and-Structs)
- **IEEE 1800-2017 packed vectors and variable part-selects.**  For a
  `[2][2][2][2]u8` packed vector, element `[a][b][c][d]` starts at bit
  `(((a*4 + b)*2 + c)*2 + d) * 8`.  The generated code emits exactly this
  expression.
  [IEEE 1800-2017](https://ieeexplore.ieee.org/document/8299595)
- **Rank-polymorphism in array languages (APL / SaC / Futhark).**  Once flattening
  is rank-polymorphic, adding dimensions is a no-op for the compiler core.  The
  4-D witness confirms t27's packed-vector lowering shares this property.
  [Futhark array language](https://futhark-lang.org/)

---

## Deliverables completed

1. **Scratch witness**
   - `specs/scratch/w550_4d_call_init_returns_array.t27` — function returning
     `[2][2][2][2]u8`, local `let` binding, and corner-element sum.
2. **Seal and baseline**
   - `.trinity/seals/scratch_w550_4d_call_init_returns_array.json`
   - `.trinity/icarus-baselines/specs/scratch/w550_4d_call_init_returns_array.json`
3. **Rust integration test**
   - `accepts_w550_four_dimensional_primitive_scalar_array_return` in
     `bootstrap/tests/icarus_lowerable.rs`.
4. **Lean 4 formal witness**
   - `w550FourDCallInitReturnsArray*` helpers in `Lemmas.lean`.
   - Lowerability and value-preservation theorems in `Soundness.lean`.
5. **Plan artifact**
   - `.claude/plans/wave-loop-550.md` with weak points, literature, variants, and
     implementation sketch.

---

## Validation matrix

| Check | Result |
|-------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 10 passed; 0 failed |
| `lake build Trinity.IcarusLowerable.Soundness` | 8572 jobs; 0 sorry |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | Icarus 58/58 PASS; cocotb 58/58 PASS; seal 638/638; 24 pre-existing yosys smoke failures unchanged |

---

## Cooperation variants for Wave Loop 551

### Variant A — Independent VCD cross-check for deterministic `bench` blocks (recommended)

**Hypothesis.**  All W5xx cocotb cross-checks target `test` blocks.  `bench`
blocks contain latency/throughput assertions that are currently skipped by the
reference model, leaving no independent verification that generated Verilog
preserves expected cycle-level behavior for performance-sensitive specs.

**Scientific grounding.**
- **cocotb deterministic benchmarking.**  Fixed-input test sequences and cycle
counters can evaluate deterministic `bench` assertions without wall-clock timing.
[cocotb docs](https://docs.cocotb.org/)
- **RVFI-DII independent trace comparison.**  Separating reference-model and
implementation traces enables independent cycle-level verification.
[RVFI](https://github.com/SymbioticEDA/riscv-formal)

**Deliverables.**
1. Extend `scripts/cocotb_ref_model.py` to parse `bench` blocks and evaluate
deterministic assertions inside them, skipping non-deterministic or timing-only
benches.
2. Add `specs/scratch/w551_bench_scalar_call_cross_check.t27` positive witness.
3. Update `bootstrap/src/suite.rs` to run cocotb against `bench` blocks when
`--cocotb` is enabled.
4. Keep `test` and `bench` probes clearly distinguished in VCD output.

**Validation contract.**
- `./scripts/tri test --icarus-simulate --cocotb --fast` passes the new bench
witness.
- Existing `test` cocotb count remains unchanged.
- `cargo test -p t27c --bin t27c` and `cargo test -p tri` stay green.

---

### Variant B — Signedness alignment for typed VCD probe registers

**Hypothesis.**  W538/W539 introduced typed scalar VCD probes.  Probe `reg`
declarations are currently always unsigned even when the probe metadata marks
the expression as signed.  The cocotb cross-check works because the reference
model interprets raw VCD bits using the declared type, but the mismatch between
the actual Verilog declaration and the metadata is a latent correctness issue for
external VCD consumers and `$display` probe logs.

**Scientific grounding.**
- **IEEE 1800-2017 signed net declarations.**  A `reg signed [N:0]` declaration
preserves signedness in procedural assignments and `$display`.
[IEEE 1800-2017 §6.11](https://ieeexplore.ieee.org/document/8299595)
- **VCD value interpretation.**  VCD stores raw bit vectors; consumers rely on
declared signal type to interpret signed values.  Emitting signed probe regs
makes the generated Verilog self-describing.

**Deliverables.**
1. Emit `reg signed` for scalar VCD probes in `gen_verilog_test` when probe
metadata says signed.
2. Add `specs/scratch/w551_signed_probe_reg.t27` positive witness with a signed
probe expression.
3. Reseal affected specs.

**Validation contract.**
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the new signed-probe witness.
- No change in existing cocotb probe count or values.

---

### Variant C — Generic module-level const/var assignment for lowerable packed values

**Hypothesis.**  W544 introduced mutable module vars and test-block call
assignments, but whole-array assignment to a module-level `var` of lowerable
packed type from a function call is not yet exercised.  Closing this gap lets
module-level state hold small packed arrays/structs returned by helpers.

**Scientific grounding.**
- **Module-level state in HLS.**  Top-level variables with initializers and
assignment semantics are the closest RTL equivalent to module-level regs.
Adding assignments from function-call returns aligns t27 with how HLS tools
infer registers from scalar variables.
  [Vitis HLS](https://docs.xilinx.com/r/en-US/ug1399-vitis-hls/Arrays-and-Structs)

**Deliverables.**
1. Add `specs/scratch/w551_module_var_packed_call_assign.t27` where a module-level
   `var` of `[3]u8` / packed scalar struct type is reassigned from a function call
   inside a `test` block.
2. Update the Python reference model to track such assignments (extend
   `mutable_module_names` beyond `ConstDecl` to include mutable `VarDecl`).
3. Fix any Verilog backend gaps for module-level packed `var` assignment.

**Validation contract.**
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the new witness.
- Existing module-level const/var witnesses remain unchanged.

---

## Recommendation

**Choose Variant A for Wave Loop 551.**  After five loops proving rank-
independence of primitive scalar array returns, the most valuable next step is
broadening the verification surface from `test` blocks to deterministic
`bench` blocks.  This directly improves confidence in cycle-level behavior and
uses the same cocotb infrastructure already in place.

---

*φ² + φ⁻² = 3 | TRINITY*
