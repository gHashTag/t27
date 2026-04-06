# T27 kernel — Rocq / Coq formal sketch

This directory is an **optional** formal layer aligned with **`docs/KERNEL_AXIOMS_AND_AGENT_EXPERIENCE_PROTOCOL.md`** (semantic **K1–K4** only; process laws **K5/K6** stay in markdown + CI).

## Build

**CI / recommended:** Docker image **`coqorg/coq:8.19-ocaml-4.14-flambda`**, then:

```bash
opam update -y && opam install -y coq-flocq
eval $(opam env)
cd coq
coq_makefile -f _CoqProject -o CoqMakefile
make -f CoqMakefile
```

**Local (macOS):** `brew install coq` gives Coq without Flocq — install **`coq-flocq`** via **opam** into the same switch, or rely on CI.

Requires **Coq ≥ 8.18** (8.19.x tested in CI) and **coq-flocq** for **`Kernel/PhiFloat.v`**.

## Status

| File | Content |
|------|---------|
| `Kernel/Trit.v` | Inductive `trit`, exhaustiveness, `trit_mul` / `trit_add` sketch |
| `Kernel/Phi.v` | `phi` on **Reals**; **`phi_squared_identity`** proved; `phi_tolerance`; no **`Admitted`** |
| `Kernel/PhiFloat.v` | Flocq **`binary64`**: `phi_f64`, `b64_mult`/`b64_plus`, **`phi_identity_contract`** (see spec) |
| `Kernel/Semantics.v` | Tiny `expr` + `eval` + determinism lemma |
| `Kernel/KernelSpec.v` | Empty `Module Type` placeholder for future t27c interface |
| `Theorems/*` | Stubs toward THEOREM-K1…K3 |

## PHI / Flocq roadmap

See **`docs/nona-03-manifest/PHI_IDENTITY_FLOCQ_BRIDGE_SPEC.md`** and **`docs/T27_KERNEL_FORMAL_COQ.md`**.

## Docs

See **`docs/T27_KERNEL_FORMAL_COQ.md`**.
