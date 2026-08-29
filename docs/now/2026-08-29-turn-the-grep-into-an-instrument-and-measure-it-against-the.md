# NOW -- Turn the grep into an instrument, and measure it against the past (2026-08-29)

## Turn the grep into an instrument, and measure it against the past (Refs #2754)

- tri abandoned list: recovery sites whose own comment names a t27 construct -- the pattern behind four defects on this line
- validated on a520590ef, before any of the fixes: names 4 sites including fn name() given ... then ... verbatim
- two of four, honestly: the other two comments sit at the top of the clause loop and no window can attribute them without inventing an association
- backticks alone matched Rust and doc fixtures; requiring a t27 keyword took it from 5 hits (1 real) to 1 hit (1 real)
