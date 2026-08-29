# NOW -- A shift is not symmetric, and my grep was reading the headline (2026-08-30)

## A shift is not symmetric, and my grep was reading the headline (Refs #2864)

- The return warning carried no line: parse_return_statement never set node.line, so all eleven printed ':?' and none could be looked up. One line to capture it before advance(); every warning is now addressable.
- With addresses, eight of the eleven were 'return x - (y >> shift);' in cordic_fixed and cordic_top. promote_types treated a shift like '+', so the SHIFT AMOUNT's type won: i16 >> u32 became U32. A shift's result has the type of the value being shifted -- C, Rust and Zig all agree. 11 -> 3 warnings, zero regressions.
- SELF-INFLICTED: my probe harness ran 'grep -oE "returns .. |Typecheck OK" | head -1', and the FIRST output line is the header 'Typecheck OK (0 errors, 2 warnings)'. Every probe read as passing while warning underneath. It cost four contradictory measurements and a dead hypothesis about parameter shadowing, and the correct hypothesis was the first one I had.
- The three that remain are genuine narrowing in the specs: ts_ps / period_ps returned as u32, and 'sign * prod' returned as i32 in gemm and systolic_array.
