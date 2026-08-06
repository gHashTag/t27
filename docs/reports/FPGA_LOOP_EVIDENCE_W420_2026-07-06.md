# FPGA Loop Evidence — Wave Loop 420 (2026-07-06)

**Issue:** #1361  
**Branch:** `wave-loop-420`  
**Variant:** C — no physical board evidence; bench still blocked (P12 unwired, DLC10 cable missing, no relay).

---

## Evidence claims

| Claim | Status | Method |
|-------|--------|--------|
| VCD `$comment` terminator is exact-token | ✅ Verified | `cargo test -p tri test_parse_vcd_comment_with_embedded_end_token` |
| Real-valued VCD nets auto-threshold | ✅ Verified | `cargo test -p tri test_parse_vcd_real_auto_threshold` |
| PVT half-period bound is monotone in process corner (`ff ≤ tt ≤ ss`) | ✅ Verified | `lake build Trinity.TernaryFPGABoot` + `cargo test -p tri test_pvt_half_ns_monotone_in_process_corner` |
| No regressions in CSV/VCD/fpga tests | ✅ Verified | `cargo test -p tri vcd` 13/13, `cargo test -p tri csv` 11/11, `cargo test -p tri fpga::tests` 48/48 |
| `./scripts/tri test` does not add yosys failures | ✅ Verified | 16 pre-existing gen-verilog yosys smoke failures (weak point #1245), no new ones |
| Physical CCLK capture for `OSCFSEL=6/7` | ❌ Not possible | P12 unwired, no DLC10 cable, no relay |
| Bitstream still ready | ✅ Regressed unchanged | `fpga/verilog/ternary_mac_demo_top.bit` from prior wave |

---

## Command outputs

### VCD tests

```
$ cargo test -p tri vcd
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running unittests src/lib.rs (...)
running 13 tests
test fpga::tests::test_parse_vcd_comment_with_embedded_end_token ... ok
test fpga::tests::test_parse_vcd_real_auto_threshold ... ok
test fpga::tests::test_parse_vcd_basic ... ok
test fpga::tests::test_parse_vcd_ignores_unselected ... ok
test fpga::tests::test_parse_vcd_missing_signal ... ok
test fpga::tests::test_parse_vcd_no_transitions ... ok
test fpga::tests::test_parse_vcd_real_threshold_filter ... ok
test fpga::tests::test_parse_vcd_scalar_bit_high ... ok
test fpga::tests::test_parse_vcd_scalar_bit_low ... ok
test fpga::tests::test_parse_vcd_scalar_multiple_changes ... ok
test fpga::tests::test_parse_vcd_threshold_filter ... ok
test fpga::tests::test_raw_to_lean_csv_output ... ok
test fpga::tests::test_raw_to_lean_period_jitter ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured
```

### PVT tests

```
$ cargo test -p tri pvt
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.04s
     Running unittests src/lib.rs (...)
running 10 tests
test fpga::tests::test_pvt_half_ns_antitone_in_vccint ... ok
test fpga::tests::test_pvt_half_ns_monotone_in_process_corner ... ok
test fpga::tests::test_pvt_half_ns_monotone_in_temperature ... ok
test fpga::tests::test_pvt_low_ns_antitone_in_vccint ... ok
test fpga::tests::test_pvt_low_ns_monotone_in_process_corner ... ok
test fpga::tests::test_pvt_low_ns_monotone_in_temperature ... ok
test fpga::tests::test_pvt_setup_ns_antitone_in_vccint ... ok
test fpga::tests::test_pvt_setup_ns_monotone_in_process_corner ... ok
test fpga::tests::test_pvt_setup_ns_monotone_in_temperature ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured
```

### Lean 4 build

```
$ lake build Trinity.TernaryFPGABoot
[2967/2967] Building Trinity.TernaryFPGABoot
Build succeeded.
```

### Full tri pipeline

```
$ ./scripts/tri test
...
gen-verilog yosys smoke summary: 16 pre-existing failures (all from #1245, none new)
Other phases: PASS
```

---

## Hardware blocker

```
$ cargo run -p dlc10 -- idcode
DLC10 cable not found (VID=0x03FD)
```

The cable and P12 wiring are still missing. Until the physical layer is
restored, silicon evidence will be produced from captures, not live toggling.

---

*φ² + φ⁻² = 3 | TRINITY*
