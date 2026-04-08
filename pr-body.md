## Sprint 8 Complete — 8/8 Acceptance Criteria PASS

### What changed
- `ffi/src/lib.rs` — integer-only core (22 extern "C" functions, L8 compliant)
- `bindings/python/src/lib.rs` — consolidated to FFI calls (no direct encode/decode)
- `bindings/python/golden_float/numpy_dtype.py` — gf_array() + to_float32() helpers
- `bindings/python/tests/test_numpy.py` — E2E NumPy tests
- `specs/interop/gf_cross_language.t27` — 6 conformance tests
- `docs/MIGRATION.md` — migration guide
- `docs/WHITEPAPER/gf_paper_v3_imrad_draft.md` — section 7 added (Cross-Language + FPGA Safety)
- `.github/workflows/phi-loop-ci.yml` — FPGA-Safety lint step
- `AGENTS.md` — L8: INTEGER-ONLY-CORE law

### Verification
GF32(φ) = 0x3FCF1BBD — bit-identical across Python, JS, Rust, C.

### Rules compliance
- L1: Closes #361
- L4: TDD — all specs have test blocks
- L8: FPGA-Safety — no f32/f64 arithmetic in core

### CI
- phi-loop-ci.yml now includes FPGA-Safety lint
- Python bindings require PYO3_ABI3 feature
