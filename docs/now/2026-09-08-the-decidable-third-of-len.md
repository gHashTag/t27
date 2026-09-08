# NOW -- The decidable third of `len` (2026-09-08)

## The decidable third of `len` (Closes #3489)

- Five passes named the `len` family and moved on. Splitting it by **what the base actually is** shows a third of it needs no decision: `string` lowers to `const char*`, and C's answer for its length is `strlen`. **223 sites have a `string` parameter as the base; 449 have a slice**, which is the part that genuinely needs a representation (#3464).
- **Three spellings, not one**: `s.len()` parses as ONE `ExprCall` named `s.len` (**1322** in the specs), `s.len` as an `ExprFieldAccess` (**687**), and `len(s)` as a free function (142 diagnostics). The fourth time in this campaign that one rule had several spellings and only one was taught. The first two now go through **one helper**.
- Errors **11 642 -> 11 401**, **14 files better and none worse**, 247 `strlen(` emitted across 15 files. `igla_coder_eval` 178 -> 103.
- **`#include <string.h>` is decided by the same two helpers the emitter uses.** A missing include is an undeclared function -- the very family this repair shrinks -- and deciding it in a second place is how that happens. A mutant that drops the include is killed by five tests.
- **The third spelling was measured and NOT handled.** Of the 302 argument shapes of `len(x)`, **zero** are a `string` parameter: 171 are identifiers that are not parameters, 89 are not plain identifiers, 19 are slices, 14 are `u32`. A branch for it would be unreachable today, so it is filed rather than written.
- **A mutant that dropped the per-item reset survived, and the fixture was the reason.** The leak test declared an `i32` named `s` in a test block and asserted `s == 3` -- a leaked set only shows up where the rule would FIRE, so nothing caught it. Rewritten to `assert(s.len() == 0)`, which becomes `strlen(s)` on an `i32` if the set leaks; the mutant dies.
- 1337 `.len` sites remain (886 calls, 451 fields) and are left loud on purpose: `strlen` on a slice would read past the end of anything that is not NUL-terminated bytes.
