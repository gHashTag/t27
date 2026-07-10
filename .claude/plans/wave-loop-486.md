# Wave Loop 486 Plan — Variant B (default)

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY
**Branch:** `wave-loop-486`
**Issue:** #1456 (to be opened)

## Goal

After W485 closed the next soft-failure classes, W486 continues the Icarus/Verilog
backend hardening on the remaining reachable gaps:

1. **Bench-local fixed-size arrays crossing function boundaries.**
   A bench declares `let data : [4]u32 = [...];` and calls `fn f(a: [4]u32)`.
   The array-parameter binding pass currently rejects this because bench-local
   arrays are neither module-level consts nor function-local arrays.
2. **Imported namespace helper erasure.**
   Namespace-qualified helper functions (e.g., `module::helper`) used only in
   host-side contexts are currently emitted as sized-zero placeholders. Extend
   W485 host-only detection to skip them entirely when they are dead to
   synthesizable contexts.
3. **Module-scope wildcard `_` bindings with array literal initializers.**
   Where the parser already accepts `let _ = [N]T{...};` at module scope, ensure
   the backend does not emit a named `_` identifier. Full struct-literal
   wildcard support is parser-blocked and remains for a future wave.

## Decomposition

### Subtask 1 — Bench-local arrays as array-parameter arguments

**Owner:** backend (`bootstrap/src/compiler.rs`)

1. Pre-collect bench-local array names per bench before the array-parameter
   binding pass.
2. Change the call-site tuple from `(&Node, Option<&Node>)` to
   `(&Node, Option<&Node>, Option<String>)` where the third element is the
   containing bench name for top-level bench-block calls.
3. In the signature builder, when a call is inside a bench block and an array
   argument identifier matches that bench's local array set, push `"__local__"`
   instead of the raw name.
4. The existing local-packed array-parameter path will then:
   - clone the callee with a packed-vector input,
   - pack the hoisted bench-local registers at the call site, and
   - slice the packed vector inside the function body.
5. Add witness spec `specs/scratch/w486_bench_array_param.t27`.

### Subtask 2 — Namespace-qualified helper erasure

**Owner:** backend (`bootstrap/src/compiler.rs`)

1. In the host-only pre-pass, also collect all imported namespace-qualified
   functions that are referenced in the current module and determine which are
   used only in non-Verilog contexts (invariants, host-only functions, or
   wildcard statements).
2. Add a map `host_only_namespace_calls: HashSet<String>` of qualified names
   that should be skipped.
3. In `gen_verilog_expr` `ExprCall`, treat qualified names in this set like
   unqualified host-only functions: statement-context comment no-op,
   expression-context sized-zero placeholder.
4. Add a witness spec that imports a helper module and uses it only in an
   invariant or wildcard statement.

### Subtask 3 — Module-scope wildcard array literals

**Owner:** backend (`bootstrap/src/compiler.rs`)

1. In `gen_verilog_const`, when `node.name == "_"` and the initializer is an
   `ExprArrayLiteral` or `ExprIdentifier` referring to a module-level array,
   emit an anonymous packed temporary or memory (matching the function-scope
   wildcard logic) instead of discarding it with a comment.
2. Keep the existing comment-only path for host-only/namespace calls.
3. Add a regression witness if the parser accepts the construct; otherwise
   document the parser limitation in the close-out report.

### Subtask 4 — Global reseal and verification

1. Run `./scripts/tri test`.
2. Reseal if needed.
3. Run `cargo test -p t27c --bin t27c`.
4. Confirm zero `UNSUPPORTED_ICARUS` placeholders remain.

## Acceptance criteria

- [ ] `specs/scratch/w486_bench_array_param.t27` parses, typechecks, generates
      Verilog, and passes yosys + Icarus smoke.
- [ ] `./scripts/tri test` reports 661 / 661 non-smoke PASS, 141 / 141 yosys
      smoke PASS, 141 / 141 Icarus smoke PASS, 0 documented baseline failures.
- [ ] 661 / 661 seal matches.
- [ ] `cargo test -p t27c --bin t27c` 1525 / 0 / 2.
- [ ] Zero `UNSUPPORTED_ICARUS` placeholders across all specs.

## Literature context

- **Cambridge VFE synthesizable Verilog subset (V0).** Direct binding of
  module-level memories to function parameters is synthesizable; the backend
  already does this for module const arrays. Bench-local arrays are hoisted to
  module scope, so the same binding mechanism applies once the pre-pass knows
  their names.
- **FIRRTL/Chisel property types.** Namespace-qualified helpers are analogous to
  non-synthesizable property methods; erasing them before Verilog emission is
  consistent with treating strings and dynamic methods as host-only.

## References

- W485 close-out: `docs/reports/WAVE_LOOP_485_CLOSEOUT.md`
- W486 cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W486_2026-07-07.md`
- Icarus Verilog quirks: https://steveicarus.github.io/iverilog/usage/icarus_verilog_quirks.html
- Cambridge VFE V0 paper: https://www.cl.cam.ac.uk/~djg11/pubs/synthesizable_verilog_syntax_and_semantics.pdf
- FIRRTL spec: https://github.com/chipsalliance/firrtl-spec/blob/main/spec.md

*φ² + φ⁻² = 3 | TRINITY*
