# NOW -- one-away overstated its own bucket (2026-09-06)

## one-away overstated its own bucket (Refs #3359)

- The command reported 56 specs as carrying exactly one error. Taking one of them --
  `server/http.t27`, sole error `expected expression, found keyword \`fn\`` -- and
  repairing it revealed `expected expression, found \`@\`` underneath. The spec was never
  one repair away.
- rustc abandons a file at the first PARSE error. A diagnostic with an `[E####]` code came
  from a pass that ran to completion; one without a code did not. So for a spec whose sole
  error has no code, "one error" is a LOWER BOUND, not a count.
- Measured on today's master: of **50** specs reported as carrying one error, **24** are
  exact and **26** are lower bounds. The honest headline is 24, not 50.
- Caught before the command merged, which is the only reason it will not ship saying the
  wrong thing. Two more controls and one more unit test assert the distinction, so it
  cannot come back silently.
- The rule is mechanical and needs no list: the presence of an error code IS the answer to
  "did the compiler keep going".
