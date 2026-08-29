# NOW -- A bare call was read as a clause head (2026-08-29)

## A bare call was read as a clause head (Refs #2754)

- deque_init(&data, &front) inside a braceless test block: an Ident, so it parsed as a CLAUSE HEAD and the block fell to the discard, taking the asserts after it
- the calls are what SET UP the state the assertions check -- without them the asserts could not run even if they survived
- discarded 32485 -> 30451 (-2034), cc 157 -> 158, everything else held; the ratchet built two hours earlier named all six improved specs by how much
