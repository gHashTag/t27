# FPGA Loop Closeout — Wave Loop 581 (15-D array-of-struct return call deduplication)

**Issue:** #1552  
**Branch:** `wave-loop-581`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## What was delivered

Wave Loop 581 closes a 15-D `[2]^15 Pt` array-of-struct return call deduplication
witness. The packed vector is **1,048,576 bits** wide (1 MiBit), **32,768**
scalar-struct elements, and **sixteen times** the IEEE 1800-2017 minimum
packed-vector width.

- `specs/scratch/w581_bench_15d_aos_call_dedup.t27` — deterministic bench/test
  witness (~5.7 MB / ~295k lines).
- `.trinity/seals/scratch_w581_bench_15d_aos_call_dedup.json` — seal ceremony.
- `.trinity/icarus-baselines/specs/scratch/w581_bench_15d_aos_call_dedup.json` —
  Icarus simulation baseline.
- `bootstrap/tests/icarus_lowerable.rs` — integration test
  `accepts_w581_bench_15d_aos_call_dedup`.

No compiler or reference-model changes were required.

---

## Chosen variant

**Variant A — 15-D array-of-struct return call deduplication.**

The witness exercises the same rank-agnostic paths as W566–W580, now at rank
15:

- `emit_local` multi-D AoS wholesale assignment,
- `call_returning_cse_value_info` `[N1]...[Nk]Pt` descriptor,
- `try_emit_struct_array_access` recursive slice indexing,
- `gen_verilog_expr` `ExprArrayLiteral` nested concatenation,
- `scripts/cocotb_ref_model.py` row-major arbitrary-precision evaluator.

The W573–W580 local-`expected` workaround is reused: the 15-D array literal is
bound to a local variable before `assert_eq`, so Icarus 12.0's `$display` VPI
path never receives a raw 1-MiBit nested concatenation.

---

## Implementation notes

### Expected-value arithmetic

For `Pt { x: i16, y: i16 }` and element index `e`, `x = 2*e`, `y = 2*e+1`.
Because fields are signed 16-bit, element indices must keep `y ≤ 32767`, i.e.
`e ≤ 16383`.

- `pentadeca[0][1][0][1][1][1][1][1][1][1][1][1][1][1][1]`: flat index `12287`,
  `x = 24574`.
- `pentadeca[0][1][0][1][0][1][0][1][0][1][0][1][0][1][0]`: flat index `10922`,
  `y = 21845`.

A Python row-major script generates the literal and verifies the probes.

### Witness structure

```t27
pub fn make_pentadeca() -> [2][2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt {
    return [2][2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt{ ... };
}

test pentadeca_test {
    let pentadeca : [2]^15 Pt = make_pentadeca();
    assert_eq(pentadeca[...].x, 24574);
    assert_eq(pentadeca[...].y, 21845);
    assert_eq(pentadeca, make_pentadeca());
    let expected : [2]^15 Pt = [2]^15 Pt{ ... };
    assert_eq(make_pentadeca(), expected);
}

bench "pentadeca_bench" { ... }
```

The literal is emitted twice (test `expected` and bench `expected`) plus the
function body, which accounts for the ~5.7 MB file size.

---

## Verification results

| Gate | Result |
|---|---|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 41 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke failures unchanged |
| Direct `t27c icarus-simulate` on W581 witness | PASS |
| Direct `t27c icarus-cocotb` on W581 witness | PASS |
| `lake build Trinity.IcarusLowerable.Soundness` | 8572 jobs, 0 `sorry` |

---

## What changed

- `bootstrap/src/compiler.rs`: **no change**.
- `scripts/cocotb_ref_model.py`: **no change**.
- `bootstrap/stage0/FROZEN_HASH`: **unchanged**.
- Added `specs/scratch/w581_bench_15d_aos_call_dedup.t27` with seal and Icarus
  baseline.
- Added `accepts_w581_bench_15d_aos_call_dedup` to
  `bootstrap/tests/icarus_lowerable.rs`.

---

## Scientific / engineering background

- **IEEE Std 1800-2017 §7.4.1** requires compliant tools to support packed arrays
  of at least 65,536 bits; 1,048,576 bits is sixteen times that minimum and
  tests the implementation ceiling rather than the language floor.
  ([StackExchange discussion](https://electronics.stackexchange.com/questions/705776/is-there-any-restriction-on-the-maximum-size-of-a-systemverilog-packed-array))
- **Icarus Verilog** v12.0 accepts the 1-MiBit packed vector once the wide
  literal is bound to a named local before any VPI formatting task. The raw
  nested concatenation inside `$display` still triggers the VPI buffer overflow
  observed in W573–W580.
- **CIRCT `HWLegalizeModules`** recursively legalizes multi-dimensional packed
  arrays with no explicit depth cap, matching t27's recursive literal emission.
  ([source](https://circt.llvm.org/doxygen/HWLegalizeModules_8cpp_source.html))
- **C++23 `std::mdspan` `layout_right`** generalizes the same row-major product
  used by t27's linear-index formula to rank 15.
  ([cppreference](https://en.cppreference.com/cpp/container/mdspan/layout_right))

---

## Risks encountered

| Risk | Outcome |
|---|---|
| Icarus rejects 1-MiBit packed vector | Resolved: the local-`expected` workaround continues to work at 1 MiBit. |
| Signed 16-bit overflow in indexed probes | Caught by cocotb reference model; fixed by selecting element indices with `e ≤ 16383`. |
| Spec generation nesting error | Initial script double-nested the outer literal, producing an extra 32-bit zero padding in generated Verilog. Fixed by emitting the root literal separately and calling the recursive generator for the two rank-14 children. |
| Gate runtime / memory | Direct simulation and cocotb completed within the standard timeout; file parsing is slower but acceptable. |

---

## Three cooperation variants for Wave Loop 582

1. **Variant A — Recommended: 16-D array-of-struct return call deduplication.**  
   Extend the rank-agnostic verification one dimension higher:
   `[2][2][2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt` (2,097,152 bits, 65,536
   elements). This is the next natural zero-change rank stress test if 15-D
   passes cleanly.

2. **Variant B: 15-D array-of-struct return with a non-power-of-two outer
   dimension.**  
   Add `[3][2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt` (1,572,864 bits, 49,152
   elements). The non-p2 outer extent is the strongest stress test for
   product-based width/index arithmetic at rank 15, following the W569/W571
   pattern.

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

Create branch `wave-loop-582` from `wave-loop-581` and implement whichever W582
variant is selected in `.trinity/current-issue.md`.
