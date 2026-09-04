# Eight ledger entries no compiler change can retire

Seven passes of this loop shipped tooling. Every one touched `cli/tri` and the
skill; not one touched `bootstrap/` or `specs/`. The subject of the loop is the
compiler, so this pass measured the compiler instead of the rulers.

## Where the product stands (650 corpus specs, master 4d63859f3)

```
generates Zig / Rust / C / Verilog   581   89.4%   (each)
  ... and Zig accepts it             308   47.4%
  ... and Zig ANALYSES it            190   29.2%
  ... and rustc accepts it           224   34.5%
  ... and cc accepts it              290   44.6%
  ... and iverilog accepts           380   58.5%
  ... AND has a data port             74   11.4%
Zig AND Verilog accept               258   39.7%
ALL FOUR accept                      184   28.3%
specs with tokens DROPPED             76   11.7%   (23,831 tokens)
specs that do not parse               69
```

The gap already named in the report is the backlog: 581 generate, 184 are
accepted by all four toolchains. The sharpest single column is the one W693/T180
put there — iverilog accepts 380, and 74 of those can move a value across the
module boundary. The other 306 compile without being able to carry a value.

## The eight

`docs/reports/suite_expectations.json` holds 152 entries at a cap of 152: 76
`parse-no-discard`, 69 `parse`, 7 `typecheck`. The `parse` slice is exactly
current — all 69 re-run by exit code, 0 of 69 parse today.

Eight of them lead a top-level line with `import` or `algorithm`. Neither word is
one of the lexer's 25 keywords, and neither appears in **any** of the 581 specs
that do parse. Mechanical conversion to the language's own names was run over all
eight: `use` is accepted where `import` was not — `hybrid_bigint` moves its error
from line 10 to line 49 — but **0 of 8** parse afterwards. Each carries further
non-conformance behind the first. There is no mechanical repair; these retire by
rewriting or withdrawing the spec, and by nothing the compiler can do.

The ledger cannot say that. Every entry carries an `expires` date, which reads as
a promise that a fix is coming. For these eight no fix is coming from this side of
the tree, and because `max_entries` re-blesses at `min(previous, observed)`,
compiler work alone can never take the cap below eight.

Rewriting them changes what those specs claim, so it is not proposed here. Refs
#3204 records the distinction and the list.

## Three claims this audit killed before writing them down

**"Twenty-six ledger entries are stale."** A `near line N` matcher returned
nothing for the entries whose message carries no line, and reading that silence
as *passes* would have reported 26 stale entries. Exit codes say 0 of 69 pass.
The matcher was broken, not the ledger — the third time a silent matcher has been
read as an empty population in this tree.

**"Eighteen specs use constructs the language lacks."** Leading-word alone does
not discriminate: `type` appears in 9 specs that parse, `class`, `interface` and
`package` in one each. Only `import` and `algorithm` score zero across the 581,
which is what makes eight the defensible number and eighteen the careless one.

**The report has five rows.** It has fourteen. The filter that stripped the
progress lines `... 51/650` also stripped every continuation row the report
indents the same way — `... and Zig accepts it`, `... AND has a data port`. The
percentages above exist only because the filter was rewritten to anchor on the
`N/650` shape. A window narrower than its population, read as the population.

## Audited clean

W643's non-empty floor was checked against every phase *target list* rather than
against the phase count — the two differ, since one list feeds many phases.
`specs_compiler`, `specs_only`, `specs_root` and the four narrow lists (yosys
smoke, icarus-simulate, icarus-cocotb, phase 6/7) are each guarded by
`require_targets`. The fix travelled. A clean audit is also a result.
