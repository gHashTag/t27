# NOW -- The third call-site was the biggest (2026-08-30)

## The third call-site was the biggest (Closes #2931)

- `cc accepts` **174 -> 242**, `ALL FOUR accept` **69 -> 115** -- the largest single move in this loop
- one template, three cc messages: undeclared call, `incomplete type 'void'`, `expected expression`; a census grouping by message text called it worth zero
- the class was closed in Zig (W585) and Verilog (W660) and never enumerated at the third site; W660's own comment names its sibling and stops
- Verilog's bare `0` does not port: C bindings carry no declared width, so a plain `0` traded 86 undeclared-call errors for 68 int-to-pointer ones and moved the accept count by ZERO
- the type comes from the consumer's declared parameter, the way Zig recovers it
- `-> void` is written two ways and I checked only one: an omitted return type leaves no ledger entry, an explicit `-> void` leaves one saying "void"; checking only for absence left 80 specs failing
