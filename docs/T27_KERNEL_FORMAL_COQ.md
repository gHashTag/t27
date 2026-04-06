# T27 kernel in Rocq / Coq — bridge document

**Status:** Scaffold — executable formal layer lives in **`coq/`**. English-only.  
**Normative prose:** [`KERNEL_AXIOMS_AND_AGENT_EXPERIENCE_PROTOCOL.md`](KERNEL_AXIOMS_AND_AGENT_EXPERIENCE_PROTOCOL.md) (layer **K1–K4** vs process laws **K5/K6**).  
**Compiler verification standards & ring plan (primary):** [`COMPILER_VERIFICATION_STANDARDS.md`](COMPILER_VERIFICATION_STANDARDS.md).  
**Short index:** [`COMPILER_VERIFICATION_LANDSCAPE_AND_T27_PLAN.md`](COMPILER_VERIFICATION_LANDSCAPE_AND_T27_PLAN.md).

## Standards cross-reference (K1–K4 ↔ DO-333 / DO-330 / Flocq)

| Layer | Coq / repo | Regulatory / practice mapping |
|-------|------------|--------------------------------|
| **K1** `T27.Kernel.Trit` | Inductive `trit` + exhaustivity | **DO-333** — theorem proving; finite model checking for small state spaces |
| **K2** `T27.Kernel.Phi` | `Coq.Reals` φ lemmas **proved**; `phi_tolerance` on `R` (5·2⁻⁵³·φ²); **`PhiFloat.v`** — Flocq `binary64`, **`phi_identity_contract`** | **DO-333** — theorem proving; **Flocq** for IEEE `f64` vs **PHI-IDENTITY** (see [`PHI_IDENTITY_FLOCQ_BRIDGE_SPEC.md`](nona-03-manifest/PHI_IDENTITY_FLOCQ_BRIDGE_SPEC.md)) |
| **K3** `T27.Kernel.Semantics` | Minimal `eval` | **DO-333** — operational semantics / future abstract interpretation of passes |
| **K4** `T27.Kernel.KernelSpec` | AST / typecheck interface | Minimal kernel / TCB boundary (seL4-style narrative in prose docs) |
| **`coqc` as tool** | `.github/workflows/coq-kernel.yml` | **DO-330** C2-style “verification tool” — typically **TQL-4/5** in aviation-shaped programs; pin version in TVR |
| **`t27c` as codegen** | `bootstrap/`, `tri gen-*` | **DO-330** **C1** — output in deliverable; **IEEE 1012-style** V&V planning treats generator evidence as part of methods |

**K5 / K6** remain **process** only (workflows, **SOUL**, **ISSUE-GATE**) — **not** Rocq axioms.

## Why Rocq (formerly Coq)

- **CIC** inductive types give a **construction-level** proof that `trit` has exactly three inhabitants (`Kernel/Trit.v`) — stronger than postulating a fourth value absent.
- **Reals** (`Coq.Reals`) carry the algebraic **φ² = φ + 1** proof (**AXIOM-K2**); **`phi_tolerance`** is the engineering bound on **`R`** (IEEE scale); **`PhiFloat.v`** is the Flocq **`binary64`** bridge with **`phi_f64`** and **`phi_identity_contract`** (computational proof; deeper `Bmult_correct` lemmas → later ring per spec).
- Ecosystem maturity for **compiler verification** (e.g. CompCert lineage) matches the long-term goal of relating **`t27c`** to a checked model.
- **Extraction** to OCaml (and onward) can share code paths with the bootstrap, *if* the team commits to maintaining extracted artifacts.

**Lean 4** remains viable for Mathlib-heavy mathematics; the choice here is **not exclusive** — this repo hosts a **Coq-shaped** sketch first because CIC + extraction tradition maps cleanly to compiler-adjacent work.

Official references: [Rocq / Coq reference manual](https://rocq-prover.org/doc/V8.19.0/refman/index.html), [Lean project](https://lean-lang.org/).

## Mapping axioms → modules

| Doc axiom | Coq module | Note |
|-----------|------------|------|
| **K1** Ternary completeness | `T27.Kernel.Trit` | `Inductive trit` + `trit_exhaustive` |
| **K2** Phi identity | `T27.Kernel.Phi` | `phi_squared_identity` **proved**; `phi_inv_sq_sum_three`; `phi_tolerance` defined |
| **K3** Referential transparency | `T27.Kernel.Semantics` | Minimal `eval`; full λ + contexts = future |
| **K4** Minimal kernel | `T27.Kernel.KernelSpec` | Empty `Module Type` — fill with AST / `typecheck` |

**K5 / K6** — **do not** encode as Coq axioms; keep in **`SOUL.md`**, **`T27-CONSTITUTION.md`**, and GitHub workflows.

## Theorem status (see also multi-model synthesis)

| Claim | Formal target | `coq/` status |
|-------|----------------|---------------|
| THEOREM-K1 | Ternary sufficiency for HSLM-style ops | Lemmas about `trit_mul` only |
| THEOREM-K2 | φ-distance order on formats | `PhiDistance.v` stub |
| THEOREM-K3 | Codegen idempotency | Parameterized `t27c_gen` + reflexivity |
| THEOREM-K4 | Issue traceability | **Out of scope** for proof assistant |

## CI

Workflow **`.github/workflows/coq-kernel.yml`** uses **`coqorg/coq:8.19-ocaml-4.14-flambda`**, installs **`coq-flocq`** via **opam**, then `coq_makefile` + `make`. Local builds need the same (or add Flocq to `COQPATH`).

## Floating-point and φ (Flocq)

For claims that involve **IEEE-style floats**, tolerances, or rounding (not pure `R` algebra), plan to align with **[Flocq](https://flocq.gitlabpages.inria.fr/)** — the standard Rocq/Coq library for floating-point specifications and proofs. That matches **PHI-IDENTITY** engineering practice (tolerance-based checks in code) with a formal **model** of the numeric contract, instead of mixing `Reals` lemmas with ad-hoc float reasoning.

## Next steps

1. Extend **`Kernel/PhiFloat.v`** with **`Bmult_correct`** / **`Bplus_correct`** / relative-error style lemmas (reusable across formats), per [`PHI_IDENTITY_FLOCQ_BRIDGE_SPEC.md`](nona-03-manifest/PHI_IDENTITY_FLOCQ_BRIDGE_SPEC.md).  
2. Align `trit_add` / balanced ternary with **`specs/`** math specs (issues **#138**, **#143**).  
3. Replace `GenIdempotency.v` parameters with an actual abstract syntax of `.t27`.  
4. Consider extraction only **after** the proof obligations above are stable.

---

*No thesis-mill or forum links — cite DOI / arXiv / official manual for paper claims.*
