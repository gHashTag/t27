# ring-090 -- HIR Simulator primitives

> **Wave 17** (2026-05-22, Closes #721): the third honestly-imported Wave-11
> crate. Waves 15 and 16 landed [ring-088](../ring-088-rust) (GF16 codec +
> MAC) and [ring-089](../ring-089-rust) (TNN ISA). Wave 17 lands ring-090:
> the simulator data-model + helpers defined by
> [`specs/fpga/simulator.t27`](../../specs/fpga/simulator.t27).
>
> Anchor: `phi^2 + 1/phi^2 = 3`.

## What

A real, compileable, tested implementation of the simulator primitives the
spec describes:

1. **`SimState`** -- 5-variant enum (`Idle`/`Running`/`Paused`/`Done`/`Error`)
   with `tag()` / `from_tag()`; tag values `0..=4` mirror the spec's
   `enum(i8) SimState` byte-for-byte.
2. **`SimConfig`** -- 7-field config (`name`, `max_cycles`, `clock_freq_hz`,
   `trace_enabled`, `vcd_output`, `break_on_error`, `vcd_path`); the default
   clock frequency is `DEFAULT_CLOCK_FREQ_HZ = 100_000_000` Hz, matching
   the spec's hard-coded constructor.
3. **`SimResult`** -- post-run report (`cycles`, `state`, `errors`,
   `assertions_fired`, `coverage_points`).
4. **`ProbePoint`** / **`TraceEntry`** -- single-signal probe and
   single-row trace record.
5. **Constructors** -- `sim_config`, `sim_config_with_trace`, `sim_ok`,
   `sim_error`, `probe`, `trace_entry`. All `const fn` where the spec's
   field-set is purely literal.
6. **Queries** -- `is_idle`, `is_done`, `is_error`, `has_errors`, `passed`.
7. **Time conversions** -- `sim_time_ns`, `sim_time_us`, `sim_time_ms`,
   `cycles_for_time_ns`. See **Time-conversion overflow note** below.
8. **`validate_sim_config`** -- counts invalid fields (empty name, zero
   max_cycles, zero clock_freq_hz).
9. **`identity_witness`** -- the universal anchor (`phi^2 + 1/phi^2 == 3`
   to f64 1e-15) required of every t27 ring crate.

The crate is `#![no_std]` (test cfg pulls `std` for the harness only),
`#![forbid(unsafe_code)]`, `#![deny(missing_docs)]`. **No external
dependencies.**

## Time-conversion overflow note (R5-HONEST)

The source spec uses `u32` for the multiplication

```text
sim_time_ns(cfg, cycles) = cycles * 1_000_000_000 / clock_freq_hz
```

but the spec's *own* canonical assertion is `sim_time_ns(_, 100) == 1000`
at `clock_freq_hz = 100_000_000`. With pure `u32` arithmetic,
`100 * 1_000_000_000 = 1e11` overflows `u32::MAX ~= 4.29e9` and the
assertion would fail. We faithfully implement the formula with a `u64`
intermediate and narrow back to `u32` at the end; the public signature
stays `u32 -> u32` exactly as in the spec, but the intermediate
arithmetic is the minimum width needed to make the spec's *own*
canonical test pass. Results that don't fit `u32` saturate at
`u32::MAX`. This is documented inline in `sim_time_ns` and
`cycles_for_time_ns`.

## Honest scope

* **No scheduler, no VCD writer, no event queue, no clock-domain
  crossing logic, no RTL execution.** Those layers live in adjacent
  specs (`vcd_trace.t27`, `clock_domain.t27`, `formal.t27`) and are
  *deliberately* out of scope for Wave 17.
* **No new spec.** Enum tags, struct field order, default values, and
  formula shapes mirror `specs/fpga/simulator.t27` byte-for-byte
  (L6 CEILING).
* The Wave-11 narrative quoted **2143 LOC** for ring-090. The honest
  Wave-17 measurement is **547 LOC**. The earlier number was a guess,
  not a measurement.

## Tests (19, all pass)

| Test                                | What it proves                                     |
|:------------------------------------|:---------------------------------------------------|
| `sim_config_creation`               | spec `test sim_config_creation`                    |
| `sim_config_with_trace_creation`    | spec `test sim_config_with_trace`                  |
| `sim_ok_result`                     | spec `test sim_ok_result` (6 sub-asserts)          |
| `sim_error_result`                  | spec `test sim_error_result` (5 sub-asserts)       |
| `probe_creation`                    | spec `test probe_creation`                         |
| `trace_entry_creation`              | spec `test trace_entry_creation`                   |
| `sim_time_ns_canonical`             | spec `test sim_time_ns` -> `1000 ns`               |
| `sim_time_us_canonical`             | spec `test sim_time_us` -> `1000 us`               |
| `sim_time_ms_canonical`             | spec `test sim_time_ms` -> `1000 ms`               |
| `cycles_for_time_ns_canonical`      | spec `test cycles_for_time_ns` -> `100 cycles`     |
| `validate_config_ok`                | spec `test validate_config_ok`                     |
| `validate_config_empty_name`        | spec `test validate_config_empty_name`             |
| `validate_config_zero_cycles`       | spec `test validate_config_zero_cycles`            |
| `invariant_max_cycles_positive`     | spec `invariant max_cycles_positive`               |
| `invariant_sim_time_positive`       | spec `invariant sim_time_positive`                 |
| `invariant_cycles_for_time_positive`| spec `invariant cycles_for_time_positive`          |
| `invariant_validate_non_negative`   | spec `invariant validate_non_negative`             |
| `identity_witness_holds`            | universal anchor (phi^2 + 1/phi^2 == 3 to 1e-15)   |
| `sim_state_tag_roundtrip`           | `SimState::from_tag . tag = id` and unknown -> None |

13 mirrored from spec `test` blocks + 4 from spec `invariant` blocks +
1 universal anchor + 1 bonus type-safety round-trip = 19 total.

## Build

```bash
cd rings/ring-090-rust
cargo check --all-targets   # green on Rust 1.83.0
cargo test --lib            # 19 passed, 0 failed
```

Local verification on Rust 1.83.0 (matching
[`Dockerfile.rust`](../../Dockerfile.rust)) -- the Wave-13 `rings-rust`
matrix will re-confirm on every PR that touches this crate.
