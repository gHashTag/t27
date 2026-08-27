# NOW -- The only corpus metric that does not lie reported half the backends (2026-08-28)

## The only corpus metric that does not lie reported half the backends (Refs #2161)

- Refs #2161. `t27c corpus` calls itself "the only corpus metric that does not lie" and measured Zig and Verilog. Rust had no compile gate anywhere in the repository and neither did C -- which is exactly how an empty match for every switch, a dropped body for every for, and a u64 typed as int all shipped with a green exit
- Added gen-rust -> rustc --emit=metadata and gen-c -> cc -fsyntax-only, plus the row the two-backend table could not show: how many specs satisfy ALL FOUR toolchains. That is what "one spec, four targets" claims and nothing counted it
- On a 39-spec sample: Zig accepts 12, rustc accepts 0, cc accepts 1, iverilog accepts 7, and ALL FOUR accept ZERO. The zero is the number this change exists to print
- The "BOTH backends accept" label became "Zig AND Verilog accept": with four columns the word both no longer names anything, and leaving it would have made the new rows read as a subset of it
