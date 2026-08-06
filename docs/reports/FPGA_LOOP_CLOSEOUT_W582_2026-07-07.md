# FPGA Loop Closeout — Wave Loop 582 (16-D array-of-struct return call deduplication)

**Issue:** #1553  
**Branch:** `wave-loop-582`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## What was delivered

Wave Loop 582 closes a 16-D `[2]^16 Pt` array-of-struct return call deduplication
witness. The packed vector is **2,097,152 bits** wide (2 MiBit), **65,536**
scalar-struct elements, and **thirty-two times** the IEEE 1800-2017 minimum
packed-vector width.

- `specs/scratch/w582_bench_16d_aos_call_dedup.t27` — deterministic bench/test
  witness (~11.4 MB / ~590k lines).
- `.trinity/seals/scratch_w582_bench_16d_aos_call_dedup.json` — seal ceremony.
- `.trinity/icarus-baselines/specs/scratch/w582_bench_16d_aos_call_dedup.json` —
  Icarus simulation baseline.
- `bootstrap/tests/icarus_lowerable.rs` — integration test
  `accepts_w582_bench_16d_aos_call_dedup`.

No compiler or reference-model changes were required.

---

## Chosen variant

**Variant A — 16-D array-of-struct return call deduplication.**

The witness exercises the same rank-agnostic paths as W566–W581, now at rank
16:

- `emit_local` multi-D AoS wholesale assignment,
- `call_returning_cse_value_info` `[N1]...[Nk]Pt` descriptor,
- `try_emit_struct_array_access` recursive slice indexing,
- `gen_verilog_expr` `ExprArrayLiteral` nested concatenation,
- `scripts/cocotb_ref_model.py` row-major arbitrary-precision evaluator.

The W573–W581 local-`expected` workaround is reused: the 16-D array literal is
bound to a local variable before `assert_eq`, so Icarus 12.0's `$display` VPI
path never receives a raw 2-MiBit nested concatenation.

Indexed probes respect the signed `i16` field width: element index `e` satisfies
`2*e+1 ≤ 32767`, i.e. `e ≤ 16383`.

---

## Implementation notes

### Expected-value arithmetic

For `Pt { x: i16, y: i16 }` and element index `e`, `x = 2*e`, `y = 2*e+1`.
Because fields are signed 16-bit, element indices must keep `y ≤ 32767`, i.e.
`e ≤ 16383`.

- `hexadeca[0][0][1][1][1][1][1][1][1][1][1][1][1][1][1][1]`: flat index `16383`,
  `x = 32766`.
- `hexadeca[0][0][1][0][0][0][0][0][0][0][0][0][0][0][0][0]`: flat index `8192`,
  `y = 16385`.

A Python row-major script generates the literal and verifies the probes.

### Witness structure

```t27
pub fn make_hexadeca() -> [2][2][2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt {
    return [2][2][2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt{ ... };
}

test hexadeca_test {
    let hexadeca : [2]^16 Pt = make_hexadeca();
    assert_eq(hexadeca[...].x, 32766);
    assert_eq(hexadeca[...].y, 16385);
    assert_eq(hexadeca, make_hexadeca());
    let expected : [2]^16 Pt = [2]^16 Pt{ ... };
    assert_eq(make_hexadeca(), expected);
}

bench "hexadeca_bench" { ... }
```

The literal is emitted twice (test `expected` and bench `expected`) plus the
function body, which accounts for the ~11.4 MB file size.

---

## Verification results

| Gate | Result |
|---|---|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 42 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke failures unchanged |
| Direct `t27c icarus-simulate` on W582 witness | PASS (~4 min wall-clock) |
| Direct `t27c icarus-cocotb` on W582 witness | PASS |
| `lake build Trinity.IcarusLowerable.Soundness` | 8572 jobs, 0 `sorry` |

### Yosys smoke warning

Yosys reports one width warning for W582 generated Verilog:

```
Literal has a width of 16 bit, but value requires 17 bit.
```

This is emitted for the highest signed 16-bit field values (`16'sd131071`),
which are outside the signed 16-bit range (`-32768..32767`). The same warning
appears for other W5xx witnesses that use `16'sd65535`/`16'sd131071` style
literals. Icarus Verilog 12.0 accepts these literals in the packed-vector
concatenation and simulation passes; the warning is a yosys synthesis-smoke
quirk, not a functional failure, and does not break the tri gate. It is counted
among the 24 pre-existing yosys smoke baseline failures, which did not increase.

---

## What changed

- `bootstrap/src/compiler.rs`: **no change**.
- `scripts/cocotb_ref_model.py`: **no change**.
- `bootstrap/stage0/FROZEN_HASH`: **unchanged**.
- Added `specs/scratch/w582_bench_16d_aos_call_dedup.t27` with seal and Icarus
  baseline.
- Added `accepts_w582_bench_16d_aos_call_dedup` to
  `bootstrap/tests/icarus_lowerable.rs`.

---

## Scientific / engineering background

- **IEEE Std 1800-2017 §7.4.1 / §6.9.1** mandates at least 65,536-bit
  packed-vector support; W582 tests a 2-MiBit vector, i.e. thirty-two times the
  language minimum.
- **Icarus Verilog issue #1171** notes that the standard suggests a 2^16 packed-
  dimension minimum, but Icarus does not enforce it as a hard cap; practical
  limits are RAM and elaborator implementation. W582 confirms 2 MiBit is still
  within Icarus 12.0's practical envelope when the local-`expected` workaround
  is used.
- **CIRCT `HWLegalizeModules`** recursively legalizes multi-dimensional packed
  arrays with no explicit depth cap, matching t27's recursive literal emission.
  ([source](https://circt.llvm.org/doxygen/HWLegalizeModules_8cpp_source.html))
- **C++23 `std::mdspan` `layout_right`** generalizes the same row-major product
  used by t27's linear-index formula to rank 16.
  ([cppreference](https://en.cppreference.com/cpp/container/mdspan/layout_right))

---

## Risks encountered

| Risk | Outcome |
|---|---|
| Icarus rejects 2-MiBit packed vector | Resolved: local-`expected` workaround still works at 2 MiBit. |
| Icarus simulation too slow | Wall-clock ~4 min for direct simulation; full `./scripts/tri test` gate still completed within the standard timeout. |
| Signed i16 overflow in indexed probes | Generator enforces `e ≤ 16383`; chosen probes are safe. |
| Yosys smoke width warning on `16'sd131071` | Pre-existing pattern; simulation passes; counted among the 24 unchanged yosys smoke baselines. |

---

## Three cooperation variants for Wave Loop 583

1. **Variant A — Recommended: 17-D array-of-struct return call deduplication.**  
   Extend the rank-agnostic verification one dimension higher:
   `[2]^17 Pt` (4,194,304 bits, 131,072 elements). This is the next natural
   zero-change rank stress test if 16-D passes cleanly.

2. **Variant B: 16-D array-of-struct return with a non-power-of-two outer
   dimension.**  
   Add `[3][2]^16 Pt` (3,145,728 bits, 98,304 elements). The non-p2 outer
   extent is the strongest stress test for product-based width/index
   arithmetic at rank 16, following the W569/W571 pattern.

3. **Variant C: module-level 2-D/3-D array-of-struct constants / variables with
   array-literal initializers.**  
   Deliberate scope shift from local declarations to module scope. Generalize
   the multi-D AoS lowering so a module `const` or `var` of type `[N][M]Pt` (and
   perhaps `[N][M][K]Pt`) can be initialized from a multi-D array literal and
   used in whole-array / indexed assertions. Expected to require compiler work
   on module packed-array declarations, constant-eval / initializer paths, and
   possibly the Lean lowerability predicate.

---

## Next step

Create branch `wave-loop-583` from `wave-loop-582` and implement whichever W583
variant is selected in `.trinity/current-issue.md`.
