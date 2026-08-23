# NOW — the last file where both findings met (2026-08-23)

`run_conformance_vvp.py` carried two open findings at once: **no negative control in any form**, and two unguarded external calls. It was invisible to the sweep for the whole campaign because it is named `run_*` rather than `check_*` — the naming proxy again, in the file where it cost the most.

- **Four verdicts covered end to end**, one per branch reachable without a simulator: no arguments is usage and not a pass; a module outside the registry is refused; unbuildable RTL is a build failure; and a **missing simulator is named**, which before this commit was a `FileNotFoundError` — red, with nothing said about whether the RTL was wrong or the tool was absent.
- Three of its exits are `2` and two are `1`, so the code alone cannot say which branch spoke. Every case asserts the message and names its siblings as forbidden.
- **The build-failure case needs the simulator PRESENT**, and says so: without that guard it would silently become a second copy of the missing-simulator case, and two cases measuring one branch read as two branches covered. When the tool is absent the case reports UNRUN and **fails** the control rather than skipping.

**The surviving mutant matched the written declaration for the third consecutive time.** One survivor, at the `NOTHING WAS EXECUTED` branch — named in the docstring as needing a planted corpus. Three gates in a row where what I wrote as uncovered and what the tool measured as uncovered were the same line. That agreement is worth more than either alone: the declaration is checked by something that cannot read it.

Gates with no control in any form: **1** — down from 3 when property-based selection exposed them, and from 4 of 12 when the campaign began.
