# Wave Loop 541 Closeout — Module-level wide packed values for independent VCD cross-check

**Issue:** #1512  
**Branch:** `wave-loop-541`  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was delivered

Wave Loop 540 enabled multi-signal VCD probes for function-returned packed scalar
structs and struct literals.  Wave Loop 541 closed the largest remaining gap:
module-level `const` and `var` declarations of lowerable packed scalar struct (or
fixed-size scalar array) type are now visible to the Python reference model, and
whole-struct assignments inside test blocks update that model before each
assertion.  As a result, assertions such as `assert_eq(src, Wide{...})`,
`assert_eq(dst, Wide{...})`, and `dst = make(); assert_eq(dst, Wide{...})` get an
independent VCD cross-check rather than relying only on the self-checking log.

### Compiler (`bootstrap/src/compiler.rs`)

- Extended `expr_width_signed` `ExprIdentifier` arm so that identifiers whose
  declared type is a lowerable packed scalar struct are sized to the packed vector
  width.  This triggers the W540 multi-slice probe emission for module-level wide
  values.
- Confirmed that `module_types` already contains both `const` and `var` declarations
  (module-level `var` is represented as a mutable `ConstDecl` in the AST).
- Updated `bootstrap/stage0/FROZEN_HASH` after the compiler surface change.

### Reference model (`scripts/cocotb_ref_model.py`)

- Added `_is_lowerable_scalar_struct_type`, `_packed_type_width_signed`, and
  `_contains_kind` helpers.
- `EvalContext.__init__` now binds module-level `const`/`var` initializers of
  lowerable packed scalar struct or fixed-size scalar array type into `self.vars`.
  Initializers that contain function calls are skipped to avoid recursive context
  construction.
- `EvalContext` tracks `mutable_module_names` so that whole-struct assignments to
  mutable module vars can update the model state.
- `_collect_assertions` now processes statements in order within each test block:
  preceding `StmtAssign` nodes to mutable module-level packed vars are evaluated and
  the binding is updated before the next `assert_eq` is recorded.
- `_type_of_expr` now returns packed widths for lowerable scalar structs and scalar
  arrays, and `_resolve_base_type` no longer loses the declared type when a module
  var is bound in `ctx.vars`.
- `_eval_struct_lit_bv` / `_eval_array_lit_bv` from W540 are reused to pack module
  initializer literals.

### Witnesses and seals

- `specs/scratch/w541_module_wide_struct_const.t27`: module-level `const` of an
  80-bit lowerable packed scalar struct, asserted against a matching struct literal.
- `specs/scratch/w541_module_wide_struct_var.t27`: module-level `var` initialized
  from the same kind of struct literal, asserted against a matching literal.
- `specs/scratch/w541_module_wide_struct_assign.t27`: module-level `var` receives a
  whole-struct assignment from a function call inside the test block.
- All three witnesses are sealed and have recorded Icarus baselines.

---

## Validation

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494 passed, 0 failed, 2 ignored |
| `cargo test -p tri` | 78 passed, 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 4 passed, 0 failed |
| `./scripts/tri test --icarus-lowerable --cocotb --fast` | 39 Icarus PASS, 0 FAIL; 39 cocotb PASS, 0 FAIL; 0 seal mismatches |
| `lake build Trinity.IcarusLowerable.Soundness` (in `proofs/lean4`) | 8572 jobs, 0 `sorry` |

The 24 pre-existing yosys smoke baseline failures are unchanged and documented.

---

## Weak points and literature

### Residual weak points

1. **Function-call initializers are skipped.**  A module-level `const src : Wide = make();`
   still falls back to log-only verification because evaluating the initializer would
   recursively create a new `EvalContext`.  A future wave can break this recursion by
   evaluating function calls without re-entering the full module-binding loop.

2. **Only whole packed values are tracked.**  Field-by-field assignments such as
   `dst.data[2] = 7;` are not reflected in the reference model, so assertions on the
   whole struct after such partial updates are not VCD-checked.

3. **Slice reconstruction assumes 64-bit aligned slices.**  This matches the current
   compiler emission but is an implicit contract between the Verilog backend and the
   Python parser.

4. **No formal connection between the Python reference model and Lean `module_value_equiv`.**
   The cocotb gate is a simulation-based oracle; the Lean proofs still cover the
   Icarus-lowerable combinatorial/sequential subset independently.

### Related scientific and industrial work

- **Leung, Bounov, Lerner — C-to-Verilog Translation Validation (MEMOCODE 2015).**
  Post-hoc equivalence checking between a high-level program and HLS-generated
  Verilog, closely analogous to the t27 cocotb reference-model gate.
  [DOI](https://doi.org/10.1109/memcod.2015.7340466)

- **Melchert et al. — Automated Translation Validation of a Compiler for Statically
  Scheduled Accelerators (FMCAD 2025).**
  SMT-based symbolic equivalence checking across every compiler stage of an
  accelerator toolchain; motivates moving from simulation baselines toward formal
  stage-by-stage validation.
  [NSF PAR](https://par.nsf.gov/servlets/purl/10663798)
  [PDF](https://repositum.tuwien.at/bitstream/20.500.12708/219556/1/Melchert%20Jackson%20-%202025%20-%20Automated%20Translation%20Validation%20of%20a%20Compiler%20for...pdf)

- **Herklotz et al. — Formal Verification of High-Level Synthesis (OOPSLA 2021).**
  The Vericert verified HLS compiler; underlines the importance of bit-accurate
  width/signedness tracking when connecting source semantics to Verilog.
  [PDF](https://johnwickerson.github.io/papers/vericert_oopsla21.pdf)
  [DOI](https://doi.org/10.1145/3485494)

- **Mishra, Dutt, et al. — A methodology for validation of microprocessors using
  equivalence checking (MTV 2003).**
  Generating a golden reference model from an ADL specification and comparing it to
  hand-written RTL; supports the approach of deriving the trusted oracle from the
  source language.
  [PDF](https://www.cise.ufl.edu/research/cad/Publications/mtv03.pdf)

- **VCDDiff (GitHub).**
  Industrial-style VCD waveform diff utility; the t27 `[PROBE]` slice reconstruction
  is a specialized, parser-free version of the same idea.
  [vcddiff](https://github.com/joonho3020/vcddiff)

---

## Next step

See `docs/reports/FPGA_LOOP_COOPERATION_W542_2026-07-07.md` for three concrete
cooperation variants and a recommended path for Wave Loop 542.

---

*φ² + φ⁻² = 3 | TRINITY*
