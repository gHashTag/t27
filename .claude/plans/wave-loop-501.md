# Wave Loop 501 — Decomposed Plan

**Goal:** remove the hard-coded `main` entry-point assumption from the generic
Icarus structural-equivalence theorem, so value preservation holds for any
emitted (non-host-only) function in a lowerable combinational module.

**Issue:** #1470  
**Branch:** `wave-loop-501`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak-point analysis

### 1.1 Current theorem shape

`module_value_equiv_statement` in
`proofs/lean4/Trinity/IcarusLowerable/Soundness.lean` currently says:

```lean
theorem module_value_equiv_statement (env : Env) (m : Module)
    (h : Module.isLowerable env m)
    (hunique : Module.hasUniqueFunctionNames m)
    (hcomb : Module.isCombinational env m)
    (hctx : Module.callContext env m)
    (mainFn : Function)
    (hm : m.findFunction "main" = some mainFn)
    (hmain : ¬ Env.isHostOnly env mainFn.name) :
    evalModuleFunctionTotal defaultFuel env m "main" [] =
    evalVModuleTotal defaultFuel env (emitModule env m) "main"
```

Weaknesses:
- The theorem is **hard-coded to the string `"main"`**.  Any host-side or
  generated harness that wants to verify a helper directly must either prove a
  separate per-witness theorem with `native_decide` or rely on `main` wrapping
  the helper.
- The `main` function is **assumed non-host-only**, but the same property is
  exactly what `Module.emittedFunctions` already guarantees for every
  function it contains.
- The generic forward-simulation invariant `all_equiv` is already fully generic
  over the function name; only the wrapper theorem at the end constrains the
  name.  This makes the restriction **syntactic, not fundamental**.

### 1.2 Practical impact

- W499 added `w499_unconditional_function_emission.t27` to show that
  unreachable functions are emitted.  The generic theorem, however, cannot state
  equivalence for those unreachable functions directly; it only talks about
  `main`.
- Generated C/Zig host code sometimes calls module helpers directly (e.g.
  `get_y()` in unit tests).  A `main`-only theorem does not cover those calls in
  the Icarus model.
- Closing this gap is the last obvious well-formedness simplification before
  moving on to sequential constructs (Variant B) or adversarial hardening
  (Variant C).

---

## 2. Literature / related work

### 2.1 CompCert `Unusedglob` and entry-point independence

CompCert's `Unusedglob` pass removes unreferenced globals but its correctness
proof (`Unusedglobproof`) does not make the semantic preservation theorem
depend on a single entry point.  The simulation relation `match_states` carries
a `KEPT` clause ensuring the currently executing function only references kept
globals, and `find_function_inject` shows function resolution is preserved for
any kept symbol.  This is the standard way to make a translation-validation
argument independent of which symbol happens to be named `main`
([CompCert Unusedglobproof](https://compcert.org/doc/html/compcert.backend.Unusedglobproof.html),
[Leroy 2009, *Formal verification of a realistic compiler*](https://6826.csail.mit.edu/2017/papers/compcert-CACM.pdf)).

### 2.2 Translation validation with native computation in Lean 4

`t27` uses `native_decide` for concrete witness theorems.  Recent Lean 4 work
("one axiom per native computation") makes each `native_decide` invocation
produce a separate, enumerated axiom rather than relying on the opaque
`Lean.trustCompiler` hook.  This improves external checker compatibility and
makes the trust story cleaner
([leanprover/lean4#12217](https://github.com/leanprover/lean4/pull/12217)).

### 2.3 Icarus Verilog packed/unpacked limitations

W500 closed the last documented Icarus baseline by avoiding unpacked arrays of
packed structs with indexed member access.  The literature / issue tracker
confirms that Icarus still cannot reliably elaborate unpacked arrays of packed
structs, unpacked structs, or array parameters; the robust workaround is to
keep values in packed vectors
([steveicarus/iverilog#1134](https://github.com/steveicarus/iverilog/issues/1134),
[steveicarus/iverilog#266](https://github.com/steveicarus/iverilog/issues/266)).

---

## 3. Decomposed implementation steps

### Step 1 — Generalize the wrapper theorem in Equivalence.lean
**File:** `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`  
**Change:** replace the hard-coded `"main"` in `module_value_equiv_proved` with
a parameter `fnName : String`, require `m.findFunction fnName = some fn` and
`¬ Env.isHostOnly env fn.name`, and derive the same lookup in the emitted
module.  Keep a convenience corollary `module_value_equiv_main` for the
`"main"` case.

### Step 2 — Update the top-level statement in Soundness.lean
**File:** `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`  
**Change:** restate `module_value_equiv_statement` in the generalized form and
add `module_value_equiv_main` as a corollary.  The existing witness theorems
that call `module_value_equiv_statement` can switch to the corollary, or be
reproved with `native_decide` if concrete.

### Step 3 — Add a non-main witness module
**File:** `specs/scratch/w501_non_main_entry_function.t27`  
**Design:** a module with at least two emitted functions, e.g. a helper
`get_y()` and a `main()` that is also emitted.  The equivalence property will
be stated for `get_y`, not for `main`, to exercise the generalized theorem.
The module must be lowerable and pass Icarus smoke.

### Step 4 — Model the witness in Lean and prove equivalence
**File:** `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`  
**Change:** add `w501NonMainEnv`, `w501NonMainModule`, and theorems
`w501_non_main_entry_lowerable` and `w501_non_main_entry_value_equiv` using
`native_decide`.  The latter applies the generalized theorem or is proved by
computation.

### Step 5 — Regenerate seals and run gates
**Commands:**
1. `lake build Trinity.IcarusLowerable.Soundness`
2. `./scripts/tri verify --lean-lowerable`
3. `./scripts/tri test --fast`
4. `cargo test -p t27c --bin t27c`

**Acceptance:**
- `lake build` green with zero `sorry` in IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable` passes with zero disagreements.
- `./scripts/tri test --fast` reports 698/698 non-smoke PASS, 178/178 yosys,
  178/178 Icarus, 698/698 seal matches.
- `cargo test` reports 1525/0/2.

### Step 6 — Close-out documentation
**Files:**
- `docs/reports/WAVE_LOOP_501_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W502_2026-07-13.md`
- `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`
- memory file `wave-loop-501.md` + `MEMORY.md` index

---

## 4. Risk assessment

| Risk | Mitigation |
|------|------------|
| `all_equiv` needs a `main`-specific valuation for globals | The globals evaluation is independent of the function name; the generalized proof reuses the same `h_globals` step. |
| Witness module accidentally depends on `main` being present | Design the witness so `main` exists but the theorem talks about `get_y`. |
| `native_decide` on new witness times out | Keep the witness tiny (a few struct fields and one function call). |
| Lean build breaks because of renamed theorem | Keep the old `module_value_equiv_statement` name as the generalized theorem and add `module_value_equiv_main` as a corollary. |

---

## 5. Acceptance criteria

- `module_value_equiv_statement` is parameterized by `fnName : String` and a
  proof that the named function is emitted, with no `main`-specific or
  host-only hypotheses beyond what `Module.emittedFunctions` already implies.
- A new scratch witness `w501_non_main_entry_function.t27` passes all gates and
  has a `native_decide` equivalence theorem for its non-`main` function.
- All existing witnesses continue to pass.
- `./scripts/tri test` reports 0 failures.

---

*φ² + φ⁻² = 3 | TRINITY*
