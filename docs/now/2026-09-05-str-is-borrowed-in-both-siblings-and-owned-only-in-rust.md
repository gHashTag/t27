# NOW -- `str` is borrowed in both siblings and owned only in Rust (2026-09-05)

Sixth compiler fix of the pass. The evidence sits three lines apart in the same file.

## The defect (Closes #3239)

- `compiler.rs:8322` maps `"str" | "string"` to `[]const u8` for Zig, `compiler.rs:18248` maps it to `const char*` for C, and `compiler.rs:24426` mapped it to the **owned** `String` for Rust
- `String` is not constructible in a `const` item, which is where the corpus mostly uses it, so specs generated `pub const DEFAULT_NOTEBOOK: String = "t27-QUEEN-BRAIN";` and rustc answered `expected String, found &str`
- **48 of the 75** specs whose first error was E0308 carried that exact mismatch
- Rust's borrowed spelling is `&'static str`; measured by name over 650 specs, rustc accepts **288 -> 314, +26, 0 regressions**

## The judgement, stated rather than hidden (Closes #3239)

- `&'static str` is narrower than `String`: a field of that type can hold only a string that lives for the program
- that narrowing is what both siblings already chose -- neither `[]const u8` nor `const char*` owns its bytes -- and it is the only spelling that works in the `const` position the corpus actually uses
- zero regressions across 650 specs says nothing in the corpus needed the owned form

## A boundary my own predictor was missing (Refs #3239)

- #3219 established that a fix's yield tracks the STUB-BODIED share of its class; this class is **3% stub-bodied** (20 of 797 functions), so I predicted **+2 to +8** and measured **+26**
- the rule applies to a fix that repairs a SIGNATURE while leaving a body behind: there a real body carries further defects and the fix cannot clear the file
- it does NOT apply to a fix that repairs a COMPLETE DECLARATION. `pub const X: T = literal;` is correct or not on its own and no body's state bears on it, which is why 26 files cleared whose bodies are not stubs at all
- two predictions with the rule, two misses in opposite directions, and the boundary is what separates them

## Two hazards met while landing it (Refs #3239)

- the patch saved from the previous branch would not apply after that branch squash-merged, because the squash moved every line number; re-applying by hand against the named anchor is what worked
- worse, `tri seals drift --fix` was then run while `target/release/t27c` still held the OTHER branch's binary, so it sealed output the source tree could not produce -- **the seal tool reads the built binary, not the source**, and that is the third time in this pass a stale binary has acted as the ruler
