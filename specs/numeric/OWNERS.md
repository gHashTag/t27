# OWNERS — specs/numeric/

## Primary

**N-Numeric** — GoldenFloat GF4–GF32 and related numeric specs.

## Dependencies

- `specs/base/types.t27`, `specs/math/constants.t27`.
- `conformance/*gf*` vectors (**E-Evidence**).

## Generates

- `gen/zig/*gf*.zig`, `gen/c/*gf*.c`, `gen/verilog/*gf*.v` (via `t27c`).
