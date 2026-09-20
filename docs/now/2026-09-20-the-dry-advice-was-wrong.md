# NOW -- The DRY advice I shipped yesterday was wrong (2026-09-20)

## Cross-module reuse does not generate, so the brief no longer tells a bee to try it (Closes #4296)

- #4293 put one line into every issue the feeder writes: "If it exists, reuse it (`use module::name;`)". Measured today on a two-spec minimal case: `use m::f;` and `use m;` both generate `// use f: no references in this module`, no `@import`, and an unqualified call; `zig test` answers `use of undeclared identifier 'double_it'`. `t27c spec-status` calls the same file IMPLEMENTED.
- That is the worst shape an instruction can have: it passes the cheap gate and fails the oracle, so a bee following it looks right until the review runs the compiler.
- The brief now says what is true - do not copy it, do not try to import it, name the file and line where it lives and implement only what this spec's criteria ask - and the duplicate ratchet still refuses a new copy.
- What this does NOT fix: the 576 duplicate bodies stay. Removing them needs `use a::b;` to emit an import, which is a compiler change and is filed separately.
