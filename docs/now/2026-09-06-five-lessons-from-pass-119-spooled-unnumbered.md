# NOW -- Five lessons from pass 119, spooled unnumbered (2026-09-06)

## Five lessons from pass 119, spooled unnumbered (Closes #3236)

- A refusal recorded only in prose is invisible to the tool that advises against it -- `dispatch: NO` told the next reader to undo #3325 and put a dispatch back in front of `cargo publish`.
- `%ad` and `%cd` render in their own recorded offsets; comparing them as wall-clock strings invented a 7-hour skew that epochs refuted 0 of 20.
- Assert the edit MOVED, not that the anchor exists -- and guard `find()` with `i >= 0`, because `s[-1:j]` is empty and `replace("")` inserts at position 0.
- A private `CARGO_TARGET_DIR` hides the binary from `.githooks/pre-commit:52`, so the commit silently does not form while push reports success.
- `cli-tri` is not a required context, so a red census cannot block a merge; measured, this refuted my own published claim -- 1 failing commit of 12, not an era.
