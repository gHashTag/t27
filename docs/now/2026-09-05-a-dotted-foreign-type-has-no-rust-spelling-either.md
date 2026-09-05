# NOW -- A dotted foreign type has no Rust spelling either, and the predictor for these fixes is stub bodies (2026-09-05)

Fourth compiler fix of this pass, fourth instance of one shape, and the first
measurement of why the yield of such a fix varies so much.

## The defect (Closes #3219)

- `param_type_to_c` opens with the arm: a dotted foreign type has no C spelling, and `void*` is "the honest lowering and what a hand-written binding uses"
- `t27_type_to_rust` never got one, so the path arrived verbatim and rustc stopped at the first dot -- `pub allocator: std.mem.Allocator,` -- and never read the rest of the file
- **one field accounted for 15 of the 24 specs** in the class, and 34 corpus specs declare it
- Rust's `void*` is `*mut ()`, which needs no import; bracket forms are matched by earlier arms, exactly as the C rule excludes a leading `[` from its own dotted check
- measured by name over 650 specs: rustc accepts **252 -> 275, +23, 0 regressions**

## Why both of my predictions were far too low (Refs #3219)

- I predicted 0-2 for #3216 and measured 10; I predicted 4-8 here and measured 23
- the reasoning was "errors queue, so a signature fix only moves the error deeper", and that is the wrong predictor
- a signature-level fix clears a whole file only when the file has nothing else wrong, which is exactly the case when its bodies are STUBS

| fix | unblocked / first-errors | stub bodies among the gainers |
|---|---|---|
| serde (#3208) | 13 / 84 = 15% | not measured |
| pointer (#3216) | 10 / 39 = 26% | 44 of 44 = 100% |
| dotted (#3219) | 23 / 24 = 96% | 100 of 100 = 100% |

- so the predictor is **what fraction of the class's specs are stub-bodied**, not how deep errors queue
- corpus-wide only **8%** of functions are stubs; that is a different population and does not bear on the claim, and I nearly used it to refute a statement scoped to ten specs

## The Rust column across this pass (Refs #3219)

- 224 at the start, 237 after #3208, 242 after #3213, 252 after #3216, **275** here
- **+51 in total, 0 regressions at every step**, each measured by spec name so a gain and a regression could not cancel inside a total
