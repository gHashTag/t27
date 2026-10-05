# NOW -- t27b: an undefined; body stub the reference never analyzes (2026-10-05)

## undefined; as a statement (Closes #6308)

- `t27b corpus specs --blockers` at master 0851055e3 ranked `ExprIdentifier statement` first (+19 files). Its dominant shape is `undefined;`, the body stub a port leaves where plumbing was; the rest are Rust-style tail expressions (`fn f(v: u8) -> u32 { v }`) and `given/when` prose, left for later under the same name.
- t27c's Zig backend emits `undefined;` as is. Zig rejects it only in a fn its lazy analysis reaches; under `zig test` neither `pub` nor `main` is a root, so a stub nothing analyzed names never stops the file.
- t27b now computes the fns a test, invariant, bench or module-level declaration can reach by name (an over-approximation) and lowers `undefined;` elsewhere to a trap (`TrapKind::Stub`) no test can hit. Where something analyzed can reach it, t27b refuses it as `ExprIdentifier(undefined) statement`; the reference reports BLOCKED (does not compile) there too.
- `t27b corpus specs` at master 0851055e3: pass 277 -> 296, pass_vacuous 279 -> 279, fail 6 -> 6, rejected 652 -> 633, front-end error 40 -> 40, JIT/interpreter mismatch 0 -> 0, timeout 3 -> 3. Each of the 19 newly passing files agrees test for test with `t27c test-report` built from the same master.
- New test `undefined_stub_only_where_zig_never_looks` in `cli/t27b/tests/source.rs`: stubs reached only from `main` run both tests, and three reached shapes (through another fn on an untaken branch, from an invariant, directly in a test) are refused; each source was checked against `t27c test-report`.
