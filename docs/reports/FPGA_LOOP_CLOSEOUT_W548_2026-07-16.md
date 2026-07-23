# Wave Loop 548 Closeout — Multi-dimensional primitive scalar array function returns

**Issue:** #1519  
**Branch:** `wave-loop-548`  
**Closeout date:** 2026-07-16  
**Source variant:** `docs/reports/FPGA_LOOP_COOPERATION_W548_2026-07-07.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 548 extended W545–W547's primitive scalar array function-return support
from one-dimensional arrays to multi-dimensional arrays (`[N][M]T`) in the
Icarus-lowerable subset.  The work fixed two independent indexing bugs, added
positive scratch witnesses for unsigned and signed 2-D returns, recorded
Icarus baselines, added Rust classifier integration tests, and proved
lowerability and value preservation in Lean 4.

---

## Weak points investigated

1. **Verilog backend bit-slice linearization was wrong for 2-D packed primitive
   arrays.**  `try_emit_primitive_array_access` in `bootstrap/src/compiler.rs`
   emitted `m[((i * 3) + j * 8) +:8]` for `[2][3]u8`, scaling the *inner* index by
   the element width instead of scaling the flat element index.
2. **Python reference model did not handle multi-D primitive array indexing.**
   `_eval_index_bv` in `scripts/cocotb_ref_model.py` explicitly gave up when
   `len(dims) > 1`, breaking the independent VCD cross-check for any 2-D element
   access.
3. **No witness exercised the end-to-end path.**  Existing 2-D witnesses were
   scalar-struct arrays; primitive scalar multi-D returns were unrepresented.

## Scientific grounding

- **CIRCT `hw.array_create` / `hw.array_get`.**  Aggregate arrays lower to flat
  bit-vectors; element extraction uses a linearized row-major index scaled by
  element width.  t27 now mirrors this for primitive scalar arrays.
  [CIRCT HW Dialect](https://circt.llvm.org/docs/Dialects/HW/)
- **IEEE 1800-2017 packed vectors.**  Variable part-selects (`[base +: width]`)
  require a *bit* base address, so the flat element index must be multiplied by
  `elemW`.  [IEEE 1800-2017](https://ieeexplore.ieee.org/document/8299595)
- **Vitis HLS array partitioning.**  Multi-dimensional arrays flatten into a
  single bank/vector; the packed-vector lowering in t27 is the register-file
  analogue.  [Vitis HLS](https://docs.xilinx.com/r/en-US/ug1399-vitis-hls/Arrays-and-Structs)

---

## Deliverables completed

1. **`bootstrap/src/compiler.rs`** — fixed `try_emit_primitive_array_access` so
   the variable part-select base is `flat_idx * elem_w`, producing the correct
   `m[(((i * 3) + j) * 8) +: 8]` for `[2][3]u8`.
2. **`bootstrap/stage0/FROZEN_HASH`** — updated after the compiler edit.
3. **`scripts/cocotb_ref_model.py`** —
   - added `_collect_index_chain` to gather all indices from a nested
     `ExprIndex` chain in source order;
   - rewrote `_eval_index_bv` to compute the row-major flat element index across
     all dimensions and extract the signed/unsigned bit slice;
   - updated `_eval_array_lit_bv` to recursively pack multi-dimensional literals
     while keeping one-dimensional scalar arrays masked to their element width.
4. **Scratch witnesses**
   - `specs/scratch/w548_2d_call_init_returns_array.t27`
   - `specs/scratch/w548_2d_signed_element_read.t27`
5. **Seals and baselines**
   - `.trinity/seals/scratch_w548_2d_call_init_returns_array.json`
   - `.trinity/seals/scratch_w548_2d_signed_element_read.json`
   - `.trinity/icarus-baselines/specs/scratch/w548_2d_call_init_returns_array.json`
   - `.trinity/icarus-baselines/specs/scratch/w548_2d_signed_element_read.json`
6. **Rust integration test** — `accepts_w548_multi_dimensional_primitive_scalar_array_return`
   in `bootstrap/tests/icarus_lowerable.rs`.
7. **Lean 4 formal witnesses** — `w548TwoDCallInitReturnsArray*` and
   `w548TwoDSignedElementRead*` helpers in `Lemmas.lean`, plus lowerability and
   value-preservation theorems in `Soundness.lean`.

---

## Validation matrix

| Check | Result |
|-------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 8 passed; 0 failed |
| `lake build Trinity.IcarusLowerable.Soundness` | 8572 jobs; 0 sorry |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | Icarus 56/56 PASS; cocotb 56/56 PASS; seal 636/636; 24 pre-existing yosys smoke failures unchanged |

---

## Cooperation variants for Wave Loop 549

### Variant A — 3-D primitive scalar array function returns (recommended)

**Hypothesis.**  W548 closed the 2-D case by generalizing the index chain and
bit-slice scaling.  Extending the same row-major linearization to three
dimensions (`[2][3][4]u8`) is a straight geometric continuation and exercises the
multi-dimensional path with more than two indices.

**Scientific grounding.**  HLS and CIRCT flatten arbitrary-rank arrays to a single
linear address space; the general formula `flat = Σ idx[k] * Π dims[k+1:]` works
for any rank.  [CIRCT HW Dialect](https://circt.llvm.org/docs/Dialects/HW/)

**Deliverables.**
1. Add `specs/scratch/w549_3d_call_init_returns_array.t27` positive witness.
2. Extend the Python reference model tests to ensure `_collect_index_chain`
   handles three-level `ExprIndex` nesting.
3. Add Lean 4 helpers and value-preservation theorem for a 3-D return.
4. Record Icarus baseline and seal.

**Validation contract.**
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the new 3-D witness.
- `lake build Trinity.IcarusLowerable.Soundness` stays green / 0 `sorry`.
- No regression in existing 1-D/2-D primitive array witnesses.

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
2. Add `specs/scratch/w549_bench_scalar_call_cross_check.t27` positive witness.
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
2. Add `specs/scratch/w549_signed_probe_reg.t27` positive witness with a signed
probe expression.
3. Reseal affected specs.

**Validation contract.**
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the new signed-probe witness.
- No change in existing cocotb probe count or values.

---

## Recommendation

**Choose Variant A for Wave Loop 549.**  It is the natural geometric
continuation of W545/W546/W547/W548, has a small and focused deliverable (one
3-D witness + one Lean theorem), and provides the highest confidence that the
multi-dimensional index linearization is truly rank-independent.

---

*φ² + φ⁻² = 3 | TRINITY*
