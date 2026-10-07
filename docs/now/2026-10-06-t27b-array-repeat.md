# NOW -- t27b: `[1] ** n` when the parser keeps the elements as text (2026-10-06)

## text-form array repeats (Closes #7112)

- New conformance spec `specs/tri/t27b/conformance/array_repeat_text.t27`. It gives 5 pass with 0 vacuous under `t27c test-report`, and the same 5 under t27b, with 17 runtime asserts.
- The parser keeps `[1]` and `[K, f()]` as text, and the reference pastes the text back as `.{ 1 } ** n`. t27b's `repeat_lit` now parses that text back into elements with `text_lit`, so it sees the same elements. The element checks of `text_elem` apply unchanged.
- `[] ** n` is still refused, as is a count that does not match the declared length. The reference refuses both too.
- 8 mutants each fail exactly the mutated test, in t27b and in the reference alike. Of 13 negative controls, 7 pass in both and 5 are refused by both. In the last one, a local `K` shadows the module `K`: the reference fails it and t27b refuses it.
- Code is in `cli/t27b/src/lower.rs`. Tests are in `cli/t27b/tests/source.rs`. Both files are listed in `tools/policy/foreign-exceptions.txt` under the owner's approval on #6063.
- Ledger `docs/reports/t27b_expectations.json`: `clade-meshd/src/transport.t27` and the new spec move to pass. `gen_fuzz.t27` now stops at `ExprCall(@intCast)`. The not-pass count goes from 48 to 47 on top of master aecf75e84.
