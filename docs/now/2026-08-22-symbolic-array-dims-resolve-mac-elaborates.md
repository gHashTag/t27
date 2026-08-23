# NOW -- symbolic array dims resolve; mac.v elaborates with zero errors (2026-08-22)

## const-name dimensions substitute before codegen (Closes #2275)

- The second half of the mac defect was one gate, not many: array dimensions
  spelled as const names (`[NUM_MAC_UNITS]MACUnit`) never parsed -- the whole
  packed-array machinery keys off `parse_array_type`, which wants digits -- so
  the AoS declaration emitted `reg [31:0]` plus a TODO and every element-field
  access flattened to an unbound name.
- `resolve_symbolic_dims` now substitutes integer-literal module consts into
  every type/dimension string once, before codegen. mac_units becomes a real
  `reg [1343:0]` (8x168) with per-element part-select initializers, and
  **iverilog elaborates mac.v with zero errors** for the first time.
- The folded struct-literal initializer writes 0 with a comment; for this spec
  that is exact (every field of the literal is zero / STATUS_READY = 0).
- Blast radius measured before landing: 13 specs corpus-wide carry symbolic
  dims (2 under specs/fpga). Full 32-module yosys smoke stays 32/32. M5
  performed.
- Unlocks #2241: the conformance vvp lane's blocker module now elaborates.
