# FPGA Loop Evidence — Wave Loop 438 (2026-07-05)

**Issue:** [#1407](https://github.com/gHashTag/t27/issues/1407)  
**Branch:** `wave-loop-438`  
**PR:** [#1410](https://github.com/gHashTag/t27/pull/1410) (predicted)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was changed

Wave Loop 438 executed **Variant B** from the W437 cooperation plan: turn the
W437 dry-run synthetic path into a CI artifact gate inside `tri fpga smoke-gate`.

- `tri fpga smoke-gate` gained `--synthetic-operating-point` and `--verify-lean`.
- When `--synthetic-operating-point` is used, the dry-run CCLK sweep runs with a
deterministic synthetic PVT context and the JSON sweep report is asserted to
carry `operating_point.source == "synthetic"` for every variant.
- When `--verify-lean` is used, the gate generates a synthetic raw-ns `.lean`
theorem and runs `verify-lean --expected-source synthetic` on it, producing a
machine-checkable end-to-end artifact trail.
- Edge-case unit tests were added for `verify_lean`: no theorem, missing summary
+ missing source comment, and mismatched expected source.
- The `tri fpga verify-lean --json` schema was documented in
`fpga/HARDWARE_SSOT.md`.
- The competitor landscape report was refreshed; no new competitor signals
surfaced before 2026-07-05.

The 7 residual `gen-verilog` yosys smoke failures from #1245 were intentionally
left untouched; Variant C (master-merge of the full fix set) remains a
dedicated future wave.

---

## Verification commands and results

### Rust unit tests

```bash
cargo test -p tri
```

Result: **126 passed; 0 failed** (3 new tests added for `verify_lean` edge cases).

### Full repo sweep (CI-like)

```bash
./scripts/tri test
```

Result:
- Parse / Typecheck / GF16 / Gen Zig / Gen Rust / Gen Verilog / Gen C / Seal: **576/576 PASS**
- Gen Verilog Yosys Smoke: **49 passed, 7 failed** (documented baseline from #1245)
- FPGA Board-Less Smoke Gate: **0 failed**

### Lean 4 build

```bash
cd proofs/lean4 && lake build Trinity.TernaryFPGABoot
```

Result: **Build completed successfully (2967 jobs).**

### End-to-end smoke-gate artifact gate

```bash
cargo run -p tri -- fpga smoke-gate --synthetic-operating-point --verify-lean --process-corner ss
```

Result: **PASS**. The gate completed the bit-config audit, the dry-run synthetic
sweep (8 variants with `source = synthetic`), generated a synthetic `.lean`
theorem, ran `verify-lean --expected-source synthetic`, and finished the yosys
synthesis smoke check.

### Default smoke gate (regression check)

```bash
cargo run -p tri -- fpga smoke-gate
```

Result: **PASS**. Default behaviour is unchanged; no synthetic source assertion
or verify-lean step is performed without the new flags.

---

## Known limitations

- Physical cold-POR capture remains blocked: the DLC10 JTAG cable is still not
detected on the host (`VID=0x03FD`) and the board P12 power header is still
unwired.
- The 7 residual `gen-verilog` yosys smoke failures remain the documented
baseline. They will be addressed in a dedicated master-merge wave (Variant C),
not mixed with the boot-evidence trail.

---

## Artifacts produced

- `cli/tri/src/fpga.rs` — smoke-gate synthetic + verify-lean integration and unit
tests.
- `fpga/HARDWARE_SSOT.md` — §3.6.23 documents the `verify-lean --json` schema.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` — refreshed with W438 boundary note.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` — updated to `wave-loop-438`.

---

*φ² + φ⁻² = 3 | TRINITY*
