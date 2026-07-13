# Wave Loop 517 — Packed AOS parameter whole-array-field reads

**Issue:** #1486 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-517` created from `wave-loop-516`  
**Variant:** A (recommended) from `docs/reports/FPGA_LOOP_COOPERATION_W517_2026-07-07.md`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Enable reading a complete fixed-size scalar array field from a packed
array-of-structs **function parameter** and returning it as a packed vector.
Example that must lower cleanly:

```t27
pub struct S {
    tag : u32,
    coords : [3]u32,
}

fn read_aos_coords(arr : [2]S, i : u8) -> [3]u32 {
    return arr[i].coords;
}
```

After W516 the same pattern already works for **function-local** AOS; the
residual boundary is parameters.

---

## Scientific anchors

- **IEEE Std 1800-2017 §7.4 / §7.4.4 / §7.4.6 / §11.5.1 / §11.5.2**: a packed
  array/struct is a contiguous vector; an unpacked memory is indexed by word
  first, then part-selected inside the packed word. Whole-field reads of a
  packed subfield are synthesizable as a constant or variable part-select.
- **Stuart Sutherland, *Synthesizable SystemVerilog* (SNUG 2013)** and
  **Sutherland/Mills, *Can My Synthesis Compiler Do That?* (DVCon 2014)**: packed
  arrays with subfields synthesize to flat vectors; element/subfield selection
  is portable across synthesis tools.
- **Michael J.C. Gordon, *The Semantic Challenge of Verilog HDL* (LICS 1995)**:
  foundational treatment of Verilog vectors and memories as vectors-of-vectors,
  relevant to why part-select must address the packed word after indexing the
  memory.
- **Icarus Verilog quirks / issue #536 / #298**: Icarus supports part-select of
  a single memory word (`mem[i][7:0]`) but rejects slices across unpacked
  dimensions. Our lowering keeps AOS parameters as one packed vector, so it
  avoids the problematic unpacked-memory slice.

---

## Residual boundary

In `bootstrap/src/compiler.rs` the helper `array_of_struct_field_slice` (line
~8995) explicitly rejects reading a whole array-typed direct field unless the
index list includes every inner index:

```rust
let inner_count = f_dims.len();
if indices.len() != dims.len() + inner_count { return None; }
```

For `arr[i].coords` (`indices.len() == 1`, `dims.len() == 1`,
`inner_count == 1`) this returns `None`. `try_emit_local_packed_array_param_field`
falls back to the generic field-access path, which sees an unresolved parameter
base and emits an `UNSUPPORTED_ICARUS` placeholder.

---

## Decomposed plan

### Phase 1 — OBSERVE / probe
- Create two scratch witnesses in `specs/scratch/`:
  - `w517_param_aos_array_field_read.t27` — function with AOS parameter,
    variable outer index, return whole `coords` field.
  - `w517_bench_local_aos_param_array_field_read.t27` — bench-local AOS passed
    to such a function (exercises the `__local__` binding path for bench-local
    arrays).
- Generate Verilog and run `t27c icarus-lowerable` to confirm the current
  failure mode (`UNSUPPORTED_ICARUS: unresolved field access arr.coords`).

### Phase 2 — Backend fix
- Modify `array_of_struct_field_slice` in `bootstrap/src/compiler.rs`:
  - When the matched field is array-typed and `indices.len() == dims.len()`,
    treat it as a whole-field read.
  - Compute `field_width` as the total packed width of the field
    (`packed_width(field_type, struct_fields)`).
  - Set `field_offset` to the field's packed offset; no inner offset needed.
- The variable-index mux path in `try_emit_local_packed_array_param_field`
  already iterates over outer-index combinations and calls
  `array_of_struct_field_slice`; supporting the whole-field case there will
  automatically extend the mux branches to slice the full field width.

### Phase 3 — Local init / return compatibility
- Verify that `var c : [3]u32 = read_aos_coords(arr, i)` unpacks correctly:
  the W516 scalar-array call-init path should handle a `[3]u32` return value.
- If not, extend the local-init path to unpack a packed scalar-array temporary
  into per-element local registers.

### Phase 4 — Test / classify
- `cargo test -p t27c --bin t27c`
- `./scripts/tri verify --lean-lowerable`
- `./scripts/tri test --icarus-lowerable --fast`
- Per-spec `t27c icarus-lowerable --json` for the two new witnesses.

### Phase 5 — Reseal
- Save seals for the new scratch specs.
- Reseal any existing non-scratch specs whose generated Verilog layout changed.

### Phase 6 — Closeout / cooperation variants
- Write `docs/reports/WAVE_LOOP_517_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W518_2026-07-07.md` with three
  proposed W518 variants.
- Update `.trinity/current-issue.md` to W518.

### Phase 7 — Learn / memory
- Update `.trinity/experience.md` with the AOS parameter field-width lesson.
- Save W517 memory file and update `MEMORY.md` index.

### Phase 8 — Land
- Create branch `wave-loop-517` from `wave-loop-516`.
- Stage, run L1/L3 commit hook, and commit with `Closes #1486`.

---

## Expected final gates

| Gate | Expected result |
|------|-----------------|
| `cargo test -p t27c --bin t27c` | 1525 passed, 0 failed, 2 ignored |
| `lake build Trinity.IcarusLowerable.Soundness` | green, zero `sorry` |
| `./scripts/tri verify --lean-lowerable` | passed, lowerable count stable |
| `./scripts/tri test --icarus-lowerable --fast` | acceptable; no new failures; new witnesses lowerable |

---

*φ² + φ⁻² = 3 | TRINITY*
