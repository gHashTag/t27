# NOW -- Two sites, one read: the census abstained where the answer was on the page (2026-09-04)

## `a guard, but two fetches` 4 -> 0, and the total held at 25 (Refs #2994)

- all four ambiguous sites sit in `issues.rs`'s `numbers` and `dated`, as the two arms of one `let raw = if instant.is_some() { gh } else { gh };` -- **two sites in the source, one read at run time**, and the single `read_is_complete` covers whichever arm ran
- **the precedent's repair would be wrong here:** `fn ready` was SPLIT so each guard had one subject; splitting these duplicates the guard and the parse, because the arms are one query with different filters -- with `--as-of` the state filter has to come off
- **two questions under one number:** `fetch_sites_in` counts SOURCE sites and feeds the published 25; the guard question needs reads that can RUN. `exclusive_fetch_sites_in` is a second function used only by `classify_fetch`, so the total is untouched
- **predicted before the change and held to the digit:** ambiguous **4 -> 0**, asks-whether-the-page-filled **4 -> 8**, sites **25 -> 25**, other buckets unmoved; 7+5+8+0+3+2 = 25
- **mutation removed two clauses:** `then > 0 && else > 0` (`min` already answers it) and `starts_with("let ")` (an assignment binds one value too). Both survived their own mutation, which is what said they were decoration
- three constructed counterexamples replaced them: a nested `} else {` at a deeper indent, a `let` line merely containing `if `, and a one-line if/else that opens nothing. **7 of 7 mutants, 10 tests**
- a fixture written from memory failed **5 of 7** tests: a fetch site is a line whose whole trimmed content is `"--limit",`. Read the matcher, then write the fixture
