# Wave Loop 485 Plan — Variant B (default)

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY
**Branch:** `wave-loop-485`
**Issue:** #1455 (to be opened)

## Goal

After W484 eliminated all sized-zero `UNSUPPORTED_ICARUS` placeholders, the
remaining Icarus/Verilog gaps are soft-failure classes that prevent IGLA and
some neural/bench specs from simulating cleanly:

1. **Host-side recursive helpers used only in proof/invariant contexts.**
   IGLA specs define string/array utility helpers (`*_inner`,
   `sacrebleu_precision`, `count_ngram_matches`, `parse_*_inner`, etc.) that are
   needed for t27 host verification but are not synthesizable. The Verilog
   backend currently emits them as normal functions, then replaces every call
   with a sized-zero placeholder because the helpers contain dynamic string
   methods or recursive loops. The result is noisy generated Verilog and
   simulation-time failures when a runtime test indirectly depends on a helper
   result.
2. **Module-scope wildcard `_` bindings.**
   `let _ = foo();` in a test/bench body drops the call result in t27 source,
   but the Verilog backend currently emits the call anyway, producing a
   placeholder expression statement that Icarus may reject or that corrupts
   generated logic when the call has side effects.
3. **Bench-local array declarations crossing function boundaries.**
   Some bench specs hoist locals to module scope but still emit references
   inside function bodies; the hoisted-name resolution for array variables is
   not fully hardened.

This wave will fix the first two classes and add a regression witness for the
third. The acceptance gate is the same as every wave: 658 / 658 non-smoke PASS,
138 / 138 yosys smoke PASS, 138 / 138 Icarus smoke PASS (no new baseline
failures), 658 / 658 seal matches, `cargo test -p t27c --bin t27c` green.

## Decomposition

### Subtask 1 — Detect and skip host-side proof helpers

**Owner:** backend (`bootstrap/src/compiler.rs`)

1. Add a heuristic that marks a function as **host-only** when *all* of the
   following hold:
   - Its name ends with `_inner` or it contains dynamic string/array method
     calls (`*.len`, `*.contains`) or builtin calls (`@intCast`, `@min`,
     `@mod`) that are known not to lower cleanly, **and**
   - it is not called from any `test`, `invariant`, or `bench` body that is
     itself emitted to Verilog (i.e., it is only reachable from runtime
     non-Verilog contexts or from other host-only helpers).
2. For host-only functions, do **not** emit a Verilog function/task at all.
3. For calls to host-only functions, emit a placeholder only if the call
   appears in an expression context; in statement context emit a comment-only
   no-op so the call does not produce a sized-zero assignment.
4. Add witness spec `specs/scratch/w485_host_helper_shadow.t27` that defines
   a recursive helper and uses it only inside an invariant and a host test, then
   verifies the Verilog module still simulates.

### Subtask 2 — Module-scope wildcard `_` bindings

**Owner:** backend (`bootstrap/src/compiler.rs`)

1. In `gen_verilog_stmt` / `StmtLocal`, detect `name == "_"`.
2. If the initializer is an `ExprCall` whose callee is a host-only or
   namespace-qualified function, emit only a comment and no placeholder.
3. If the initializer is an `ExprCall` that *is* emitted, bind the result to an
   anonymous packed temporary whose width matches the call's return type, and
   never reference it again.
4. If the initializer is a struct/array literal, emit an anonymous temporary
   with the existing packed lowering but no name binding.
5. Add witness spec `specs/scratch/w485_wildcard_binding.t27` covering
   `let _ = host_helper();`, `let _ = emitted_fn();`, and
   `let _ = StructLit{...};`.

### Subtask 3 — Bench-local array hoisting hardening (adversarial witness)

**Owner:** backend + test

1. Create `specs/scratch/w485_bench_local_array_hoist.t27` that declares a
   bench-local fixed-size array, uses it inside a function called from the
   bench, and asserts a result.
2. Verify that the generated Verilog references the hoisted module-level name
   (`bench_<n>_<name>`) and that the array elements initialize correctly.
3. If a bug is found, fix `verilog_local_raw_base` / `bench_local_names`
   propagation; otherwise keep the witness as a regression guard.

### Subtask 4 — Global reseal and verification

1. Run `./scripts/tri test`.
2. If any seal mismatches, run `find specs -name "*.t27" -exec t27c seal --save {} \;`.
3. Run `cargo test -p t27c --bin t27c`.
4. Confirm zero `UNSUPPORTED_ICARUS` placeholders remain.

## Literature context

- **Cambridge VFE synthesizable Verilog subset (V0).** Formal semantics for a
  synthesizable Verilog subset show why the backend must avoid unbounded
  recursion, dynamic strings, and unresolved identifiers.
- **Proof-assistant extraction / helper erasure (Peregrine, MetaRocq).**
  Helpers used only in specifications or proofs are routinely erased before
  code generation. W485 applies the same idea: identify helpers whose bodies
  are not synthesizable and skip them in the Verilog target.
- **FIRRTL/Chisel lowering.** FIRRTL treats strings as non-synthesizable
  property types and lowers fixed-size vectors to packed Verilog. This
  justifies the t27 design choice of keeping string/array helper functions
  host-only.

## Acceptance criteria

- [ ] `specs/scratch/w485_host_helper_shadow.t27` parses, typechecks, generates
      Verilog, and passes yosys + Icarus smoke.
- [ ] `specs/scratch/w485_wildcard_binding.t27` passes the same gates.
- [ ] `specs/scratch/w485_bench_local_array_hoist.t27` passes the same gates.
- [ ] `./scripts/tri test` reports 658 / 658 non-smoke PASS, 138 / 138 yosys
      smoke PASS, 138 / 138 Icarus smoke PASS, 0 documented baseline failures.
- [ ] 658 / 658 seal matches.
- [ ] `cargo test -p t27c --bin t27c` 1525 / 0 / 2.
- [ ] Zero `UNSUPPORTED_ICARUS` placeholders across all specs.

## References

- W484 close-out: `docs/reports/WAVE_LOOP_484_CLOSEOUT.md`
- W485 cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W485_2026-07-07.md`
- Icarus Verilog quirks: https://steveicarus.github.io/iverilog/usage/icarus_verilog_quirks.html
- Cambridge VFE V0 paper: https://www.cl.cam.ac.uk/~djg11/pubs/synthesizable_verilog_syntax_and_semantics.pdf
- FIRRTL spec: https://github.com/chipsalliance/firrtl-spec/blob/main/spec.md
- Peregrine extraction abstract: https://types2026.cse.chalmers.se/abstracts/61.pdf

*φ² + φ⁻² = 3 | TRINITY*
