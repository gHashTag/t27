# NOW -- The oracle was in the corpus (2026-08-30)

## Two lessons from the signed-shift fix (Refs #2868)

- `gen-verilog` mapped `>>` to Verilog's `>>`, which fills with zeros however the operand is declared; the arithmetic shift is `>>>`
- simulated on the actual generated module: `cordic_x_next(100, -64, shift=2)` returned -16268 where the spec, C and Zig all say 116
- the decisive evidence was in the CORPUS, not the compiler: all 559 generated modules held 2 occurrences of `>>>`, and both were inside string literals holding this project's own hand-written golden CORDIC RTL, which writes `y0 >>> 1` for the identical rotation
- the project knew the right operator and the backend emitted it zero times; a hand-written reference in the repository is an oracle, and it costs one grep
- four defects fixed this pass and NOT ONE moved an acceptance number: the step-as-body, the dropped width suffix, the logical shift, the truncated test
- every one produced output the target compiler was happy with, so a gate asking "does it compile" could not see any of them
- what did see them: a second backend to disagree with, a hand-written artefact to compare against, and simulating the module instead of reading it
