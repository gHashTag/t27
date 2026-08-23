# NOW -- A tool that truncates its own list (2026-08-24)

## A tool that truncates its own list (Closes #2407)

- The corpus ratchet reported 27 unexpected passes and printed a shorter list. Removing every printed name left two, from the same 27; the summary count and the listing disagreed and I trusted the listing.
- Third time this shape has cost a wrong conclusion — an awk view six columns wide, a head on a diff, now a tool capping its own output. Compare the summary number to the number of lines you extracted; the gap is the finding.
- Section 80 gives the loop that fixes it, with two guards: stop if nothing is extractable rather than spinning, and stop if a NEW failure appears rather than ratcheting a real regression into a ledger unattended.
