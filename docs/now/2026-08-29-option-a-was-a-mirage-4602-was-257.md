# NOW -- Option A was a mirage: 4602 was 257 (2026-08-29)

## Option A was a mirage: 4602 was 257 (Refs #2754)

- brace-body looked like a channel nobody had opened, at 4602 tokens; 4514 of them sit in specs that ALSO have a bdd-block-fallback
- a braced statement inside a braceless block that fell back is the SAME event reached through a second function, and the split counted it as two
- the channel is now contextual: bdd-block-fallback 23852 + brace-body/in-fallback 4345 = 93% is one class, and the independent brace-body is 257 tokens across 10 specs
