# NOW -- an unannotated local keeps the type it was copied from (2026-08-23)

## 213 -> 186 elaboration errors; the call-only version measured zero (Refs #2325)

- `var result = mem;` recorded an EMPTY type, so `result.port_count` fell past
  the part-select branch and flattened to the unbound `result_port_count`
  (hir.v, memory.v, timing.v). The local now inherits its type from the
  initializer: a callee's return type for a call, or the param/local it copies.
- Method note worth keeping: the call-only arm was written first and measured
  ZERO change over the 32-module set. The shape that actually dominates this
  corpus is the plain copy of a parameter -- writing the obvious arm and
  measuring it before adding the second is what showed which one mattered.
- Ordering verified rather than assumed: params are registered at the top of
  gen_verilog_fn BEFORE the locals loop reads them, so the lookup is sound and
  not an accident of a stale map.
- 213 -> 186 over the fpga set, yosys 32/32, both vector modules pass,
  mac self-test still elaborates with zero errors. M5 performed.
