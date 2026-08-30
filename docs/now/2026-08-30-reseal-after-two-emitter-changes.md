# NOW -- Reseal after two emitter changes (2026-08-30)

## Reseal after two emitter changes (Refs #2929)

- `*T` parameters now reach C as `T*` (#2929) and a constant must fit its declared type (#2925); both move generated output, so 149 seals drifted
- read the acceptance columns first, as the gate instructs: `corpus-ratchet` is GREEN on master and `cc accepts` is unchanged at 174, so nothing worsened
- 73 seal files written, 445 twinned specs already consistent; after: 1318 seals, 1224 hold, 94 known-broken already baselined
- this is the second time in two passes that my own emitter change left master red on `seal-coverage`; the repair is one command and belongs in the SAME pull request as the emitter change
