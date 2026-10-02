# NOW -- The JTAG relay spec was Zig, not t27 (2026-09-21)

## specs/port/tools/jtag/link_relay.t27 has never compiled, on any backend (Fixes #4559)

- It landed in #4533 today already broken: `?struct{ role: []u8, idcode: u32 }` at line 104, plus 23 `@intCast(x as T)`, 14 `try std.testing.expect`, `m.tms(&[...])`, `while (i < n) : (i += 1)`, `.{ .role = r }`, `null`/`.unwrap()`/`.or(0)`. Zig end to end, in a file named `.t27`.
- Rewritten against `read_user1.t27` -- its neighbour in the same directory, same JTAG/MPSSE domain, and a spec that does generate. `t27c check` reports 0 errors and 0 warnings; `gen-c`, `gen-rust`, `gen-verilog` and `gen` all exit 0.
- The optional is gone, not papered over: `DrWord { ok, value }` keeps the distinction the Python made with `None`, so a caller that ignores `ok` reads a zero rather than a word the hardware never sent. The two IR loops, spelled four times between `xfer` and `request`, are one `chain_ir(idx, chain, sel)`.
- One of the seven dead tests asserted a wrong constant: `0x1F3DF` for IDCODE at idx=1 on a 3-chain, where `0x1F000 | (0x0E << 6) | 0x1F` is `0x1F39F`. A test that cannot compile cannot be wrong out loud. The arithmetic in the rewrite was checked against an independent computation of the same loop.
- `specs/ml/optimizer/adamw.t27` was fixed by #4540 and left in `tools/specs_generate_baseline.txt`. `check_specs_generate.py` reports "baseline now generates" and returns before it reaches "newly does not generate", so every run stopped at adamw and never looked at link_relay. The stale line is pruned here; that is what made this visible.
- What this does NOT establish: that the generated C compiles. `undefined` lowers to `{0};`, which is not valid C, and `read_user1.t27` -- classed *Working* -- fails `cc` with 6 errors for the same reason. That is a backend defect across every spec that stubs a body, filed separately, not this file's.
