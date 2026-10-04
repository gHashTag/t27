# NOW -- gen-verilog sizes struct and uN entry ports (2026-10-04)

## gen-verilog sizes struct and uN entry ports by one rule (Closes #5904)

- `VerilogCodegen::entry_port_width` is the one rule for whether an `on_comb` parameter becomes a module port and how wide it is; the entry-points census calls it instead of its private `is_sized_primitive` list.
- A struct whose fields are all strictly sized is a port of the summed field width, checked against the width the generated function declares; nested structs, slices and floats stay refused.
- `uN` / `iN` for N in 1..=128 are N bits wide; `ternary_mac` with a `TernaryWeight` operand now has `input wire [7:0] w` and synthesizes to 222 yosys cells instead of NO DATA PORTS.
- A refused entry point is reported on stderr as `t27c gen-verilog: ENTRY POINT REFUSED -- <param>: <type>`; the in-file comment and exit status are unchanged.
- A parameterless `on_comb` is called as `on_comb(1'b0)`, which also fixes iverilog for `d_full_path` and `d_static_one`.
- Corpus: gen-verilog output changed for 8 specs, refused specs 1 -> 0; 9 seals and `FROZEN_HASH` resealed. Part of epic #5905.
