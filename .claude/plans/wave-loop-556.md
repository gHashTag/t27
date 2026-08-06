# Wave Loop 556 Plan — Multi-site call-return array deduplication

Issue #1527 | branch `wave-loop-556` | next branch `wave-loop-557`

---

## Charter

Implement **Variant A**: when the same `f()` packed-array expression appears at
multiple sites in one deterministic `bench` block, reuse a single packed
temporary and emit only one assignment. The witness asserts both an element
(`mat()[i][j]`) and the whole array (`assert_eq(mat(), expected)`) in the same
bench without duplicating the call or its temporary.

---

## Weak points discovered

1. **Temporaries are only created for indexed calls.** `predeclare_call_array_tmps`
   and `materialize_call_array_tmps_in_expr` walk `ExprIndex` chains and register
   temps only when the root is `ExprCall`. A bare `ExprCall` used as the actual
   expression of `assert_eq(mat(), expected)` is never registered.
2. **Whole-array asserts evaluate the call multiple times.** Even within a single
   `assert_eq`, the call text is rendered for the probe assignment and again for
   the comparison expression. Without a shared temporary, the simulator evaluates
   the function twice.
3. **No visibility into temporary reuse.** There is no witness that exercises both
   `mat()[i][j]` and `assert_eq(mat(), ...)` in the same bench, so the
   deduplication path is only tested implicitly.
4. **Scope of the call-temp map is per block but the substitution logic is
   incomplete.** The map `call_array_tmp_names` is keyed by full call expression
   text, so it is already capable of sharing; the missing piece is populating and
   consulting it for bare `ExprCall` sites.
5. **The generated Verilog currently relies on pure calls, but this is not
   documented.** The Icarus-lowerable subset is intended to be side-effect-free,
   yet deduplicating calls changes evaluation count. The boundary should be
   documented before W556 closes.

---

## Engineering / scientific background

- **Common subexpression elimination in HDL compilers.** Verilator implements this
  via a data-flow graph CSE pass (`V3DfgCse.cpp`) that hashes vertices by type,
  data type, size, and source hashes, then replaces equivalent vertices. A
  simpler textual-key approach is sufficient for t27 because the Icarus-lowerable
  bench/test block is a small, pure, single-block scope.
- **Scoreboard / reference-model patterns in FPGA verification.** Cocotb
  recommends a pure-Python reference model feeding a scoreboard; t27's
  deterministic bench cross-check is a lightweight instance where the AST-derived
  expected value is compared against VCD probes.
- **Packed-array temporaries in SystemVerilog.** Packed arrays are contiguous bit
  vectors, so a single `reg [W-1:0]` can hold a complete multi-dimensional
  primitive scalar array and be referenced by element slices or as a whole.

Sources:
- [Verilator V3DfgCse.cpp](https://github.com/verilator/verilator/blob/dbd48233/src/V3DfgCse.cpp)
- [Verilator internals overview](https://github.com/verilator/verilator/blob/dbd48233/docs/internals.rst)
- [Cocotb scoreboard docs](https://docs.cocotb.org/en/v1.2.0/_modules/cocotb/scoreboard.html)
- [NDK-FPGA cocotb guide](https://cesnet.github.io/ndk-fpga/devel/basic_cocotb_test.html)
- [SystemVerilog packed arrays](https://github.com/mbits-mirafra/SystemVerilogCourse/wiki/02.Array)

---

## Implementation tasks

### A. Create W556 scratch witness

`specs/scratch/w556_bench_multi_site_array_dedup.t27`:

- Module `mat() -> [2][3]u8` returns a known 2-D array.
- `bench` block contains:
  - `assert_eq(mat()[1][2], 0xAB);`
  - `assert_eq(mat(), [2][3]u8{...});`
- Equivalent `test` block for static validation.
- Optional: also exercise a signed variant or a second unsigned witness that uses
  the temporary at more than two sites.

### B. Extend compiler to share packed-array call temporaries

In `bootstrap/src/compiler.rs`:

1. **Extend `predeclare_call_array_tmps`** to also handle `NodeKind::ExprCall`
   whose return type is a primitive scalar array. Register a temp with the same
   key as indexed uses so the same `mat()` call gets one temp.
2. **Extend `materialize_call_array_tmps_in_expr`** to materialize bare
   `ExprCall` nodes that have a temp entry before the statement that uses them.
3. **In `gen_verilog_test_stmt`**, when emitting the actual expression of an
   `assert_eq` (both for probe assignment and comparison), substitute the
   predeclared temp name for bare `ExprCall` that returns a primitive scalar array
   and has an entry in `call_array_tmp_names`.
4. Keep `gen_verilog_expr` itself unchanged so `collect_expr_text` continues to
   render original call text for key generation.
5. Ensure the temporary is declared with the full packed width/signedness.

### C. Validate temporary reuse

Inspect generated Verilog for the witness:
- Only one `call_array_tmp_*` temporary declared.
- Only one assignment `<tmp> = mat();` emitted.
- Both `mat()[1][2]` and `assert_eq(mat(), ...)` reference the same temp.

### D. Baselines, seals, integration test

- Run direct `t27c icarus-simulate` / `t27c icarus-cocotb` on the witness.
- Save t27 seal.
- Record Icarus baseline if the witness passes the `gen-verilog` pre-flight
  (likely yes if it uses no named test/bench locals).
- Add `accepts_w556_bench_multi_site_array_dedup` integration test in
  `bootstrap/tests/icarus_lowerable.rs`.

### E. Validation matrix

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
- Direct `./target/release/t27c icarus-simulate specs/scratch/w556_*.t27`
- Direct `./target/release/t27c icarus-cocotb specs/scratch/w556_*.t27`
- `lake build Trinity.IcarusLowerable.Soundness`

---

## Three cooperation variants for Wave Loop 557

### Variant A — Recommended: general bench CSE for scalar calls
Extend the same temporary-deduplication machinery to scalar-return function
calls inside bench blocks, not only packed arrays. Witness: multiple
`assert_eq(f(), expected)` and `assert_eq(f() + g(), ...)` in one bench share
a single `call_tmp_*` per pure call.

### Variant B: whole-array comparison for 3-D / 4-D signed arrays
Extend W555 whole-array probes to 3-D and 4-D signed primitive scalar arrays,
verifying row-major slice reconstruction in the Python model for ranks 3 and 4.

### Variant C: deterministic-bench side-effect boundary
Add an AST classifier that rejects (or documents) `bench` blocks containing function
calls with side effects, `#` delays, or unbounded loops, and update
`docs/ICARUS_LOWERABLE_BOUNDARY.md` to state that the call-deduplication
optimization is only valid for pure calls.

---

## Skills to save at closeout

Pattern: *"The same function-call expression returning a packed primitive
scalar array can be used for both element indexing and whole-array comparison in
one bench block; register the temporary by full call expression text for bare
ExprCall sites as well as ExprIndex chains, then substitute the temp name during
assert_eq emission."*
