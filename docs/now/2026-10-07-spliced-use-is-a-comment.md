# NOW -- a `use` whose items the splice declared becomes a comment (2026-10-07)

## bootstrap/src/use_resolve.rs (Closes #7281)

- t27c splices each item that `use a::b::Item;` names from `specs/a/b.t27` into the importer, but the `use` line stayed in the resolved text.
  - When the body reads `Item.x`, the Zig backend lowers that `use` as an import.
  - So it wrote `const Trit = @import("Trit.zig");` beside the spliced `Trit`, and zig stopped at "duplicate struct member name".
- The resolver now writes that line as a comment: `// use a::b::E; -- E spliced below by t27c (#7281)`.
  - A brace list keeps the items the splice did not declare: `use a::b::{K, Q};` becomes `use a::b::{Q}; // K spliced below by t27c (#7281)`.
  - A whole-module `use`, an alias (`as`) and an item the splice did not declare stay as they were.
  - Each line maps to one line, so a diagnostic's line number still names the importer's line.
- Two unit tests in `use_resolve.rs`, one per behaviour. With the call removed, both fail and the other 38 pass.
- Measured on the Railway lab on the #7242 tree (403b27f29 plus #7242). The same tree was built twice, without and with the change, and run over all 1484 `.t27` files under `gen`, `gen-c`, `gen-rust`, `gen-verilog` and `typecheck`:
  - The exit code changes on none of the 5 x 1484 runs, and stderr changes on none.
  - Output changes in 7 files, under `gen` only. Every changed line is a removed `// use X: no references in this module` comment (29 lines). `gen-c`, `gen-rust` and `gen-verilog` do not lower `use`.
  - `typecheck` stdout differs on 2 files (`compiler/ast.t27`, `specs/bus/pubsub.t27`), where two "recursive struct" warnings swap places. That is not this change: six runs of the unchanged binary on `pubsub` print 2 different orders. Filed as #7283.
- On the tree with #7191's three slices, a `const X = @import("X.zig");` beside a declared `X` drops from 3 specs (5 names) to 0.
  - `t27c test-report` on those 3 specs still blocks, now on the next defect in each body.
  - `bigint`: "struct 'spec.BigInt' has no member named 'zero'".
  - `hybrid_bigint`: "pointless discard of local constant".
  - `runner`: "unused function parameter".
- No seal changes. A seal hashes a spec's own output before any `use` is spliced.
