# NOW -- mac_reset executes: a pure accumulator reader closes the observability gap (2026-08-22)

## 15 -> 18 cases; uart's vectors are measured as aspirational (Refs #2241)

- The reset vectors check the accumulator AFTER mac_reset, and no function
  exposed it -- the only read path (mac_cycle's return) also writes. The spec
  gains `mac_acc_read(unit) -> i32`, a pure reader; the mac_reset group joins
  the registry as staged sequences (dirty -> reset -> read). 18/18 pass.
- mac's unexecuted debt is down to the two slice-argument groups
  (mac_dot_product, mac_matrix_vector).
- uart measured honestly: its vectors describe BIT-LEVEL protocol behavior
  (framing errors, sync stages, tx bit patterns) that the combinational spec
  exposes through no function -- the same aspirational-artifact class as the
  old formal props. Recorded in #2241; executable only after the module gains
  real sequential I/O (#2266/#2238 lanes).
