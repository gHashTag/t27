# NOW — sweeping for the missing-tool shape (2026-08-23)

Yesterday's gate raised `FileNotFoundError` when a compiler was absent — exit 1 with no verdict, so *"the tool is missing"* and *"the arithmetic is wrong"* left the same colour and the same silence. That is a shape, so it got swept for.

- **Mechanically: 17 invocations of an external tool by bare name, 11 with no `try/except OSError`.** Eleven is a raw number whose meaning is not established — the same trap as *54 tools*.
- **Five of the eleven have a `shutil.which` precondition**, which is a better design than catching the exception. Counting those as defects would have been the false-accusation direction.
- **Six have neither.** Three are `git` — nearly universal, named rather than fixed. The other three matter: one `cc`, and an `iverilog`/`vvp` pair inside a gate that also has **no control at all**, so two findings converge on one file.
- **The measurement disagreed with the count.** Running each with the tool stripped from PATH: one crashes as predicted, one reports the absence correctly (its `which` does dominate the call, which the static read could not show), and one exits 2 on usage before reaching the tool — the probe never tested what it meant to. **Eleven candidates, one confirmed crash.**

Fixed with the same shape as yesterday: catch, name the absence, and return the value the caller already treats as *uncompared* rather than *disagreed*. The gate now reaches its own "the extraction is broken, not the tree" verdict — the right class, which is the point.

The control case reuses the **agreeing** fixture, so the only thing wrong with that world is the missing tool, and `DIFFERENT behaviours` is named absent: reporting an absence as a disagreement is what the branch exists to prevent.

**The rule:** a static count of unguarded calls is a list of candidates, not findings. Run each in the world it describes — a `which` above the call, an argument check before it, or a wrapper you did not read all make the same static pattern harmless, and only running tells you which.
