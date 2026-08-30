# NOW -- Zig has no empty statement (2026-08-31)

## Zig has no empty statement (Closes #2974)

- a statement that lowered to nothing became a bare ';', which is not a Zig statement -- one such line killed the whole file and every check in it; 491 lines across 81 specs
- two source forms, now labelled apart: 475 childless (bench prose) and 16 with a child that rendered to nothing (defer in a test body); 475+16 = 491, one notice per semicolon removed
- zig test --test-no-exec 165 -> 190 (+25, 0 regressions), build-obj 282 -> 308 (+26, 0); +25 is exactly what an adversarial sweep attributed to this line before the code existed
- M3 and M4 survived the first mutation run: the 'keeps its semicolon' test used an assignment, which reaches StmtAssign and never touches the arm; and the invented bench fixture did not parse at all
