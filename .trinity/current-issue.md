# Wave Loop 495 — Semantic equivalence for function calls and W493 witnesses

**Issue:** #1465  
**Branch:** `wave-loop-495`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Close the function-call equivalence gap left by Wave Loop 494. W494 defined a
scalar bit-vector semantics for the t27 AST and the shallow Verilog AST, and
proved value preservation for a scalar-struct-literal witness using
`native_decide`. W495 extends that result to the constructs that W493 made
Icarus-lowerable: function calls, struct-return field access, and struct-
literal fields initialized from scalar-struct identifiers/local variables/
module-level constants.

---

## Why now

The W493 compiler fixes proved that the backend can lower these patterns, but
the formal model still cannot evaluate a Verilog `function` call because the
shallow AST did not store function definitions. Once the model knows how to
inline a Verilog function body, the four W493 positive witnesses become
straightforward `native_decide` equivalence targets. This is the highest-
leverage follow-through from W494 and keeps the backend and formalization
tracks aligned.

---

## Scope

1. **Shallow Verilog AST** — add `VFunction` definitions to `VModule`.
2. **Emitter model** — emit t27 functions as `VFunction` definitions instead of
   flattening every reachable body into top-level items.
3. **Semantics**:
   - Generalize t27 `evalExpr` field access to infer the struct type of a
     function-call result (not only constructor calls).
   - Derive array element width from the base expression's type instead of
     hard-coding 8 bits.
   - Inline Verilog function bodies in `evalVExpr .call`.
   - Run a named function in `evalVModule` after evaluating module-level
     items.
4. **Witnesses** — model the four W493 positive specs in Lean and prove
   value preservation for `get_y()` / `main`:
   - `w493_nested_struct_field_from_identifier_lowerable`
   - `w493_local_scalar_struct_field_lowerable`
   - `w493_module_scalar_struct_field_lowerable`
   - `w493_module_aos_element_field_lowerable`
5. **Generic theorem** — state `Module.isLowerable env m →
   evalFunction env m main [] = evalVModule env (emitModule env m) "main"`
   and prove the base scalar case plus the function-call extension.

---

## Literature context

- **Lutsig** (Lööw, CPP 2021) shows that a verified Verilog-to-netlist compiler
  can carry a semantic-equivalence theorem in HOL4 — our goal is a much smaller
  source-to-shallow-Verilog equivalence, but the theorem shape is the same.
- **Vericert** (Herklotz et al., OOPSLA 2021) adapts the Lööw–Myreen Verilog
  semantics into CompCert and proves C-to-Verilog HLS correct; our combinational,
  inlined-call subset avoids the clocked-event complexity they handle.
- **"The Essence of Verilog"** (Chen et al., OOPSLA 2023) gives an operational
  semantics tested against Icarus/Verilator; it justifies treating our subset as
  combinational and finite.
- **Kami** (Choi et al., ICFP 2017) and **Sparkle/Verilean** demonstrate that
  proof-assistant hardware DSLs can extract to Verilog/FPGA with machine-checked
  guarantees; CktFormalizer (Xiong et al., 2026) extends that idea with a
  dependently typed Lean-to-Verilog pipeline.
- **Melchert et al. (FMCAD 2025)** use SMT-based translation validation across
  compiler stages; our per-witness `native_decide` equivalence lemmas are a
  lightweight, proof-assistant analog of the same idea.

---

## Acceptance

- `lake build Trinity.IcarusLowerable.*` is green.
- `./scripts/tri test --fast` keeps the W494 gate:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS.
  - 176 / 177 Icarus smoke PASS (1 documented baseline failure).
  - `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- At least three new `native_decide` value-preservation theorems for W493
  positive witnesses are added to `Soundness.lean`.
- Close-out report and three W496 cooperation variants are written.

---

*φ² + φ⁻² = 3 | TRINITY*
