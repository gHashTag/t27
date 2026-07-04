# FPGA Loop Evidence — 2026-07-05 (Wave Loop 399)

**Issue:** #1298  
**Branch:** `wave-loop-399`  
**Board:** QMTech Wukong V1 / XC7A200T-FGG676-1 (no physical session this wave)

## Hypothesis

W398 provided board-less CCLK tooling. W399 automates the cold-POR CCLK sweep so
that the only manual step is the physical power-cycle / cable handling.

## Evidence gathered

| Command | Status | Notes |
|---------|--------|-------|
| `tri fpga cclk-sweep ... --dry-run` | PASS | Generated 6 synthetic logs; summary table printed; first working variant identified. |
| `tri fpga sweep-report` | PASS | Produced markdown report from 6 dry-run logs. |
| `tri fpga measure-cclk` | PASS | Printed CCLK pin and DSLogic settings. |
| `tri fpga measure-cclk --csv /tmp/fake_cclk.csv` | PASS | Estimated 25.000 MHz / 50.0 % on synthetic 25 MHz square wave. |
| `cargo build --release -p tri` | PASS | No compiler errors. |
| `./scripts/tri test` | PASS | 575/575 PASS, 56 yosys smoke targets. |

## Open questions for W400

1. Which raw `OSCFSEL` value first reaches `DONE=1` on the physical board after a
   true cold POR?
2. What is the actual CCLK frequency for that working variant?
3. Does the working variant remain reliable across multiple power cycles?

## Conclusion

W399 tooling is ready for the physical CCLK sweep. The next wave should run
`tri fpga cclk-sweep` on the board and capture CCLK with `tri fpga measure-cclk`.

---

*phi^2 + 1/phi^2 = 3 | TRINITY*
