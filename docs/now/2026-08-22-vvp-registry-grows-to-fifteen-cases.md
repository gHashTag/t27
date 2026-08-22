# NOW -- the vvp registry grows to 15 cases; staging fixes an order bug the gate itself caught (2026-08-22)

## pack_trit, mac_cycle, mac_status join the executed set (Refs #2241)

- Three more groups execute: pack_trit (3), mac_cycle (2), mac_status (2) --
  mac coverage 8 -> 15 cases, all passing.
- The gate caught its own first bug: mac_status_initially_ready read
  STATUS_DONE because state-writing ops (mac_cycle, mac_multiply) ran before
  the "initially" read. Cases now carry a stage (fresh reads -> stateless ->
  stateful ops -> post-op reads); the vector's own id encodes which.
- Debt shrinks and stays visible: mac's unexecuted groups are now 3
  (dot_product and matrix_vector need slice arguments; reset needs an
  accumulator reader).
