# NOW -- The compile-debt ledger described a compiler master had already moved past (2026-08-22)

## The compile-debt ledger described a compiler master had already moved past (Closes #2365)

- Bisect pins the behaviour change at da4d3a850 (2026-08-09, whose message says '0 regressions'). It is an ancestor of master and NOT of 9a757a3dd, the commit where the ledger was measured: that branch forked on 9 August, ran ten days on a compiler without the change, and brought its ledger with it. Verified by building t27c at both commits — seven specs generate on the 19 August compiler and on master's do not.
- Second defect, independent: git ls-files sweeps the parser's own negative fixtures into the compile-debt ledger — 12 damaged files, 7 malformed generic-const declarations, 2 EOF hazards. Recording those as debt inverts their meaning, since one of them generating would mean a parser bug shipped. Excluded from enumeration, and 124 entries the ledger held as broken leave it because master's compiler generates them.
