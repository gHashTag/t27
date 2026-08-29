# NOW -- The step became the body (2026-08-30)

## Four backends dropped the body of every `while (c) : (step)` (Refs #2860)

- `while (cond) : (step) { body }` has THREE children: condition, continue expression, body
- every `while` emitter read `children[1]` as the body and never looked at `children[2]`
- so `while (i < corpus.len) : (i += 1) { scores[i] = compute_similarity(...); }` became `while (i < corpus.len) { i += 1; }` -- a loop that advances a counter and computes nothing
- Zig, C, Verilog and Rust, all four; the audit reported it for Zig only
- 6 sites in 4 specs, measured; none of the six bodies contains a `continue` or a `break`, which is what makes "emit the step last" correct in the three languages that have no continue expression
- Zig gets the real `: (step)` form; C, Verilog and Rust get the step as the last statement of the body, with the `continue` limitation written next to it
- the continue expression arrives WRAPPED: emitting `children[1]` directly printed `/* unsupported expr: Module */`, so a shared `unwrap_single` descends to the statement
- nothing moved: zig ast-check 224/559, cc accepts 171, suite 2455/0. The wrong loop was already valid in all four languages
