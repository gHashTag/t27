# Wave Loop 555 Plan — Whole-array bench assignments

Issue #1526 | branch `wave-loop-555` | next branch `wave-loop-556`

---

## Charter

Support `assert_eq` on a complete 2-D primitive scalar array value inside a
`bench` block. Reuse the W540 multi-slice VCD probe path to capture the wide
packed array in Icarus and reconstruct it in the Python reference model.

---

## Weak points discovered

1. **Wide array assert_eq is only exercised for scalar structs (W540).** A
   primitive scalar array is also a packed vector, but no witness asserts
   equality on the whole array value. The compiler's probe pre-declaration and
   the Python evaluator's reconstruction path are therefore untested for arrays.
2. **`expr_width_signed` treats primitive arrays as non-probe-able.** For an
   identifier that is a primitive scalar array, `expr_width_signed` returns
   `None`, so the probe pre-declaration skips the wide-path and falls through to
   scalar handling. This prevents the VCD probe from capturing the full packed
   value.
3. **`gen_verilog_expr` for array identifiers may not emit a valid packed-vector
   expression when the local is unpacked.** In W554 the local was packed
   (initialized from a function call). If the local is initialized by an array
   literal, the compiler lowers it as an unpacked Verilog array (`reg [7:0] tmp
   [0:2][0:3];`). A direct reference to `tmp` in an assignment is illegal; we
   must ensure the local chosen for whole-array comparison is packed.
4. **The Python reference model compares expected value against VCD slices but
   the expected literal may be a 2-D array literal.** `_eval_array_lit_bv`
   already packs multi-D literals into a `Bv` (W548), so the evaluator side is
   mostly ready, but `_type_of_expr` for an array identifier returns `None`,
   preventing the cross-check from knowing the full width/signedness of the
   actual expression.
5. **Signed whole-array comparison is untested.** W540 used an unsigned scalar
   struct. A signed primitive array like `[2][3]i8` needs the probe slices to be
   interpreted with the element signedness, not as unsigned bits.
6. **`assert_eq` code generation for wide values uses `!=` on packed vectors,
   but the failure diagnostic prints `%0d` on a wide vector, which is only the
   low 32 bits.** The diagnostic for wide arrays should be acceptable as long as
   the PASS/FAIL decision is correct, but it is a weak spot in debuggability.

---

## Engineering / scientific background

* **Packed-vector equality in Verilog.** A 2-D primitive scalar array that is
  stored as a single packed `reg [W-1:0]` can be compared with `==` against a
  concatenation of the same width. SystemVerilog packed arrays are guaranteed
  contiguous bits (Stack Overflow / AMD UG901), so the equality is well-defined.
* **Row-major bit-vector layout.** T27 stores element `[0]` at the LSB and
  concatenates higher indices to the left (MSB). Multi-dimensional literals are
  nested concatenations where the outermost dimension contributes the highest
  bits (W548). The compiler's `emit_packed_array_literal_concat` and the Python
  `_eval_array_lit_bv` already implement this consistently.
* **Deterministic bench cross-check.** The cocotb / direct-iverilog gate records
  VCD probes and compares them against a reference-model evaluation of the
  expected literal. Wide values are split into ≤64-bit slices because VCD scalar
  probes are easier to parse reliably than vector values (W540). The reference
  model reconstructs the full value from slice offsets.
* **Prior work.** FPGA verification methodologies commonly use a scoreboard
  pattern (DVCon EU 2025, "A Python based Design Verification Methodology") and
  probe interfaces for wide buses (NDK-FPGA, microcotb). T27's VCD probe
  approach is a lightweight deterministic instance of the same idea: a golden
  reference model evaluates expected values from the AST and compares against
  observed simulation VCD slices.

Sources:
- [Cocotb simulator support — waveform generation](https://docs.cocotb.org/en/stable/simulator_support.html)
- [NDK-FPGA cocotb tips & tricks (probe framework)](https://cesnet.github.io/ndk-fpga/devel/cocotb_tips_and_tricks.html)
- [DVCon EU 2025 — FPGA Firmware Verification: a common approach](https://dvcon-proceedings.org/wp-content/uploads/DVConEU_2025_paper_95.pdf)
- [A Python based Design Verification Methodology (2021)](https://doi.org/10.51201/jusst/21/05358)
- [microcotb — hardware-in-the-loop VCD probes](https://github.com/psychogenic/microcotb)
- [SystemVerilog packed vs unpacked arrays](https://stackoverflow.com/questions/477646/packed-vs-unpacked-vectors-in-system-verilog)
- [AMD Vivado UG901 — Packed and Unpacked Arrays](https://docs.amd.com/r/2022.2-English/ug901-vivado-synthesis/Packed-and-Unpacked-Arrays?contentId=9BCZVwsdbiywm3XQtiCvWg)
- [VLSI Trainers — SystemVerilog Series SV-23 (row-major file I/O)](https://vlsitrainers.com/systemverilog-series-%c2%b7-sv-23/)

---

## Implementation tasks

### A. Create W555 scratch witnesses

Three specs under `specs/scratch/`:

* `w555_bench_whole_array_unsigned.t27` — function returning `[2][3]u8`, bench
  `let tmp : [2][3]u8 = mat(); assert_eq(tmp, [2][3]u8{ ... });`.
* `w555_bench_whole_array_signed.t27` — function returning `[2][3]i8`, bench
  `let tmp : [2][3]i8 = mat(); assert_eq(tmp, [2][3]i8{ ... });`.
* `w555_bench_whole_array_nested_call.t27` — same, but the actual expression is
  the function call itself (`assert_eq(mat(), ...)`) to exercise the W553
  temporary + wide probe path.

Each witness includes an equivalent `test` block so the static assertion also
passes.

### B. Compiler changes

In `bootstrap/src/compiler.rs`:

1. Extend `expr_width_signed` for `ExprIdentifier` so that a primitive scalar
   array local/parameter/module variable returns its packed width and element
   signedness (matching `packed_width` / `packed_signed`).
2. Ensure `gen_verilog_probe_prelude` declares a wide multi-slice probe when the
   actual expression width exceeds 64 bits (it already does this for any
   `width > 64`, so step B1 enables it for array identifiers).
3. In `gen_verilog_test_stmt`, when emitting the assignment to the wide probe
   temporary, call `gen_verilog_expr` on the actual expression. For a primitive
   array identifier that is stored as an unpacked array, `gen_verilog_expr`
   currently may emit just the identifier. Verify / adjust so that the chosen
   local is packed. The simplest approach: require the witness to initialize the
   local from a function call (packed), which is the W555 focus. Document that
   array-literal-initialized whole-array comparison is out of scope for this
   wave.
4. Preserve `$signed(...)` for signed packed slices in the wide probe code path
   if needed. The probe slices are currently declared unsigned and the Python
   reference model uses the expected value's signedness for interpretation.

### C. Reference model changes

In `scripts/cocotb_ref_model.py`:

1. Extend `_type_of_expr` for `ExprIdentifier` to return the packed width and
   signedness when the identifier's declared type is a primitive scalar array.
   Use `_packed_type_width_signed` or `_scalar_array_info`.
2. Ensure `_eval_expr_bv` for an `ExprIdentifier` that is a primitive scalar
   array returns the whole packed `Bv` from `ctx.vars`. The local value is
   already bound by `_collect_assertions` using `_eval_expr_bv` on the
   initializer.
3. The VCD cross-check `_cross_check` already handles multi-slice probes and
   signed interpretation via the expected `Bv`; no change expected.

### D. Baselines, seals, integration test

* Run direct `t27c icarus-simulate` / `t27c icarus-cocotb` on the witnesses to
  confirm the multi-slice probes are captured and reconstructed.
* Save t27 seals for the new witnesses.
* Add `accepts_w555_bench_whole_array_cross_check` in
  `bootstrap/tests/icarus_lowerable.rs`.
* Note in the closeout report: W555 witnesses are direct-simulation only if the
  tri suite's `gen-verilog` pre-flight still rejects named test/bench locals (the
  same pre-existing limitation documented in W554).

### E. Validation matrix

* `cargo build --release -p t27c`
* `cargo test -p t27c --bin t27c`
* `cargo test -p tri`
* `cargo test -p t27c --test icarus_lowerable`
* `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
* Direct `./target/release/t27c icarus-simulate specs/scratch/w555_*.t27`
* Direct `./target/release/t27c icarus-cocotb specs/scratch/w555_*.t27`
* `lake build Trinity.IcarusLowerable.Soundness`

---

## Three cooperation variants for Wave Loop 556

### Variant A — Recommended: multi-site call-return array deduplication
When the same `f()` packed-array expression is indexed at multiple sites in
one bench, reuse a single packed temporary and emit only one assignment. The
W553 temporary map already deduplicates by call expression text; add a dedicated
witness with many reads and verify the temporary is assigned once.

### Variant B: signed whole-array comparison for higher ranks
Extend the W555 whole-array bench probe to 3-D and 4-D signed primitive scalar
arrays. Verify row-major slice reconstruction in the Python model for ranks 3
and 4.

### Variant C: timed/non-deterministic bench classifier
Introduce an AST classifier that rejects (or skips) `bench` blocks containing
`#` delays or unbounded loops from the deterministic cocotb gate, and document
the boundary in `docs/ICARUS_LOWERABLE_BOUNDARY.md`.

---

## Skills to save at closeout

Pattern: *"A whole-array `assert_eq` inside a `bench` block is just a wide
packed-vector VCD probe. Once `expr_width_signed` / `_type_of_expr` recognize a
primitive scalar array identifier as a probe-able packed vector, the W540 multi-
slice path handles capture and the existing `_eval_array_lit_bv` handles the
expected literal reconstruction."*
