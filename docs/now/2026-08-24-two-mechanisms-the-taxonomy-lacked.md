# NOW — two mechanisms the taxonomy lacked (2026-08-24)

The `number-audit` skill sorts twenty-two cases into four ways a true sentence goes false. I have been citing it all campaign without testing it. Testing it against this campaign's own failures found two that fit none of the four.

- **How it was tested, and why re-reading would not have worked.** Not by re-reading the four mechanisms and nodding, but by taking each failure this campaign produced and asking which mechanism it belongs to. Two had no home.

- **8.5 — the claim was TRUE and its meaning was not.** *"26,546 tokens of specification never reach codegen."* Every word measured. Two thirds of that total is `forall`-quantified statements the compiler documents, in the function that skips them, as *"not runtime-checkable"*. The actionable number was 9,547.

  It fits none of the four: it was checked against the thing it describes, the world had not moved, the instrument was honest, and the verification was right. **A wrong number is caught by anyone who re-runs the command; a right number with the wrong meaning survives re-running, because re-running reproduces it.**

- **8.6 — the observation was right and the REASON was invented.** A tool reported two gates as having "no success path to break". True. The explanation beside it — *"forcing a ternary's whole line to 1 is the silent operator seen backwards"* — was fabricated and wrong, and because it read as a considered distinction it stopped anyone measuring for a full day.

  *The check:* for every limitation you write down, name the experiment that would show it is not one, and run it. A reason you cannot falsify in one command is a hypothesis wearing a fact's clothes.

- **Why these are mechanisms and not variants.** The section's own criterion is that the fixes do not transfer. 8.4's remedy is to adversarially re-derive the accusation — and re-deriving 26,546 gives 26,546, re-deriving "two gates show no path" gives two gates. Both needed someone to read the code that produced the behaviour, which is a different action from checking the arithmetic.

- **The index was stale one line after the fix.** `MEMORY.md` said "4 механизма", and the skill's own frontmatter description said "Four failure mechanisms" — the description being the text loaded into context to decide whether the skill is relevant at all. Both corrected. That is mechanism 8.2 in miniature, found immediately after adding the section that names it.

Refs #2492
