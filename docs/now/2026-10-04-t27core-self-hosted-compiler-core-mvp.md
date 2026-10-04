# NOW -- t27core: self-hosted compiler core MVP (2026-10-04)

## t27core: self-hosted compiler core MVP (Closes #5981)

- `specs/compiler/core/t27core.t27` is a t27 compiler written in t27: lexer, parser, checks and C emitter for the subset `t27c gen-c` lowers without loss; first step of epic #5980.
- Fixpoint holds: gen-c builds the core, and the core run on its own source writes gen-c's output byte for byte.
- `bootstrap/tests/core_selfhost.rs` gates the fixpoint, the core's own tests, 6 fixtures, 16 refusals with codes, and a corpus differential (58 specs accepted, all byte-identical to gen-c).
- Out-of-band check, not in CI: 18,000 grammar-fuzzed programs, 3,403 accepted, 0 differ from gen-c.
- Shapes gen-c lowers with loss are refused rather than copied; three new ones are recorded on #5980: `- -x` emitted as `--x`, declarations after `endmodule` dropped, and top-level consts truncated after a leading literal or name.
