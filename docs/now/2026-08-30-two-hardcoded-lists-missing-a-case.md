# NOW -- Two hardcoded lists, each missing a case (2026-08-30)

## The Rust backend, never audited before (Refs #2844)

- `has_body` decides whether a function has a body by matching statement kinds against a fixed list, and StmtAssign was absent
- so a function whose body is assignments ONLY tested as bodiless and was emitted as `{ unimplemented!() }` -- a stub rustc accepts everywhere and that panics at run time
- 53 functions in 35 specs, and Zig and C lower every one: the clock `tick()` in the FPGA testbenches, `uart_reset`, six state setters, both `on_clock` hardware steps
- `expr_is_bool` had no arm for ExprFieldAccess, so `!debouncer.enabled` became `((debouncer.enabled) == 0)` -- E0308, a hard rustc error
- the emitter prints `pub enabled: bool,` into the same file and never consulted it; now it remembers, per file
- keyed by NAME and not by (struct, field) because ZERO of the 650 specs declare one field name as bool in one struct and something else in another; 35 names collide ACROSS files, which is why the set is cleared per file
- measured: unimplemented stubs 870 -> 817, integer zero-tests on a bool field 25 -> 2, rustc accepts 214 -> 216
- this is #2844 one level up: that fixed the CONTENT of the assignment arms; this is the gate that stopped them being reached
