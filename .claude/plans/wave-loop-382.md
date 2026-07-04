# Wave Loop 382 Plan

**Issue:** #1274
**Selected variant:** B — proof push to 272 generic ∀ + array/RAM lowering prototype
**Branch target:** `wave-loop-382` from `trinity-rust-rings`

---

## Phase breakdown

### 1. Issue (completed)
- GitHub issue #1274 created for Wave Loop 382.
- `.trinity/current-issue.md` updated.

### 2. Spec
- Add `specs/scratch/w382_ram_lowering.t27` exercising:
  - module-level `var mem : [8]u16`
  - write `mem[addr] = data`
  - read `out = mem[addr]`
  - optionally a tiny FIFO wrapper to exercise sequential read/write

### 3. TDD
- Include `test` assertions in the regression spec:
  - Write then read same address returns data.
  - Write to distinct addresses keeps values separate.
  - FIFO push/pop ordering.

### 4. Code
- Modify `bootstrap/src/compiler.rs`:
  - Detect module-level `var` with array type `[N]T` in `gen_verilog_var` / `gen_verilog_module`.
  - Emit `reg [W-1:0] mem [0:N-1];` for array vars.
  - Handle `ExprIndex` (`mem[i]`) in `gen_verilog_expr` to emit `mem[i]`.
  - Handle `StmtAssign` where LHS is `ExprIndex` to emit `mem[i] = x;`.
  - Ensure width inference for element type `T`.

### 5. Gen
- Run `./scripts/tri gen` (or `t27c gen-verilog` on the scratch spec) to produce `gen/` artifacts.

### 6. Seal
- Run `t27c seal --save` for all 28 affected specs (27 IGLA + scratch).

### 7. Verify
- Run `t27c suite --repo-root .` until 0 failures.
- Run `lake build Trinity.TernaryInference` until pass.
- Run yosys smoke on the new RAM regression spec.

### 8. Land
- Commit with `Closes #1274`.
- Push topic branch `wave-loop-382`.
- Open PR #1275 against `trinity-rust-rings`.

### 9. Learn
- Update `.trinity/experience.md`.
- Save memory file `wave-loop-382.md`.

---

## Risk mitigation

- If array lowering proves too broad for one wave, reduce to single-port memory only (no FIFO).
- If proof count target risks Lean timeout, fall back to Variant A (proof-only) and defer RAM work.
- Keep duplicate theorem-name check before appending to avoid W381 deduplication issue.

---

*phi² + 1/phi² = 3 | TRINITY*
