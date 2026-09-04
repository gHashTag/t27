# NOW -- The shared word for could-not-run had no code 2 (2026-09-04)

## The shared word for could-not-run had no code 2 (Closes #3126)

- tools/_prereq.py exists to split skip (the environment is incomplete) from broken (the product failed), and answered the first with 1 -- the same code as the second, and the one reserved for a check that RAN and said no
- It is imported by 15 files with 26 live skip call sites across 7 gates, so the wrong word was taught to every one of them; .githooks/pre-commit already branches on 2 to say nothing was examined
- skip under --require now exits 2 and says so; broken stays at 1; nothing regresses because any non-zero fails a step and neither reader of 2 calls this path
- Four mutations kill, including the control that catches a module exiting 2 for everything. Wider and unfixed: one condition answered 2, 1 or 0 by ten different gates
