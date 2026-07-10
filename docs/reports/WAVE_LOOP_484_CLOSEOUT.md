# Wave Loop 484 — Close-out Report

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY
**Variant:** B (default) — continue making the remaining `UNSUPPORTED_ICARUS` placeholders functional.

---

## 1. Goal

Eliminate the remaining sized-zero `UNSUPPORTED_ICARUS` placeholders from the
Icarus Verilog backend by lowering dynamic `.len()` / `.contains()` on known
strings and fixed-size arrays into real, synthesizable logic. Keep all
non-smoke tests, yosys smoke, Icarus smoke, seals, and Rust unit tests green.

At the start of W484, four specs still emitted `UNSUPPORTED_ICARUS` placeholders
for string/array `.len()` / `.contains()` calls. The end-state is **zero**
`UNSUPPORTED_ICARUS` placeholders across all 658 specs.

---

## 2. What was changed

### 2.1 `bootstrap/src/compiler.rs`

| Change | Root-cause class | Effect |
|--------|------------------|--------|
| Added `module_known_string_literals` and extended `known_string_literals`. | String `.len()` / `.contains()` calls on module const/var and function-local string variables could not recover the literal value at Verilog emission time, so they fell back to a placeholder. | Track module-level and function-level identifiers whose initializer is a string literal; the value is used to resolve `.len()` and `.contains()` statically. |
| Updated `parse_expr_postfix` / `flatten_field_access_name`. | `"abc".len()` was parsed as a call to free function `len`, dropping the string-literal receiver. | String-literal receivers are encoded as quoted dotted names (`"abc".len`) so the backend can recover the value. |
| Extended `try_gen_verilog_static_len`. | Only fixed-size arrays had static `.len()` lowering; strings fell through. | Now resolves string literals, module const/var strings, and function-local string variables to a constant width. |
| Extended `try_gen_verilog_static_contains`. | `.contains(needle)` only worked for fixed-size module-level scalar arrays; strings and u8 arrays fell through. | Resolves string `.contains()` statically and emits an OR-reduction over fixed-size scalar/u8 arrays, using per-element register names for function-local arrays and indexed memory access for module-level arrays. |
| Fixed `gen_verilog_local_multi_dim_init`. | 1-D local array literals stored their values in `extra_size` rather than nested children, so the per-element register initializer produced a zero-value warning. | Falls back to `array_literal_elements` when children are empty, so `let arr : [3]u32 = [10, 20, 30];` initializes `arr_0`, `arr_1`, `arr_2` correctly. |

### 2.2 Witness specs

- `specs/scratch/w484_dynamic_len.t27` — covers literal, module const, module
  var, function-local string `.len()`, plus module-level and function-local
  fixed-size array `.len()`.
- `specs/scratch/w484_static_contains.t27` — covers literal, module const,
  module var, function-local string `.contains()`, plus module-level u8 and u32
  array `.contains()`.

Both specs pass yosys and Icarus smoke and have seals under `.trinity/seals/`.

### 2.3 Global reseal

The compiler changes altered generated Verilog for every spec that previously
contained `UNSUPPORTED_ICARUS` placeholders for string/array methods (notably
`specs/numeric/gf16.t27`, `specs/enrichment/youtube_transcript.t27`,
`specs/fpga/testbench/mac_tb.t27`, and `specs/vsa/similarity_search.t27`). All
`.trinity/seals/*.json` were refreshed after the final green suite run.

---

## 3. Verification

```bash
./scripts/tri test
```

| Phase | Result |
|-------|--------|
| Parse | 658 / 658 PASS |
| Typecheck | 658 / 658 PASS |
| GF16 conformance | OK |
| Gen Zig | 658 / 658 PASS |
| Gen Rust | 658 / 658 PASS |
| Gen Verilog | 658 / 658 PASS |
| Gen Verilog Yosys Smoke | 138 / 138 PASS, **0 failures** |
| Gen Verilog Icarus Smoke | 138 / 138 PASS, **0 documented baseline failures** |
| FPGA board-less smoke gate | OK |
| FPGA standalone lake-package build | OK |
| FPGA smoke gate replay | OK |
| Gen C | 658 / 658 PASS |
| Seal verify | 658 / 658 PASS |
| Fixed point | 0 divergences |

```bash
cd bootstrap && cargo test -p t27c --bin t27c
```

- **1525 passed, 0 failed, 2 ignored.**

Additional metric:

- **Total `UNSUPPORTED_ICARUS` placeholders across all 658 generated Verilog specs: 0.**

The suite reports `acceptable: true` — zero documented Icarus baseline failures
and no other failures.

---

## 4. Literature review

The W484 work continues the multi-stage DSL-lowering thread from W479–W483:

- **Static method resolution on bounded containers.** Dependent-type and
  refinement-type systems (e.g., F*, Lean 4 `Array.size`) keep container size in
  the type so `.len()` becomes a compile-time proof obligation rather than a
  runtime call. t27 arrays are fixed-size by declaration, so the same
  reasoning applies at the Verilog backend: `.len()` is a constant derived from
  the declared dimensions.
- **String membership as finite-set check.** In a synthesizable HDL context,
  string `.contains` is a finite-language membership test. Lowering it to an
  OR-reduction over the known byte sequence is the standard way to make it
  synthesizable; the backend now does this automatically for known strings.
- **Value-recovery from the AST for literal receivers.** Compilers that flatten
  method-call syntax into `receiver.method` names (common in small-language
  backends) must preserve literal receivers. Encoding the quoted literal in the
  flattened name is a lightweight way to keep the value available during code
  generation without adding a new AST node.

The literature supports the design decision to lower only *known* strings and
*fixed-size* arrays: the sizes are statically recoverable, the OR-reduction is
synthesizable, and no runtime memory or string library is required.

---

## 5. What was not closed (and why)

- **Host-side recursive helper shadowing in IGLA specs.** Helpers used for proof
  automation are still not emitted to Verilog. This class is independent of the
  `.len()` / `.contains()` work and remains for a future wave.
- **Module-scope wildcard `_` bindings.** Wildcard results are still left
  unbound in generated Verilog. This is a separate scoping/lowering gap.
- **Dynamic `.len()` / `.contains()` on unknown strings or variable-length
  containers.** Only identifiers initialized by a known string literal or
  declared fixed-size arrays are lowered; runtime string/variable-size
  containers would require a different backend strategy.

These classes are deliberately larger than one wave and are tracked as future
Icarus/Verilog backend extensions.

---

## 6. Next-wave cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_W485_2026-07-07.md` for three W485
variants.

---

*φ² + φ⁻² = 3 | TRINITY*
