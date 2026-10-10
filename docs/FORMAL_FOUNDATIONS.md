# Formal Foundations

Record of the formal-verification gate campaign. Each numbered proposition
states what a coverage gate guarantees, what witnessed it, and what the wave
found. Propositions cited below for context (Props. 80, 83, 89b, 82d, 98,
111, 122a, 123) were recorded in the campaign's wave reports and issues;
this file carries the record forward starting at Prop. 124 (issue #2094).

---

## Prop. 124: Gates name their subjects (2026-10-08)

### Motivation

Prop. 123 showed a gate passing its coverage floor while missing exactly the
artifact it was written for: `compared > 0` held on twenty other connections
while the `dma_controller` instantiation went unparsed. A floor on a total
says nothing about coverage of the thing you care about.

### Finding 1: three gates now name their subject

Each gate now carries a witness -- a named artifact whose presence is
required for the gate to pass -- so a gate can no longer pass while its
subject escapes it:

| gate | witness | why that one |
|---|---|---|
| `units_scan` | the `dma_controller` instantiation is parsed | Prop. 122a's connection |
| `width_scan` | `l2` is among the declarations examined | Prop. 80's defect site |
| `bound_scan` | `accumulator` is among the registers classified | Prop. 83's register |

All three verified by renaming the subject in a scratch copy: 3/3 fire.

### Finding 2: widening the units vocabulary was mostly a negative result

Enumerating the 141 skipped connections was supposed to reveal unchecked
quantities. It revealed `clk`, `rst_n`, `rd_data`, `wr_en`, `a`, `b`, `sum`,
`cin`, `cout` and AXI handshakes -- not quantities at all.

An earlier report framed this as "the vocabulary covers 14%", implying 86%
of quantities were unchecked. The truth is that most connections are not
quantities and are correctly skipped. One family was genuinely missing --
addresses -- and adding it took the compared count from 23 to 42 with 0 new
disagreements.

### Finding 3: two of the tests were wrong before either gate was

- `width_scan`: the witness appeared not to fire because its reduction floor
  caught the mutation first -- the gate failed correctly, by a different
  guard, and the test demanded one specific message.
- `bound_scan`: the witness appeared silent because the mutation renamed only
  the register's declaration, while that gate identifies registers from
  assignments -- the mutation never removed the subject from its view.

In both cases the instrument was right and the check of it was wrong -- the
mirror of Prop. 89b.

### Finding 4: an edit that silently did nothing

The `width_scan` witness was first inserted with `str.replace()` on an anchor
that did not match, with no assertion on the count -- so `names_seen` stayed
empty and the witness fired against the shipped tree.

The campaign has written down "assert your injection landed" three times
(Props. 82d, 98, 111); it was violated in the wave that cites it. Re-applied
with `assert s.count(old) == 1`.

### Gate status

All fourteen gates green. units_scan: 42 compared, 1 known-open, 0 new.
doc_gate: 124/124. claims_check: 9 claims, 0 stale.
