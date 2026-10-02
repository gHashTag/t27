# NOW -- a compiler defect names its stage, and the compiler is one file (2026-09-24)

## 59 more issues stopped naming nothing (Closes #4766)

- `bootstrap/src/compiler.rs` is **1.79 MB** and holds the lexer, the parser, the typechecker and every generator. An issue reporting that `t27c parse` accepts a body of zero statements, or that `gen-c` emits `0 bad;`, is about that file -- and says so by naming the STAGE, which is how a person writes it and why every path rule here read it as naming nothing.
- "gen-c: `null` has no C spelling", "The lexer turns 0o777 into 0", "The emitter dropped every `x as T` cast: 645 in 67 specs" -- 44 of the 286 were this, and writing them took the count to 59 once the other rules were applied in the same run.
- They will conflict with one another, because they really do all touch one file. The Queen runs them one at a time. That is correct, and it is not what "unreachable" meant.
- **Board: 580 open, 237 without a boundary (was 565 of 649 yesterday).**
- What the remaining 237 are, counted rather than guessed at: **62 `Wave Loop N` run logs** and **57 `formal: ... (Prop. N)` proof notes** -- reports of what happened, not units of work, and a boundary is the answer to "which files does this work own". The rest are defect reports whose fix location is a judgement: #3225 names fourteen specs the compiler reads wrongly, and the fix is in neither the specs nor any file it mentions.
