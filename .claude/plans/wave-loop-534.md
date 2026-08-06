# Wave Loop 534 — Decomposed Plan

**Date:** 2026-07-07  
**Issue:** #1505  
**Branch:** `wave-loop-534`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Weak points found in the audit

1. **The Rust lowerability classifier is not structural.**
   `is_icarus_lowerable` in `bootstrap/src/suite.rs` only checks for the
   substring `UNSUPPORTED_ICARUS` in generated Verilog and then asks
   `iverilog` to accept the file. It never inspects the t27 AST, so it cannot
   be aligned with the Lean 4 `Module.isLowerable` predicate.

2. **No documented contract for the lowerability boundary.**
   There is no single file listing which constructs are in/out of the
   Icarus-lowerable subset. Future backend changes can silently expand the
   subset or break existing negative witnesses without review.

3. **Missing adversarial negative witnesses.**
   The corpus already has enum-field and string-field negative witnesses
   (W532/W533), but lacks systematic coverage for:
   - `f32`/`f64` struct fields,
   - host-only helper calls from synthesizable code,
   - casts to non-lowerable types,
   - unresolved imports,
   - unbounded / non-range `while` loops,
   - whole-struct assignment of non-lowerable structs at module scope.

4. **No Rust integration test aligns the classifier with the Lean predicate.**
   The Lean proof knows exactly which AST constructs are lowerable, but the
   Rust gate is only an oracle on generated Verilog. A drift between the two
   would only be noticed when a proof or a simulation breaks.

5. **Negative witnesses are not exercised by the Icarus simulation gate.**
   They only appear in the lowerability classifier path; there is no explicit
   test that asserts they are rejected.

---

## Relevant scientific / technical literature

1. **Herklotz et al. — *Formal Verification of High-Level Synthesis* (OOPSLA 2021).**
   Vericert proves a backward-simulation theorem from Verilog back to C by
   composing forward simulations over HTL. The Vericert restriction to a
   *syntactically identified lowerable subset* of C is the closest analogue to
   our Icarus-lowerable t27 subset. Takeaway: the boundary predicate must be a
   structural, checkable property of the source AST, not a post-hoc test on the
   generated target.

2. **Chen et al. — *The Essence of Verilog* (OOPSLA 2023).**
   λ_V gives a reference operational semantics for Verilog and compares it
   against Icarus and Verilator. Takeaway: simulator acceptance is a necessary
   but insufficient lowerability criterion; the real contract is over the
   source language constructs that the emitter promises to handle.

3. **Lööw — *The Simulation Semantics of Synthesisable Verilog* (OOPSLA 2025).**
   Argues that the Verilog standard is internally inconsistent and that
   formalization must therefore be explicit about which constructs are in the
   synthesizable subset. Takeaway: write down the lowerability boundary in a
   contract document and validate the compiler against it, not the other way
   around.

4. **SiFive Kami — `Syntax.WfMod` predicates (ICFP 2017).**
   Kami uses decidable fixpoint well-formedness predicates (`WfBaseModule`,
   `WfMod_new`) with equivalence proofs to inductive versions. Takeaway: keep
   a computable/decidable structural predicate in the proof assistant and prove
   it equivalent to the compiler's classifier.

5. **CIRCT / K-CIRCT (arXiv:2404.18756).**
   CIRCT separates dialects (`hw`, `comb`, `sv`) and uses progressive lowering
   plus IRN verification. Takeaway: a lowerability boundary is best expressed
   as a source-dialect predicate before lowering, and the compiler should
   explicitly reject out-of-subset programs with a named error class.

---

## Variant A — Structural Icarus lowerability boundary (recommended)

**Goal:** Make the Icarus lowerability subset explicit, documented, and
falsifiable by both Rust and Lean 4.

**Subtasks (decomposed):**

1. **Add a structural Rust lowerability classifier in `bootstrap/src/compiler.rs`.**
   - Walk the parsed AST (not generated Verilog) and return a typed result:
     `Lowerable`, `Unsupported { construct, span }`.
   - Reject: enum/string/float fields in packed contexts, host-only helper calls
     in synthesizable code, unresolved imports, non-lowerable casts, unbounded
     dynamic loops, and whole-struct assignment of non-lowerable structs.
   - Keep the existing `UNSUPPORTED_ICARUS` markers as the *backend* fallback,
     but make the new classifier authoritative.

2. **Expose the classifier as a `t27c` subcommand.**
   - Add `t27c icarus-lowerable <file>` that prints a machine-readable JSON
     verdict and exits `0`/`1` so the test suite can use it directly.

3. **Add adversarial negative scratch witnesses under `specs/scratch/`.**
   - `w534_negative_f32_field.t27` — scalar struct with `f32` field.
   - `w534_negative_host_only_helper.t27` — call to a host-only helper from a
     non-host function.
   - `w534_negative_cast_to_string.t27` — cast to non-lowerable type.
   - `w534_negative_unresolved_import.t27` — use of an unresolved imported name.
   - `w534_negative_unbounded_while.t27` — `while` without a bounded range.
   - `w534_negative_nonlowerable_struct_assign.t27` — module-scope whole-struct
     copy of a struct with a string field.
   - Each witness contains an empty test block so the file parses and
     type-checks, but the synthesizable subset is rejected.

4. **Add a Rust integration test in `bootstrap/src/compiler.rs` or a new**
   `bootstrap/tests/icarus_lowerable.rs`.
   - Parse each W534 negative witness and assert the classifier returns
     `not_lowerable`.
   - Parse each existing lowerable W5xx/W3xx witness and assert the classifier
     returns `lowerable`.
   - Run as `cargo test -p t27c --test icarus_lowerable`.

5. **Add Lean 4 `¬ Module.isLowerable env m` theorems in**
   `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`.
   - Instantiate the simplified AST for each W534 negative witness.
   - Prove non-lowerability by computation (`native_decide` or direct `simp`).
   - Cross-check that the Lean predicate agrees with the Rust classifier.

6. **Document the boundary.**
   - Write `docs/ICARUS_LOWERABLE_BOUNDARY.md` with:
     - precise grammar of lowerable expressions/statements,
     - list of excluded constructs and why,
     - invariant that the Rust classifier and Lean predicate agree on every
       corpus spec.

7. **Verification.**
   - `cargo build --release -p t27c` and update `FROZEN_HASH`.
   - `cargo test -p t27c --bin t27c` and new `--test icarus_lowerable`.
   - `cargo test -p tri`.
   - `./scripts/tri test --icarus-simulate --icarus-lowerable --fast`.
   - `lake build Trinity.IcarusLowerable.Soundness`.
   - Reseal affected specs.

**Why recommended:** It closes the largest remaining soundness gap: a proof is
only as strong as the gate it protects. Making the boundary structural,
documented, and cross-checked between Rust and Lean is the prerequisite for
any future expansion of the lowerable subset.

---

## Variant B — cocotb reference-model cosimulation

**Goal:** Add an independent Python reference-model simulation layer on top of
the existing Icarus gate.

**Scope:**
1. Generate a cocotb-compatible testbench wrapper for t27 specs that pass the
   `--icarus-lowerable` classifier.
2. Implement a minimal Python reference model that mirrors t27 arithmetic and
   packed-vector layout for the lowerable subset.
3. Drive the DUT with pseudo-random inputs and compare outputs cycle-by-cycle.
4. Keep the existing Icarus gate as the fast first line; run the cocotb gate in
   CI on a scheduled cadence.

**Why valuable:** Reference-model cosimulation is the standard way to catch
value-level semantic drift independently of the Verilog emitter.

---

## Variant C — Harden module-level sequential behavior in Lean 4

**Goal:** Extend the Lean 4 formal semantics and proofs to cover module-level
procedural initialization and whole-struct assignment.

**Scope:**
1. Model module-level `var` initialization and top-level assignment in the
   shallow Verilog semantics (`VModule`).
2. Prove that module-level packed scalar-struct copy and function-call
   initialization preserve the t27 source-level value.
3. Add a non-scratch corpus spec (e.g., `specs/igla/w534_module_scalar_struct_soundness.t27`)
   imported into `Trinity.IcarusLowerable.Completeness`.
4. Keep the proof green and add a `native_decide` discharge for the witness.

**Why valuable:** The Lean proof currently covers function-local and
combinational behavior. Closing the module-level sequential gap makes the
value-preservation story complete for W511–W533 shapes.

---

## Recommended variant

**Variant A.** The lowerability boundary is the weakest point of the current
proof stack: the Rust classifier is fast and pragmatic, but it is not yet tied
to an adversarial formal predicate or a documented contract. Hardening that
boundary first keeps risk low, produces durable documentation, and creates a
regression test that any future packed-vector expansion must satisfy. Variants B
and C should follow once the boundary is contractually pinned.

---

*φ² + φ⁻² = 3 | TRINITY*
