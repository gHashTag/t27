# Wave Loop 502 — Decomposed Plan

**Goal:** harden the Icarus lowerability gate with adversarial non-main witnesses and keep the classifier/smoke-gate boundary disagreement-free.

**Issue:** #1471  
**Branch:** `wave-loop-502`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak-point analysis

### 1.1 Current state after W501

- The generic structural-equivalence theorem `module_value_equiv_statement` is now
  fully entry-point agnostic: it applies to any emitted (non-host-only) function.
- `./scripts/tri verify --lean-lowerable` exports **253 lowerable specs** and
  skips 294 specs that still contain unmodeled placeholders.
- The Icarus smoke gate is at **179 / 179 PASS with 0 documented baselines**.

### 1.2 Weaknesses

- **Thin witness coverage for non-`main` entry points.**  W501 proved the theorem
  for one helper (`get_y`).  The classifier and smoke gate have not been stressed
  with:
  - a non-`main` function called from another emitted function,
  - a chain of three emitted functions ending in a non-`main` leaf,
  - a helper that takes a scalar struct parameter,
  - a module with multiple non-`main` entry points.
- **Classifier/smoke boundary is only as strong as the witness set.**  The 294
  intentionally skipped specs sit right next to the 253 lowerable ones.  A
  future emitter change could silently move a spec across the boundary.
- **No regression test exercises the generalized theorem with multiple emitted
  functions in one module.**  Such a test would catch accidental reintroduction of
  reachability or host-only assumptions.

### 1.3 Practical impact

- Adding adversarial witnesses now prevents regressions in both the Rust
  classifier (`bootstrap/src/compiler.rs`) and the Lean predicate
  (`Predicate.lean`).
- Each witness comes with a `native_decide` equivalence theorem that directly
  applies `module_value_equiv_statement` to a non-`main` function, keeping the
  formal contract exercised.

---

## 2. Literature / related work

### 2.1 Randomized compiler fuzzing and adversarial witnesses

The standard pattern for hardening a compiler correctness gate is differential
random testing with equivalence-preserving oracles:

- **Csmith** (PLDI 2011) generates UB-free C programs and compares outputs across
  compilers/optimization levels.  It found 325+ GCC/LLVM bugs
  ([Yang et al., 2011](https://doi.org/10.1145/1993316.1993532)).
- **YARPGen** (OOPSLA 2020) interleaves value tracking with generation to avoid
  undefined behavior without pervasive safe-math wrappers and found 220+ bugs in
  GCC, LLVM, and Intel C++
  ([Regehr et al., 2020](https://users.cs.utah.edu/~regehr/yarpgen-oopsla20.pdf)).
- **CsmithEdge** (EMSE 2022) deliberately relaxes UB-avoidance constraints to
  produce adversarial witnesses that Csmith cannot generate, then validates them
  with sanitizers and Frama-C Eva
  ([Sun et al., 2022](https://doi.org/10.1007/s10664-022-10146-1)).
- **Orange4 / equivalence-transformation testing** generates programs by applying
  semantics-preserving transformations to a seed, tracking runtime values to
  avoid UB without heavy dynamic checks
  ([Ishiura et al., 2018](https://ist.ksc.kwansei.ac.jp/~ishiura/publications/C2018-11a.pdf)).

For t27 the equivalent is not random generation but **hand-crafted adversarial
witnesses** that sit exactly on the lowerability boundary and are proved
semantically equivalent in Lean 4.

### 2.2 Icarus Verilog packed/unpacked limitations

The smoke gate relies on the emitter keeping all values in packed vectors.
Icarus still cannot reliably elaborate unpacked arrays of packed structs with
indexed member access.  The robust workaround is packed arrays + packed structs
([steveicarus/iverilog#1134](https://github.com/steveicarus/iverilog/issues/1134),
[steveicarus/iverilog#266](https://github.com/steveicarus/iverilog/issues/266)).

### 2.3 Entry-point-independent semantic preservation

CompCert's `Unusedglobproof` shows that semantic preservation can be stated for
any kept function symbol, not only a hard-coded `main`.  t27 adopted the same
pattern in W499/W501 by emitting every non-host-only function and proving
`module_value_equiv_statement` for arbitrary function names
([CompCert Unusedglobproof](https://compcert.org/doc/html/compcert.backend.Unusedglobproof.html)).

---

## 3. Decomposed implementation steps

### Step 1 — Add four adversarial scratch specs

**Files:** `specs/scratch/w502_*.t27`

| Spec | Purpose |
|------|---------|
| `w502_non_main_called_from_emitted.t27` | A non-`main` function `helper` is called from another emitted function `caller`; equivalence is proved for `caller`. |
| `w502_non_main_chain_leaf.t27` | Chain `top → mid → leaf` where `leaf` is a non-`main` function; equivalence proved for `leaf`. |
| `w502_non_main_helper_struct_param.t27` | Helper takes a scalar-struct parameter and returns a scalar; equivalence proved for the helper. |
| `w502_multiple_non_main_entries.t27` | Module exposes two non-`main` functions `a` and `b`; equivalence proved for both. |

Each spec must:
- be accepted by the parser/typechecker,
- be classified as `lowerable` by `./target/release/t27c icarus-lowerable`,
- pass Icarus smoke (`./scripts/tri test`),
- have a unique function-name invariant and a clean call context.

### Step 2 — Model each witness in Lean

**File:** `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`

For every spec add:
- environment (`w502...Env`),
- module (`w502...Module`),
- function definitions (`w502...Helper`, `w502...Caller`, `w502...Main`, etc.).

Keep the modules tiny so `native_decide` stays fast.

### Step 3 — Prove lowerability and value equivalence for non-main functions

**File:** `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`

For each witness and target function add two theorems:
- `<prefix>_lowerable`: `Module.isLowerable env m` by `native_decide`.
- `<prefix>_value_equiv`: equality between `evalModuleFunctionTotal` and
  `evalVModuleTotal` for the non-`main` function, applying
  `module_value_equiv_statement` with concrete `findFunction`,
  `hasUniqueFunctionNames`, `isCombinational`, and `callContext` proofs by
  `simp` + `native_decide`.

### Step 4 — Regenerate the completeness import and run gates

**Commands:**
1. `lake build Trinity.IcarusLowerable.Soundness`
2. `./scripts/tri verify --lean-lowerable`
3. `./scripts/tri test`
4. `cargo test -p t27c --bin t27c`

**Acceptance:**
- `lake build` green with zero `sorry` in IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable` passes with zero disagreements.
- `./scripts/tri test` reports 699+/699+ non-smoke PASS, 179+/179+ yosys smoke,
  179+/179+ Icarus smoke, 0 baseline failures.
- `cargo test` reports 1525/0/2.

### Step 5 — Close-out documentation

**Files:**
- `docs/reports/WAVE_LOOP_502_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W503_2026-07-13.md` (three W503 variants)
- `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`
- memory file `wave-loop-502.md` + `MEMORY.md` index

---

## 4. Risk assessment

| Risk | Mitigation |
|------|-----------|
| Witness triggers an Icarus placeholder and becomes not-lowerable | Keep witnesses in the already-working scalar/struct-call subset; avoid array-typed fields. |
| `native_decide` on new witness times out | Keep modules tiny (< 5 functions, scalar struct with one field). |
| Adding specs increases seal count and breaks seal verify | Compute and commit new seal JSON files. |
| Classifier says lowerable but smoke fails | Fix the emitter; if unfixable, document as a new baseline. |

---

## 5. Acceptance criteria

- At least four new scratch adversarial witnesses pass parser, typecheck,
  classifier, Icarus smoke, and seal verify.
- Each witness has a Lean `native_decide` equivalence theorem for a non-`main`
  function that applies `module_value_equiv_statement`.
- `./scripts/tri verify --lean-lowerable` reports zero disagreements.
- `./scripts/tri test` reports 0 failures and 0 new Icarus baselines.
- Close-out report and three W503 cooperation variants are committed.

---

*φ² + φ⁻² = 3 | TRINITY*
