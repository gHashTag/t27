# NOW -- A discarded modifier with no consumer is latent (2026-08-30)

## Two parser losses, 45 sites, zero live consequence (Refs #2880)

- `pub` on a struct field: a bare `if KwPub { advance(); }` with no write, 41 sites
- the `!` error-union marker: `has_error_union` computed, used to advance, then dropped, 4 sites
- both are the shape of #2867, where the lexer dropped a width suffix -- and that one WAS a defect because the Zig shift path re-invented the width and a function panicked
- these are not: the Rust backend emits `pub` on every field regardless, Zig and C have no field visibility, and three of the four `!` sites are bodiless declarations no backend emits while the fourth is noreturn
- the count is not the finding; for a lost piece of information the finding is the CONSUMER, and looking for one costs two commands -- generate both ways and diff
- filed as latent, with where to look on the day a backend wants either
