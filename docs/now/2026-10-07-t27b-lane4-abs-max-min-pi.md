# NOW -- t27b lowers @abs, @max, @min and std.math.pi (2026-10-07)

## t27b lane 4: Zig builtins the reference prints as written (Closes #7394)

- `@abs(x)` shares the bare `abs(x)` lowering; an integer operand stays refused (Zig's `@abs` of an `iN` is a `uN`).
- `@max` / `@min` of two floats of one type: a NaN gives the other operand, equal operands give the first, as Zig 0.16 Debug does; of two integers of one type, the larger / smaller.
- `std.math.pi` and `std.math.e` are Zig's own comptime_float literals, rounded to binary128.
- Conformance first: `specs/tri/t27b/conformance/builtin_abs_max_min_pi.t27`, 12/12 on `t27c test-report`, 0 vacuous.
- `specs/ml/rl/dqn.t27` and `specs/port/trinity/src/tri/string_dualities.t27` move to `pass`.
