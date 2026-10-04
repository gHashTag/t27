# NOW -- Rust public constant names (2026-10-04)

## Result

- Fixes #5994. Valid immutable source constants such as type_documentation or mixedCount were emitted with their public names intact, but failed Rust's non_upper_case_globals lint under -D warnings.
- Emit a naming allowance on the affected immutable value declaration only. Preserve its exact name, type and value. Uppercase constants, mutable module variables, function type aliases, other backends and unrelated warnings keep their existing behavior.
- The executed fixture retains lower/mixed/uppercase constant APIs and an uppercase mutable module variable. Lowercase mutable statics are a separate policy question and retain their existing strict naming error.

## Actual validation

- The accepted baseline passes three controls and fails two constant-name regressions using actual native generation and Rust 1.96. Five final tests pass: independent runtime checks across 258 values, public constants and caller-visible state, a type-correct constant mutation that compiles but fails runtime assertions, an uppercase control, and actual rejection of unsigned-comparison and lowercase-mutable-static warnings.
- M1 native release and M2 native parse/typecheck pass. Full M3 no-fail-fast: 2822 passed, zero failed, two ignored across 122 targets. Full M4 native corpus ratchet: 95 expected failures, unchanged cap126 and no unexpected failure/pass/expiry/discard/gate drift. The whole corpus remains non-green; the compiler retains inherited build warnings.
- Complete native hash comparison over all 804 referenced sealed specs changes Rust output in 25 specs and 52 aliases only. Source, C, Zig and Verilog hashes stay identical. Actual RAW Rust hashes match native seal generation. Every inserted attribute precedes an immutable pub const; comments and all other tokens match, and actual standalone Rust diagnostics introduce no new class or count.
- All 52 old aliases match the accepted native baseline before save. Native reports are identical on both sides: 23 specs BLOCKED by inherited Zig compilation errors, two specs PASS, none fail or time out in this affected set. Save and verify exactly 52 aliases without force; full coverage stays 1449 seals, 1325 holding and 124 known broken, with the debt ledger byte-identical.
- A broader unpublished version also changed mutable static declarations. Its native report audit exposed two inherited infinite waits and the existing timing_tb failure. That version was archived; those specs and their seals are unchanged by this narrower correction.
- A stricter common-wrapper compile-only preview of 102 current canonical tri-net modules stays at 78 passing modules with zero regressions. integration_framework loses its naming diagnostic but retains three unused-assignment diagnostics. This is not full module execution or compiler PIN adoption proof.
- Align to accepted repeat canonical d72b871 by native fast-forward, preserving exact owned source bytes and all incoming/own append-only event records. The t27c dependency graph and incoming bootstrap sources/tests are unchanged. Rebuild the current canonical native TRI because upstream changed its hook policy; use the repository's actual hooks.

## Next

- Run exact-head Linux checks and accept the focused constant PR through normal required checks.
- Address source warning defects and a separately audited complete tri-net compiler PIN migration.
- Diagnose native test timeouts and existing testbench failures separately; physical radio validation requires boards.
