# Wave Loop 539 Closeout — Typed 64-bit VCD probe + full Python expression evaluator

**Issue:** #1510  
**Branch:** `wave-loop-539`  
**Status:** closed  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Delivered

1. **Typed VCD probe emission in `bootstrap/src/compiler.rs`**
   - Added `VerilogCodegen::expr_width_signed` and `field_scalar_array_info` to
     infer the scalar bit width and signedness of every `assert_eq` actual
     expression in the Icarus-lowerable subset.
   - Replaced the fixed `reg [63:0]` probe declarations with width-typed probes
     (`reg [W-1:0]` or scalar `reg`) and added `probe_specs` metadata.
   - Probes now correctly cover signed `i16` function returns, signed scalar
     array elements, and scalar-struct field accesses.

2. **Full Python expression evaluator in `scripts/cocotb_ref_model.py`**
   - Introduced a bit-vector value representation (`Bv`) that tracks width and
     signedness independently of Python's arbitrary-precision `int`.
   - Implemented a recursive evaluator for the Icarus-lowerable expression subset:
     literals, variables, parameterless function calls, struct field access,
     scalar array indexing, binary/unary operators, casts, switch, and ternary.
   - Added signed/unsigned comparison, signed division/remainder, and arithmetic
     right shift to match Verilog semantics.
   - Updated the VCD parser to record per-identifier widths and to interpret
     probe values with the correct width and signedness.

3. **Validation gates green**
   - `cargo build --release -p t27c`: green.
   - `cargo test -p t27c --bin t27c`: 1494 passed; 0 failed; 2 ignored.
   - `cargo test -p tri`: 78 passed; 0 failed.
   - `cargo test -p t27c --test icarus_lowerable`: 4 passed; 0 failed.
   - `./scripts/tri test --icarus-lowerable --cocotb --fast`: 35 Icarus PASS,
     35 cocotb PASS, 0 seal mismatches.
   - `lake build Trinity.IcarusLowerable.Soundness`: green, 0 `sorry`.
   - 24 pre-existing yosys smoke baseline failures remain unchanged.

4. **FROZEN_HASH updated**
   - `bootstrap/stage0/FROZEN_HASH` now reflects the new
     `bootstrap/src/compiler.rs` hash after the W539 changes.

---

## Literature used

1. **SyoSil, *Python-based verification environment using PyUVM and cocotb***  
   <https://www.syosil.com/images/resources/osv_whitepaper-1.0.3.0.pdf>
2. **Gadde et al., *Towards Efficient Design Verification – Constrained Random Verification using PyUVM***  
   <https://arxiv.org/html/2407.10317v1>
3. **DVCon EU 2025, *FPGA Firmware Verification: a common approach for simulation and hardware tests***  
   <https://dvcon-proceedings.org/wp-content/uploads/DVConEU_2025_paper_95.pdf>
4. **Verilator / cocotb issues on signed-width comparison**  
   <https://github.com/verilator/verilator/issues/5968>,  
   <https://github.com/verilator/verilator/issues/4174>,  
   <https://github.com/cocotb/cocotb/discussions/5268>
5. **angr `claripy`** as the bit-vector AST evaluation precedent  
   <https://api.angr.io/projects/claripy/en/latest/api.html>

---

## Known limitations / next-wave seeds

- The evaluator currently only handles parameterless function calls; a future
  wave can extend it to scalar arguments and module-level parameter-bound
  references.
- Wide packed-struct arrays are still skipped because a single probe cannot
  capture more than 64 bits.  Variant C (multi-signal probes) is documented in
  the cooperation report for Wave 540.
- The Python evaluator does not yet model loop-carried variables, `break`/
  `continue` flags, or bounded `while` loops; those assertions fall back to the
  authoritative log-based self-check.

---

*φ² + φ⁻² = 3 | TRINITY*
