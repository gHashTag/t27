# NOW -- one &str field no longer makes a whole struct unaddressable (2026-08-22)

## strings pack at zero width; 573 -> 228 elaboration errors across the fpga set (Closes #2424)

- Measured first: 100 of 438 structs are rejected for lowering ONLY by string
  fields while all their other fields are primitive scalars. The damage is not
  the missing string -- lim.max_luts on such a struct flattens to the unbound
  identifier lim_max_luts, so the NUMERIC half is unreachable too.
- A string has no hardware representation, so it now packs at zero width and is
  skipped when accumulating offsets. Reading one still flattens to an unbound
  name -- exactly as broken as before, which is honest for a value hardware
  cannot hold -- and no existing layout can shift, because a struct with a
  string field does not lower today at all.
- Measured after: 573 -> 228 elaboration errors over the 32-module set (-60%),
  power_analysis 14 -> 1, yosys smoke 32/32 unchanged, both executed vector
  modules still pass. M5 performed.
