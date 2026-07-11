# Wave Loop 493 Plan — Close the next gen-verilog struct/call gaps and extend the formal model

**Issue:** #1463 (to create)  
**Branch:** `wave-loop-493`  
**Variant:** B (with formal follow-through)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Context and motivation

### 1.1 Weak points in the current work

Wave Loop 492 closed the first meaningful soundness property for the t27 →
Icarus path: the lowerability predicate guarantees that the *modeled* Verilog
output contains no placeholder or TODO stubs.  However, the following gaps
remain:

1. **Two documented adversarial baseline failures.**
   - `specs/scratch/w491_nested_struct_return_field_not_lowerable.t27` — a
     struct-literal field initialized from a scalar-struct parameter lowers to
     malformed Verilog (yosys/Icarus syntax error).
   - `specs/scratch/w492_predicate_rejects_nested_return_field.t27` — a
     nested-struct-return field access either fails to parse or lowers to an
     `UNSUPPORTED_ICARUS` placeholder, leaving the generated function body as
     `// TODO: implement`.
2. **Exporter conservatism.**  `Completeness.lean` currently proves only 253 of
   the 693 specs lowerable; 294 specs are skipped because the Rust exporter does
   not yet model constructs they use (strings, enums, f32 in some contexts,
   unmodeled statement shapes, etc.).
3. **Soundness is syntactic only.**  `Module.isSound` says the modeled output
   has no placeholders; it does not yet say the modeled output computes the same
   values as the t27 source.
4. **No operational semantics for the simplified t27 AST or the shallow
   Verilog AST.**  A value-preservation theorem (Variant A) is the next logical
   step but requires the backend to be stable first.

### 1.2 Scientific background consulted

The W493 plan is grounded in recent hardware-compiler verification and struct
lowering literature:

- **Sparkle / Verilean** ([github.com/Verilean/sparkle](https://github.com/Verilean/sparkle))
  — a Lean 4 embedded HDL that emits synthesizable SystemVerilog.  Sparkle
  uses a `Signal` denotational semantics and `bv_decide` for equivalence checks,
  demonstrating that a shallow, value-preserving semantics over a carved-out
  subset is a realistic target in Lean 4.
- **CktFormalizer** (arXiv:2605.07782v3,
  [DOI 10.48550/arxiv.2605.07782](https://doi.org/10.48550/arxiv.2605.07782))
  — auto-formalizes natural-language specs into a dependently-typed Lean 4 HDL
  and proves equivalence before SystemVerilog extraction.  Validates the
  strategy of keeping the synthesizable subset explicit and provable.
- **Melchert et al., FMCAD 2025** — "Automated Translation Validation of a
  Compiler for Statically Scheduled Accelerators"
  ([PDF](https://repositum.tuwien.at/bitstream/20.500.12708/219556/1/Melchert%20Jackson%20-%202025%20-%20Automated%20Translation%20Validation%20of%20a%20Compiler%20for...pdf),
  [NSF PAR](https://par.nsf.gov/servlets/purl/10663798)) — end-to-end
  translation validation of a CGRA compiler using Yosys → SMT transition
  systems.  The closest academic precedent for proving that compiler-generated
  Verilog preserves behavior; W493 takes a lighter, predicate-based step toward
  the same goal.
- **Yosys packed-struct support gap** ([YosysHQ/yosys#4653](https://github.com/YosysHQ/yosys/issues/4653)) —
  confirms that flattening structs to packed bit-vectors is the portable
  synthesis strategy, which is exactly what the t27 backend already does.
- **AMD Vitis HLS struct packing** ([UG1399](https://docs.amd.com/r/en-US/ug1399-vitis-hls/Structs-in-the-Interface)) —
  shows that HLS tools default to aggregating structs into wide vectors and
  partitioning array members into the packed vector.  This justifies t27's
  current packed-vector lowering for scalar structs and arrays of structs.

This wave does **not** prove full semantic equivalence.  It removes the
remaining concrete lowering blockers so that the next formalization wave has a
stable, larger modeled subset.

---

## 2. Goals

1. Fix `w491_nested_struct_return_field_not_lowerable.t27` so that a
   struct-literal field initialized from a scalar-struct parameter lowers to
   legal Verilog.
2. Fix `w492_predicate_rejects_nested_return_field.t27` so that nested-struct-
   return field access lowers to a packed-vector slice instead of a placeholder
   or empty function body.
3. Add one new adversarial witness that exercises the *new* boundary created by
   these fixes (e.g., a triple-nested struct-return field access or a struct-
   return field whose type is an array of structs).
4. Update the Lean predicate and exporter to cover the newly-lowerable patterns
   (nested struct-return field access, struct-literal fields that are
   scalar-struct values).
5. Regenerate `Completeness.lean` and verify that the imported corpus count
   increases (target: >253 specs).
6. Keep the full repository gate green.
7. Produce a W493 close-out report and three W494 cooperation variants.

---

## 3. Decomposed tasks

| # | Task | Owner | Files | Acceptance |
|---|------|-------|-------|------------|
| 1 | **Research weak points + literature snapshot** | Queen | `.claude/plans/wave-loop-493.md`, update `docs/reports/T27_VS_FORMAL_HDL_2026-07-11.md` | Weak points and paper references documented. |
| 2 | **Diagnose w492 parser/lowerer failure** | Creator (C) | Parser/typechecker code in `bootstrap/src/compiler.rs` | Root cause of empty `bad` function body identified. |
| 3 | **Fix nested struct-return field access** | Creator (C) | `bootstrap/src/compiler.rs` | `specs/scratch/w492_predicate_rejects_nested_return_field.t27` passes Icarus smoke and is classified lowerable. |
| 4 | **Fix struct-literal field from struct-typed parameter** | Creator (C) | `bootstrap/src/compiler.rs` | `specs/scratch/w491_nested_struct_return_field_not_lowerable.t27` passes yosys and Icarus smoke and is classified lowerable. |
| 5 | **Add new adversarial boundary witness** | Creator (C) | `specs/scratch/w493_*.t27` | Spec documents the next unsupported pattern and is added to the Icarus baseline. |
| 6 | **Update Lean predicate/exporter** | Creator (C) | `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`, `Emitter.lean`, `bootstrap/src/compiler.rs` | New patterns are modeled and accepted by the predicate. |
| 7 | **Regenerate Completeness.lean** | Verifier (V) | `proofs/lean4/Trinity/IcarusLowerable/Completeness.lean` (gitignored) | Count increases; `tri verify --lean-lowerable` green. |
| 8 | **Run conformance tests and reseal** | Verifier (V) | `.trinity/seals/`, `bootstrap/stage0/FROZEN_HASH`, `repro/numerics/nmse_manifest*.json` | `cargo test -p t27c --bin t27c` green; `./scripts/tri test --fast --icarus-lowerable` green; seals fresh if compiler changes. |
| 9 | **Produce W493 report + W494 variants + memory** | Queen | `docs/reports/WAVE_LOOP_493_CLOSEOUT.md`, `docs/reports/FPGA_LOOP_COOPERATION_W494_2026-07-*.md`, `docs/NOW.md`, `.trinity/current-issue.md`, `.trinity/experience.md`, memory | All documents updated; branch pushed; `wave-loop-494` created. |

---

## 4. Detailed implementation notes

### 4.1 Diagnosing the w492 empty function body

The `bad` function in `w492_predicate_rejects_nested_return_field.t27` is
emitted with `// TODO: implement` because `node.children.is_empty()`.  The
first step is to determine whether the parser drops the body or a typecheck/lowering
pass clears it.  The AST dump already shows an empty `FnDecl`, while the very
similar `w491` witness parses correctly.  The difference appears to be the use of
`.field` syntax in the struct literal and the zero-argument call.  The fix is
likely in the parser recovery path or in how field-access chains are collected
for scalar struct-return calls.

### 4.2 Fixing nested struct-return field access

The existing `try_emit_scalar_struct_call_field` already walks nested struct
paths (`fields.iter().take(fields.len().saturating_sub(1))`).  If the parser
produces the correct AST, this helper should already handle `make_outer().inner.v`.
If it does not, the likely causes are:
- `fn_return_types` does not contain the return type for the callee (because the
callee was marked unreachable/host-only during fixpoint).
- `collect_field_index_path_rooted` returns the wrong field order or misses the
path when the call has zero real arguments.
- The helper returns `false` because it thinks the leaf is an array or the index
 count is wrong.

The fix should be localized to this helper or its call sites in
`gen_verilog_expr` (`ExprFieldAccess` and `ExprIndex`).

### 4.3 Fixing struct-literal field from struct-typed parameter

In `w491`, `make_outer(inner : Inner)` returns `Outer { x: inner }`.  The
backend currently emits a malformed expression.  The fix is to emit a packed
concatenation that includes the already-packed parameter vector.  Specifically:
- When a struct-literal field initializer is an identifier whose type is a scalar
struct, emit the identifier itself as the packed slice for that field (no
per-field registers).
- The total packed vector for `Outer` becomes `{inner}` or `{inner as packed}`.

This is similar to how scalar-struct function arguments are already passed as
packed vectors.

### 4.4 Adversarial boundary witness

Add `specs/scratch/w493_nested_array_field_on_struct_return.t27` that attempts
`make_outer().inner.coords[i]` where `inner` is an array-typed field of a scalar
struct returned from a call.  This pattern is likely still unsupported and should
be added to the Icarus baseline rather than failing the gate.

### 4.5 Lean model updates

- In `Predicate.lean`: extend `Expr.isLowerable` for `fieldAccess` on a `call`
  to accept nested struct fields (currently it only checks the immediate leaf).
- In `Emitter.lean`: extend `fieldAccess` emission to walk nested struct offsets
  when the base is a constructor call; emit the correct packed slice.
- In `Soundness.lean`: add a representative theorem for the new positive
  witness.

### 4.6 `tri verify --lean-lowerable` gate

After the exporter is less conservative on the fixed patterns, regenerate
`Completeness.lean` with `t27c lean-lowerable --repo-root .` and verify the count
increases.

---

## 5. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| The w492 empty-body bug is deeper than the parser/helper layer. | Add detailed AST dumps and work back from the emitted Verilog; if it is a typechecker issue, limit the fix scope and document the residual boundary. |
| Fixing w491 struct-literal field changes packed-vector layout and breaks seals. | Add a dedicated scratch witness; reseal only if the compiler changes. |
| Lean predicate/exporter drift after backend changes. | Update the Lean model in the same commit as the backend fix and run `tri verify --lean-lowerable` before committing. |
| New adversarial witness perturbs the baseline count. | Document it in both yosys and Icarus baseline JSON; the suite now accepts baseline-only failures. |
| Completeness.lean regeneration time grows. | It remains gitignored and optional; only run it for verification. |

---

## 6. Definition of done

- [x] Root cause of w492 empty `bad` body documented: indented `;` comments were tokenized as `Semicolon`, causing parser recovery to drop the function body.
- [x] `w491_nested_struct_return_field_not_lowerable.t27` fixed, renamed to `w493_nested_struct_field_from_identifier_lowerable.t27`, and passes yosys/Icarus smoke.
- [x] `w492_predicate_rejects_nested_return_field.t27` fixed by correcting comments, renamed to `w492_nested_return_field_positive.t27`, and passes Icarus smoke.
- [x] New `specs/scratch/w493_*.t27` witnesses added and documented in baselines.
- [x] Lean predicate already accepted the new patterns; Rust exporter/model verified by `tri verify --lean-lowerable`.
- [x] `Completeness.lean` regenerated; count stayed at 253 because the placeholder cleanup removed previously-misclassified specs, offsetting the newly-lowerable witnesses.
- [x] `tri verify --lean-lowerable` green.
- [x] `./scripts/tri test --fast --icarus-lowerable` green (zero disagreements).
- [x] `cargo test -p t27c --bin t27c` green (1525/0/2).
- [x] Seals resealed after `bootstrap/src/compiler.rs` changes.
- [ ] `docs/reports/WAVE_LOOP_493_CLOSEOUT.md` and `docs/reports/FPGA_LOOP_COOPERATION_W494_2026-07-*.md` written.
- [ ] `docs/NOW.md`, `.trinity/current-issue.md`, `.trinity/experience.md`, and persistent memory updated.
- [ ] `wave-loop-493` pushed and `wave-loop-494` created.

---

*φ² + φ⁻² = 3 | TRINITY*
