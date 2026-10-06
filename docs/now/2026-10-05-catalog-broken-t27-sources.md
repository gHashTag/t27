# NOW -- catalog Broken in gHashTag/t27: five malformed sources fixed (2026-10-05)

## Five specs the parser rejected now pass every backend (Closes #6437)

- Re-measured the 59 t27 paths the SPECS catalog calls Broken with master t27c on the Railway lab (parse plus 7 backends): 24 are negative fixtures under `bootstrap/tests/`, 23 already pass on master, 7 fail only gen-verilog on `[N][]const u8` (#6438), 5 were malformed source.
- `examples/fpga/qmtech_minimal/design.t27` was `#`-comment pseudo-config; it is now module `QmtechMinimalDesign` (heartbeat pattern, UART timing, resource budget; 6 tests pass under zig, 3 invariants) and defers pins to `specs/boards/` and `fpga/HARDWARE_SSOT.md`.
- `specs/ar/datalog_engine.t27` (`loop`/`break` -> `while`), `specs/ar/ternary_logic.t27` (dropped `type Trit = Trit`), `specs/test_framework/graph_drift_detection.t27` and `verilog_bench_harness.t27` (`format!` -> `format`, tuple `for` and `if let` -> `keys()`/`contains_key`, `T?` -> `?T`); comment non-ASCII replaced (L3).
- Seals re-saved with `t27c seal --save` on the lab; the stale `qmtech_minimal_design.json` (module `design`) is replaced by `qmtech_minimal_QmtechMinimalDesign.json`.

## Follow-up: ledgers shrunk after #6440 turned master red (Closes #6452)

- #6440 fixed the specs but left them in the ledgers: Corpus Ratchet saw 4 UNEXPECTED PASSES and 2 new failures one phase later, and check_specs_generate saw 5 baseline specs that now generate.
- `specs/ar/ternary_logic.t27`: two invariants carried Rust-style parameter lists the parser dropped (62 tokens); they now use free variables like their siblings.
- `specs/test_framework/graph_drift_detection.t27`: Rust `match` -> `if` chain; `compute_graph_hash` takes a node count, since `HashMap<K, V>` in a parameter list splits into two parameters.
- `docs/reports/suite_expectations.json` 113 -> 109 entries (cap follows); 5 lines removed from `tools/specs_generate_baseline.txt`; 6 seals re-saved. Lab: `t27c suite --ratchet --corpus-only` RATCHET CLEAN 109/109, check_specs_generate OK.
