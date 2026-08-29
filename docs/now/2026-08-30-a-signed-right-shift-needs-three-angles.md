# NOW -- A signed right shift needs three angle brackets (2026-08-30)

## The Verilog backend filled with zeros where the spec said sign (Refs #2860)

- `>>` was mapped to Verilog's `>>` unconditionally; Verilog's `>>` is LOGICAL and fills with zeros however the operand is declared
- the signedness repair the backend already has fires only on ordered relations `< <= > >=`, so no shift was ever fixed up
- simulated on the ACTUAL generated module: `cordic_x_next(100, -64, shift=2)` returned **-16268**; the spec, C and Zig all say **116**
- y = -64 is 16'hFFC0 = 65472; 65472 >> 2 = 16368; 100 - 16368 = -16268
- this project's own hand-written golden CORDIC RTL writes `y0 >>> 1` for the identical rotation, so the right operator was known and the expression path emitted it ZERO times across 559 modules
- `>>>` occurrences corpus-wide: 2 -> 369 across 48 specs; both of the old two were inside string literals
- iverilog acceptance unchanged 373/559 -- the wrong shift was always legal Verilog, which is why no gate could have caught it
- tempered, and the audit said so first: 42 of the 45 affected specs shift non-negative values today, so they are LATENT. The three igla/race CORDIC specs are value-corrupting now
