# Wave Loop 554 Plan — Bench-local primitive scalar arrays

Issue #1525 | branch `wave-loop-554` | next branch `wave-loop-555`

---

## Charter

Extend the deterministic `bench` cross-check to support a named local binding
that receives a packed primitive scalar array returned by a function call,
e.g. `let tmp : [N]T = f();`. Verify that subsequent element reads inside the
same `bench` block are emitted correctly in Verilog and reconstructed correctly
by the Python reference model.

---

## Weak points discovered

1. **Bench locals are only partially exercised for scalar structs.** W533 added
   `let tmp : Pt = make(...);` for packed scalar structs, but no previous loop
   exercises a `bench` local whose type is a primitive scalar array.
2. **Packed primitive array locals share the same lowering decision as
   function-local arrays (W546) but are not applied to bench blocks.** The
   `emit_local` path already emits packed-vector `reg` + whole-vector assignment
   when the initializer is not an array literal, but it needs to be stress-
   tested inside an `initial` block used for simulation.
3. **The Python reference model already binds test/bench locals, but element
   reads from a local packed array inside a bench are not exercised end-to-end.**
   `_eval_index_bv` resolves the local type via `test_local_types`, so the
   evaluator should work, but no witness locks the behavior.
4. **Mixed signed/unsigned bench-local array reads are untested.** The W553
   signed fixes apply to direct `f()[i]`; a named local adds another indirection
   that could change signedness inference in both compiler and reference model.
5. **Multi-site reads from a bench local are untested.** Reading `tmp[0]`,
   `tmp[1]`, and `tmp[2]` from the same local should reuse the packed vector
   without re-emitting the function call.

---

## Engineering / scientific background

* **Packed-vector array temporaries in Verilog.** A function returning a
  primitive scalar array lowers to a packed vector (W545). When that vector is
  bound to a local name inside a procedural block, it must be stored in a packed
  `reg` so that subsequent element reads use bit-slice indexing. This is the
  same technique used for function-local packed arrays (W546) and for the W553
  call-return expression temporaries.
* **Row-major flat indexing.** Element `[i]` of `[N]i8` is at bit offset
  `i * 8`, matching the packed layout where element 0 is the LSB. The compiler's
  `try_emit_primitive_array_access` already implements this for packed vectors;
  we only need the bench local to be recognized as a packed-vector base.
* **Reference-model local binding.** The Python evaluator stores the whole packed
  array as a `Bv` and extracts element slices at the declared element width and
  signedness. `_eval_index_bv` already handles this via `_resolve_full_type`,
  which looks up `test_local_types` for the current block.
* **Deterministic bench semantics.** T27 `bench` blocks are pure combinational
  initial blocks (no `#` delays). This aligns with cocotb/reference-model
  cross-checks that run a single time-step simulation.

---

## Implementation tasks

### A. Create W554 scratch witnesses
Three specs under `specs/scratch/`:
* `w554_bench_local_array_unsigned.t27` — `let tmp : [3]u8 = seq_u8();` inside a
  `bench`, assert `tmp[0] == 1`, `tmp[1] == 2`, `tmp[2] == 3`.
* `w554_bench_local_array_signed.t27` — `let tmp : [3]i8 = seq_i8();` inside a
  `bench`, assert `tmp[0] == -1`, `tmp[1] == -2`, `tmp[2] == -3`.
* `w554_bench_local_array_2d.t27` — `let tmp : [2][3]u8 = mat();` inside a
  `bench`, assert `tmp[0][0] == 1` and `tmp[1][2] == 6`.

Each witness includes both a `test` and a `bench` block with equivalent asserts.

### B. Compiler support verification
* Confirm `emit_local` in `bootstrap/src/compiler.rs` declares a packed `reg`
  for `let tmp : [N]T = f();` inside a test/bench block and emits `tmp = f();`.
* Confirm `try_emit_primitive_array_access` reads from the local packed reg via
  bit-slices and wraps signed element slices with `$signed(...)`.
* The W553 `call_array_tmp_*` mechanism must not fire for named locals; the
  local itself serves as the packed temporary.

### C. Reference model verification
* Confirm `_collect_assertions` binds the local packed-array value for both
  `TestBlock` and `BenchBlock`.
* Confirm `_resolve_full_type` finds the local type from `test_local_types` for
  bench blocks.
* Confirm `_eval_index_bv` extracts the correct element value and signedness.

### D. Baselines, seals, and integration tests
* Generate Icarus baselines for the three witnesses via the suite gate.
* Save t27 seals.
* Add `accepts_w554_bench_local_array_cross_check` in
  `bootstrap/tests/icarus_lowerable.rs`.

### E. Validation matrix
* `cargo build --release -p t27c`
* `cargo test -p t27c --bin t27c`
* `cargo test -p tri`
* `cargo test -p t27c --test icarus_lowerable`
* `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
* `lake build Trinity.IcarusLowerable.Soundness`

---

## Three cooperation variants for Wave Loop 555

### Variant A — Recommended: whole-array bench assignments
Support `assert_eq` on a complete 2-D primitive scalar array value inside a
`bench`, using the W540 multi-slice probe path for wide signed packed arrays.

### Variant B: multi-site call-return array deduplication
When the same `f()` packed-array expression is indexed at multiple sites in
one bench, reuse a single packed temporary and emit only one assignment. The
W553 temporary map already deduplicates by call expression text; extend coverage
and add a dedicated witness with many reads.

### Variant C: timed/non-deterministic bench classifier
Introduce an AST classifier that rejects (or skips) `bench` blocks containing
`#` delays or unbounded loops from the deterministic cocotb gate, and document
the boundary.

---

## Skills to save at closeout

Pattern: *"A `bench`-local primitive scalar array initialized from a function call
is just a packed-vector `reg` in Verilog; the compiler's existing packed-array
local lowering and the Python evaluator's `_resolve_full_type` handle it once the
witness exists."*
