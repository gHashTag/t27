# NOW -- zig-primitive-bindings (2026-10-04)

## Result

- Quote declared primitive value bindings and their references in Zig, with fresh item and nested block scope. Preserve builtin type expressions, aliases and casts.
- Source fixture and six real Zig tests cover typed/untyped locals, tuple bindings, parameters, mutable rebinding, unused discard, test/bench assignments, independent257 values and two source mutants rejected by executed assertion panics.
- M1 release PASS. Final M3:2834 passed,0 failed,2 ignored,124 targets. M4 clean expected-failure ratchet:95 observed,cap126, no unexpected/expired/discard/gate drift. The whole corpus is still not green.
- Actual107 tri-net Zig modules and1209 tests PASS, including19 SHA256 tests. Native generation keeps102Rust,75C and113Verilog outputs byte-identical to the accepted C correction. Product PIN and budgets untouched.
- Native804 sealed-spec audit: only7 Zig specs/13 aliases change. Save and verify without force;6 BLOCKED/1 PASS kept honest. Coverage1449/1325/124 and debt ledger unchanged.

## CI execution

- Original019e Linux bootstrap job recorded2828 passed/6 failed/2 ignored/124 targets. All six new tests failed spawning missing zig. Install actual Zig0.16 before the unchanged full ratchet; no test skip, ledger adjustment or compiler change.
- Existing Icarus lowerability tests include missing gitignored witness skips. Their cargo count must not be presented as359 executed RTL simulations. Independent tri-net113 simulations and the new Zig controls have separate actual execution receipts.

## Canonical alignment

- Merge upstream c107 history unchanged; its loop-tools workflow adds three shell steps. Combined native census is268 run steps/247 runner-selected shell steps (master267/246 plus one Zig install). Own compiler f668 and upstream CLI source/manifest/lock are preserved; preserve genuine upstream authors and all events.

## Corpus prerequisites

- Exact474673 Linux runs all six native Zig controls PASS. Its three existing corpus_zig_bodies tests now execute and refuse the absent iverilog prerequisite; cargo reports2831 passed/3 failed/2 ignored/124 targets. Install actual Icarus before unchanged tests; final native census269 run steps/248 runner-selected shell steps. No test skips or failing-set changes.

## Limits and next

- Separate upstream SHA256 source still BLOCKED on pointer errors; accepted-baseline source-only value rename reproduces all three. Temporal alias substitution in the existing optimizer is also unfixed. Broader keyword/dotted issue2631 remains open.
- Publish issue6040 fix from dmitrii-f-t27, verify exact-head Linux CI, then ordinary canonical merge. No compiler PIN adoption until the remaining two Verilog and24 strict Rust module blockers are fixed.
- No radio hardware or full model inference claimed.
