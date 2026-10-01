# NOW -- the Trinity manifest reads build.zig's code, not its comments, and counts the vendored t27 copies apart (2026-10-01)

## tools/trinity_manifest.py -- two defects the consumer's capability index found (gHashTag/trinity#989)

- Comments. `parse_build_zig` ran its patterns over the raw text of build.zig, so a `b.step`
  or an `installArtifact` in a commented-out line was counted. At the pinned revision 976df517
  that invented two steps (`needle-mcp`, `trinity-mcp`) and two guarded installs
  (`trinity-canvas`, `trinity-canvas-wasm-check`), and the reachability walk, which read comments
  too, reached one .zig file only through a commented `@import`. build.zig and every walked .zig
  file are now read with their
  `//` comments (`///` and `//!` too) blanked in place, offsets and line numbers kept; a `//`
  inside a string, after an escaped quote, or in a multiline string line is text and stays.
- Vendored copies. Every tracked .t27 outside the website mirror was canonical, so trinity's
  byte-identical copies of these specs under `external/t27/` (locked by its contracts.t27) read
  as trinity's own: at trinity main 37bb4a94 the checker reported 110 canonical, 70 of them the
  copies. `external/t27/` is now a second never-canonical prefix, counted apart as
  `T27_VENDORED_FILES`, and a `CANONICAL_SPEC` pointing into it is refused (MIRROR_AS_CANONICAL).
  At trinity main the corrected reading is 40 canonical and 70 vendored.
- `--self-check` plants three new defects (sixteen in all), checks the comment blanker on a
  fixture holding every shape it must tell apart, and runs `inventory()` end to end over a
  planted repository. Sixteen mutants of the fix, each killed by the assertion aimed at it.

## What the corrected inventory changed

- `conformance/trinity/inventory.json`, regenerated from a clean checkout at the pin: 66 steps
  (was 68), 3 guarded installs (was 5), 748 reachable and 2082 unreachable .zig files (was 749
  and 2081), 0 vendored. `report.json` and the S03 `build_graph.json` (its profiles come from the
  inventory) regenerated from it; `project.t27` and the README state the corrected counts and
  what they read before.
- `mcp.needle-mcp` and `mcp.trinity-mcp` no longer own the commented-out steps. Their acceptance
  is the build that installs the binary (`zig build -Dci=true && test -x zig-out/bin/<name>`),
  which is what the cited CI run measured; their evidence source says the run steps are
  commented out at the pin and that no run of either server is measured.
- `research.unreferenced-sources` states 748 and 2082. The four edited specs are re-sealed with
  the current compiler, and `seal --verify` matches all four.

## Follow-up in gHashTag/trinity

- After this merges: `python3 tools/contracts.py vendor --t27 <clone> --revision <new master>`,
  then drop `trinity/mcp.needle-mcp` and `trinity/mcp.trinity-mcp` from `KNOWN_BLOCKED` in
  `specs/reproduce/capabilities.t27`: both are in `RUN`, their new acceptance runs and passes,
  and a known-blocked card that measures complete fails the index until its line is removed.
