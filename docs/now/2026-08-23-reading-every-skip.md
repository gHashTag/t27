# NOW — reading every `skip` as the sentence it makes (2026-08-23)

Yesterday's rule was mechanical: **every `skip` claims that the thing missing is not the subject of this check.** Fourteen calls in the tree; read each as that sentence and two do not survive.

- Both are a **spec file tracked in git**. Measured, not argued: renaming the file aside makes both gates exit **0** without a flag. A deleted spec is not a bare machine — it is the repository missing the thing the gate exists to verify.
- Both now call `broken()`: fatal with or without `--require`. **`--require` should not be what saves you from a deleted source file.**
- **The extraction, at the one safe moment.** Four hand-copied `skip()`s had just been made to behave identically, which is the only time a deduplication is a pure deduplication. `tools/_prereq.py` now holds both words, with the rule that separates them written where the code is rather than in a comment inside one of four copies — which is how the four came to disagree.

**The extraction broke a control on its first run, and the control said so.** `verify_multitarget`'s planted tree carried the script and not the module it now imports, so the child died at import with empty stdout — and the case refused to read that as a skip. A plant must carry everything the thing under test needs; adding a shared dependency changes what "everything" means.

Two assertions also moved: messages now name the script from `argv` rather than a constant. A control asserting exact text is a control that notices a message changing — which is the point, even when the change is mine.

All five self-checks green; the three cheap gates still exit 0 on a healthy tree.
