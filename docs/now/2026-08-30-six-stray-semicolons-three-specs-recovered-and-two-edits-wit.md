# NOW -- Six stray semicolons, three specs recovered, and two edits withdrawn (2026-08-30)

## Six stray semicolons, three specs recovered, and two edits withdrawn (Refs #2864)

- The census's work queue had two rows needing no owner decision: a closing brace with nothing open (4 specs) and a statement terminated twice (2). Both probed: 'struct X { };' and '} else { };' and 'print(x);;' each fail, and each parses with the extra semicolon removed.
- The census names only the FIRST occurrence. Fixing every occurrence of the same form: pubsub 2, mac_tb 6, uart_tb 5 -- all three now parse. 615 -> 618 specs, zero regressions, 21 lines changed and every one is exactly one character shorter.
- Two edits WITHDRAWN. hybrid_arithmetic and relay_observer advanced but still do not parse, and the edit made their seals stale for no gain. Reverted rather than carried: an edit that buys nothing and costs a re-seal is not a repair.
- The corpus ratchet caught the three fixed specs immediately as UNEXPECTED PASS -- yesterday's lesson working. Entries removed, max_entries 150 -> 147. git grep over the three names found four more files that mention them; none is read by any gate, so they are snapshots, not ledgers.
