# NOW -- skill renumber deleted a section while its count guard passed (2026-09-05)

## skill renumber deleted a section while its count guard passed (Refs #3195)

- running the previous commit fix on its own branch destroyed one of that branch section: the section quotes three heading lines inside a fenced block as evidence, skillnum::sections counts every line starting with two hashes and a number whether fenced or not, and the rebuild dropped the real section while the quoted headings filled its seats
- the existing guard compared section COUNTS and passed, because three quoted headings went in and one real section came out. A total cannot see a substitution -- the same lesson as the dead test and the phantom test, one level up, in the tool rather than in a report
- titles_lost compares the SET of titles and refuses the write, naming what would go. Renumbering is invisible to it: every number changes and no title does. That is the guard every hand-written resolver in this loop already had and the shipped command did not
