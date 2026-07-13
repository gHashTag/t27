# Wave Loop 518 — Decomposed Plan

**Issue:** #1487 (placeholder)  
**Branch:** `wave-loop-518`  
**Selected variant:** A — clear remaining W508 `break`/`continue` yosys/Icarus smoke baselines  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Scientific anchors

- IEEE Std 1800-2017 §10.6/§10.7 — `break`/`continue` are SystemVerilog loop-control statements; Verilog-2001 has no direct equivalents and relies on `disable <block>`.
- Sutherland & Mills, *Synthesizable SystemVerilog: Busting the Myth that SystemVerilog is only for Verification* (SNUG 2013), §4.5: `break`/`continue` are synthesizable and recommended over the error-prone `disable` statement.
- Sutherland, *Modeling with SystemVerilog in a Synopsys Synthesis Design Flow* (SNUG Europe 2006), §8.4: `break`/`continue` replace `disable` in synthesizable code.
- Icarus Verilog 12.0 (current tool-chain) rejects both plain `break` statements and `disable fork;` in procedural code, so a portable flag-based encoding is required.
- Yosys accepts `break`/`continue` in `-sv` mode but the t27 Verilog target is intentionally Verilog-2001-compatible; the flag encoding keeps both simulators green.

---

## Root cause

The Verilog backend currently emits:

```verilog
break;          // StmtBreak -> "disable fork;"
continue;       // StmtContinue -> "/* continue */;"
```

- `disable fork;` is rejected by both yosys (syntax error) and Icarus.
- `/* continue */;` is a no-op, so `continue` semantics are lost, causing runtime assertion failures.

Two additional Icarus failures (`w468_local_ram_style.t27`,
`w514_function_local_packed_aos_ram_style.t27`) are caused by Icarus rejecting
attribute specifiers (`(* ram_style = ... *)`) on declarations inside
functions. These are kept in the same wave because they are the only remaining
Icarus smoke failures after the break/continue fix.

---

## Design

### Break / continue flag encoding

For every emitted loop, introduce two per-loop `reg` flags:

- `__break_flag_N`  — set by `break`; cleared at loop entry.
- `__continue_flag_N` — set by `continue`; cleared at the start of each
  iteration.

Loop emission becomes:

```verilog
reg __break_flag_N;
reg __continue_flag_N;
__break_flag_N = 0;
__continue_flag_N = 0;
for (i = 0; i < N && !__break_flag_N; i = i + 1) begin
    __continue_flag_N = 0;
    if (!__break_flag_N && !__continue_flag_N) begin <stmt_0> end
    if (!__break_flag_N && !__continue_flag_N) begin <stmt_1> end
    ...
end
```

- `break` sets `__break_flag_current = 1;`.
- `continue` sets `__continue_flag_current = 1;`.
- Guards are applied to each statement individually so that a `continue` in
  the middle of the body skips the remaining statements of the current
  iteration.
- Flags are scoped by loop-nesting depth using a stack; a `break` in an inner
  loop only affects the innermost loop.

### Function-local pragma suppression

Icarus rejects `(* attr *)` on declarations inside functions. Track whether the
emitter is currently inside a function body; if so, suppress pragma attributes
on local declarations while still honoring them at module scope.

---

## Implementation phases

### Phase 1 — Loop flag infrastructure

1. Add `loop_flag_stack: Vec<(String, String)>` and `loop_flag_counter: u32` to
   `Compiler` state.
2. Add helper to push/pop current loop flag names and write flag declarations.

### Phase 2 — Loop emission rewrite

1. In `gen_verilog_while_stmt`:
   - declare and initialize loop flags,
   - append `&& !__break_flag_N` to the `while` condition,
   - reset `__continue_flag_N = 0;` at the top of the body,
   - wrap each body statement with the guard.
2. In `gen_verilog_for_stmt` and `gen_verilog_for_range_stmt`: same pattern.

### Phase 3 — Break / continue statement emitters

1. `StmtBreak` → emit `__break_flag_current = 1;`.
2. `StmtContinue` → emit `__continue_flag_current = 1;`.
3. Add a safety fallback when the flag stack is empty (should not happen for
   lowerable specs).

### Phase 4 — Function-local pragma suppression

1. Add `in_function_body: bool` state; set true while emitting a function/task
   body, false after.
2. In local-declaration emitters, skip writing `extra_pragma` when
   `in_function_body` is true.

### Phase 5 — Validation

1. `cargo build --release` and `cargo test -p t27c --bin t27c`.
2. `./target/release/t27c icarus-lowerable` on the three W508 specs.
3. `./scripts/tri test --icarus-lowerable --fast`.
4. `./scripts/tri verify --lean-lowerable`.

### Phase 6 — Reseal and baselines

1. Reseal the three W508 specs and any other specs whose generated output
   changed.
2. Update `docs/reports/gen_verilog_smoke_baseline.json` and
   `docs/reports/gen_verilog_iverilog_smoke_baseline.json` to empty
   `expected_failures` (or remove W508 entries).

### Phase 7 — Reports and memory

1. Write `docs/reports/WAVE_LOOP_518_CLOSEOUT.md`.
2. Write `docs/reports/FPGA_LOOP_COOPERATION_W519_2026-07-07.md` with three W519
   variants.
3. Update `.trinity/current-issue.md` to W519.
4. Save memory in `.claude/projects/-Users-playra-t27/memory/wave-loop-518.md`
   and append to `MEMORY.md`.
5. Update `.trinity/experience.md`.

### Phase 8 — Land

1. Commit on `wave-loop-518` with `Closes #1487`.
2. Create `wave-loop-519` branch.

---

## Risk

- **Medium.** The flag encoding touches every synthesizable loop and changes
  generated output for all specs with `break`/`continue` in hardware contexts.
- The per-statement guard may interact with deferred temporary assignments; the
  existing `aos_stmt_start` insertion point must be measured inside the guard.
- Nested loop flag scoping must be correct to avoid cross-loop flag leakage.

---

*φ² + φ⁻² = 3 | TRINITY*
