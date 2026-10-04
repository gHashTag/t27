# NOW -- t27c drops unused candle-core and candle-nn (2026-10-04)

## t27c drops unused candle-core and candle-nn (Closes #5899)

- `bootstrap/Cargo.toml` declared `candle-core` and `candle-nn`, but no file under `bootstrap/src` imports either crate, so every build of t27c compiled an ML stack nothing called.
- Both lines are removed. `Cargo.lock` goes from 432 to 358 packages (-74); no source file changes.
