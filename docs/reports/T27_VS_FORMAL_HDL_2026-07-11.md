# t27 vs. formal HDL / compiler verification research snapshot

**Date:** 2026-07-11  
**Branch:** `wave-loop-491`  
**Issue:** #1461  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Why this matters now

Wave Loops 466–490 closed the largest functional holes in the t27 → Verilog
pipeline. The Icarus smoke gate is currently clean (166/166 PASS, zero documented
baselines), but the contract that makes it clean is implicit in
`bootstrap/src/compiler.rs`. This snapshot collects the weak points of the current
implementation and the scientific / tooling precedents for locking that contract
in a machine-checkable form.

---

## 2. Weak points in the current t27 → Icarus path

### 2.1 Implicit lowerability rules

The following functions decide what is lowerable, but their logic is only
available as Rust code:

| Function | File:line | What it decides |
|----------|-----------|-----------------|
| `fn_body_has_unlowerable_construct` | `bootstrap/src/compiler.rs:7551` | Recursion, dynamic `.len()`/`.contains()`, namespace calls, builtins, enum values, string literals, string `+`. |
| `compute_host_only_functions` | `bootstrap/src/compiler.rs:7622` | Reachability fixpoint + interface filtering on `string`/enum types. |
| `try_emit_scalar_struct_call_field` | `bootstrap/src/compiler.rs:8506` | Scalar struct-return calls: which field accesses are legal in expression context. |
| `try_emit_array_of_struct_call_field` | `bootstrap/src/compiler.rs:8332` | Array-of-struct-return calls: indexed field access. |
| `gen_verilog_local_struct_var_decl` | `bootstrap/src/compiler.rs:5027` | Struct locals with array-typed fields. |

### 2.2 Defensive fallbacks still in the emitter

These paths are not exercised by the current green gate, but they are the
surface on which future frontend changes will break:

| Marker | File:line | Situation |
|--------|-----------|-----------|
| `/* TODO: {} initializer ... */` | `:11817`, `:11863`, `:11980`, `:11991` | Aggregate initializers for `localparam`/`parameter`. |
| `/* TODO: array literal ... */` | `:17198` | Array literals in expression context not yet handled. |
| `UNSUPPORTED_ICARUS` | `:15436`, `:15557`, `:15605`, `:16309`, `:16835`, `:16895` | Dynamic methods, namespace/host-only calls, unresolved field accesses. |
| `// TODO: implement` | `:12598` | Empty function bodies. |

### 2.3 No machine-checked contract

`docs/BACKEND_CONTRACT.md` states the intent (preserve logical behavior,
distinguish synthesizable vs simulation-only), but it does not enumerate the
lowerable subset. The W491 wave addresses this gap.

---

## 3. Scientific and tooling precedents

### 3.1 Lean 4 hardware description and verification

**Sparkle / Verilean**  
- Repository: [github.com/Verilean/sparkle](https://github.com/Verilean/sparkle)  
- A Lean 4 embedded HDL compiler that generates readable SystemVerilog.  
- Uses a `Signal` denotational semantics, `bv_decide`, and LTL for equivalence / lowerability checks.  
- Explicitly validates with Icarus Verilog round-trip simulation.  
- Relevance: demonstrates that a synthesizable-subset predicate plus Icarus
  simulation is a realistic, Lean-provable target.

**CktFormalizer**  
- Paper: arXiv:2605.07782 ([DOI 10.48550/arxiv.2605.07782](https://doi.org/10.48550/arxiv.2605.07782))  
- Uses a Lean 4 HDL for LLM-driven hardware generation with formal proofs.  
- Reports 95–100% backend realizability by restricting designs to a type-safe,
  synthesizable subset and validating with Icarus (`iverilog -g2012`, `vvp`).  
- Relevance: confirms that lowerability discipline is the bottleneck, not proof
  automation.

### 3.2 Synthesizable Verilog subsets and proof-producing translation

**A Proof-Producing Translator for Verilog Development in HOL**  
- Lööw & Myreen, ITP 2019 ([RHUL paper](https://www.cs.rhul.ac.uk/home/upac096/papers/formalise19.pdf))  
- Defines a behavioral synthesizable subset of Verilog with operational semantics.  
- Validates the semantics against Icarus Verilog, Vivado, and Verilator.  
- Relevance: the closest academic precedent to a carved-out lowerability subset
  validated against Icarus.

**FIRRTL ABI and spec**  
- [chipsalliance/firrtl-spec](https://github.com/chipsalliance/firrtl-spec)  
- Standardized IR with a formal SystemVerilog ABI (aggregate-preserving vs scalarized).  
- Relevance: an intermediate language benefits from an explicit ABI contract to
  the final backend; t27 needs the same kind of contract to Icarus.

### 3.3 Ternary / multi-valued logic synthesis

**An RTL-Based General Synthesis Methodology for Device-Independent Ternary Logic Circuits**  
- Park et al., *IEEE Access* 2025 ([DOI 10.1109/access.2025.3597293](https://doi.org/10.1109/access.2025.3597293))  
- First RTL-to-gate-level ternary synthesis flow; ~63% cell-count reduction vs.
  prior MUX-based synthesis.

**Tlsys: A Synthesis Framework for Ternary Logic from RTL to CNFET-Based Gate-Level Netlist**  
- Li et al., *Chinese Journal of Electronics* 2025 ([DOI 10.23919/cje.2025.00.418](https://doi.org/10.23919/cje.2025.00.418))  
- End-to-end ternary RTL-to-netlist framework verified on >500k-gate netlists.

**Takahe**  
- [github.com/Zaneham/takahe](https://github.com/Zaneham/takahe)  
- Universal synthesis tool supporting binary, ternary (Setun style), duodecimal,
  stochastic, and quantum flows; outputs nextpnr JSON for Lattice iCE40 4-LUT.

**Trinity B002**  
- gHashTag, Zenodo 2025 ([DOI 10.5281/zenodo.19224235](https://doi.org/10.5281/zenodo.19224235))  
- Zero-DSP ternary MAC built from FPGA LUTs, Yosys/NextPNR/OpenXC7 flow.

Relevance: t27's long-term direction includes a native ternary backend. Every
one of these flows relies on an explicit lowerability / realizability contract.

### 3.4 Wave pipelining and elastic pipelines

**Wave-Pipelining: A Tutorial and Research Survey**  
- Burleson, Ciesielski, Klass, Liu, *IEEE Trans. VLSI* 1998 ([tutorial](https://www.cs.princeton.edu/courses/archive/fall01/cs597a/wave.pdf))  
- Formal timing constraints for wave-pipelined circuits; valid clock periods are
  non-continuous and bounded by max-min delay spread.

**T-spec / T-piper**  
- Nurvitadhi, CMU 2010 ([thesis](https://users.ece.cmu.edu/~jhoe/distribution/2010/nurvitadhi.pdf))  
- Automatic pipeline synthesis with integrated formal verification via SMV
  compositional model checking.

Relevance: the FPGA side of t27 already has PVT-envelope theorems in `lake/`;
  compiler-stage correctness for timing-sensitive paths will eventually need the
  same refinement-map discipline.

---

## 4. Takeaways for W491

1. The immediate risk is **silent drift**, not a red gate.  
2. The right response is an **explicit, machine-checkable lowerability predicate**
   rather than a full semantics proof.  
3. **Lean 4** is a viable host for the predicate; Sparkle and CktFormalizer have
   already done the hard part of proving lowerability/equivalence in Lean.  
4. **Icarus Verilog** is the accepted oracle for validation, matching academic
   precedent.  
5. The W491 Variant A plan is the smallest step that materially reduces the
   drift risk: formalize the predicate, add a classifier, and gate on agreement.

---

*φ² + φ⁻² = 3 | TRINITY*
