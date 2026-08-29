# NOW -- One name, two definitions -- and my own scanner was wrong twice (2026-08-29)

## One name, two definitions -- and my own scanner was wrong twice (Refs #2774)

- tri types dup: 299 struct definitions, 21 names defined more than once, 16 CONFLICTED (field lists differ) and 5 DUPLICATED (same fields twice)
- the first version counted 284 where grep counted 299: a newtype (struct CallID(str);) and a one-line empty body (struct PollSlow {}) each swallowed every definition after them
- found by cross-checking against grep BEFORE shipping -- the wrong number was entirely plausible and nothing else would have caught it
- the quantifier census had its own copy of that scanner and the same two bugs; both now use one implementation, and its 15 became 16
