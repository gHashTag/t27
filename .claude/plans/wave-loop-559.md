# Wave Loop 559 Plan — Signed whole-array comparison for 3-D and 4-D arrays

Issue #1530 | branch `wave-loop-559` | next branch `wave-loop-560`

---

## Charter

Implement **Variant A**: extend the W555 whole-array `assert_eq` probe to
signed primitive scalar arrays of rank 3 and 4. Verify that the generated
Verilog captures the full packed vector correctly, that Icarus simulation passes,
and that the Python reference model reconstructs the expected 3-D/4-D signed
array literal from the VCD probes using the same row-major layout used by the
backend.

---

## Weak points discovered

1. **W555 only exercised 2-D arrays.**
   The existing witnesses (`w555_bench_whole_array_signed`, `_unsigned`, `_nested_call`,
   `_wide`) are all rank 2. The compiler code paths (`expr_width_signed`,
   `gen_verilog_expr` `ExprArrayLiteral`, `emit_packed_array_literal_concat`,
   `try_emit_primitive_array_access`) are rank-independent, but no regression
   spec locks rank-3/rank-4 whole-array behavior inside a `bench` block.
2. **The Python reference model has rank-3/rank-4 element-index support but no
   whole-array literal evaluator for ranks > 2.**
   `_eval_array_lit_bv` already recurses through arbitrary dimensions, and
   `_primitive_array_info` handles any rank. What is missing is an explicit
   cross-check that the signed multi-slice probe reconstruction matches a 3-D
   or 4-D signed array literal.
3. **Wide-probe slicing is rank-agnostic but must be tested above 64 bits.**
   A `[2][3][4]i8` array is only 48 bits, so it fits in one probe. A
   `[2][2][2][2]i32` array is 256 bits and will force four 64-bit slices,
   exercising the W540 reconstruction path at rank 4 with signed elements.
4. **Signed element semantics at whole-vector comparison are subtle.**
   Verilog compares a `reg signed [N:0]` as a signed vector, which matches a
   t27 `[...]i8` array only if every element is stored sign-extended in place.
   The backend already wraps element part-selects with `$signed(...)`; the
   whole-vector comparison must rely on the declared `signed` probe / temporary
   reg to preserve sign.
5. **Direct function-call actual expressions need the W557 temporary path.**
   `assert_eq(cube(), literal)` already triggers the packed call temporary for a
   rank-3 return, so this variant also confirms that call temporaries work for
   higher-rank arrays.

---

## Engineering / scientific background

- **Multi-dimensional packed arrays in SystemVerilog.** IEEE 1800-2017 treats a
  packed array as a contiguous vector. A slice of a signed packed array is
  unsigned unless wrapped in `$signed(...)`; the whole signed packed vector
  compares naturally as signed. The t27 backend therefore declares the probe / tmp
  reg with `signed` when the element type is signed and packs elements
  LSB-first, row-major.
- **cocotb / VCD wide packed array reconstruction.** cocotb presents packed
  arrays as flattened bit vectors; manual slicing and reshaping is required to
  recover multi-dimensional signed values (cocotb discussion #2933, PR #4746).
  t27's Python model already does this via `_eval_array_lit_bv`, which packs
  inner arrays recursively and uses `_primitive_array_info` to infer width and
  signedness.
- **Bit-accurate reference models for matrix/tensor hardware.** MMA-Sim
  (arXiv 2511.10909) validates GPU matrix cores against a software reference by
  comparing bit-accurate row-major packed results. The same principle applies
  here: the reference model computes the packed-vector value of the expected
  literal and compares it to the VCD-captured actual value.
- **Ara SIMD ALU signed packed comparison.** The Ara vector unit uses explicit
  `$signed(...)` wrappers around per-element packed slices when signedness
  matters, mirroring the t27 backend's `try_emit_primitive_array_access` path.

Sources:
- [Verilator packed-array regression](https://github.com/verilator/verilator/blob/dbd48233/test_regress/t/t_array_packed_sysfunct.v)
- [Ara SIMD ALU signed packed comparison](https://github.com/pulp-platform/ara/blob/ec0e37916b901961beab3974acc44ab5ea422db6/hardware/src/lane/simd_alu.sv)
- [MMA-Sim: Bit-Accurate Reference Model](https://ar5iv.labs.arxiv.org/html/2511.10909)
- [cocotb multidimensional packed arrays discussion](https://github.com/cocotb/cocotb/discussions/2933)
- [cocotb vpiPackedArray support PR](https://github.com/cocotb/cocotb/pull/4746)
- [IEEE 1800-2017 §7.4 / §11.7](https://fpga.mit.edu/6205/_static/F23/documentation/1800-2017.pdf)
- [VLSI SOURCE SystemVerilog arrays](https://vlsisource.com/systemverilog/arrays/)

---

## Implementation tasks

### A. Create W559 scratch witnesses

- `specs/scratch/w559_bench_whole_array_3d_signed.t27`:
  - `pub fn cube() -> [2][3][4]i8` returning a small 3-D signed array.
  - `test` and `bench` blocks asserting `assert_eq(tmp, literal)` where
    `tmp : [2][3][4]i8 = cube();`.
- `specs/scratch/w559_bench_whole_array_4d_signed.t27`:
  - `pub fn hyper() -> [2][2][2][2]i32` returning a 256-bit signed 4-D array.
  - `test` and `bench` blocks asserting equality against the literal.
- Optional `specs/scratch/w559_bench_whole_array_3d_signed_direct_call.t27`:
  - Same as the 3-D witness but the actual expression is `cube()` directly, to
    exercise the W557 packed call temporary for rank-3 returns.

### B. Validate compiler paths

No compiler change is expected. Verify that:
- `expr_width_signed(ExprIdentifier)` for `tmp` returns the full packed width
  and signed=true for `[2][3][4]i8` and `[2][2][2][2]i32`.
- `gen_verilog_expr(ExprArrayLiteral)` for the expected literal emits a packed
  concatenation with the correct nesting and `signed` element constants.
- The actual expression emits a single signed probe / tmp reg and that the
  comparison is a single `!=` between two signed packed vectors.

### C. Validate Python reference model

- Ensure `_eval_array_lit_bv` correctly packs the expected 3-D/4-D signed
  literals into a `Bv` with `signed=true`.
- Ensure the VCD reconstruction (`_read_vcd_probe`) combines multi-slice probes
  for the 4-D witness and that the resulting signed value matches the expected
  packed vector.

### D. Save seals and baselines

- Save t27 seals for each witness.
- Record Icarus baselines for the witnesses that pass the `gen-verilog` pre-flight
  without named test/bench locals (likely the direct-call variants). Witnesses
  with named `let` bindings may still pass direct simulation but be excluded
  from the automated suite tally (same W555 limitation).

### E. Add integration test

Add `accepts_w559_bench_whole_array_higher_rank_signed` in
`bootstrap/tests/icarus_lowerable.rs` covering all W559 witnesses.

### F. Validation matrix

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
- Direct `t27c icarus-simulate specs/scratch/w559_*.t27`
- Direct `t27c icarus-cocotb specs/scratch/w559_*.t27`
- `lake build Trinity.IcarusLowerable.Soundness`

---

## Three cooperation variants for Wave Loop 560

### Variant A — Recommended: scalar-struct return call deduplication
Apply the W556–W558 block-scoped call temporary machinery to lowerable packed
scalar-struct return calls used at multiple sites in a `test` or `bench` block.
The temporary would be a packed-vector register whose width equals the struct
element width.

### Variant B: whole-array comparison for array-typed struct fields
Extend the W555 whole-array probe to scalar-struct variables whose fields are
fixed-size scalar arrays, enabling `assert_eq(tmp, literal)` where `tmp` is a
scalar struct with array-typed fields.

### Variant C: timed/non-deterministic bench classifier
Introduce an AST classifier that rejects (or skips) `bench` blocks containing
`#` delays or unbounded loops from the deterministic cocotb gate, and update
`docs/ICARUS_LOWERABLE_BOUNDARY.md` to state that the W556–W558 deduplication
optimization is only valid for pure calls.

---

## Skills to save at closeout

Pattern: *"Extending a whole-array probe to higher ranks is primarily a
regression-lock wave: the existing rank-independent code paths usually already
support the new rank, but a witness with a wide signed 4-D array exercises both
the signed whole-vector comparison and the multi-slice VCD reconstruction at
once."*
