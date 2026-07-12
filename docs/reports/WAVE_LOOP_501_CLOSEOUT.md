# Wave Loop 501 Close-out Report

| Field | Value |
|-------|-------|
| Issue | #1470 |
| Branch | `wave-loop-501` |
| Ring | 12 (gen-verilog / Icarus semantics) |
| Date | 2026-07-13 |
| Anchor | φ² + φ⁻² = 3 | TRINITY |

---

## 1. What was attempted

Wave Loop 500 closed the last documented Icarus baseline and left the generic
structural-equivalence theorem in its cleanest shape yet, but the theorem still
hard-coded the entry point to the string `"main"` and assumed `main` was not a
host-only helper.  This meant generated host code or test harnesses that call a
module helper directly could not be covered by the generic theorem without
wrapping the helper in `main`.

Wave Loop 501 removed that restriction by parameterizing the theorem over any
emitted function name.  The result is a `main`-independent value-preservation
contract for the Icarus-lowerable combinational subset.

---

## 2. What was actually changed

### 2.1 Generic equivalence proof (`proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`)

- `module_value_equiv_proved` now takes a parameter `fnName : String` and a
  function `fn : Function` instead of hard-coding `"main"` / `mainFn`.
- The proof derives lookup of the emitted `VFunction` for `fnName` and applies
  the existing fuel/AST forward-simulation invariant to `fn.body`.
- A convenience corollary `module_value_equiv_main` preserves the original
  `main`-specific shape.

### 2.2 Top-level soundness statement (`proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`)

- `module_value_equiv_statement` is now the generalized theorem.
- `module_value_equiv_main_statement` is the `main` corollary.
- Added a non-main witness:
  - `w501_non_main_entry_lowerable` — the witness module is lowerable.
  - `w501_non_main_entry_value_equiv` — value preservation for the `get_y`
    function is proved by applying the generalized theorem directly.

### 2.3 Witness model (`proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`)

- Added `w501NonMainEnv`, `w501NonMainModule`, and the three functions
  `make_pt`, `get_y`, and `main`.
- The `get_y` helper calls `make_pt` and returns a scalar field; the theorem
  states equivalence for `get_y`, not for `main`.

### 2.4 Regression spec (`specs/scratch/w501_non_main_entry_function.t27`)

- A `.t27` module matching the Lean witness shape.
- Test block checks both `get_y()` and `main()`.
- Seal: `.trinity/seals/scratch_w501_non_main_entry_function.json`.

---

## 3. Literature / related work

The change follows the same CompCert-style design pattern already cited in
Wave Loop 499: semantic preservation should not depend on which symbol happens
to be named `main`.  CompCert's `Unusedglobproof` uses a dynamic `KEPT`
membership invariant and `find_function_inject` so that every reachable symbol
resolves identically in the source and transformed programs, without
hard-coding the entry point
([CompCert Unusedglobproof](https://compcert.org/doc/html/compcert.backend.Unusedglobproof.html),
[Leroy 2009](https://6826.csail.mit.edu/2017/papers/compcert-CACM.pdf)).

The concrete witness theorems continue to rely on Lean 4 `native_decide`; the
recent "one axiom per native computation" refactor makes each computational
witness transparent as a separate axiom, improving external-checker trust
([leanprover/lean4#12217](https://github.com/leanprover/lean4/pull/12217)).

---

## 4. Verification results

| Gate | Result |
|------|--------|
| `lake build Trinity.IcarusLowerable.Soundness` | green, zero `sorry` |
| `./scripts/tri verify --lean-lowerable` | passed, 254 lowerable specs |
| `./scripts/tri test --fast` non-smoke | 699 / 699 PASS |
| `./scripts/tri test --fast` yosys smoke | 179 / 179 PASS (0 baseline) |
| `./scripts/tri test --fast` Icarus smoke | 179 / 179 PASS (0 baseline) |
| `./scripts/tri test --fast` seal verify | 699 / 699 match |
| FPGA board-less smoke gate / replay | OK |
| `cargo test -p t27c --bin t27c` | 1525 / 0 / 2 |

Full `./scripts/tri test` (including standalone lake-package build) was also run
and passed with the same counts.

---

## 5. Residual boundaries

- Conditionals and loops remain outside the modeled operational semantics.
- The theorem still requires the chosen function to be emitted (non-host-only),
  which is exactly the `Module.emittedFunctions` contract.
- The witness uses only scalar struct fields; array-typed direct fields continue
  to use memory-mode lowering.

---

## 6. Next-wave cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_W502_2026-07-13.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
