# NOW -- Eight ledger entries no compiler change can retire (2026-09-05)

Seven passes of this loop shipped tooling. Every one touched `cli/tri` and the
skill; not one touched `bootstrap/` or `specs/`. The subject of the loop is the
compiler, so this pass measured the compiler instead of the rulers.

## Where the product stands (650 corpus specs, master 4d63859f3) (Refs #3204)

- generates Zig / Rust / C / Verilog: 581 each, 89.4%
- Zig accepts 308 (47.4%), and ANALYSES 190 (29.2%)
- rustc accepts 224 (34.5%); cc accepts 290 (44.6%)
- iverilog accepts 380 (58.5%), and 74 of those (11.4%) have a data port -- the other 306 compile without being able to carry a value across the module boundary
- Zig AND Verilog accept 258 (39.7%); ALL FOUR accept 184 (28.3%)
- 76 specs (11.7%) drop tokens, 23,831 of them; 69 specs do not parse at all
- the gap the report already names is the backlog: 581 generate, 184 are accepted by all four toolchains

## Eight entries whose `expires` date is a promise nobody can keep (Refs #3204)

- the ledger holds 152 entries at a cap of 152: 76 `parse-no-discard`, 69 `parse`, 7 `typecheck`
- the `parse` slice is exactly current -- all 69 re-run by exit code, 0 of 69 parse today
- eight of them lead a top-level line with `import` or `algorithm`; neither is one of the lexer's 25 keywords, and neither appears in ANY of the 581 specs that parse
- mechanical conversion to the language's own names was run over all eight: `use` is accepted where `import` was not -- `hybrid_bigint` moves its error from line 10 to line 49 -- but 0 of 8 parse afterwards, each carrying further non-conformance behind the first
- so these retire by rewriting or withdrawing the spec, and by nothing the compiler can do, while every entry's `expires` date reads as a promise that a fix is coming
- `max_entries` re-blesses at `min(previous, observed)`, so compiler work alone can never take the cap below eight
- rewriting them changes what those specs claim, so it is not proposed here

## Three claims controls killed before they were written down (Refs #3204)

- "twenty-six ledger entries are stale": a `near line N` matcher returned nothing for the entries whose message carries no line, and reading that silence as *passes* would have reported 26; exit codes say 0 of 69 pass, so the matcher was broken and the ledger was not
- "eighteen specs use constructs the language lacks": leading-word alone does not discriminate -- `type` appears in 9 specs that parse, `class`, `interface` and `package` in one each; only `import` and `algorithm` score zero across the 581, which is what makes eight defensible and eighteen careless
- "the corpus report has five rows": it has fourteen; the filter that stripped the progress lines `... 51/650` also stripped every continuation row indented the same way, including `... AND has a data port`, and the percentages above exist only because the filter was re-anchored on the `N/650` shape

## W643's non-empty floor, audited clean (Refs #3204)

- checked against every phase TARGET LIST rather than the phase count, since one list feeds many phases and the two numbers differ
- `specs_compiler`, `specs_only`, `specs_root` and the four narrow lists (yosys smoke, icarus-simulate, icarus-cocotb, phase 6/7) are each guarded by `require_targets`
- the fix travelled; a clean audit is also a result
