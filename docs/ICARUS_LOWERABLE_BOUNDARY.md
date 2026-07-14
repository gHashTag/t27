# Icarus-Lowerable Boundary

This document defines the **Icarus-lowerable subset** of t27: source constructs
that the Trinity bootstrap compiler can lower to synthesizable Icarus Verilog
(`iverilog -g2012`) and that the Lean 4 `Trinity.IcarusLowerable` predicate
can certify.

Anchor: φ² + φ⁻² = 3 | TRINITY

## 1. Design principle

The boundary is **structural and source-AST based**.  The authoritative
lowerability predicate walks the parsed t27 module and rejects constructs that
have no synthesizable Verilog lowering.  Generated Verilog is used only as a
cross-check (`iverilog` compilation) to catch backend regressions, not as the
definition of lowerability.

This closes the soundness gap where the old oracle classifier (generated
Verilog + `iverilog`) accepted semantically unlowerable specs because the
backend emitted syntactically valid placeholder Verilog.

## 2. Structural rules (Rust implementation)

Implementation: `bootstrap/src/compiler.rs`

- **Types**
  - Lowerable scalar primitives: `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `bool`.
  - Fixed-size arrays of lowerable scalar primitives or **scalar structs** are lowerable.
  - A scalar struct is lowerable iff every field is a scalar primitive or a fixed-size
    array of scalar primitives (checked recursively).  Fields of type `f32`, `string`,
    enum, nested struct, pointer, slice, etc. make the struct non-lowerable.
  - Return types, parameter types, and local declaration types must be lowerable.

- **Functions**
  - Calls to functions not declared in the same module are rejected, unless they are a
    small set of backend-injected builtins.
  - Qualified calls (`namespace::name`) are rejected.
  - Calls to host-only helpers (imports from `host::`, `print`, etc.) are rejected.

- **Control flow**
  - `if`/`else` and range-for (`for i in a..b`) are lowerable.
  - Iterator-style `for` is not lowerable.
  - `while` loops are accepted structurally only when the condition is not a constant
    `true` literal.  Bounded termination is checked by the Lean soundness layer.
  - `break`/`continue` are lowerable only inside loop bodies.

- **Expressions**
  - Casts to a non-lowerable type (e.g. `as String`) are rejected.
  - String literals, enum literals, and unsupported placeholders are rejected in
    synthesizable contexts.
  - Struct literals are lowerable only when the struct itself is lowerable.

- **Module-level constructs**
  - `const`, `var`, and `pub fn` declarations are part of the synthesizable model.
  - `test`, `bench`, and `invariant` blocks are host-side harness code and are not
    checked by the lowerability predicate.

## 3. CLI gate

```bash
t27c icarus-lowerable [--json] <file.t27>
```

Exit status is `0` for both lowerable and non-lowerable inputs (the classifier
always succeeds when the file parses).  The textual verdict is:

```
lowerable
not_lowerable: <reason>
```

JSON output:

```json
{
  "path": "specs/scratch/w534_negative_f32_field.t27",
  "lowerable": false,
  "reason": "not lowerable"
}
```

## 4. Test-suite integration

`bootstrap/src/suite.rs::is_icarus_lowerable` now uses the structural classifier
as the gate and then runs `gen-verilog` + `iverilog -g2012 -o /dev/null` as a
sanity filter.  Only specs that pass both steps are admitted to the Icarus
simulation regression gate.

## 5. Adversarial witnesses

Negative witnesses under `specs/scratch/w534_negative_*.t27` exercise the
boundary and must be rejected by both the Rust classifier and (where modeled) the
Lean predicate:

| Witness | Construct rejected |
|---|---|
| `w534_negative_cast_to_string.t27` | cast to `String` |
| `w534_negative_f32_field.t27` | scalar struct with an `f32` field |
| `w534_negative_host_only_helper.t27` | call to a host-only helper (`host::print`) |
| `w534_negative_nonlowerable_struct_assign.t27` | assignment involving a non-lowerable struct type |
| `w534_negative_unbounded_while.t27` | `while (true)` unbounded loop |
| `w534_negative_unresolved_import.t27` | call to an unresolved imported function |

## 6. Lean 4 model

The simplified AST predicate is in `proofs/lean4/Trinity/IcarusLowerable/`:

- `Predicate.lean` — `Ty.isLowerable`, `Expr.isLowerableFuel`, `Stmt.isLowerableFuel`,
  `Module.isLowerable`.
- `Lemmas.lean` — representative `Module.isLowerable` and `¬ Module.isLowerable`
  witnesses proved by `native_decide`.
- `Soundness.lean` — value-preservation and sequential-equivalence theorems for
  the lowerable subset.

The Lean predicate is kept as close as possible to the Rust structural
classifier.  Where the two diverge, the Rust classifier is the operational
source of truth and the Lean model is tightened in follow-up waves.

## 7. Pre-existing yosys smoke baseline

The Icarus-lowerable gate is independent of the Yosys synthesis smoke gate.
Several legacy `w3xx` scratch specs fail Yosys smoke because they exercise
language features (keyword collisions, tuple returns, destructuring, unpacked
local arrays) that are outside the current Icarus-lowerable subset.  Those
failures are tracked as pre-existing baselines and are not part of this wave.
