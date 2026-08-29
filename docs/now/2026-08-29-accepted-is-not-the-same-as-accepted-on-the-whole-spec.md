# NOW -- Accepted is not the same as accepted on the whole spec (2026-08-29)

## Accepted is not the same as accepted on the whole spec (Refs #2754)

- corpus now ends with the discard beside the acceptance columns: 87 specs, 32485 tokens, and a line saying what that means
- --per-spec gains a dropped column, so the diff that names a moved row also shows what that row throws away
- docs/PARSER_DISCARD_LANDSCAPE.md: rowan makes discarding unrepresentable, tree-sitter represents it and still does not report it (tree-sitter#4049), bison puts it in the grammar
