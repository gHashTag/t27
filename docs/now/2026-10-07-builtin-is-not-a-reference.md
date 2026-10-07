# NOW -- a builtin call `@pow(...)` no longer splices a spec's own `pow` (2026-10-07)

## bootstrap/src/use_resolve.rs (Closes #7292)

- The resolver collects every identifier a spec reads and splices the declaration of each one it finds in an imported spec. It read `@pow(x, y)` as a read of `pow`.
  - `specs/memory/formula_embed.t27` calls `@pow` and imports `PHI` from `math::constants`. So t27c spliced that spec's own `pow`, and with it `floor`, `exp_approx`, `ln_approx` and `E`, which `pow` reads. Nothing in the importer calls any of them.
- A token right after `@` is now left out of the identifier set. The arguments of the call are still read: `@pow(base, @as(f64, exp))` reads `base`, `f64` and `exp`.
- Two tests in `use_resolve.rs`: a unit test of the identifier set, and a splice of a two-spec tree where `@pow(...)` must not splice `pow` and a bare `pow(...)` must. With `builtin = false` in place of the check, both fail and the other 40 pass. `unresolved_use` (2) and `dotted_module_name` (9) pass.
- Measured on the Railway lab on the #7191 fourth-slice tree. The #7281 build and the same tree with this change were run over all 1363 `.t27` files under `gen`, `gen-c`, `gen-rust`, `gen-verilog` and `typecheck`:
  - The exit code changes on none of the 5 x 1363 runs, and stderr changes on none.
  - Output changes in 7 files, under `gen`, `gen-c` and `gen-rust`; `gen-verilog` does not change. Every changed line is a removed line.
  - `formula_embed` loses `pow`, `floor`, `exp_approx`, `ln_approx` and `E` (97 lines of `gen`). It keeps `abs`, which its test calls by name.
  - Six specs lose a spliced `abs` they read only as `@abs`: `ml/activation/gelu_activation`, `ml/activation/sigmoid_activation`, and `tri/math/bezier`, `constants`, `matrix` and `probability`. Each imports `math::constants` whole.
  - `typecheck` stdout differs on `formula_embed` only: 3 "cannot assign to immutable" warnings go, which named lines of the spliced `pow` body. 14 warnings become 11.
- `t27c test-report` gives the same result on all 7 before and after: 6 are blocked on the same zig error, and `sigmoid_activation` fails the same way.
- `cc -fsyntax-only` on the `gen-c` output reports the same number of errors on all 7. On 6 of them gcc warns more, e.g. `probability` from 10 warnings to 62.
  - The cause is in the C backend: it writes `@abs(x)` and `@pow(x, y)` into the C as they stand. gcc reads the `@` as a stray character and calls C's own `int abs(int)`.
  - The dead `double abs(double)` that the false splice declared gave that call a prototype. Filed as #7297: 85 of 1362 `gen-c` outputs carry a real builtin's `@` spelling, 769 sites.
- No seal changes. `t27c seal --verify` prints "all hashes MATCH" on all 7 with both builds.
