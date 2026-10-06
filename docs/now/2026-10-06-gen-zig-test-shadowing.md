# NOW -- gen-zig test-block shadowing, landed as an owner exception (2026-10-06)

## bootstrap/src/compiler.rs gen-zig fix (Closes #6295, debt #5980)

- Zig refused a test or bench block whose local shadowed a module declaration ("local constant shadows declaration of 'HITS'"). Locals that shadow a module name are now renamed (`_lv`), a write to a module `var` stays a write, and a bench block picks `var` or `const` from its assignment count.
- `specs/compiler/zig_test_shadowing.t27` is the regression spec; module vars HITS/READY, fns twice/hit, three tests and a bench.
- `bootstrap/src/compiler.rs` is hand-written Rust, landed under the owner exception from #6579 (label `owner-approved-foreign`); the debt is #5980.
- Reference pass count on the t27c lab, same base: 770 -> 776 (formal_tb, vcd_trace_tb, ternary_mac_synth and the new spec go to pass; two specs from master are counted too), 0 regressions. Nine fpga/port specs now compile and fail or hang their own tests (#6560); their seals are forced and ledgered as tests-fail.
