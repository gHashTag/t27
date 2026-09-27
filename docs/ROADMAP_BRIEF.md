# The roadmap brief: what the swarm is building, and how to rewrite code into `.t27`

Every roadmap task quotes this file (`tools/queen/feed_roadmap.py` embeds its two
sections below into each issue it files), so a bee sees it in its prompt. Edit it
here; the next task carries the change. Every claim about the language was measured
on 2026-09-27 with `t27c test-report`, the command the review runs.

Two rules for editing it: no heading may contain the words the Queen reads as a
criteria heading (`acceptance`, `criteria`, `done when`, `критерии`), and no line may
start with `## Boundary`. Either one would change what she reads out of every issue
that quotes this text. `python3 tools/queen/feed_roadmap.py --self-test` checks both.

## The goal of the game

**Everything below the interface is written once, in `.t27`, and generated to its
target**: Rust for servers; Zig, C and Verilog for the core and the silicon. That is
law L0 (`docs/T27-CONSTITUTION.md`). Two exceptions are named and nothing else is
exempt: the seed (`t27c` itself, `bootstrap/`, stays hand-written Rust) and the
interface (Swift for trios, TSX for the web), which stage 8 has to decide.

The roadmap walks the stack in eight stages. Each stage is a goal issue; the work is
filed as one task per file, like the one you are reading.

| Stage | Goal | What | Where it lives | Rewrite | Generated to |
|---|---|---|---|---|---|
| 1 | #4543 | the swarm's own tools | gHashTag/t27 | Python, shell | t27c gen-rust |
| 2 | #4544 | the Queen and her bees (api.t27.ai) | gHashTag/BrowserOS | TypeScript, Go | t27c gen-rust |
| 3 | #4545 | the app and the bot (app.t27.ai) | gHashTag/999-multibots-telegraf | TypeScript, JavaScript, Rust | t27c gen-rust |
| 4 | #4546 | VIBEE services on Fly.io | gHashTag/vibee-gleam | TypeScript, JavaScript, Gleam | t27c gen-rust |
| 5 | #4547 | the Zig core | gHashTag/trinity, gHashTag/trios | Zig, C | t27c gen, gen-c |
| 6 | #4548 | trios Rust rings | gHashTag/trios, gHashTag/trios-railway | Rust | t27c gen-rust |
| 7 | #4549 | silicon | gHashTag/t27, gHashTag/trinity | Verilog | t27c gen-verilog |
| 8 | #4550 | interfaces | trinity, trios, the sites | HTML, CSS, Swift | undecided |

**Progress is measured, not declared.** `apps/website/scripts/roadmap-stack.mjs` in
gHashTag/trinity counts the bytes of every hand-written language in every repository
and publishes them at `t27.ai/roadmap/stack.json`, drawn by the ROADMAP tab. A stage
closes when its languages are gone from that count, or what remains is a written
exception.

**How one file moves.** A task names ONE original and ONE target,
`specs/port/<repo>/<path>.t27`. You write that `.t27`. The Queen's review runs the
task's commands exactly as written, compiles the generated code and runs your tests;
the publisher then opens a pull request from your branch. Swapping the generated code
in for the original, in its own repository, is a later step and not yours: the
original is never edited by a port.

**What a lost turn looks like** - each of these was measured on the swarm, and each
costs the task: it is held, and later filed again under a new number.

- **An empty branch.** Nothing written, or everything written somewhere else. A
  partial port that parses, with a few passing tests and the missing functions named
  in your report, is worth more than nothing.
- **Files outside the boundary.** `debug_output.txt`, `stdout.txt`, `test.t27`, a copy
  of the original, an edit to another spec: each is reported and the work is not
  published. Scratch goes under `/tmp`.
- **Another language inside a `.t27`.** Of the 35 files under `specs/port/tools/` on
  2026-09-27, 3 built: 7 did not parse and 25 generated code that did not compile,
  mostly because their bodies were Zig or Rust pasted in. `spec-status` said
  IMPLEMENTED for 24 of them. Only `test-report` is proof.
- **Looking for the original on disk.** If it is in another repository, it is not in
  your checkout. The task quotes it in full.

## How to rewrite code into .t27

### Port the decision, not the plumbing

The ports that build share one design (`specs/port/tools/check_fix_carries_source.t27`,
6 of 6 tests pass). The logic that DECIDES becomes pure `.t27` functions. Everything
that touches the world - files, network, subprocess, database, HTTP framework, clock,
randomness, environment - is either a parameter the caller passes in, or a declared
function with an `undefined;` body that no test reaches. That is not a shortcut: the
decision is the part that can be generated and tested, and it is the part that was
worth porting.

| In the original | In `.t27` |
|---|---|
| exit codes, thrown errors, `Result`, `Err` | `const EXIT_FAIL: i32 = 1;` and a verdict function returning `i32`; or `struct { ok: bool, value: T }`; or `?T` with `null` |
| a dict, an object, a JSON record | a named `struct` with typed fields - only the facts the logic reads |
| a list that grows (`append`, `push`) | a fixed-capacity `struct { items: [N]T, len: usize }`, or a count |
| a list of strings as a parameter | `lines: []str`; a test passes `&lines` where `var lines: [2][]const u8 = ["a", ""];` |
| a regex match, a parsed file, a subprocess result | the ANSWER as a `bool` or a count parameter |
| `print`, logging | return a marker string or a code; the caller prints |
| `argv`, `main()` | a verdict function taking `argc: u32` and the flags |
| `async`/`await`, promises, callbacks | the synchronous decision; waiting is the caller's job |
| a class with methods | a `struct` plus free functions `Type_method(self: Type, ...)` |
| a union or enum type, Rust `match` | `enum` and `switch` with `.Variant =>` arms, or `if`/`else` |
| a Verilog `module` | `module Name { ... }` in `.t27`; `t27c gen-verilog` emits `module Name (` |

Keep every name the task lists: the name is how the port is checked. Keep the
behaviour: a test asserts what the original returns for inputs you choose. When a
construct cannot be expressed, port the rest, and name the missing piece and its line
in your report rather than faking it.

### What compiles today

Verified with `t27c test-report`:

- functions: `fn f(x: i64) -> i64 { ... }` (`->`, `→` or no arrow); `pub fn` to export
- `const`, `var`, `let` (same as `const`), `x += 1`, casts `n as i64`
- `if (c) { } else if (c) { } else { }`, also as an expression: `const w = if (x < 0) "neg" else "pos";`
- `while (i < n) { i += 1; }`, `break`, `continue`, `for x in xs { }`, `for (xs) |x| { }`
- `for i in 0..n` only when `n` is `usize`; over an `i32` it does not compile
- `switch (c) { .Red => 1, .Green => 2, else => 3, }` - arms are `.Variant`, a number, a char, or `else`
- integers `u8`..`u64`, `usize`, `i8`..`i64`, `f32`, `f64`, `bool`; `0xFF`, `0b101`, `1_000`
- strings: `str` (becomes `[]const u8`); escapes are only `\n \t \\ \"`
- `==` on strings when one side is a literal, a string parameter or a string struct field
- arrays `[4]u32 = [2, 3, 5, 7]`, slices `[]const u32`, `.len`, `[_]u32{0} ** 16`
- `struct Point { x: i32, y: i32, }` and `Point { x: 3, y: -4 }`; `enum Color { Red, Green, }`
- `?T` with `null`; tuple returns `fn divmod(a: u32, b: u32) -> (u32, u32)` and `const (q, r) = divmod(7, 2);`
- tests: `test name { assert(f(1) == 2); }`, `assert_eq(a, b)`; `invariant name { ... }` is checked at compile time
- comments: `//`, `/* */`, `#`
- a module line: `module port::name;` or `module Name { ... }`

### What does not, and what you see instead

- generics (`<T>`, `comptime T`), closures, traits, `impl`: parse error or wrong code
- `Result`, `Ok`, `Err`, `Option`, `Some`, `unwrap`, `!T`, `error.X`, `try`, `catch`: undeclared identifier or parse error
- `print(...)`, `println!`, `format!`, `console.log`: undeclared; `std.debug.print` compiles in Zig only and breaks the C and Rust outputs
- string concatenation (`"a" + b`), f-strings, template literals: does not compile
- growable lists (`ArrayList`, `.push`, `[]` literals that grow), maps: do not compile; no map or list library in `specs/` builds today
- `match`: parses, and is then DROPPED - the test fails with no error; use `switch`
- `"\r"` in a string is not a carriage return; the test fails; use the char `'\r'`
- `fn f() -> T;` with no body: parse error; write `{ undefined; }`, and never call it from a test
- `use a::b::fn_name;` does not resolve; `use a::b;` (the file `specs/a/b.t27`) does. Prefer not to import: say in your report what you would reuse
- a local string compared with `==` to a `const` string: does not compile; compare with a literal

### A complete port that passes

```
module port::clamp_count;

pub fn clamp(x: i64, lo: i64, hi: i64) -> i64 {
    if (x < lo) {
        return lo;
    }
    if (x > hi) {
        return hi;
    }
    return x;
}

// Python's `line.strip()` is falsy exactly when every character is whitespace.
pub fn is_blank(line: []const u8) -> bool {
    for c in line {
        if (c != ' ' and c != '\t' and c != '\r' and c != '\n') {
            return false;
        }
    }
    return true;
}

pub fn count_nonempty(lines: []str) -> usize {
    var n: usize = 0;
    for line in lines {
        if (!is_blank(line)) {
            n += 1;
        }
    }
    return n;
}

test clamp_edges {
    assert(clamp(-3, 0, 10) == 0);
    assert(clamp(5, 0, 10) == 5);
    assert(clamp(42, 0, 10) == 10);
}

test is_blank_cases {
    assert(is_blank(" \t\n"));
    assert(!is_blank("  x "));
}

test count_nonempty_mixed {
    var lines: [5][]const u8 = ["a", "", "  ", "b\n", "\t"];
    assert(count_nonempty(&lines) == 2);
}
```

It ports `def clamp(x, lo, hi)` and `def count_nonempty(lines)` and passes
`t27c test-report` with every test green.

### Before you report

1. `t27c parse <spec> > /dev/null && echo parses` - it prints the whole tree on success.
2. `t27c test-report <spec>` - read the words, not the exit code, which is 0 either
   way: every test `pass`, nothing `FAIL`, no `BLOCKED`.
3. `t27c coverage <spec>` - every function has a test.
4. `git status --short` - one line: the file in the Boundary.
