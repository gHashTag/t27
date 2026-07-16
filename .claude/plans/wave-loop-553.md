# Wave Loop 553 Plan — Signed/unsigned mixed bench probes

Issue #1524 | branch `wave-loop-553` | next branch `wave-loop-554`

---

## Charter

Verify that `$signed(...)` wrappers and VCD value reconstruction work correctly
inside deterministic `bench` blocks for signed scalar/array returns. Add
explicit signed/unsigned mixed witnesses to the Icarus/cocotb gate.

---

## Weak points discovered

1. **No signed bench witnesses.** Existing signed witnesses (W547, W532) only
   use `test` blocks. The `$signed(...)` / signed VCD path inside `bench` is
   untested end-to-end.
2. **Scalar signed return in bench untested.** A function returning `i8`/`i16`
   negative value inside a bench assertion — the Verilog probe is
   `reg signed [...]`, and the Python expected `Bv` is signed.
3. **Signed primitive array element in bench untested.** A bench asserting
   `a[0] == -1` for a `[3]i8` array requires `$signed(...)` on the Verilog slice
   and signed reconstruction from VCD.
4. **Signed packed scalar struct field in bench untested.** A bench asserting
   equality on a struct whose field is `[3]i16` with negative values.
5. **Mixed signed/unsigned in one bench untested.** No witness exercises both
   signed and unsigned asserts in the same block.

---

## Engineering / scientific background

* Two's-complement sign extension in Verilog: `$signed(...)` casts a bit vector
  to a signed interpretation, required for signed packed-array slices.
* VCD signed reconstruction: raw VCD bits are unsigned patterns; the reference
  model reinterprets them at the declared width using two's-complement. The
  `Bv.as_int()` method already implements this.
* Packed struct signedness: a packed vector representing a struct is emitted as
  unsigned, but individual field slices are `$signed(...)` when accessed. The
  Python model conservatively marks the whole packed value as signed if any
  field is signed, which is correct for the cross-check.

---

## Implementation tasks

### A. Create W553 scratch witnesses
Three specs under `specs/scratch/`:
* `w553_bench_signed_scalar_return.t27` — function returns a negative `i8`
  value; both `test` and `bench` assert it.
* `w553_bench_signed_array_element.t27` — function returns `[3]i8`; both
  `test` and `bench` assert `a[0] == -1`.
* `w553_bench_signed_struct_field.t27` — packed scalar struct `Pt { data: [3]i16 }`;
  both `test` and `bench` assert equality with a literal containing negative
  values.

### B. Reference model verification
* Confirm `_type_of_expr` returns signed width for scalar signed returns, array
  elements, and struct fields.
* Confirm `_eval_expr_bv` produces signed `Bv` values.
* Confirm `_interpret_vcd_value` sign-extends raw VCD bits using the expected
  `Bv` signedness.

### C. Compiler verification
* Confirm generated Verilog uses `reg signed [...]` for signed scalar probes.
* Confirm packed scalar struct field slices use `$signed(...)` when accessed.

### D. Baselines, seals, and integration tests
* Generate Icarus baselines for the three witnesses via the suite gate.
* Save t27 seals.
* Add `accepts_w553_bench_signed_cross_check` in
  `bootstrap/tests/icarus_lowerable.rs`.

### E. Validation matrix
* `cargo build --release -p t27c`
* `cargo test -p t27c --bin t27c`
* `cargo test -p tri`
* `cargo test -p t27c --test icarus_lowerable`
* `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
* `lake build Trinity.IcarusLowerable.Soundness`

---

## Three cooperation variants for Wave Loop 554

### Variant A — Recommended: bench-local primitive scalar arrays
Allow `let tmp : [N]T = f();` inside a `bench` block where `f` returns a packed
primitive scalar array, and cross-check element reads against the reference
model.

### Variant B: whole-array bench assignments
Support `assert_eq` on a complete 2-D primitive scalar array value (not just a
scalarized sum) inside a bench.

### Variant C: timed/non-deterministic bench classifier
Introduce an AST classifier that rejects (or skips) `bench` blocks containing
`#` delays or unbounded loops from the deterministic cocotb gate, and
 document the boundary.

---

## Skills to save at closeout

Pattern: *"Signed values in deterministic bench blocks share the same
`$signed(...)` wrapping and two's-complement VCD reconstruction as test blocks;
adding signed witnesses locks the behavior."*
