# ring-097-chain-of-thought

Bounded Chain-of-Thought reasoning primitives — proof trace with K3 ternary logic.

Mirrors `specs/ar/proof_trace.t27` byte-for-byte.

## Primitives

- **`MAX_STEPS = 10`** — DARPA CLARA bound on reasoning chain length
- **`Trit`** — K3 ternary logic: `True`, `Unknown`, `False`, plus `Null` sentinel
- **K3 connectives**: `k3_and` (min lattice), `k3_or` (max lattice), `k3_not`
- **`ProofStep`** — `step_id`, `operation` (ASCII, up to 24 chars), `inputs` (up to 3 trits), `output: Trit`, `timestamp_us`
- **`ProofTrace`** — fixed-capacity 10-step buffer + `start_timestamp_us` + `end_timestamp_us` + `verified` flag
- **Operations**: `new_proof_trace`, `add_step`, `verify_trace`, `trace_length`, `is_at_capacity`, `finalize_trace`, `step_at`, `format_trace`, `trit_to_string`

## Verification semantics

`verify_trace` returns `VerifyStatus::Valid` iff:
- the trace is non-empty,
- the step count is ≤ `MAX_STEPS`,
- every step has a non-`Null` output (per spec invariant).

Otherwise: `Empty`, `TooManySteps`, or `NullOutput(index)`.

## no_std + no heap

The crate is `#![no_std]` and `#![deny(warnings)]`. No allocations: traces hold a `[ProofStep; MAX_STEPS]` array, operation names are interned as `[u8; MAX_OP_NAME]` + length, inputs are `[Trit; MAX_INPUTS_PER_STEP]` + count. The `format_trace` rendering writes into a caller-supplied buffer (`FORMAT_TRACE_BUFFER` bytes worst-case).

A private `pow_u64` (fast exponentiation by squaring) replaces libm for the anchor identity.

## Anchor #9

`cot_phi_identity` routes the Trinity identity `phi^2 + 1/phi^2 = 3` through a 6-step bounded reasoning chain:
1. Symbolic premise: `phi > 1`
2. Symbolic premise: `1/phi < 1`
3. `k3_and` over both premises
4. **Numeric witness** via `pow_u64(phi, 2) + pow_u64(phi, -2)`, asserting < 1e-9 from 3.0
5. `k3_or` admitting alternative paths
6. Conclusion

The trace is then verified and finalised. A separate mass-conservation hook verifies that φ²-weighted Pos and φ⁻²-weighted Neg priorities sum to exactly 3.0.

## Build & test

```
cargo check
cargo test --lib
```

## Constitutional compliance

- L1 TRACEABILITY — `Closes #735`
- L2 GENERATION — zero edits under `specs/`, `gen/`, `coq/`
- L3 PURITY — ASCII source, English doc-comments
- L4 TESTABILITY — `#[test]` blocks
- L5 IDENTITY — `phi^2 + 1/phi^2 = 3` exercised
- L6 CEILING — spec constants byte-for-byte
- L7 UNITY — no `*.sh` files

---

`phi^2 + 1/phi^2 = 3`
