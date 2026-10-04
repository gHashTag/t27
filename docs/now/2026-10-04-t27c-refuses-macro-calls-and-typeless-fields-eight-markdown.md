# NOW -- t27c refuses macro calls and typeless fields; eight Markdown specs become modules (2026-10-04)

## t27c refuses macro calls and typeless fields; eight Markdown specs become modules (Closes #5923)

- Parse refuses a Rust macro call (`format!(..)`, `assert!(..)`, `panic!(..)`), which it used to read as a name followed by `!`, naming the macro. Typecheck refuses a struct field with no type or an integer-literal type, which gen-rust wrote as `pub f: ,` (#3225). Generation does not run typecheck, so no generated output moves.
- Eight files named .t27 that were Markdown documents are now t27 modules, with 54 test blocks between them: api/tri_net_api, benchmarks/bench_nn, benchmarks/gf16_bfloat16_nmse, conformance/e2e_scenarios and four under physics/.
- `tri misread` splits each pair into refused and silent, and trusts a refusing typecheck only after a clean spec has been seen to typecheck and show no shape. Measured: 41 pairs, 35 refused, 6 silent (the six are fixed in #5968).
- Suite ledger: +25 typecheck entries, -8 parse entries, max_entries 126 -> 112. Lean completeness ledger held at 77 by a swap: physics_lqg_cs_bridge now agrees with its theorem and is retired; benchmarks_gf16_bfloat16_nmse became classifiable and enters, because the spec computes in f64, which Icarus does not lower, and its theorem was written over the Markdown document.
