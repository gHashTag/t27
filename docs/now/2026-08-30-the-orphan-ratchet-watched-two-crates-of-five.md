# NOW -- The orphan ratchet watched two crates of five (2026-08-30)

## The orphan ratchet watched two crates of five (Refs #2900)

- `tri mods orphan` had `let crates = ["bootstrap", "cli/tri"]` while Cargo.toml names five members. It printed their sum -- `7 of 132 files` -- as the repository's Rust population; the real number is 136.
- The guard directly below that list refuses a crate that has been REMOVED -- 'the crate list in this command is stale, and a report of zero orphans would be that staleness' -- and there was none for one being ADDED. A guard written as a list goes stale by addition.
- Sharper: the gate's own message reads 'A crate the ledger does not name is a crate this gate does not watch', and it could only ever fire for crates already in the hardcoded list. The message named the failure mode it could not detect.
- The list is now read from Cargo.toml `members`. Census covers 5 crates, 136 files. Measured first: the three newly covered crates hold zero orphans, so this lands green rather than red-on-arrival.
- `src/bin/*.rs` is a root: cargo discovers binary targets by layout, with no `mod` line. Without that rule cli/dlc10/src/bin/dlc10.rs is a false orphan -- demonstrated by mutation.
- Historical control: the old binary reports `7 of 132`, ORPHAN CEILING CLEAN, exit 0 with a stranded file planted in cli/flash-spi. The new one exits 1 and names it.
- Self-criticism: my first test for this passed under its own mutation -- it exercised the members() helper, not the caller reading it, which is the exact class this change is about. Replaced by a differential test comparing Cargo.toml against the ceiling ledger; it fails in both directions.
