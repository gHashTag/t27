# NOW -- tri ci baseline ranks by failure history, not by trigger table (2026-08-22)

## tri ci baseline ranks by failure history, not by trigger table (Closes #2359)

- Correcting #2309. Its premise — a gate with no baseline can turn a PR red — is false for one of the four it reported. seal-staleness-warn is advisory by construction (every path ends in exit 0): 169 runs, 0 failures. Reporting it beside emit-bitexact (144 runs, 84 failures, no baseline, the one that actually blocked two merges) presented four findings that are not the same size.
- The signal was never the trigger table but whether the workflow has ever concluded failure. The command now measures that per hole, sorts loudest-first, and --strict alarms only on gates observed to fail. Fourth category added to the taxonomy: advisory by construction.
