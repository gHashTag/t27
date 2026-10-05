# NOW -- Twenty-one core specs stop discarding tokens (2026-10-02)

## Repair the specs/tri/ files the parser was throwing away (Refs #5497)

- The corpus ratchet reported 29 core specs that began discarding top-level tokens after 2026-09-12; 21 of them are under specs/tri/. Nearly all of it is test blocks: given/when/then blocks the parser cannot lower (several statements before `given`, two `then` clauses, `then result == void`, Zig types such as `std.StringHashMap` in a `given`), tests written without braces, one orphaned function body and one stray `}`.
- Where a test says something the language can express, it becomes a braced test with the same check (logging round trips and thresholds, config defaults, segment-tree update = 13, circular buffer state, rsa round trip, red-black tree operations, channel recv). Type-level checks become value-level ones (config, disjoint_set).
- Where it cannot -- function literals passed to map/bind/asks/filter, environments built as anonymous structs, bodies of `true` -- the block is removed with a comment saying why (io, reader, async_stream, base64, html, mime, xml, kd_tree): those blocks never reached codegen.
- All 21 parse with nothing discarded; every backend generates each spec as before; Zig test-report is unchanged except two removed empty tests (html, xml). disjoint_set's own typecheck error (a module-level `var` array treated as immutable) is now visible and is a typechecker defect, fixed separately. The 21 specs are resealed; published figures move by -19 test blocks and +2 `.len` reads, with notes.
