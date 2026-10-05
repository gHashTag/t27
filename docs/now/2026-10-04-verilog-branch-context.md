# NOW -- verilog-branch-context (2026-10-04)

## Result

- Carry a value-returning final if/else's tail context into its branches, including nested final if/else. Preserve function locals' existing recursive hoisting and initialization. Ordinary non-tail statements and explicit-only return branches retain their previous lowering.
- Carry native test and bench assertion context through nested branches and loops. Declare assert_eq probes in the same recursive order as emission, before procedural statements; preserve the existing failure flag and verdict counters.
- Source fixture and six actual Icarus tests cover257 independent inputs, nested branch values, two hoisted locals in each branch, u8 narrowing, u64 shifts/division with a zero guard, explicit returns, ordinary statement bodies, skipped branches, loops and a native bench. Two source mutants compile then fail executed assertions; compile failures are not accepted as runtime negatives.
- M1 release PASS. Full M3:2840 passed,0 failed,2 ignored,125 targets. M4 expected-failure ratchet clean:95 observed,cap126, no unexpected/expired/discard/gate drift. The whole corpus is still not green.
- All113 tri-net modules execute actual native Icarus simulation with1269 SOURCE tests. Measured caps34/28/28/28 remain unchanged. Of397 default generated outputs, only specs/m3_multihop.t27 Verilog changes; the other396 outputs are byte-identical to the Zig binding correction. Product sources, compiler PIN and budgets are untouched.
- Native804 sealed-spec audit: only3 Verilog specs/7 aliases change. Actual RAW output matches native hashes; synthesis-visible non-function bytes, signatures and operation tokens remain identical except the required return assignment targets and block delimiters. Native save/verify without force retains all three SOURCE reports as BLOCKED. Coverage1449/1325/124 and debt ledger remain unchanged.

## Limits and next

- RAW hash and scope comparisons do not prove synthesis of the three blocked specifications. Existing source/type defects remain; no forced PASSED verdict or wider baseline was introduced.
- Publish the focused issue6043 fix from dmitrii-f-t27, verify exact-head Linux CI and then ordinary canonical merge. Reuse the accepted Zig/Icarus CI prerequisites without duplicating workflow changes.
- Next, independently fix Rust implicit tail returns in m3_multihop and audit the actual committed Rust warning policy. Compiler PIN adoption requires complete native acceptance across the remaining defects.
- No radio hardware, first place, complete red-cell closure or full model inference is claimed.
