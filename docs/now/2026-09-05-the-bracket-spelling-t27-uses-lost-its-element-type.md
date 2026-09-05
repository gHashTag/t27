# NOW -- The bracket spelling t27 actually uses lost its element type (2026-09-05)

Second compiler fix of this pass, same shape as the first: a rule that exists in one
emitter and never travelled to its sibling.

## The defect (Closes #3213)

- two spellings share the shape `[...]` and put the element type on opposite sides of the bracket
- Zig-style `[N]T` has the SIZE in the brackets and the element after them; t27's own `[T]` -- the spelling the corpus uses, as in `proof_steps: [ProofStep]` -- has the ELEMENT in the brackets and nothing after
- `t27_type_to_rust` read only the tail, so `after` was the empty string and **52 specs received `Vec<>`**, which rustc cannot parse; everything behind it in the file was invisible to any first-error histogram
- a second, adjacent miss: the Rust mapper knew `"str"` and not `"string"`, so 34 specs using the long spelling received the bare word `string` as a Rust type

## The target was set by siblings, not inferred (Closes #3213)

- the C emitter already reads `[ProofStep]` correctly -- the same field comes out as `ProofStep* proof_steps;`, and `[string]` as `const char** args;`
- the Zig mapper already spells the alias `"str" | "string" => "[]const u8"` at compiler.rs:8322
- so neither fix required a judgement about what the spelling means; two backends had already answered it

## Measured, 650 corpus specs, by name (Closes #3213)

- rustc accepts 224 before either fix of this pass, 237 after #3208 alone, **242 with these two on top**
- **+5 from this change, 0 regressions**
- specs emitting `Vec<>` or `pub f: ,`: **52 -> 22**
- the 22 that remain are the `pub f: ,` form, an empty field type and a different cause, not touched here
- the distance between "52 stopped emitting a degenerate type" and "+5 accepted" is the distinction #3208 recorded: a first-error count is not an unblocking count

## Process (Refs #3213)

- 40 seals re-written by `tri seals drift --fix`; only `gen_hash_rust` and `sealed_at` move, and the C, Verilog and Zig hashes are untouched, as they must be for a change to the Rust mapper alone
- `bootstrap/stage0/FROZEN_HASH` moves in the same commit, digest from `t27c frozen-digest`, per FROZEN.md §5
- the branch was first built on top of the still-open #3209 and then rebuilt on master after that squash-merged, because a squash makes a sibling branch's content unrelated by ancestry -- the trap `tri skill renumber` warns about, met here in the compiler instead of the skill
