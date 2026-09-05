# NOW -- The argument for borrowing was thirty lines above the arm that ignored it (2026-09-05)

Tenth compiler fix of the pass and the seventh instance of one shape, but this one did
not have to travel between emitters. It had to travel thirty lines.

## The defect (Closes #3258)

- the `str` arm was given the borrowed form today, and its comment argues the case: "neither `[]const u8` nor `const char*` owns its bytes"
- the very next arm sends the literal `[]const u8` spelling down the owned path, so `const SPDX_HEADER : []const u8 = "..."` came out as `pub const SPDX_HEADER: Vec<u8> = "..."` -- `expected Vec<u8>, found &str`, and a `Vec` cannot be built in a const at all
- measured against master: **329 -> 330, +1, 0 regressions**
- the Zig backend already writes `const SPDX_HEADER: []const u8 = "..."` and `zig build-obj` accepts it; `gen-c` writes a `#define`, erasing the type rather than answering it

## A regression I measured earlier is what located the boundary (Closes #3258)

- I probed the broad version hours ago and DECLINED it: mapping both `[]const u8` and `[]u8` measured **+1 / -1**
- the regression was `specs/igla/race/opcodes.t27`, whose `chain: []u8` is indexed as `chain[(idx) as usize]` and cannot be a `str`
- so the decline was right for the broad rule and the narrow one is right: key on the **`const` qualifier**, not on the element type. `[]u8` stays a `Vec<u8>` and opcodes still emits `chain: Vec<u8>`
- a declined probe is not a dead end; its regression is a measurement of where the rule ends

## What the fan-out was worth (Refs #3258)

- nine agents against a pinned binary, after an earlier run of the same audit was thrown away because I rebuilt the compiler underneath it
- the agent rebuilt the 329/252/69 baseline ITSELF before trusting the one it was given, which is the check that would have caught my earlier mistake
- it regression-tested the one-arm alternative across all 24 accepted files containing `Vec<u8>`: **0 broken**
- and it named the suggestion trap explicitly: rustc renders `help: call Into::into` as a numbered line that comes LAST, so reading the last numbered line returns the already-corrected form

## The other cause in that lens is a decision, and it is cheap (Refs #3258)

- 3 specs write an array as a QUOTED STRING: `const DEFAULT_KERNEL : []u32 = "[2, 2]";`
- no sibling answers it -- Zig emits the same and `zig build-obj` rejects it; C emits a `#define` that compiles while meaning a string
- but the agent measured the decisive thing: rewriting the spec line to the canonical `[2]u32 = [2, 2]` makes the CURRENT binary emit `pub const DEFAULT_KERNEL: [u32; 2] = [2, 2]`, rc=0, **with no compiler change at all**
- so if the answer is "normalize the four spec lines", today's already-landed `[N]T` fix covers it for free; if the answer is "the emitter must parse a quoted list", that is new behaviour in `gen_const`. Four lines, and the choice is the owner's
