# NOW -- Fifteen type names, one definition each (2026-10-02)

## Give each newer definition its own name (Refs #5497)

- `tri types ratchet` failed on master: 15 type names gained a second (or third) definition, 14 of them from recent ports under `specs/port/`, and `ConfigEntry` because #4794 made `specs/account/auth.t27` readable. A name with two definitions has no determined domain (#2774).
- Rename the newer definition in each pair inside its own file, as #4612 asked for `Regex`, instead of blessing the ledger: `CubicBezier`, `OrgConfigEntry`, `Dup2CounterState`, `MetricGate`, `NotebookIssue`, `ClassifierShiftRegister`, `UartEchoInterface`, `D6TestInterface`, `LinkRelayMpsse`, `MpsseContext`, `UartOscillatorState`, `W365Path`, `W368Path`, `RectPoint`, `RegistryPool`, `VerdictPullRequest`, `SimpleRSAKeyPair`, `ServerStatusResponse`.
- Proof that nothing else moved: for every file, all four backends generate the same output before and after, byte for byte, once the new name is mapped back to the old one (Verilog also lowercases it into register names). `queen-ci-verdict.t27` was refused by every backend before and after.
- Drop the three rows that no longer conflict (`AdamWConfig`, `JitCache`, `ParseResult`) from `docs/reports/type_conflicts.json`, `type_conflicts_classified.json` and the tables in `docs/TYPE_CONFLICTS.md`. `tri types ratchet`: CLEAN at 77; `tri types classified`: OK. Reseal the two seal files of `specs/account/auth.t27` (current before, current after).
- Renaming `OscillatorState` in uart_echo_top.t27 makes its `update_oscillator_chain` body name a different type, so that duplicate group is gone; the duplicate-body ledger drops its row (`update_oscillator_chain 2`) in the same change, as that ratchet requires.
