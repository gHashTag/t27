# NOW -- conformance vectors execute against RTL for the first time (2026-08-22)

## the vvp lane opens: mac's vectors run and pass, debt stays visible (Refs #2241)

- `tools/run_conformance_vvp.py` generates a hierarchical-call testbench per
  registry module, applies each vector case, and fails loudly on mismatch.
  mac: 8/8 cases pass (extract_trit x3, mac_multiply x4, invalid_unit x1) --
  the first execution of the checked-in vector corpus in its history.
- Negative control: a planted fault (Trit_pos flipped to 0 in a copy of the
  RTL) produces 4 FAILs and exit 1. The gate is real in both directions.
- The fpga-conformance job now runs this instead of compile-everything /
  execute-nothing: the compile+run set is the executed-vector registry
  (doctrine as formal v1 -- thin and real beats broad and vacuous); the 33
  other vector files and mac's 6 unexecuted groups are printed as open debt,
  never counted as covered.
- Enabled by today's #2275 closure: mac.v elaborates with zero errors.
