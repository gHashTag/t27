# A fix( in a source scope can merge with no source file

**2026-09-05** — Refs #3278

#3264 was titled `fix(rust): an untyped local bound to a comparison is not a bool (+3)`
and merged carrying **only this directory's note**. The edit lived in the working tree;
a `git reset --hard`, taken to get an honest baseline, destroyed it before the commit.
`git log --all -S expr_is_bool_syntactically` answered nothing, four merges later.

Every control that pass asked about the **binary**, and the binary was right — it had been
built from the working tree. None asked about the **commit**.

The population decides the rule. Any `fix(` with no source file names 12 commits on
master and 11 are correct to land that way: `fix(seals)` reseals JSON, `fix(freeze)`
rewrites one hash, `fix(paper)` edits a manuscript. Narrowed to scopes that name the
compiler — rust, c, zig, verilog, parser, compiler, lexer, typecheck — it names
**1 of 100**, and that one is the defect.

Verified on the pair: 49e5fff28 (the false claim) is refused; 0d7b13a4e (the restore that
carried it) passes.
