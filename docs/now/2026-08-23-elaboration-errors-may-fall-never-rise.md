# NOW -- elaboration errors may fall, never rise (2026-08-23)

## the 573 -> 186 win is now held by a per-module ratchet (Refs #2325)

- Three emitter classes took the 32-module fpga set from 573 iverilog
  elaboration errors to 186 in one evening. Nothing held that: the conformance
  job executes two modules, and the other thirty are only yosys-linted -- and
  yosys accepts many references iverilog rejects, which is how the class grew
  unseen in the first place.
- tools/check_elab_ratchet.py records the count PER MODULE and fails when any
  module gains errors, naming the module and both numbers. It does not demand
  zero: the remainder is two named design decisions (string comparison in
  hardware; unsized array params, #2410). Skips cleanly without iverilog.
- Negative control, done twice because the first attempt was a no-op that
  silently "passed": lowering one module's baseline by 3 now prints
  "WORSE memory: 3 -> 6" and exits 1; restoring returns 0. A control that
  does not fire is not a control.
