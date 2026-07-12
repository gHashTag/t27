# Wave Loop 500 Close-out Report

| Field | Value |
|-------|-------|
| Issue | #1458 |
| Branch | `wave-loop-500` |
| Ring | 12 (gen-verilog / Icarus semantics) |
| Date | 2026-07-13 |
| Anchor | φ² + φ⁻² = 3 | TRINITY |

---

## 1. What was attempted

Wave Loop 499 closed with a single documented Icarus baseline failure:
`specs/scratch/w493_local_aos_element_field_not_lowerable.t27`. The spec
exercises a local array of scalar structs whose indexed element is used inside
a struct literal:

```t27
pub fn make_outer(i : u32) -> Outer {
    let choices : [2]Inner = make_choices();
    return Outer { x: choices[i] };
}
```

The local array is lowered in **register mode**: each element's scalar fields
are emitted as per-element registers (`choices_0_y`, `choices_1_y`). The
struct-literal leaf emitter did not recognize a local register-mode
array-of-struct element, so it fell back to an `UNSUPPORTED_ICARUS` placeholder.

Wave Loop 500 added the missing recognition and re-packing path, closed the
boundary, and renamed the witness to reflect its new lowerable status.

---

## 2. What was actually changed

### 2.1 Verilog emitter (`bootstrap/src/compiler.rs`)

- `gen_verilog_pack_struct_array_element` now distinguishes three storage
  layouts:
  1. **Module-level AOS** — flat per-field memories (`base_flatfield[addr]`).
  2. **Local memory-mode AOS** — direct per-field memories
     (`base_field[addr][inner]` for array-typed fields).
  3. **Local register-mode AOS** — per-element per-field registers
     (`base_idx_flatfield`).
- For register mode, the function flattens the element struct fields at pack
  time and emits the correct register names. Literal indices produce a single
  concatenation; variable indices produce the existing priority mux over all
  possible element positions.
- The variable-index mux fallback zero is now sized
  (`{N{1'b0}}`) to avoid the Icarus error
  "Concatenation operand has indefinite width" that unsized `0` triggers
  inside a concatenation/ternary.
- Existing module-level and memory-mode paths are unchanged.

### 2.2 Witness

- Renamed `specs/scratch/w493_local_aos_element_field_not_lowerable.t27` to
  `specs/scratch/w493_local_aos_element_field_lowerable.t27`.
- Updated module name, comments, and test block to document the closed
  boundary.
- Regenerated seal:
  `.trinity/seals/scratch_w493_local_aos_element_field_lowerable.json`.

### 2.3 Reseals

- `specs/scratch/w476_adversarial_aggregate_tail.t27`
- `specs/scratch/w476_nested_whole_struct_assign.t27`

These two specs also use variable-index AOS element packing. The sized zero
fallback changed their generated Verilog, so their seals were regenerated.

---

## 3. Literature / related work

The fix is an instance of a common compiler-backend pattern: the same logical
value (a struct) has more than one concrete storage layout (flat memory vs.
per-element registers), and the code that re-materializes the value as a
packed vector must branch on the layout. The register-mode path mirrors the
per-element register naming already used for local scalar arrays and for local
AOS declarations; the packer now mirrors that naming instead of assuming a
memory-style index.

The sized-zero fallback follows Verilog width rules: unsized constants inside
concatenations or ternary branches have indefinite width and are rejected by
Icarus. Using a replication constant (`{N{1'b0}}`) gives the fallback the same
width as the struct's packed representation.

---

## 4. Verification results

| Gate | Result |
|------|--------|
| `lake build Trinity.IcarusLowerable.Soundness` | green, zero `sorry` |
| `./scripts/tri verify --lean-lowerable` | passed, 253 lowerable specs |
| `./scripts/tri test` non-smoke | 698 / 698 PASS |
| `./scripts/tri test` yosys smoke | 178 / 178 PASS (0 baseline) |
| `./scripts/tri test` Icarus smoke | 178 / 178 PASS (0 baseline) |
| `./scripts/tri test` seal verify | 698 / 698 match |
| FPGA board-less smoke gate / replay | OK |
| FPGA standalone lake-package build | OK |
| `cargo test -p t27c --bin t27c` | 1525 / 0 / 2 |

The gen-verilog Icarus smoke gate now has **zero documented baseline
failures**.

---

## 5. Residual boundaries

- The generic equivalence theorem still assumes `main` is not host-only.
- Conditionals and loops remain outside the modeled operational semantics.
- Register-mode re-packing covers scalar-struct AOS elements; array-typed
  direct fields continue to use memory-mode lowering.

---

## 6. Next-wave cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_W501_2026-07-13.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
