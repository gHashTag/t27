# NOW -- Build Paper was one character: a four-column table declared three (2026-09-06)

## Build Paper was one character: a four-column table declared three (Refs #3270)

- build-paper.yml has been red on master since 2026-04-14, never green. tri red why names the current cause as 'Build PDF from LaTeX'; the two runs before it failed with no step reported at all, so the cause had already changed once.
- docs/WHITEPAPER/latex/main.tex line 278 declares \begin{tabular}{lll} while the header and all four body rows carry FOUR columns. Counted each row: 4, 4, 4, 4, 4. One character.
- Found by fanning tri red why across every workflow red on master: 10 diagnosed, 5 FIX, 4 LEAVE, 1 RESTRICT_TRIGGER, and zero DELETE or DISABLE - no workflow should be removed.
