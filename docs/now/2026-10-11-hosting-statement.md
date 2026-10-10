# NOW -- specs/hosting/statement.t27: the hosts' epoch statement, paid by kind, settlement shut (2026-10-11)

## specs/hosting/statement.t27 (Closes #8744; gHashTag/trios#1761)

- Each epoch (a day from 2026-10-10T00:00Z) the ledger closes once, in order, into a statement: one leaf per (host key, kind) with its mTRI and receipt ids, under an RFC 6962 root (corpus_receipt.t27's prefixes and split), chained by `prev` and signed with the Queen's Ed25519 key. A host checks its own leaf: signature, owner, path shape, root.
- Kinds come from network/mvp.t27 (`fee_kind_of`: the first agreeing receipt is the executor's, later ones verifiers'; ENTRY_SLASH for a disagreeing one) and payout.t27; a known-answer job pays verifiers only. Settlement is shut behind VALUE_GATE_OPEN, a spec constant, until counsel; nothing moves value and TON (tri-bridge.t27) is the chain of record. credit.t27's comments now say so.
- `t27c test-report`: 9 tests, 0 FAIL, 0 vacuous, 82 runtime asserts, 3 invariants. 11 negative controls: 10 FAIL by name, 1 BLOCKED by the invariant `nothing_moves_value`.
