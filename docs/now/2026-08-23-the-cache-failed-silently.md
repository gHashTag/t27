# NOW — the cache failed silently, three ways, in six lines (2026-08-23)

A full re-measurement re-measured gates whose hashes **matched entries already in the cache**, and the entry count climbed 30 → 40 → 80 *during* that run. It was rebuilding a cache it should have loaded.

Reading the six lines that load and save it found three silent failure paths:

```
read_to_string(..).ok()          unreadable file  -> empty map
serde_json::from_str(..).ok()    corrupt file     -> empty map
let _ = fs::write(..)            failed write     -> next run starts empty
```

**Each degrades to "no data", which is indistinguishable from "nothing measured yet".** A corrupt cache looked exactly like a first run — the same shape as a gate that cannot fail, one layer down in the tooling that measures gates.

**The cause was mine, repeatedly.** `fs::write` truncates in place, and I killed `--all` runs on a ten-minute timeout three times. Each kill landed some chance between the truncate and the write and left half a JSON document, which the next run swallowed.

Fixed both halves: **write-then-rename**, atomic on one filesystem, so the file is either the old complete document or the new one; and every path now says what happened — a missing file is a first run and stays silent, an unreadable or unparseable one prints what it found and why it is re-measuring.

Verified on a planted repository in three states: fresh measures, a repeat says `[cached]`, and a **deliberately truncated** cache prints the warning and re-measures. The truncation was done by cutting the file to 40 bytes — what a killed run does by accident.

**The rule:** `.ok()` on a read and `let _ =` on a write are the two commonest ways a tool loses data without saying so. Neither is wrong where the absence is expected — but the *present and broken* case has to be told apart from the *absent* one, and only the code that opens the file can tell them apart.
