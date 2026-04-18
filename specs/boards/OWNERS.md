# specs/boards/ — Board Profiles

Domain: FPGA board pin assignments, clock constraints, I/O standards.

| Profile | File | Target | Status |
|---------|------|--------|--------|
| Minimal | `xc7a100t_minimal.t27` | QMTECH XC7A100T (LED+UART) | All pins prjxray-verified |
| Full | `xc7a100t_full.t27` | QMTECH XC7A100T (LED+UART+SPI+MAC) | 4 SPI + 32 MAC pins missing from prjxray-db |
| Arty A7 | (via `--board arty-a7`) | Digilent Arty A7-100T | All minimal pins prjxray-verified |

Pin coverage details: `docs/fpga/PIN_COVERAGE.md`

Primary: FPGA team
Related: `specs/fpga/`, `specs/pins/` (when available)
