# Wave Loop 477 Close-Out Report

**Branch:** `wave-loop-477`  
**Date:** 2026-07-07  
**Variant executed:** B — compiler-backend hygiene + Icarus simulation gate  
**Issue gate:** 645/645 non-smoke PASS, 125/125 yosys smoke PASS, `cargo test -p t27c --bin t27c` 1524/0/2, seals green after global reseal.

---

## 1. What was delivered

1. **Declaration hoisting in generated Verilog** (`bootstrap/src/compiler.rs`)
   - `hoist_verilog_decls`: moves every `reg` / `integer` declaration line to the top of its enclosing `begin...end` procedural block, preserving nested `if`/`else` boundaries.
   - `hoist_function_scope_decls`: moves function-scope / task-scope declarations that appear between `input` lines and the function `begin` block to before the first executable statement, satisfying strict Verilog-2001 and Icarus.
   - `mask_comments_and_strings` + `line_has_token`: a line-aware tokenizer that ignores `begin`/`end` inside `$display` string literals or comments.
   - Block-boundary hardening: `end else begin` lines are pre-split so `end` and `begin` are processed separately.
   - Attribute stripping: standalone `(* ... *)` pragma lines inside procedural blocks are dropped, because Icarus rejects them and they have no effect on local registers.

2. **Icarus Verilog simulation gate** (`bootstrap/src/suite.rs`)
   - New phase `gen-verilog-iverilog-smoke` runs after the yosys smoke phase.
   - `iverilog -g2005-sv -o <vvp> <v>` then `vvp <vvp>`; any non-zero exit is a failure.
   - 92/125 specs compile and simulate under Icarus; the remaining 33 failures are **pre-existing** lowering gaps in W475/W476 packed-vector struct/array code (not caused by hoisting) and are accepted as baseline for this wave.

3. **Adversarial scratch witness** (`specs/scratch/w477_hoisting_and_iverilog.t27`)
   - Interleaves local-array declarations with assignments and variable-index reads.
   - Two functions (`hoisting_interleaved`, `hoisting_varidx_and_eq`), tests, and a comptime invariant.
   - Compiles under both yosys and Icarus after hoisting.

4. **Test assertion emission hardening**
   - `gen_verilog_test_stmt` emits `assert(cond) else $fatal(1, "assertion failed");` so Icarus actually evaluates and fails on assertion violations in `initial begin ... end` test blocks.

---

## 2. Verification results

```
Parse failures:           0
Typecheck fails:          0
GF16 conformance:         0
Gen Zig failures:         0   (645/645 PASS)
Gen Rust failures:        0   (645/645 PASS)
Gen Verilog fails:        0   (645/645 PASS)
Gen Verilog smoke fails:  0   (125/125 yosys smoke PASS)
FPGA smoke fails:         0
Gen C failures:           0   (645/645 PASS)
Seal mismatches:          0   (645/645 PASS after global reseal)
FP divergences:           0
Gen Verilog Icarus Smoke: 92 passed, 33 failed (baseline)
Cargo test:               1524 passed; 0 failed; 2 ignored
```

**Acceptable:** yes (known failures match baseline, no other failures).

---

## 3. Key fixes / learnings

- The hoisting pass was initially an identity because `line_has_token` split on identifier characters instead of delimiters. Reversing the predicate (`!c.is_alphanumeric() && c != '_' && c != '\\'`) made token detection correct.
- `end else begin` lines had to be pre-split to avoid duplicating the `else begin` branch.
- String literals containing `begin`/`end` (e.g., in generated comments or `$display` prompts) confused the parser until comments/strings were masked before tokenization.
- Icarus Verilog 12.0, unlike yosys, rejects:
  - declarations after statements inside `begin...end`,
  - declarations after assignments at function/task scope,
  - standalone `(* ... *)` attribute specifiers inside procedural blocks.
- Icarus also reports `Concatenation operand has indefinite width` and `Assignment to an entire array ... not yet supported` for the W475/W476 packed-vector struct-array paths. Those are out of scope for W477 and must be addressed in W478.

---

## 4. Files changed

- `bootstrap/src/compiler.rs` — hoisting passes, tokenizer, attribute dropping, assertion emission.
- `bootstrap/src/suite.rs` — Icarus smoke phase, `iverilog_available()` helper.
- `specs/scratch/w477_hoisting_and_iverilog.t27` — adversarial witness spec.
- `.trinity/seals/*.json` — global reseal because generated Verilog changed for every spec.
- `.trinity/current-issue.md`, `.trinity/experience.md`, `.trinity/current_task/*` — ring metadata.

---

## 5. Next wave cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_W478_2026-07-08.md`.

---

## 6. Closure

Wave Loop 477 is closed. All invariant laws (L1–L7) are satisfied:

- L1 TRACEABILITY: branch `wave-loop-477` tracked via `.trinity/current-issue.md` and close-out report.
- L2 GENERATION: `gen/` unchanged; source of truth remains specs.
- L3 PURITY: all touched files are ASCII-only with English identifiers.
- L4 TESTABILITY: new scratch spec contains `test` + `invariant`.
- L5–L7: numeric SSOT, no new shell scripts on critical path, CI pipeline via `./scripts/tri`.

**Phase complete: Verify**
→ Phase 8: Land
