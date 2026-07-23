# Wave Loop 549 Closeout — 3-D primitive scalar array function returns

**Issue:** #1520  
**Branch:** `wave-loop-549`  
**Closeout date:** 2026-07-16  
**Source variant:** `docs/reports/FPGA_LOOP_CLOSEOUT_W548_2026-07-16.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 549 extended Wave Loop 548's multi-dimensional primitive scalar array
function-return support to three-dimensional arrays (`[2][3][4]u8`).  The
implementation required no new backend code: the existing row-major
linearization in `bootstrap/src/compiler.rs` and the `_collect_index_chain`
helper in `scripts/cocotb_ref_model.py` generalized correctly to three indices.
The loop added a positive scratch witness, a Rust integration test, a Lean 4
value-preservation theorem, an Icarus baseline, and a seal.

---

## Weak points investigated

1. **Rank-independence of the compiler's flat-index formula.**  W548 only tested
   two dimensions; a 3-D return could have revealed an off-by-one in the inner
   dimension product or an incorrect nesting of `ExprIndex` chains.
2. **Reference-model index-chain walker.**  `_collect_index_chain` was only
   proven for two indices; a 3-D access `m[i][j][k]` could have exposed an
   ordering or recursion-depth bug.
3. **Recursive array-literal packing.**  `_eval_array_lit_bv` had to pack three
   levels of inner arrays while still masking the leaf 1-D scalar array children
   to the declared element width.
4. **Lean formal model nesting.**  The simplified AST had to represent
   `.array 2 (.array 3 (.array 4 .u8))` and the `evalExpr .index` semantics had
   to agree with the Verilog linearization.

## Scientific grounding

- **CIRCT `hw.array_create` / `hw.array_get`.**  Arbitrary-rank arrays lower to
  a flat bit-vector with a rank-independent linearized index
  `flat = Σ idx[k] * Π dims[k+1:]`.  t27 now exercises this for rank 3.
  [CIRCT HW Dialect](https://circt.llvm.org/docs/Dialects/HW/)
- **Vitis HLS array flattening.**  Multi-dimensional arrays are flattened into a
  single memory bank in row-major order; t27's packed-vector lowering is the
  register-file equivalent.  [Vitis HLS](https://docs.xilinx.com/r/en-US/ug1399-vitis-hls/Arrays-and-Structs)
- **IEEE 1800-2017 packed vectors.**  Variable part-selects require a bit base
  address, so the linearized element index must be scaled by element width for
  every rank.  The generated Verilog for the 3-D witness emits
  `m[(((0 * 12) + (0 * 4) + 0) * 8) +: 8]`, confirming the formula.
  [IEEE 1800-2017](https://ieeexplore.ieee.org/document/8299595)

---

## Deliverables completed

1. **Scratch witness**
   - `specs/scratch/w549_3d_call_init_returns_array.t27` — function returning
     `[2][3][4]u8`, local `let` binding, and corner-element sum.
2. **Seal and baseline**
   - `.trinity/seals/scratch_w549_3d_call_init_returns_array.json`
   - `.trinity/icarus-baselines/specs/scratch/w549_3d_call_init_returns_array.json`
3. **Rust integration test**
   - `accepts_w549_three_dimensional_primitive_scalar_array_return` in
     `bootstrap/tests/icarus_lowerable.rs`.
4. **Lean 4 formal witness**
   - `w549ThreeDCallInitReturnsArray*` helpers in `Lemmas.lean`.
   - Lowerability and value-preservation theorems in `Soundness.lean`.
5. **Plan artifact**
   - `.claude/plans/wave-loop-549.md` with weak points, literature, variants, and
     implementation sketch.

---

## Validation matrix

| Check | Result |
|-------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 9 passed; 0 failed |
| `lake build Trinity.IcarusLowerable.Soundness` | 8572 jobs; 0 sorry |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | Icarus 57/57 PASS; cocotb 57/57 PASS; seal 637/637; 24 pre-existing yosys smoke failures unchanged |

---

## Cooperation variants for Wave Loop 550

### Variant A — Arbitrary-rank primitive scalar array function returns (recommended)

**Hypothesis.**  W549 proved rank-3 works.  The next step is to make the
feature *explicitly* rank-independent by adding a negative/unusual shape (e.g.
`[2][2][2][2]u8`) and confirming the classifier, backend, reference model, and
Lean model all generalize without hand-coded 2-D or 3-D assumptions.

**Scientific grounding.**  CIRCT and HLS use the same rank-independent linear
address formula for any number of dimensions.  Testing rank 4 makes any latent
hard-coded rank assumption visible.
  [CIRCT HW Dialect](https://circt.llvm.org/docs/Dialects/HW/)

**Deliverables.**
1. Add `specs/scratch/w550_4d_call_init_returns_array.t27` positive witness.
2. If any hard-coded rank limit appears, remove it in `bootstrap/src/compiler.rs`,
   `scripts/cocotb_ref_model.py`, or the Lean model.
3. Add Lean 4 helper and value-preservation theorem for a 4-D return.
4. Record Icarus baseline and seal.

**Validation contract.**
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the new 4-D witness.
- `lake build Trinity.IcarusLowerable.Soundness` stays green / 0 `sorry`.
- No regression in existing 1-D/2-D/3-D primitive array witnesses.

---

### Variant B — Independent VCD cross-check for deterministic `bench` blocks

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
2. Add `specs/scratch/w550_bench_scalar_call_cross_check.t27` positive witness.
3. Update `bootstrap/src/suite.rs` to run cocotb against `bench` blocks when
`--cocotb` is enabled.
4. Keep `test` and `bench` probes clearly distinguished in VCD output.

**Validation contract.**
- `./scripts/tri test --icarus-simulate --cocotb --fast` passes the new bench
witness.
- Existing `test` cocotb count remains unchanged.
- `cargo test -p t27c --bin t27c` and `cargo test -p tri` stay green.

---

### Variant C — Signedness alignment for typed VCD probe registers

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
2. Add `specs/scratch/w550_signed_probe_reg.t27` positive witness with a signed
probe expression.
3. Reseal affected specs.

**Validation contract.**
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the new signed-probe witness.
- No change in existing cocotb probe count or values.

---

## Recommendation

**Choose Variant A for Wave Loop 550.**  It pushes the rank-independence
boundary one step further and is the natural continuation of W545/W546/W547/
W548/W549.  If rank 4 passes without code changes, we have strong evidence that
the linearization is truly general; if it fails, the failure will pinpoint the
remaining hard-coded assumption.

---

*φ² + φ⁻² = 3 | TRINITY*
