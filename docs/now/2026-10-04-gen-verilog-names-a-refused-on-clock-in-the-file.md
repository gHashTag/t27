# NOW -- gen-verilog names a refused on_clock in the file (2026-10-04)

## gen-verilog names a refused on_clock entry point in the .v file (Closes #5963)

- The in-file `// ENTRY POINT REFUSED` comment was written only inside the `NO DATA PORTS` branch, so an `on_clock` whose parameter had no derivable width but whose module still exposed a `var` output port got no comment; only stderr reported it.
- The comment now comes from one helper, `VerilogCodegen::write_entry_refusal_comment`, called whenever an entry point is refused, for `on_comb` and `on_clock` alike.
- Three new tests in `bootstrap/tests/verilog_struct_entry_ports.rs`: a refused `on_clock` slice parameter is named once in the file and once on stderr, the `on_clock` and `on_comb` comment blocks are identical, and an accepted `on_clock` writes no refusal.
- Corpus: gen-verilog over all specs under `specs/` is byte-identical before and after (no corpus spec has a refused entry point), stderr and exit codes unchanged; no seals touched, `FROZEN_HASH` updated. Part of epic #5905.
