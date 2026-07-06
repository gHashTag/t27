# FPGA Loop Cooperation Plan — Wave Loop 464 (2026-07-07)

**Issue:** #1441 (to create from W463 land commit)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context at the end of W463

Wave Loop 463 closed #1439 by selecting Variant B: compiler-backend hardening.
The wave extended the W461/W462 array-parameter clone machinery so that binding
signatures are propagated through nested same-array calls. A function `f(data)`
that internally calls another array-parameter function `lookup(data)` now has
its resolved signatures propagated to `lookup`, and the inner call is redirected
to the matching propagated clone. The `--fast` suite path is green: 591/591
non-smoke PASS, 71/71 yosys smoke PASS, FPGA smoke gate OK, 0 baseline failures,
0 seal mismatches.

The physical bench remains blocked: `dlc10 idcode` reports "DLC10 cable not
found (VID=0x03FD)", P12 is unwired, and no automated cold-POR relay gate exists.
The default `./scripts/tri test` still cannot complete in this environment
because Phase 3c-standalone stalls while `lake` downloads the `batteries`
dependency from `reservoir.lean-lang.org`; the smoke-gate report itself passes.

This cooperation plan proposes three mutually-exclusive W464 execution
strategies. Select **one** at the start of the wave.

---

## Variant A — Live CCLK capture if the bench unblocks

Execute only if the DLC10 cable is found **and** P12 + relay are wired.

### Goal
Run the first live cold-POR CCLK sweep since W434 and mint a hardware-backed
theorem fixture set under the post-W463 bitstream.

### Scope
1. Run `tri fpga cclk-sweep --json` across OSCFSEL 0–7 with the current
   post-W463 bitstream.
2. Persist fixtures under `tests/fixtures/fpga/theorem-matrix/live-w464/` with
   PVT context and raw-ns measurements.
3. Mint `XADC_LIVE_W464_OPERATING_POINT` theorem in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` plus matching Rust unit tests.
4. Add a regression gate that rejects fixture drift beyond the documented PVT
   envelope.

### Acceptance
- At least one successful live sweep completes and its fixture is committed.
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test --fast` remains 591/591 non-smoke PASS with yosys smoke OK.

---

## Variant B — Compiler backend hardening: mixed call sites + struct-literal array arguments + clone-name collision guard (default)

Execute when the bench is still blocked. This is the most likely W464 path.

### Goal
Extend the W463 propagation so functions with both direct and indirect
array-parameter call sites are merged correctly, allow struct literals to be
passed to struct-element array parameters, and harden clone naming against
long-signature collisions.

### Scope
1. **Mixed direct/indirect array-parameter call sites.** Extend the W463 merge
   logic so that if a function `g(data)` is called both directly from a
   module-level/test/bench site and indirectly through `f(data)`, the resulting
   signature set is the union of direct and propagated signatures. Add a
   dedicated regression spec covering this case.
2. **Struct-literal array arguments.** Allow array parameters whose element type
   is a struct to be passed with a literal array of struct literals, lowering to
   a field-interleaved anonymous packed ROM or memory initialized field-by-field.
3. **Clone-name collision guard.** Add a deterministic disambiguation suffix
   when the sanitized clone name derived from array-parameter signatures collides
   with an existing clone or module-level identifier. This protects against
   pathological array names containing the same underscore-delimited components.
4. Reseal all affected specs and keep `YOSYS_ALLOWED_WARNINGS` aligned with the
   cleaner output.

### Acceptance
- `./scripts/tri test --fast` reports 0 failures and `ACCEPTABLE: yes`.
- New or updated scratch specs pass `t27c gen-verilog` + yosys
  `read_verilog -sv -DSIMULATION` and are exercised by at least one `assert_eq`.
- `cargo test -p t27c --bin t27c` passes with 0 failures.

---

## Variant C — Formal boot-evidence fallback

Execute if Variant B is blocked by an AST/scope refactor that cannot be
completed safely in one wave.

### Goal
Extend the board-less Lean 4 boot-evidence lattice with synthesizability and
compiler-correctness bridge statements that cover the new W463 nested
array-parameter path.

### Scope
1. **Synthesizability theorem block.** Add propositions in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` stating that the W463 regression
   spec (`w463_nested_array_param_call.t27`) produces yosys-clean Verilog, and
   that the emitted clone structure matches the intended propagation graph.
2. **Nested-call correctness lemma.** Relate the W463 propagation rule to the
   abstract ternary MAC semantics in `TernaryInference.lean`: if `f(arr)` and
   `g(arr)` share the same array parameter, then calling `g` from `f` preserves
   the same array-parameter binding semantics as a direct call to `g(arr)`.
3. **Adversarial clone-collision witness.** Add a Lean theorem or Rust unit test
   showing that two different array-parameter signature keys cannot produce the
   same resolved clone mapping under the deterministic naming scheme.
4. Add matching Rust unit tests in `cli/tri/src/fpga.rs`.

### Acceptance
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test --fast` remains 591/591 non-smoke PASS with yosys smoke OK.
- At least 3 new Lean theorems and 3 new Rust unit tests land.

---

## Recommended selection order

1. **Variant A** if hardware becomes available during the W464 start-of-wave probe.
2. **Variant B** otherwise — it is the natural continuation of the compiler
   hardening line after W455–W463.
3. **Variant C** only if Variant B hits an unresolvable parser/AST scope blocker.

---

*φ² + φ⁻² = 3 | TRINITY*
