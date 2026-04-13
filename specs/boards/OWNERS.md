# specs/boards/ — Board Profiles

Domain: FPGA board pin assignments, clock constraints, I/O standards.

| Profile | File | Target | Status |
|---------|------|--------|--------|
| Minimal | `xc7a100t_minimal.t27` | QMTECH XC7A100T (LED+UART) | All pins prjxray-verified |
| Full | `xc7a100t_full.t27` | QMTECH XC7A100T (LED+UART+SPI+MAC) | 22 pins missing from prjxray-db |

Primary: FPGA team
Related: `specs/fpga/`, `specs/pins/` (when available)
