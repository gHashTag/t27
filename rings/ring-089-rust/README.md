# ring-089 -- TNN ISA (balanced-ternary core)

> **Wave 16** (2026-05-22, Closes #719): the second honestly-imported Wave-11
> crate. Wave 15 landed [ring-088](../ring-088-rust) (GF16 codec + MAC).
> Wave 16 lands ring-089: a small but *complete and executable* model of t27's
> balanced-ternary core ISA.
>
> Anchor: `phi^2 + 1/phi^2 = 3`.

## What

A real, compileable, tested implementation of:

1. **`Trit`** -- a wrapped `i8` in `-1..=1`, mirroring
   `TRIT_NEG`/`TRIT_ZERO`/`TRIT_POS` from
   [`specs/isa/ternary_arithmetic.t27`](../../specs/isa/ternary_arithmetic.t27).
2. **`Word27`** -- 27 trits packed into `[i8; 27]` (LSB at index 0), matching
   `TRITS_PER_WORD = 27` and `REG_WIDTH = 27` from
   [`specs/isa/registers.t27`](../../specs/isa/registers.t27). Bijective
   `from_i64` / `to_i64` over `|v| < 3^27 / 2` using Euclidean (floor)
   division -- Rust's default truncating `/` is incorrect for negative
   values and gives wrong digits, see the doc-comment on `from_i64`.
3. **`trit_add(a, b, cin) -> (sum, cout)`** -- single-trit balanced-ternary
   adder per `specs/isa/ternary_arithmetic.t27`. Sums in `-2..=2` decompose
   into a digit in `-1..=1` plus a carry in `-1..=1`.
4. **`word_add` / `word_sub`** -- 27-trit ripple-carry adder over `Word27`.
   `word_sub(a, b) = word_add(a, b.negate())`.
5. **`Opcode` (9 entries)** -- `Nop`, `Mov`, `Addi`, `Add`, `Sub`, `Neg`,
   `Load`, `Store`, `Halt`. A deliberate *subset* of the spec's ISA, not
   a claim of full coverage.
6. **`Cpu`** -- fetch / decode / execute loop with 27 registers (R0
   hardwired to zero), a 64-instruction code memory, and a 256-cell data
   memory. `Cpu::step` runs one instruction; `Cpu::run(max_steps)` runs
   until `HALT` or step budget exhaustion.
7. **`identity_witness`** -- returns `true` iff `phi^2 + 1/phi^2 == 3`
   to f64 1e-15. Required of every t27 ring crate.

The crate is `#![no_std]` (test cfg pulls `std` for the harness only),
`#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`. **No external
dependencies.**

## Honest scope (R5-HONEST)

* **No GF16 instructions, no ternary-gates ALU, no pipeline, no branch
  prediction, no Coptic encoding.** Those layers exist in the spec
  (`specs/fpga/ternary_isa.t27` etc.) but are not part of Wave 16. Future
  waves can extend the opcode table.
* **No new spec.** `TRIT_NEG`, `TRIT_ZERO`, `TRIT_POS`, `NUM_REGISTERS`,
  `TRITS_PER_WORD`, `REG_WIDTH`, balanced-add carry rules all mirror
  existing `.t27` source byte-for-byte (L6 CEILING).
* The previous Wave-11 narrative claimed ring-089 was authored "in another
  sandbox" with ~334 LOC, but no source ever reached this repo. Wave 16
  is the real import; this is the actual ring-089 (635 LOC).

## Tests (15, all pass)

| Test                                       | What it proves                                                |
|:-------------------------------------------|:--------------------------------------------------------------|
| `identity_witness_holds`                   | `phi^2 + 1/phi^2 == 3` to 1e-15                               |
| `trit_construction_rejects_out_of_range`   | `Trit::new` rejects values outside `-1..=1`                   |
| `trit_add_basic_table`                     | spot-checks of the balanced-ternary single-trit adder         |
| `word_zero_roundtrip`                      | `Word27::zero().to_i64() == 0`                                |
| `word_from_i64_roundtrip_small`            | bijective round-trip incl. negative inputs (`-13`, `-100`, ...) |
| `word_add_arithmetic_matches_i64`          | `word_add` agrees with `i64 +` across mixed signs             |
| `word_sub_arithmetic_matches_i64`          | `word_sub` agrees with `i64 -` across mixed signs             |
| `negate_is_involution`                     | `negate(negate(w)) == w` and `to_i64(negate(w)) == -to_i64(w)` |
| `trit_at_and_set_trit_bounds`              | index-27 accesses return `None`/`false`                       |
| `cpu_r0_is_hardwired_zero`                 | writes to R0 are ignored (spec invariant)                     |
| `cpu_addi_chain`                           | three `ADDI` instructions accumulate the immediate            |
| `cpu_add_sub_neg`                          | `ADD`/`SUB`/`NEG` produce the i64-equivalent values           |
| `cpu_load_store_roundtrip`                 | `STORE` then `LOAD` round-trips through data memory           |
| `cpu_halt_stops_execution`                 | `HALT` halts and freezes `pc`                                 |
| `cpu_phi_identity_integer_projection`      | **`floor(phi)+floor(1/phi)+ceil(phi^2-2)=3`** through CPU ops |

The last test is the second time the project's identity anchor is exercised
*through actual numeric kernels* (after Wave 15's `mac_dot_phi_identity`).
It uses the rational integer projection `floor(phi) + floor(1/phi) +
ceil(phi^2 - 2) = 1 + 0 + 2 = 3`, executed entirely with `ADDI`/`ADD`/`HALT`
on the `Cpu` model. The point is *not* that this equals the float anchor
exactly -- it is the smallest integer expression that lands on `3` and
exercises the full fetch / decode / execute loop.

## Build

```bash
cd rings/ring-089-rust
cargo check --all-targets   # green on Rust 1.83.0
cargo test --lib            # 15 passed, 0 failed
```

Local verification on Rust 1.83.0 (matching
[`Dockerfile.rust`](../../Dockerfile.rust)) -- the Wave-13 `rings-rust`
matrix will re-confirm on every PR that touches this crate.
