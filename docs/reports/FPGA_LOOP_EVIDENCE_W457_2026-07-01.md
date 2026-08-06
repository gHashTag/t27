# FPGA Loop Evidence — Wave Loop 457 (2026-07-01)

**Issue:** #1428
**Branch:** `wave-loop-457`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was done

Wave Loop 457 executed **Variant B** from
`docs/reports/FPGA_LOOP_COOPERATION_W457_2026-07-01.md`: with the physical bench
still unavailable, add **RAM style pragma support** for module-level arrays.

A new top-level `pragma name = "value";` statement is parsed before the next
module-level declaration. For `ram_style = "block"` and
`ram_style = "distributed"` the compiler stores the attribute and emits the
standard Verilog `(* ram_style = "..." *)` annotation on the synthesizable
`reg ... [0:N]` memory declaration.

### Files changed

- `bootstrap/src/compiler.rs`
  - Added `KwPragma` token and `pragma` keyword lexer mapping.
  - Added `extra_pragma: String` to `Node`; initialized in `Default` and `new`.
  - Added `pending_pragma` to `Parser` and `ParserCheckpoint` with save/restore.
  - Added `parse_pragma` accepting `ram_style = "block"` /
    `ram_style = "distributed"`; unknown pragmas are rejected.
  - `parse_module_body` consumes `pragma` directives before the next
    module-level declaration.
  - `parse_const_decl` and `parse_var_decl` capture the pending pragma into the
    declaration node and clear it.
  - `gen_verilog_var` emits `(* {pragma} *)` before the `reg ... [0:N]`
    declaration for true array types.
  - Added `tests_w457_ram_style` unit-test module with three tests.

- `specs/scratch/w457_ram_style_block.t27`
- `specs/scratch/w457_ram_style_distributed.t27`
  - New regression specs for block and distributed RAM style pragmas.

- `.trinity/seals/scratch_w457_ram_style_block.json`
- `.trinity/seals/scratch_w457_ram_style_distributed.json`
  - New seals for the regression specs.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W457 competitor boundary section.

---

## Verification results

| Check | Result |
|---|---|
| `cargo test -p t27c --bin t27c tests_w457_ram_style` | **PASS** (3/3) |
| `t27c gen-verilog specs/scratch/w457_ram_style_block.t27` + `yosys read_verilog -sv; synth -top w457_ram_style_block` | **PASS** (emits `(* ram_style = "block" *)`) |
| `t27c gen-verilog specs/scratch/w457_ram_style_distributed.t27` + `yosys read_verilog -sv; synth -top w457_ram_style_distributed` | **PASS** (emits `(* ram_style = "distributed" *)`) |
| `./scripts/tri test --json /tmp/tri_test_w457.json` | **ALL TESTS PASSED** |
| Parse | 579 passed, 0 failed |
| Typecheck | 579 passed, 0 failed |
| Gen Zig | 579 passed, 0 failed |
| Gen Rust | 579 passed, 0 failed |
| Gen Verilog | 579 passed, 0 failed |
| Gen Verilog Yosys Smoke | **59 passed, 0 failed** |
| FPGA Board-Less Smoke Gate | **OK** (`phases green`) |
| FPGA Standalone Lake-Package Build | **OK** (`elapsed_ms ~386371`) |
| Gen C | 579 passed, 0 failed |
| Seal Verify | 579 passed, 0 failed |
| Fixed Point | 0 divergences |
| **TOTAL FAILURES** | **0** |
| `ACCEPTABLE` | **yes** |

---

## Not done

- **Physical bench execution:** still blocked. `dlc10 idcode` reports
  "DLC10 cable not found (VID=0x03FD)", P12 is unwired, and no automated cold-POR
  relay gate exists.
- **Live XADC capture / cold-POR boot:** no real hardware operation this wave.
- **ROM style pragmas / per-port attributes:** deferred to a future wave.

---

*φ² + φ⁻² = 3 | TRINITY*
