# FPGA Loop Evidence — Wave Loop 425 (2026-07-05)

**Issue:** #1374  
**Branch:** `wave-loop-425`  
**Variant executed:** C (bench blocked; formal/tooling hardening)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Expanded OSCFSEL 0–7 default sweep (board-less dry-run)

Command:

```bash
cargo run --release -p tri -- fpga cclk-sweep --dry-run fpga/verilog/ternary_mac_demo_top.bit
```

Result:

```
[cclk-sweep] 8 variant(s) will be swept from fpga/verilog/ternary_mac_demo_top.bit
[cclk-sweep] DRY RUN: no hardware will be touched; synthetic logs will be written.

== CCLK sweep summary ==
----------------------------------------------------------------------
 OSCFSEL  bitstream                         DONE    MODE  conclusion
----------------------------------------------------------------------
       0  ternary_mac_demo_top_oscfsel00.bit       1   0b001  DONE=HIGH: board boots from flash
       1  ternary_mac_demo_top_oscfsel01.bit       0   0b001  H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants
       2  ternary_mac_demo_top_oscfsel02.bit       0   0b001  H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants
       3  ternary_mac_demo_top_oscfsel03.bit       0   0b001  H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants
       4  ternary_mac_demo_top_oscfsel04.bit       0   0b001  H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants
       5  ternary_mac_demo_top_oscfsel05.bit       0   0b001  H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants
       6  ternary_mac_demo_top_oscfsel06.bit       0   0b001  H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants
       7  ternary_mac_demo_top_oscfsel07.bit       0   0b001  H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants
----------------------------------------------------------------------

=> First working variant: OSCFSEL=0
```

The dry-run sweep now covers all 8 documented Artix-7 CCLK selections.

---

## 2. FPGA smoke gate (board-less)

Command:

```bash
cargo run --release -p tri -- fpga smoke-gate
```

Result:

```
[smoke-gate] dry-run sweep report OK (8 variants)
[smoke-gate] yosys synthesis OK
[smoke-gate] complete
```

---

## 3. Lean 4 PVT envelope proof build

Command:

```bash
lake build Trinity.TernaryFPGABoot
```

Result:

```
✔ [2967/2967] Built Trinity.TernaryFPGABoot (17s)
Build completed successfully (2967 jobs).
```

New theorems:

- `pvt_half_ns_worst_case_is_upper_envelope`
- `pvt_low_ns_worst_case_is_upper_envelope`

Both prove that the documented worst-case operating point is the upper envelope
of the PVT-aware SCK low / half-period bounds over the operating rectangle.

---

## 4. Rust unit tests

Command:

```bash
cargo test -p tri
```

Result:

```
test result: ok. 93 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## 5. Local sweep

Command:

```bash
./scripts/tri test
```

Result:

- Parse: 576 passed, 0 failed
- Typecheck: 576 passed, 0 failed
- Gen Zig: 576 passed, 0 failed
- Gen Rust: 576 passed, 0 failed
- Gen Verilog: 576 passed, 0 failed
- Gen Verilog Yosys Smoke: 49 passed, 7 failed (pre-existing #1245 weak points)
- FPGA Board-Less Smoke Gate: OK
- Gen C: 576 passed, 0 failed
- Seal Verify: 576 passed, 0 failed
- Fixed Point: 0 divergences

The 7 yosys smoke failures are tied to major gen-verilog features already
tracked in `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` and are not introduced
by W425.

---

## 6. Hardware status

- **P12 CCLK probe:** unwired.
- **Relay/remote-power gate:** not available.
- **DLC10 cable:** still missing; Digilent HS2 + openFPGALoader is the working
  path.
- **XC7A200T board:** reachable via JTAG when the HS2 cable is connected (idcode
  `0x13631093` / `0x3636093` depending on cable).

---

*φ² + φ⁻² = 3 | TRINITY*
