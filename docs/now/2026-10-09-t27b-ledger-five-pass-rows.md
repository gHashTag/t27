# NOW -- t27b ledger: five pass specs get a row, cap stays 21 (2026-10-09)

## five new specs are named in the ledger, and the cap still equals the real count (Closes #8215)

- New rows, all pass, each from its own PR's t27b-native-ratchet run: `compiler/theory/warmup`
  and `warmup_t27b` (#8182), `fpga/openxc7-synth/DSP48E1_mock` (#7970),
  `fpga/rtl/fpga_test_reporter` (#7926), `tri/t27b/verdict_key` (#8208).
- `max_not_pass` stays 21. #8173 and #8174 already brought it down to the real count of rows
  that are neither `pass` nor `pass_vacuous`, so the cap half of #8215 is done on master.
- Result: 1230 rows, unique, sorted, none missing, not_pass 21 = `max_not_pass` 21.
