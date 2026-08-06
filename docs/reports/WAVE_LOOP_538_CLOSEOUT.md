# Wave Loop 538 Closeout — VCD probe + independent cocotb reference-model check

**Issue:** #1509  
**Branch:** `wave-loop-538`  
**Status:** closed  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Delivered

1. **VCD waveform capture in generated simulation testbenches**
   - `bootstrap/src/compiler.rs`: when `emit_test_assertions` is true, the
     generated Verilog now emits `$dumpfile("dump.vcd"); $dumpvars(0);` inside
     `// synthesis translate_off` so only simulation output changes; synthesis
     seals remain stable.

2. **Scalar probe registers for every `assert_eq` actual expression**
   - `VerilogCodegen::gen_verilog_test` pre-declares `reg [63:0]` probes at the
     top of each test block and `gen_verilog_test_stmt` assigns them with the
     actual expression value before the self-checking comparison.
   - A `[PROBE] <block> <idx> = %0d` line is emitted for visibility.

3. **Independent VCD cross-check in the Python reference model**
   - `scripts/cocotb_ref_model.py`:
     - VCD capture works in both direct `iverilog`/`vvp` mode and cocotb runner
       mode.
     - A minimal built-in VCD parser reads the final probe values.
     - Probe values are compared against the independently evaluated expected
       literal from the t27 AST.
     - Negative expected values are interpreted as signed 64-bit two's
       complement to match the Verilog probe.
     - Probes whose value is `X` (typically because the actual expression is
       wider than 64 bits) are skipped gracefully; the log-based self-check
       remains the authoritative result for those cases.

4. **Icarus simulation baseline normalization**
   - `bootstrap/src/suite.rs`: `normalize_icarus_output` now filters out the
     VCD startup diagnostics and `[PROBE]` debug lines so Phase 3d baselines
     continue to track only `[TEST]` status lines.

---

## Validation gates

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 4 passed; 0 failed |
| `./scripts/tri test --icarus-simulate --icarus-lowerable --cocotb --fast` | 35 Icarus PASS, 35 cocotb PASS, 0 seal mismatches |
| `lake build Trinity.IcarusLowerable.Soundness` | 8572 jobs, 0 `sorry` |
| Pre-existing yosys smoke baseline failures | 24 unchanged |

---

## Known limitations / next-wave seeds

- The 64-bit probe width limits independent VCD comparison to scalar values that
  fit in 64 bits.  Wide packed-struct arrays and whole-array comparisons fall
  back to the log-based self-check.
- The Python expected-value evaluator currently handles literals and simple
  constant expressions; variables, function calls, and struct/array indexing
  in the expected position are skipped rather than independently evaluated.

---

*φ² + φ⁻² = 3 | TRINITY*
