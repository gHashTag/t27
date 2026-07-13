# Wave Loop 518 — Closeout Report

**Issue:** #1487  
**Branch:** `wave-loop-518`  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Selected variant

**Variant A:** clear the remaining W508 `break`/`continue` yosys and Icarus smoke
baselines, plus the function-local pragma Icarus boundaries.

---

## What changed

### 1. Flag-based `break`/`continue` lowering

Replaced the broken Verilog emission for `break` (`disable fork;`) and
`continue` (`/* continue */;`) with a portable flag encoding:

- Every loop receives two `reg` flags: `__break_flag_N` and
  `__continue_flag_N`.
- `break` sets the innermost break flag; the loop condition includes
  `&& !__break_flag_N`.
- `continue` sets the innermost continue flag; every remaining body statement
  is guarded by `if (!__break_flag_N && !__continue_flag_N)` so the rest of the
  iteration is skipped.
- `for` loops emit a guarded manual increment so `continue` still reaches the
  increment.
- Nested loops use a stack so flags never leak across scopes.

Files touched: `bootstrap/src/compiler.rs`.

### 2. Function-local pragma suppression

Icarus Verilog 12.0 rejects attribute specifiers (`(* ram_style = ... *)`) on
declarations inside function/task bodies. The emitter now tracks
`in_function_body` and suppresses pragma output for local declarations while
still honoring them at module scope. The yosys/Vivado backend continues to
receive the pragma from the spec; the attribute is simply omitted from the
Icarus-compatible Verilog text.

Files touched: `bootstrap/src/compiler.rs`.

### 3. Test expectation update

Updated `test_parse_for_range_verilog` to match the new guarded for-loop header.

Files touched: `bootstrap/src/compiler.rs`.

### 4. Baselines cleared

- `docs/reports/gen_verilog_smoke_baseline.json` — W508 yosys entries removed,
  now empty.
- `docs/reports/gen_verilog_iverilog_smoke_baseline.json` — W508, W468 and W514
  Icarus entries removed, now empty.

---

## Validation

| Gate | Result |
|------|--------|
| `cargo build --release` | ✅ |
| `cargo test -p t27c --bin t27c` | 1525 passed, 0 failed, 2 ignored |
| `./scripts/tri test --icarus-lowerable --fast` | 0 failures, 0 seal mismatches, 0 yosys/Icarus baseline failures |
| `./scripts/tri verify --lean-lowerable` | ✅ passed (251 lowerable specs) |
| Manual W508 witnesses (yosys + Icarus + simulation) | ✅ all pass |
| Manual W468/W514 function-local pragma witnesses (Icarus) | ✅ all pass |

Suite summary:

```
Parse failures:           0
Typecheck fails:          0
Gen Verilog fails:        0
Gen Verilog smoke fails:  0
Gen Verilog Icarus fails: 0
Seal mismatches:          0
Icarus lowerable:         224
Icarus smoke pass/fail:   224/0
TOTAL FAILURES:           0
YOSYS BASELINE FAILURES:  0
ICARUS BASELINE FAILURES: 0
ACCEPTABLE:               yes
```

---

## Reseal

All 744 specs under `specs/` were resealed because the generated Verilog for
any loop-containing spec changed. Only the Verilog hash moved; Zig, C, and Rust
hashes remained stable for non-loop changes.

---

## Scientific anchors

- IEEE Std 1800-2017 §10.6/§10.7 — `break`/`continue` semantics.
- Sutherland & Mills, *Synthesizable SystemVerilog: Busting the Myth that
  SystemVerilog is only for Verification* (SNUG 2013), §4.5: `break`/`continue`
  are synthesizable and recommended over `disable`.
- Sutherland, *Modeling with SystemVerilog in a Synopsys Synthesis Design Flow*
  (SNUG Europe 2006), §8.4: `break`/`continue` replace `disable`.

---

## Remaining boundaries

- No documented gen-verilog yosys or Icarus smoke baseline failures remain on
  `wave-loop-518`.
- Packed scalar struct equality/inequality in the Icarus-lowerable subset is a
  candidate for the next wave (Variant B).

---

*φ² + φ⁻² = 3 | TRINITY*
