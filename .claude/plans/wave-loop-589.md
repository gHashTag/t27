# Plan — Wave Loop 589

**Issue:** #1560  
**Branch:** `wave-loop-589`  
**Goal:** module-scope 17-D array-of-struct variable initialized from a function
call with indexed signed field writes, at the 4-MiBit cliff.

## Phase breakdown

1. **Observe** — read `CLAUDE.md`, `SOUL.md`, `AGENTS.md`; gather W585/W587/W588
   closeouts; run Agent E weak-point scan and literature review.
2. **Plan** — choose variant C (`[2]^17 Pt` module-scope mutable AoS from call);
   identify the module-scope call-initializer gap in `gen_verilog_var` /
   `gen_verilog_const`.
3. **Delegate / Implement** — patch `bootstrap/src/compiler.rs` to emit wholesale
   packed assignment for multi-D scalar-struct arrays initialized by `ExprCall`.
4. **Gen / Seal** — generate the W589 witness programmatically with leaf values
   inside signed i16; create seal and Icarus baseline; reseal affected existing
   seals whose generated Verilog changed.
5. **Verify** — `cargo build --release -p t27c`, `cargo test -p t27c --bin t27c`,
   `cargo test -p tri`, `cargo test -p t27c --test icarus_lowerable`,
   `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`,
   `./scripts/tri test --fast`.
6. **Synthesize / Land** — write closeout report, update `.trinity/current-issue.md`,
   `.trinity/experience.md`, persistent memory.

## Risk register

| Risk | Mitigation |
|------|------------|
| Icarus/Yosys rejects 4-MiBit concatenation | Stay at `[2]^17` (just at cliff); keep multi-line literal style; use function-call initializer to avoid duplicating the literal. |
| Signed i16 overflow in leaf values | Use `(2*i)%32768` / `(2*i+1)%32768`. |
| Parser silently truncates single-line mega-literal | Emit multi-line brace style matching W584. |
| Compiler change breaks existing module-var seals | Reseal affected specs after verifying diff is expected. |

## Verification target

- 76/76 Icarus PASS, 76/76 cocotb PASS, 0 seal mismatches.
- `cargo test -p t27c --test icarus_lowerable` at 49/0 with new W589 test.
