# NOW -- Two identical declarations are not ambiguous (2026-08-29)

## The refusal needed the candidates to disagree (Refs #2764)

- #2764 said "gen-c does not resolve `use`". It does -- `run_gen_c` calls `use_resolve::resolve`, and so do the Zig and Rust backends
- the real cause: `Trit` is declared VERBATIM IDENTICALLY in `base/types.t27` and `base/ops.t27`, six specs import both, and the resolver refuses to choose between them
- it writes `// UNRESOLVED Trit: declared in base/ops.t27 and base/types.t27 -- ambiguous, not spliced` into the resolved source, and codegen strips comments, so the compiler knows exactly why it cannot emit `Trit` and throws the sentence away
- when the candidates are the same declaration there is no choice to get wrong; compared line-by-line trimmed, because the corpus writes two indentation conventions
- measured: 30 ambiguous (spec, name) pairs, 10 agree and 20 genuinely differ -- `PHI` in constants vs sacred_physics stays unresolved and must
- effect: `unknown type name` errors across the 460 specs with `use` fall 944 -> 802; specs compiling with zero errors stay 82 -> 82, so no spec flipped and the claim is not that one did
- `grep '^use '` found 68 specs; the directives are indented inside `module M;` and there are 460
