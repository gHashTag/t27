# NOW -- build_verify counts measured, 2 stale seals retired from the baseline (2026-10-06)

## Seal currency: BuildVerify (Closes #6659)

- `specs/fpga/verification/build_verify.t27` failed `test_module_count`: #1399 set `TOTAL_FPGA_MODULES = 33`, `TOTAL_BOARD_CONFIGS = 3`, `TOTAL_SPECS = 66` while the test still asserted 31. None of those matched the tree. The constants are now the counts measured on master 099ac2224: 36 modules (`specs/fpga/*.t27`), 30 testbenches, 2 board configs, 68 total, and `t27c gen-verilog` succeeds on all 68 on the Railway lab, so `VERILOG_FILES == TOTAL_SPECS` holds at 68.
- `t27c test-report`: 11/11 pass. Negative control: `TOTAL_FPGA_MODULES = 35` fails `test_module_count` and `test_total_specs`.
- Resealed on the Railway t27c lab with master's compiler (099ac2224): `BuildVerify.json`, `verification_BuildVerify.json`. Their two `stale` lines leave `tools/seal_baseline.txt` (stale 30 -> 28). `tools/check_seal_coverage.py`: OK, 1456 seals, 1314 hold.
