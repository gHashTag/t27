# PHI-IDENTITY — Flocq bridge (technical specification)

**Status:** DRAFT → review. English-only.  
**Repo:** `github.com/gHashTag/t27` · **Normative prose:** [`KERNEL_AXIOMS_AND_AGENT_EXPERIENCE_PROTOCOL.md`](../KERNEL_AXIOMS_AND_AGENT_EXPERIENCE_PROTOCOL.md) (K2), [`T27_KERNEL_FORMAL_COQ.md`](../T27_KERNEL_FORMAL_COQ.md), [`NUMERIC-CORE-PALETTE-REGISTRY.md`](../nona-02-organism/NUMERIC-CORE-PALETTE-REGISTRY.md).  
**Related issues:** [#138](https://github.com/gHashTag/t27/issues/138), [#142](https://github.com/gHashTag/t27/issues/142) (context).

## Problem

- **Layer A (`Coq.Reals`):** \(\varphi^2 = \varphi + 1\) is exact. Implemented in `coq/Kernel/Phi.v` (no `Admitted`).
- **Engineering (`f64`):** \(\varphi\) is not representable exactly; operations round. The **PHI-IDENTITY** check in code uses a **tolerance**.
- **Gap:** A machine-checked link between **IEEE binary64** semantics and `phi_tolerance` requires **Flocq** (or equivalent), as used by CompCert for float proofs.

## Goals (scope)

1. **Done (Layer A):** `Phi.v` — no `Admitted`; `phi_tolerance := 5 * / IZR(2^53) * φ²` on `R`; algebraic lemmas.
2. **Done (Ring 47 / Layer C — computational):** `PhiFloat.v` imports Flocq; `phi_f64 : binary64`; `phi_sq_f64` / `phi_plus_one_f64` via `b64_mult` / `b64_plus`; theorem **`phi_identity_contract`** (for this literal, `fl(φ²)` and `fl(φ+1)` are **bit-identical**, so `Rabs` residual is `0` &lt; `phi_tolerance`). Validation: **`scripts/validate_phi_f64.py`**.
3. **Future:** `Bmult_correct` / `Bplus_correct` + relative-error bounds (reusable on other formats); wire `PhiDistance.v` to `B2R`.

## Non-goals

- Full `t27c` AST semantics; Zig codegen correctness; changing `f64` format.

## CI

Workflow **`.github/workflows/coq-kernel.yml`** uses image **`coqorg/coq:8.19-ocaml-4.14-flambda`** and **`opam install coq-flocq`** so `From Flocq Require Import …` resolves.

## References

- [Flocq](https://flocq.gitlabpages.inria.fr/) (INRIA) · LGPL-3.  
- Boldo & Melquiond — *Flocq: A Unified Library for Proving Floating-Point Algorithms in Coq*.

## Acceptance (incremental)

| ID | Criterion |
|----|-----------|
| AC-01 | `Phi.v` contains no `Admitted` |
| AC-02 | `coq-flocq` builds `PhiFloat.v` in CI |
| AC-03 | `conformance/phi_identity_vectors.json` valid JSON (see suite validator) |
| AC-04 | `T27_KERNEL_FORMAL_COQ.md` reflects K2 Reals status |

**Algebraic** error bound (`phi_identity_f64_tolerance` style from TZ §5) — **deferred**; current proof uses **computational equality** of IEEE results for the fixed `phi_f64` constant.
