# NOW -- A census that proves its own categories (2026-08-30)

## A census that proves its own categories (Refs #2864)

- tri unparsed now carries a PROBE per construct -- a minimal source the compiler must reject for the row to be named. Six candidates read off real failing lines turned out to compile in isolation: @trim(), .anthropic, if-as-expression, *Foo, &Foo and []const u8. An earlier fan-out had named []const u8 as a cause.
- And a COUNTER per construct -- a near-identical source the compiler ACCEPTS, on which the matcher must stay silent. That is the half the probe cannot check: is_use fired on every use line while only 'use a::b as C;' fails, and its probe WAS the aliased form, so the probe passed and the matcher was still wrong.
- A third state: refused ON PURPOSE. Casts to non-primitive types are a documented position (no backend lowers float arithmetic, argued beside VALID_CAST_TYPES), not a gap. Three specs moved out of the work queue and into a section that carries the citation.
- pub module N; accepted -- pub is a modifier the declaration parser already reads for fn/struct/const, and a module was the one place it was not. Zero specs gained: the single file carrying it advanced 17 lines and stopped on '**'. Said plainly rather than counted as a win.
- The design is self-invalidating and proved it twice in one run: module a::b and pub module N; both flipped to ACCEPTED and left the queue without anyone editing a list.
