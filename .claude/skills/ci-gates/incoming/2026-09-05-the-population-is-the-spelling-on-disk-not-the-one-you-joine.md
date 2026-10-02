## the population is the spelling on disk, not the one you joined

`skill_files()` built its population by joining a name:

```rust
let p = e.path().join("SKILL.md");
if p.is_file() { out.push(p); }
```

Five skill files are tracked in this repository and **two are spelled
`skill.md`**. On Linux that reads 3 of 5. On a case-insensitive filesystem it is
worse than missing them: `is_file()` returns true, and the path pushed is one
**git has never heard of** — so `git show origin/master:<path>` fails and the
file reads as newly added rather than as tracked.

The fix is to take the name that is there rather than assert one:

```rust
for f in std::fs::read_dir(e.path())?.flatten() {
    if f.file_name().eq_ignore_ascii_case("SKILL.md") { out.push(f.path()); break; }
}
```

Both lowercase files carry **zero** numbered headings, so the missing population
was empty and no past check gave a wrong answer. That is luck, not design: the
first numbered heading added to either would have been unguarded, and the gate
would have kept printing a clean result over it.

Two habits fall out. **Enumerate, do not construct** — read the directory and
filter, rather than composing a path and testing it. And when a population turns
out to have been short, **say whether the missing part was empty**, because "the
gate read 3 of 5" and "the gate was wrong" are different findings and only one
of them is true here.
