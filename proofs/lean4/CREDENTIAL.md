# Trinity — a mathlib-backed Lean 4 development

**Verifiable from a clone in one command.**

```bash
cd proofs/lean4 && lake build
```

Carried tree (branch `carry/wave-loop-898-unique` off `origin/master` 2925def9b), measured
2026-09-28: `Build completed successfully (8588 jobs).` exit 0, **24 modules tracked, 24
imported by the root** — sets equal in both directions.

| metric | carried tree | `origin/master` 2925def9b (baseline) |
|---|---|---|
| modules present | **24** | 24 |
| modules imported by the root | **24** | **9** |
| `lake build` | **yes**, exit 0 | **no** — `TernaryFPGABoot`, `H4Lagrangian` fail; `IcarusLowerable.Completeness` fails but the 9-import root never reaches it |
| `theorem` / `lemma` | **1,328** | 1,167 in the 9 imported modules (12 more sit unimported on disk) |
| `sorry` | 0 | — (build does not complete) |
| custom `axiom` | 0 | — |
| toolchain | `leanprover/lean4:v4.31.0` | same |
| dependency | mathlib4 `800238935b9c` (same pin as the wave-loop-898 green build of 2026-08-13) | same |

`sorry` is counted from the build log (`declaration uses sorry`), not by grep: the word
appears in prose comments. 0 hits.

## Where each module came from

The wave-loop-898 branch and master diverged in 8 of 24 modules. The carried tree resolves
each by *which version compiles and is newer*, not by branch loyalty:

| module | source | why |
|---|---|---|
| `TernaryMac`, `TernaryGemm`, `TernaryInference`, `NeutrinoMasses` | **master** (Sep 4 / Aug 29) | newer than the branch (Jul 5 / Aug 12) and build green |
| `TernaryFPGABoot` | **branch** (Jul 5) | master's Sep-25 version (Wave Loop 460 Variant C, #4800) does not compile — lean CI red since Sep 24 |
| `H4Lagrangian` | **branch** (Aug 12) | master's Aug-29 version does not compile |
| `IcarusLowerable/Completeness` | **branch** (Jul 23) | master's Aug-28 version fails: `native_decide` evaluates `Module.isLowerable … = false` as false at 4703:106 |
| `GoldenFloatRoundTrip` | **branch** (Aug 12) | newer than master's (Jul 29) |
| `T1Lucas`, `ZetaSumRule` | **branch, new** | previously untracked — the blocking defect below |
| other 14 | identical on both sides | — |

1,328 = the wave-loop-898 working-tree 1,301 + the four master-side modules' newer
content (Sep-4/Aug-29 edits added statements), − branch-side edits to the five replaced
modules that were already superseded.

## History this file used to carry

Until 2026-09-28 this page documented a **blocking defect**: HEAD imported
`Trinity.T1Lucas` without tracking it (22 tracked / 23 imports), so the committed tree
could not build and the "1,254 lemmas" headline was a claim about one laptop's working
tree, not a credential. The untracked files are tracked now; the root imports exactly
what the tree contains; the build is green on the committed set. That section is retired
into this paragraph so nobody re-learns its lesson: **a count with no tree named is not a
measurement, and a build that does not reach the source proves nothing about it.**

## What CI checks

`lean-proofs.yml` on master is the **one-directional** gate: it asks "did the root forget
a file", never "does an import have no file behind it". On a 22-file/23-import tree it
prints `root reaches every module` and exits 0 — measured on a pristine clone of
`40003ed1`. The wave-loop-898 branch carries a rewritten workflow (bidirectional set
comparison, git-contains check, build from `git archive HEAD`, sorry/axiom counted on the
committed tree) that exists **only there**: pushing it needs the `workflows` OAuth scope,
which the current token lacks. Until it lands, the equal-sets claim above is verified by
the two `grep -c` lines in the table, not by CI.

## What is in it

| Area | Modules |
|---|---|
| Golden-ratio algebra and exact identities | `CorePhi`, `ExactIdentities`, `T1Lucas`, `Lemmas` |
| Float round-trip for a golden-ratio number format | `GoldenFloatRoundTrip` |
| Ternary arithmetic kernels | `TernaryMac`, `TernaryGemm`, `TernaryInference`, `TernaryFPGABoot` |
| Compiler correctness (AST → semantics → Verilog) | `IcarusLowerable/` — 11 modules incl. `Soundness`, `Completeness`, `Equivalence`, `SemanticsTotal` |
| Physics derivations | `H4Lagrangian`, `H4Derivations`, `NeutrinoMasses` |
| Point-process rigidity | `ZetaSumRule` |

The largest single piece is `IcarusLowerable/`: a lowering from an AST to Verilog with
soundness, completeness and semantic-equivalence proofs. That is the part a reviewer
should look at first — it is a compiler-correctness development, not a collection of
identities.

## What this is NOT

**Nothing here is about the Riemann hypothesis, and nothing here is about zeta zeros.**
`ZetaSumRule` formalises point-process algebra — a sum rule for pair correlations of the
form `R(u) = 1 − c·K(u)`, and the resulting number-variance rigidity. Its one analytic input
(that the zeros' number variance is `o(L)`, which is Fujii 1975) enters strictly as a
**hypothesis** and is never asserted. A formalisation that quietly discharged that
hypothesis would look stronger and be worth nothing.

The 2026 unconditional proportion bound `3/2 − (1/√2)·cot(1/√2) = 0.6725007` also shipped as
a Lean 4 formalisation. **That is a shared medium, not a shared subject**, and this file
claims no more than that.

## Reproduce, naming the tree explicitly

```bash
# the carried tree, as a reviewer would get it after the PR merges
cd proofs/lean4
find Trinity -name '*.lean' | wc -l                                     # 24 modules present
grep -c '^import Trinity' Trinity.lean                                  # 24 imported by root
lake build                                                              # exit 0, 8588 jobs
grep -rcE '^[[:space:]]*(theorem|lemma)[[:space:]]' Trinity --include='*.lean' \
  | awk -F: '{n+=$2} END{print n+0}'                                    # 1328
grep -rcE '^[[:space:]]*axiom[[:space:]]' Trinity --include='*.lean' \
  | awk -F: '{n+=$2} END{print n+0}'                                    # 0
```

Nothing about how hard the theorems are. That is for a reader to judge.
