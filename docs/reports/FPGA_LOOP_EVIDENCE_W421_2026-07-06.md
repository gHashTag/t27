# FPGA Loop Evidence — Wave Loop 421 (2026-07-06)

**Issue:** #1363  
**Branch:** `wave-loop-421`  
**Variant:** C — no physical board evidence; bench still blocked.

---

## Evidence claims

| Claim | Status | Method |
|-------|--------|--------|
| VCD `$timescale` terminator is exact-token | ✅ Verified | `cargo test -p tri test_parse_vcd_timescale_with_embedded_end_token` |
| Real-valued VCD auto-threshold works with `1 us` timescale | ✅ Verified | `cargo test -p tri test_parse_vcd_real_auto_threshold_us_timescale` |
| PVT half-period bound is monotone combined | ✅ Verified | `lake build Trinity.TernaryFPGABoot` + `cargo test -p tri test_pvt_half_ns_monotone_combined` |
| No regressions in VCD/PVT/fpga tests | ✅ Verified | `cargo test -p tri vcd` 15/15, `cargo test -p tri pvt` 11/11, `cargo test -p tri fpga::tests` 51/51 |
| `./scripts/tri test` does not add yosys failures | ✅ Verified | 16 pre-existing gen-verilog yosys smoke failures (weak point #1245), no new ones |
| Physical CCLK capture for `OSCFSEL=6/7` | ❌ Not possible | `openFPGALoader --detect` reports 0 devices |
| Competitor snapshot published | ✅ Verified | `docs/reports/T27_VS_FORMAL_HDL_2026.md` |

---

## Command outputs

### VCD tests

```
$ cargo test -p tri vcd
running 15 tests
test fpga::tests::test_parse_vcd_comment_with_embedded_end_token ... ok
test fpga::tests::test_parse_vcd_timescale_with_embedded_end_token ... ok
test fpga::tests::test_parse_vcd_real_auto_threshold ... ok
test fpga::tests::test_parse_vcd_real_auto_threshold_us_timescale ... ok
test fpga::tests::test_parse_vcd_real_to_raw_ns_25mhz ... ok
...
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured
```

### PVT tests

```
$ cargo test -p tri pvt
running 11 tests
test fpga::tests::test_pvt_half_ns_monotone_in_temp ... ok
test fpga::tests::test_pvt_half_ns_antitone_in_vccint ... ok
test fpga::tests::test_pvt_half_ns_monotone_in_process_corner ... ok
test fpga::tests::test_pvt_half_ns_monotone_combined ... ok
test fpga::tests::test_pvt_half_ns_lower_bound_across_operating_rectangle ... ok
...
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured
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
Gen Verilog Yosys Smoke: 40 passed, 16 failed
TOTAL FAILURES: 16 (all pre-existing from weak point #1245)
Other phases: PASS
```

### Hardware detection

```
$ openFPGALoader --detect -c digilent_hs2
empty
Jtag frequency : requested 6.00MHz    -> real 6.00MHz
found 0 devices
```

The Digilent FTDI cable is present (`0x0403:0x6014`), but the FPGA is not
responding on the JTAG chain. The board is either not powered or not connected.

---

*φ² + φ⁻² = 3 | TRINITY*
