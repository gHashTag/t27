# NOW -- tri which -- where a subcommand goes (2026-09-04)

## tri which -- where a subcommand goes (Closes #3098)

- Three surfaces answer to 'tri' (bash arms, loop helpers, the Rust binary) plus 155 t27c subcommands, and nothing reported which one serves a name
- tri which NAME prints every route that claims the name; TRI_T27C and TRI_BIN are honoured, because a report that ignores the documented overrides describes a tree the caller is not using
- Three exits, and the third is the point: 0 served, 1 no route, 2 a binary that could answer is not built -- 'no such subcommand' and 'I could not find out' are different answers
