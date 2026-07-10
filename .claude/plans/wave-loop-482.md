# Wave Loop 482 — Decomposed Plan

**Date:** 2026-07-10  
**Branch:** `wave-loop-482` (created from `wave-loop-481`)  
**Variant:** B — make the W481 Icarus placeholders functional for imported scalar struct parameters, same-file AOS parameters, and same-file struct-return locals.  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Context and weak spots

W481 ended with **0 documented Icarus smoke failures**, but achieved it by emitting sized zero placeholders for unresolved field-access bases. The following honest lowering gaps remain:

| Gap | Where seen | W481 behavior | W482 target |
|-----|------------|---------------|-------------|
| Imported scalar struct parameter field access | `w481_icarus_aos_param_and_imported_struct.t27` (`m.value`, `m.scale`) | `32'd0 /* unresolved field access ... */` | Destructure imported packed vector into per-field wires using the imported spec's struct layout. |
| Imported struct parameter field access in `igla` | `igla/race/formal.t27` (`m.assigns`) | `32'd0 /* unresolved field access ... */` | Same as above, once cross-file struct layout is available. |
| Same-file AOS parameter element field access | `igla/coder/eval.t27` (`results[idx].test_pass`) | `32'd0 /* unresolved field access ... */` | Generate per-field memories for AOS parameters and index them. |
| Same-file struct-return local field access | Not yet exercised in scratch specs | local declared as scalar, field access emits bare `r_field` | Declare local as packed reg `[W-1:0]` and emit slice-based field reads. |

Dynamic string/array `.len()` and namespace-qualified calls remain out of scope; they are host-side constructs that cannot be synthesized.

---

## 2. Scientific grounding

### 2.1 Ternary / Gen-Verilog
- [An RTL-Based General Synthesis Methodology for Device-Independent Ternary Logic Circuits](https://research.knu.ac.kr/en/publications/an-rtl-based-general-synthesis-methodology-for-device-independent/) — ternary RTL-to-gate synthesis; shows that ternary designs can be expressed in Verilog-based HDL and lowered to standard RTL.
- [Tlsys: A Synthesis Framework for Ternary Logic from RTL to CNFET-Based Gate-Level Netlist](https://doi.org/10.23919/cje.2025.00.418) — first ternary RTL-to-netlist framework; supports large-scale designs.
- [Towards a Balanced Ternary FPGA](https://gwern.net/doc/cs/hardware/2009-beckett.pdf) — motivates ternary FPGA work that t27's FPGA path serves.
- [Trinity B002: Zero-DSP FPGA Architecture for Ternary Inference](https://doi.org/10.5281/zenodo.19224235) — defensive publication on ternary NN inference with OpenXC7/Yosys/NextPNR flow.

### 2.2 Embedded DSL lowering / multi-stage hardware DSLs
- [Chisel: Constructing Hardware in a Scala Embedded Language](http://people.eecs.berkeley.edu/~krste/papers/chisel-dac2012.pdf) — multi-stage elaboration + FIRRTL lowering.
- [Scala Defined Hardware Generators for Chisel](http://scottbeamer.net/pubs/schoeberl-micpro25.pdf) — higher-order generator patterns applicable to t27's struct/array lowering.

### 2.3 Verilog subset / semantics / contracts
- [The current state of Verilog semantics modelling in HOL4](https://community.arm.com/cfs-file/__key/communityserver-blogs-components-weblogfiles/00-00-00-37-98/Andreas-Lo_0803_o_0803_w.pdf) — formal Icarus Verilog semantics; justifies a machine-checkable lowerable subset.
- [A Proposal for a Standard SystemVerilog Synthesis Subset](https://sutherland-hdl.com/papers/2006-DVCon_SystemVerilog_synthesis_subset_paper.pdf) — portable synthesizable subset.
- [Synthesizable Verilog*](http://jgillenw.com/hfl07.pdf) — core calculus and operational semantics for synthesizability.
- [Veri-Sure: A Contract-Aware Multi-Agent Framework with Temporal Tracing and Formal Verification for Correct RTL Code Generation](https://arxiv.org/html/2601.19747) — design-contract + verifier agent architecture; analogous to the Trinity V/E/C agents.
- [Sparkle — Type-Safe, Formally Verifiable HDL in Lean 4](https://github.com/Verilean/sparkle) — Lean 4 HDL with Verilog generation, the closest competitor to t27's Lean/Verilog bridge.

---

## 3. Decomposed implementation plan

### Milestone 1 — Cross-file struct layout cache
**Files:** `bootstrap/src/compiler.rs`, possibly `bootstrap/src/suite.rs`  
**Owner:** C (codegen)  
**Tasks:**
1. When `gen-verilog` starts, collect `struct_fields` from all `.t27` specs reachable via `use` declarations in the current file.
2. Cache them in a new `Codegen` field `imported_struct_fields: HashMap<String, Vec<StructField>>` keyed by fully-qualified struct name (`module::Struct`).
3. For same-file structs, the existing `struct_fields` map continues to dominate.

**Acceptance:** `field_access_base_is_unresolved` returns `false` for an imported scalar struct parameter when its layout is present in the cache.

### Milestone 2 — Imported scalar struct parameter lowering
**Files:** `bootstrap/src/compiler.rs`  
**Owner:** C (codegen)  
**Tasks:**
1. In `gen_verilog_fn_internal`, when a parameter type is a struct found in `imported_struct_fields`, declare it as a packed input of `return_width(type)` bits.
2. Emit per-field local wires (`m_value`, `m_scale`) using `packed_field_offset` / `type_to_width` on the imported layout.
3. Update `field_access_base_is_unresolved` so imported scalar struct parameters are treated as resolved.
4. Update the first `ExprFieldAccess` fallback to prefer imported struct destructure when available.

**Acceptance:** `w481_icarus_aos_param_and_imported_struct.t27` produces real wires for `m.value` / `m.scale`, and the test can assert the imported constructor's value in Verilog (or at least does not read zeros for unresolved field accesses).

### Milestone 3 — Same-file AOS parameter variable-index field access
**Files:** `bootstrap/src/compiler.rs`  
**Owner:** C (codegen)  
**Tasks:**
1. When an array-of-struct parameter is passed and used with variable index (`pts[idx].x`), generate per-field memories in the specialized clone.
2. Route the `idx` parameter to index each per-field memory.
3. In the `ExprFieldAccess` fallback, recognize that `pts` (AOS param) + `idx` access should read from the generated per-field memory, not emit a placeholder.

**Acceptance:** `igla/coder/eval.t27` no longer emits `32'd0 /* unresolved field access results.test_pass */`; it reads from `results_test_pass[idx]` or equivalent.

### Milestone 4 — Same-file struct-return local packed declaration
**Files:** `bootstrap/src/compiler.rs`  
**Owner:** C (codegen)  
**Tasks:**
1. In `StmtLocal`, when the initializer is a same-file struct-returning call, declare the local as a packed reg of `return_width(type)` bits and assign the function result directly.
2. When field access is on such a local, emit a slice expression using `packed_field_offset`.
3. Update `field_access_base_is_unresolved` to know about these packed locals.

**Acceptance:** A new scratch spec `w482_struct_return_local_decl.t27` compiles under Icarus and reads real values from struct-return locals.

### Milestone 5 — Witness specs and regression tests
**Files:** `specs/scratch/w482_imported_struct_param.t27`, `specs/scratch/w482_aos_param_functional.t27`, `specs/scratch/w482_struct_return_local_decl.t27`  
**Owner:** C + V  
**Tasks:**
1. Write specs that exercise the three lowering classes under interpreter and Icarus.
2. Keep assertions honest: if a construct is still a placeholder, do not assert its Verilog value.
3. Seal the new specs.

### Milestone 6 — Global reseal and full gate
**Owner:** V  
**Tasks:**
1. Run `./scripts/tri test` and resolve any seal mismatches with `--save`.
2. Run `cargo test -p t27c --bin t27c`.
3. Verify Icarus smoke has zero documented baseline failures and no new regressions.

---

## 4. Risk and mitigations

| Risk | Mitigation |
|------|------------|
| Cross-file struct parsing changes codegen initialization | Only parse imports when `gen-verilog` runs; keep Zig/Rust/C paths untouched. |
| AOS parameter lowering regresses existing yosys witnesses | Add adversarial scratch specs and rerun full `tri test` before commit. |
| Imported struct parameter width mismatches | Use the same `return_width` / `packed_field_offset` helpers already used for same-file structs. |
| Scope collision for per-field local wires | Prefix with sanitized parameter name (`m_value`, `m_scale`) and use `module_declared_regs` dedup. |

---

## 5. Definition of done

- [ ] Cross-file struct layout cache populated from `use` imports.
- [ ] Imported scalar struct parameters destructure into real per-field wires.
- [ ] Same-file AOS parameters support variable-index element field access.
- [ ] Same-file struct-return locals declared as packed regs with slice field reads.
- [ ] At least three new scratch witness specs added and sealed.
- [ ] `./scripts/tri test` acceptable with 0 Icarus baseline failures.
- [ ] `cargo test -p t27c --bin t27c` green.
- [ ] `docs/reports/WAVE_LOOP_482_CLOSEOUT.md` and W483 cooperation variants written.
- [ ] `.trinity/current-issue.md`, `.trinity/ring-482.md`, `.trinity/experience.md`, and memory updated.
- [ ] Commit on `wave-loop-482` with `Closes #1452`.

---

*φ² + φ⁻² = 3 | TRINITY*
