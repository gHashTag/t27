# The dialect decision — one page for the Architect (W909)

After fourteen rungs and the forall rehearsal, the second of the two residue
masses is DIALECT CONTENT: Zig/Rust statement bodies inside keyword-form
test/invariant/bench blocks (`var x : [N]T = undefined;`, `while (…) {`,
`for (0..N) |_| {`, `@builtins`, imperative chains), concentrated in
specs/queen, specs/memory, specs/math templates and the two honestly-failing
specs/ar files.

## The rehearsal (this wave) — measured and reverted

A brace-aware VERBATIM statement capture at clause position (read, don't
run), under the same scope guards as statement clauses:

    0014 baseline        25,670 discarded tokens, 62 files
    dialect capture      23,739 (-1,931), 56 files, zero new parse-fails
    + forall capture      4,711 (-93.0% from the original 67,760),
                          consume-all 419, discarding files 32

The safe-guard ceiling recovers about half of the mapped dialect mass; the
rest opens blocks with a statement as the FIRST body line, where W905's
panel showed scope guards cannot discriminate and theft begins. Patch:
`DIALECT-PLUS-FORALL-REHEARSAL.patch` (both rehearsals, 0014-relative).

## The options

1. **Status quo** — per-line honest drops; the two specs/ar files stay
   honest parse failures. Cost 0.
2. **Verbatim capture (rehearsed)** — tokens read into `dialect-stmt` nodes,
   no semantics; seals become honest about content they cannot run. One wave.
3. **Actually teach the Zig statement subset** — var-undefined, while,
   for-capture, @builtins as real AST. A multi-rung project with backend
   consequences; only worth it if these specs are meant to GENERATE.
4. **Migrate the bodies** to the clause DSL. Author-time cost; the inventory
   argument (teach beat migrate 23,033:1,589) cuts the other way here --
   dialect bodies are not BDD content.

Recommendation: option 2 if the goal is certificate honesty; option 1 if the
files are legacy. A word here plus the forall word closes the whole residue:
**both words together take the corpus from 25,670 to 4,711 lost tokens.**

---

*φ² + φ⁻² = 3 | TRINITY*
