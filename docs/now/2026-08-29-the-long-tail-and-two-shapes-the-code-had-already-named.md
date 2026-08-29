# NOW -- The long tail, and two shapes the code had already named (2026-08-29)

## The long tail, and two shapes the code had already named (Refs #2754)

- given clk = true, rst_n = false: the loop's own comment names comma-separated bindings as a reason to stop mid-clause, and nothing acted on it -- 19 events in one spec
- given crossings : [i32] = []: the arm peeked for = immediately after the name, so an annotation between them read as not-a-binding
- 25093 -> 23926 tokens, 77 -> 76 specs, and not one acceptance column moved: zero rows differ across 650 specs
- --fallbacks --show <spec> scopes the census to one file; a census that names a target you cannot then open stops one step short
