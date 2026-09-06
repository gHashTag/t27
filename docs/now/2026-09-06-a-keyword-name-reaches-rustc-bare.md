# NOW -- A keyword as a variant or function name reaches rustc bare (2026-09-06)

## A keyword name reaches rustc bare (Refs #3347)

- The Zig emitter escapes enum variant names and explains why in a comment one line above
  its own call to `zig_ident`. The rule did not travel to Rust, and had never reached the
  function name in either backend's Rust output.
- `rust_ident` already escaped struct fields, parameters, struct-literal fields and field
  access. Two positions were missing: the enum variant and the function's own name.
- Measured: **336 to 338**, zero regressions. `variant.t27` and `async.t27` compile now.
- The two enum-variant specs still fail, and their causes CHANGED --
  `expected identifier, found keyword` became `in expressions, \`_\` can only be used`
  and `expected expression, found \`]\``. That is how the fix was confirmed there.
- Found by censusing the first rustc error of all 245 failures TOGETHER WITH the generated
  line. The pairing is what separated four classes hiding inside one error text.
- One correction to that instrument: rustc prints `aborting due to N previous errors` as
  an error line, so a naive count of distinct classes said NO spec was one fix away.
  Excluding the summary, 95 are.
- A second: "one error away" is a LOWER bound on the work. rustc stops early, so fixing the
  first error can surface others it never reached -- both enum specs show it.
