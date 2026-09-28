# Trinity — a mathlib-backed Lean 4 development

**This does NOT yet verify from a clone. One command verifies it in *this working tree*,
which is a claim about a directory on one laptop, not a credential.**

```bash
cd proofs/lean4 && lake build
```

Last working-tree run 2026-08-13: `Build completed successfully (8588 jobs).` exit 0.

> ## BLOCKING DEFECT — this does not reproduce from a fresh clone
>
> Found 2026-08-13 by adversarial verification. It invalidates the headline above for any
> reviewer who has not got this working tree, and it is **still present** as of the last
> line of this file.
>
> Two modules are **untracked**: `Trinity/T1Lucas.lean` and `Trinity/ZetaSumRule.lean`.
> Meanwhile commit `40003ed1` (2026-08-12, message *"23/23 modules, 1254 lemmas"*)
> committed the line `import Trinity.T1Lucas` into `Trinity.lean`. So **HEAD's root
> imports a file HEAD does not contain.**
>
> Measured, not inferred (2026-08-13, HEAD = `40003ed1`):
>
> ```
> git ls-files proofs/lean4/Trinity | grep -c '\.lean$'       -> 22 tracked
> find Trinity -name '*.lean' | wc -l                         -> 24 on disk
> git show HEAD:proofs/lean4/Trinity.lean | grep -c '^import'  -> 23 imports
> git cat-file -e HEAD:proofs/lean4/Trinity/T1Lucas.lean       -> MISSING
> ```
>
> **The set difference, which is the part that matters:** at HEAD,
> `{imported} \ {tracked}` = `{T1Lucas}`. `ZetaSumRule` is in neither set at HEAD — it
> exists only in this working tree.
>
> **Verified consequence, not a prediction.** A pristine tree of `40003ed1` was built on
> 2026-08-13 (`git archive HEAD:proofs/lean4` into a temp dir, mathlib oleans supplied from
> cache, all `Trinity` source from the commit):
>
> ```
> error: Trinity: some modules have bad imports
> error: no such file or directory
>   file: .../Trinity/T1Lucas.lean
> error: Trinity.lean: bad import 'Trinity.T1Lucas'
> error: build failed          exit code 1
> ```
>
> **Fix is the author's to make** (committing is not done unattended here):
>
> ```bash
> git add proofs/lean4/Trinity/T1Lucas.lean \
>         proofs/lean4/Trinity/ZetaSumRule.lean \
>         proofs/lean4/Trinity.lean
> ```
>
> The CI gates that catch this are already in place (below). They fail on HEAD today, by
> design — see the verification table.

## What CI checks, and the three different questions it asks

`.github/workflows/lean-proofs.yml`. The source-integrity gates look like variations on one
check and are not; **none of them implies another**:

| gate | asks about | question |
|---|---|---|
| 1 | the **working tree** | does the root's import set *equal* the set of `.lean` files on disk, in **both** directions? |
| 2 | the **index** | does git *contain* every module the root imports, and are there zero untracked `.lean`? |
| 3a / 3b | **HEAD** | does the committed tree — materialised by `git archive`, so nothing untracked can leak in — check, and then *build*? |
| 4 | **HEAD** | zero `sorry`, zero custom `axiom`, counted on that same committed tree. |

Gate 3b is the only build in the workflow. Building the working tree would answer a
question nobody asked. `.lake/packages` is symlinked in from the runner cache: that is the
mathlib **olean cache**, not source — every `Trinity` module is compiled from HEAD's bytes.

### Why gate 1 had to be rewritten

The 2026-08-12 version computed `comm -23 on_disk imported`. That is a set comparison, but
a **one-directional** one: it asked only *"did the root forget a file"*, never *"does an
import have no file behind it"*. On a pristine clone of `40003ed1` — 22 files, 23 imports —
it printed `root reaches every module` and **exited 0**, on a tree that cannot compile a
single line.

The failure was the missing **direction**, not counting. The old gate does catch an
equal-count divergence; renaming a module under it makes it fire. Anyone repeating this fix
should not carry away a "counts instead of sets" moral, because that one is measurably
wrong here.

### The gates were verified by making them fail

Run 2026-08-13 against the real `40003ed1`, by extracting each step's `run:` script from the
workflow YAML and executing it verbatim, so that the thing tested is the thing CI runs.
"Fixed clone" is a `/tmp` clone with the two files added and committed **there**; this
repository's index and HEAD were not touched.

| gate | this working tree | pristine clone of `40003ed1` | fixed clone |
|---|---|---|---|
| 1 | pass (24 == 24, sets equal) | **FAIL** — `+ T1Lucas` imported, no such file | pass (24 == 24) |
| 2 | **FAIL** — `T1Lucas.lean`, `ZetaSumRule.lean` imported + untracked | **FAIL** — `T1Lucas` imported, not in git | pass (24 tracked, 0 untracked) |
| 3a | **FAIL** — HEAD imports 23, contains 22 | **FAIL** — same | pass (24 in HEAD, 24 imported) |
| 3b | **FAIL** — `bad import 'Trinity.T1Lucas'`, exit 1 | **FAIL** — same | pass — `Build completed successfully (8588 jobs)`, exit 0 |
| 4 | not reached | not reached | pass — 1,301 / 0 `sorry` / 0 `axiom` |

Both gates that report a count were then mutation-tested in each direction, with the
mutation reverted afterwards to check that the gate goes quiet again:

- **gate 1** — adding an unimported module fires the `-` branch; deleting an imported
  module fires the `+` branch; reverted, silent.
- **gate 4** — appending `axiom probe_axiom : True` fires `1 custom axiom(s) declared`;
  a `declaration uses \`sorry\`` line in the build log fires `1 declaration(s) use sorry`;
  reverted, silent.

**A gate that has never been seen to fail passes for reasons unrelated to what it guards.**
Two examples from building these, both of which would otherwise have shipped as green:

- `git archive HEAD:proofs/lean4` invoked from *inside* `proofs/lean4` applies an implicit
  cwd-relative pathspec: it emits a valid **empty** tar, exits 0, and gate 3a then failed
  with a message about the repository that had nothing to do with the repository. Fixed
  with `git -C "$root"`, plus an explicit extracted-entry count so that the gate can only
  fail for its own reason.
- The first run of gate 3b was piped into `tail`, so the exit code read back was `tail`'s —
  `0` — for a build that had in fact failed. That is the same defect this workflow's own
  header warns about, committed while verifying the workflow. Re-run unpiped: exit `1`.

And one defect that was **already committed at `40003ed1`**, found only by driving the gate
into its *passing* state:

- `axioms=$(grep -rcE '^\s*axiom\s' Trinity --include='*.lean' | awk ...)`. **`grep -rc`
  exits 1 when it matches nothing**, even though it prints a `file:0` line for every file.
  Under `set -euo pipefail` that status propagates out of the pipeline and out of the
  assignment, so the step died — silently, before the summary and before any `::error::` —
  **exactly in the state it exists to certify: zero custom axioms.** A guard that goes red
  when the news is good is worse than no guard, and no amount of running it on a healthy
  tree would have revealed it, because on a healthy tree it *is* the failure. Fixed by
  `{ grep ... || true; } | awk ...`; verified by re-running it green and then making it fail
  on an injected axiom. The same construction was then found and fixed in gate 2's tracked-
  file listing, where it would have killed the step in the one case most worth reporting —
  no tracked `.lean` at all. Probed on a synthetic repo: it now reports instead of dying.

## The numbers, per tree

A count with no tree named is not a measurement. Measured 2026-08-13 17:08.

| metric | HEAD `40003ed1` | this working tree |
|---|---|---|
| modules present | **22** | **24** |
| modules imported by the root | **23** | **24** |
| builds | **no** — `bad import`, exit 1 | yes, exit 0 |
| `theorem` / `lemma` | **1,243** | **1,301** |
| `sorry` | 0 | 0 |
| custom `axiom` | 0 | 0 |
| toolchain | `leanprover/lean4:v4.31.0` | same |
| dependency | mathlib4, `require mathlib from git` | same |

`1,243 + 11 (T1Lucas) + 47 (ZetaSumRule) = 1,301`, exactly.

**The working-tree column is a moving part.** `Trinity/ZetaSumRule.lean` was last modified
21 minutes before this measurement, by concurrent work. Quote `1,301` only with the
timestamp attached; the only number that will still be true tomorrow is HEAD's, and HEAD's
is currently the number for a tree that does not build. Earlier revisions of this file said
**1,270** and the `40003ed1` commit message said **1,254**; both are stale, and neither was
ever a count of a tree a reviewer could obtain.

Reproduce, naming the tree explicitly:

```bash
# HEAD, as a reviewer would get it
d=$(mktemp -d); git archive HEAD:proofs/lean4 | tar -x -C "$d"; cd "$d"

find Trinity -name '*.lean' | wc -l                                     # modules present
grep -c '^import Trinity' Trinity.lean                                  # imported by root
grep -rcE '^[[:space:]]*(theorem|lemma)[[:space:]]' Trinity --include='*.lean' \
  | awk -F: '{n+=$2} END{print n+0}'                                    # 1243 at 40003ed1
grep -rcE '^[[:space:]]*axiom[[:space:]]' Trinity --include='*.lean' \
  | awk -F: '{n+=$2} END{print n+0}'                                    # 0
```

`sorry` is checked from the build log, not by grep: the grep hits in the tree are the word
*sorry* inside prose comments. The gate looks for `declaration uses sorry` in `lake build`
output, which is the only reliable signal.

## Why the module count is stated separately from the exit code

On 2026-08-12 this package built green while compiling **9 of its 23 modules**.
`lean_lib «Trinity»` defaults to `roots := #[Trinity]`, and `Trinity.lean` imported nine;
the entire `IcarusLowerable/` compiler-correctness subdirectory, plus `GoldenFloatRoundTrip`,
`Lemmas` and `T1Lucas`, sat outside the build target. **A build that does not reach the
source proves nothing about it.**

*(`IcarusLowerable/` is **11** files, measured as
`find Trinity/IcarusLowerable -name '*.lean' | wc -l`. `riemann-zeta-lab` v2.8 and the
2026-08-12 write-up both said "12-file"; corrected in both places.)*

So the number to check is not "did `lake build` exit 0" but "did it exit 0, on the committed
tree, whose root imports exactly the modules that tree contains".

## What is in it

| Area | Modules |
|---|---|
| Golden-ratio algebra and exact identities | `CorePhi`, `ExactIdentities`, `T1Lucas`*, `Lemmas` |
| Float round-trip for a golden-ratio number format | `GoldenFloatRoundTrip` |
| Ternary arithmetic kernels | `TernaryMac`, `TernaryGemm`, `TernaryInference`, `TernaryFPGABoot` |
| Compiler correctness (AST → semantics → Verilog) | `IcarusLowerable/` — 11 modules incl. `Soundness`, `Completeness`, `Equivalence`, `SemanticsTotal` |
| Physics derivations | `H4Lagrangian`, `H4Derivations`, `NeutrinoMasses` |
| Point-process rigidity | `ZetaSumRule`* |

\* not in the repository — see the blocking defect.

The largest single piece is `IcarusLowerable/`: a lowering from an AST to Verilog with
soundness, completeness and semantic-equivalence proofs. That is the part a reviewer should
look at first — it is a compiler-correctness development, not a collection of identities.

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

## The claim this page supports

Today: a mathlib-backed Lean 4 development of **1,243** statements at `40003ed1` with no
holes and no custom axioms — which **does not build**, because its root imports a module the
commit does not contain. The 24-module, 1,301-statement version exists only in one working
tree.

Once the two files are tracked, and not before, this becomes: a development verifiable by a
reviewer in one command, whose build target provably reaches all of its source, checked on
the committed tree rather than on anybody's laptop.

Nothing about how hard the theorems are. That is for a reader to judge.
