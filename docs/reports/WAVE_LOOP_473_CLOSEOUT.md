# Wave Loop 473 — Close-out Report (2026-07-08)

**Issue:** #1447  
**Branch:** `wave-loop-473`  
**Variant selected:** B — compiler-backend aggregate hardening (bench still blocked)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 473 closed the last practical gap in the `gen-verilog` aggregate-lowering line for module-level arrays of structs with array-typed fields. The bench remains physically blocked (no DLC10 cable / unwired P12 relay), so Variant B was selected by default.

The key realization this wave is that multi-dimensional outer arrays of structs (`[2][3]Shape`) cannot be lowered as nested unpacked memories. The per-leaf per-element memory model (`shapes_pts [0:5][0:2]`) has exactly one outer dimension; all outer array coordinates must be linearized into that first index before the inner field indices and the final scalar bit-slice are applied. Once both the read path and the write path share the same linearization rule, literal-index and variable-index assignment become symmetric and synthesizable.

The conformance suite is green at **633/633** non-smoke specs and **113/113** yosys smoke targets, with **zero** gen-verilog smoke failures and **zero** seal mismatches.

---

## What landed

### `bootstrap/src/compiler.rs`

- Added `module_struct_array_dims: HashMap<String, Vec<(usize, String)>>` to store the outer array dimensions of every module-level array of structs.
- Initialized and cleared the new registry in `VerilogCodegen::new()` and per-module reset blocks.
- Populated `module_struct_array_dims` in `gen_verilog_const` and `gen_verilog_var` after registering `module_struct_array_fields` / `module_struct_array_elem_types`.
- Updated the deep nested field read path (around the W472 module-level struct-array block) to split collected index nodes into:
  - outer indices — linearized through `gen_verilog_multi_dim_index_expr` into the first memory dimension;
  - inner indices — emitted as direct memory indices;
  - constant leaf bit slice — emitted as `[high:low]`.
- Rewrote `gen_verilog_try_struct_array_assign` to first collect a deep module-level path via `collect_field_index_path`, then emit the same linearized outer index + inner index + bit-slice assignment for both scalar leaf fields and array-typed nested fields.
- The write path now emits legal procedural targets such as `shapes_pts[((i * 3) + j)][k][31:16] = v;` instead of relying on the read-as-LHS fallback.

### Regression specs

- `specs/scratch/w473_module_var_struct_array_field_write.t27`  
  Literal-index write/read-back for `shapes[1].pts[2].x = v`.
- `specs/scratch/w473_module_var_struct_array_field_varidx_write.t27`  
  Variable-index write/read-back for `shapes[i].pts[j].x = v`.
- `specs/scratch/w473_3d_module_var_struct_array.t27`  
  Read-only higher-dimensional `[2][3]Shape` with nested `grid[i][j].pts[k].x`.
- `specs/scratch/w473_3d_module_var_struct_array_write.t27`  
  Write/read-back for `[2][3]Shape` with variable outer and inner indices.

### Seals and stage-0 hash

- All affected `.trinity/seals/*.json` files were resealed to the new gen-verilog output.
- `bootstrap/stage0/FROZEN_HASH` was refrozen to  
  `826ece852792c69a2361676983ec146c432a481e4bc4a29a677514258ae70680`.

---

## Weak spots and related work

### Project weak spots

- **Physical boot-evidence gap.** The strongest differentiation — live cold-POR CCLK sweeps on the Wukong XC7A100T — is still gated by missing hardware (DLC10 cable / unwired P12 relay). This has been the dominant blocker for nine consecutive waves. Every compiler-backend wave extends the tooling, but it does not close the physical evidence loop.
- **Lean ↔ Verilog semantic bridge.** The compiler backend is tested by simulation and yosys elaboration, but there is no formal proof that the per-field memory model preserves source read/write semantics for arrays of structs. This is the most important formal gap left in the aggregate-lowering line.
- **Adversarial witness for generated Verilog.** The smoke gate checks yosys acceptance and a small set of allowed warnings. It does not yet systematically scan emitted Verilog for undeclared identifiers, width mismatches, or illegal inline declarations before simulation.
- **Master-merge divergence.** A related but independent fix set exists on `master` (`701d79b3b`) for earlier gen-verilog defects. It was repeatedly rejected as a single-wave merge because it is insufficient for the then-current baseline and risky relative to the wave-line sub-fixes. The wave-line branch now has a cleaner zero-failure baseline, so a future re-integration strategy should be planned explicitly.

### Scientific / engineering context

- The ternary/ternary-trit HDL space remains thin in the literature. The closest public competitors are Sparkle HDL and Verilean, both Lean-native hardware-description experiments. No published work has demonstrated a spec-to-bitstream pipeline for ternary-weighted neural accelerators with sealed numeric conformance, which is t27's core claim.
- The struct-of-arrays vs array-of-structs lowering question is standard in high-level synthesis (e.g., Vivado HLS, LegUp, Circt `firtool`). t27's current backend uses a strict struct-of-arrays decomposition at the leaf-field level, which matches the register/memory model of Verilog and avoids packed arrays of structs that most synthesizers reject. The remaining formal work is to prove that this decomposition is semantics-preserving.
- Recent Lean 4 native compiler advances make a verified shallow embedding of the t27 memory model feasible as a next-wave formal target.

---

## Not done (blocked or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Live-capture `XADC_LIVE_W473_OPERATING_POINT` — bench unavailable.
- Lean 4 synthesizability/correctness lemmas for the per-field memory model — deferred to W474 Variant C if selected.
- Master-merge of the `master` gen-verilog fix set — still deferred; should be planned as its own small wave rather than merged opportunistically.

---

## Verification

- `cargo build --release`: **PASS**.
- `cargo test -p t27c`: **1871 passed, 0 failed, 2 ignored**.
- `./scripts/tri test --fast`: **ALL TESTS PASSED**
  - Parse / Typecheck / Gen Zig / Gen Rust / Gen Verilog / Gen C / Seal Verify: **633/633 PASS**.
  - Gen Verilog Yosys Smoke: **113 passed, 0 failed**.
  - FPGA Board-Less Smoke Gate: **OK**.
  - Fixed Point: 0 divergences.
  - **TOTAL FAILURES: 0** — `BASELINE FAILURES: 0`, `ACCEPTABLE: yes`.
- Full `./scripts/tri test`: **ALL TESTS PASSED**
  - 633/633 parse/typecheck/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify PASS.
  - Gen Verilog Yosys Smoke: **113 passed, 0 failed**.
  - FPGA Board-Less Smoke Gate: **OK**.
  - FPGA Standalone Lake-Package Build: **OK**.
  - Fixed Point: 0 divergences.
  - **TOTAL FAILURES: 0** — `BASELINE FAILURES: 0`, `ACCEPTABLE: yes`.

---

## Close-out artifacts

- `docs/reports/WAVE_LOOP_473_CLOSEOUT.md` (this file)
- `docs/reports/FPGA_LOOP_COOPERATION_W474_2026-07-08.md`
- `.trinity/ring-473.md`
- `.trinity/experience.md` (appended)
- `~/.claude/projects/-Users-playra-t27/memory/wave-loop-473.md`

---

## Next wave

- **Branch:** `wave-loop-474`
- **Plan:** `docs/reports/FPGA_LOOP_COOPERATION_W474_2026-07-08.md`

---

*φ² + φ⁻² = 3 | TRINITY*
