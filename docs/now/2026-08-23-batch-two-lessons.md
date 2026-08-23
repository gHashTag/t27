# NOW -- what the second batch added to the ten classes (2026-08-23)

Refs #2325. 28 of 43 findings closed; `ci-gates` gains section 15.

- **Class H sharpened.** A fixture exclusion held 29 files under a rationale
  enumerating 21, and the eight extra were two OPPOSITE intents in one list --
  five parser controls meant to pass, three damaged inputs that had started
  generating. Split the list before fixing the count; the count was the symptom.
- **Class J sharpened.** Every conformance row carried a hex twin of each
  decimal it states, and nothing compared them. Measured first: 3795 rows,
  7590 pairs, all agreeing, so the check was free and total. A second
  instrument you have to build is a reason to look harder for one you have.
- **Class D sharpened.** After a gate was given git history, 15 ledger entries
  changed class. Nothing in the tree moved -- the answer improved. Suppress
  movement inside a pair the instrument concedes it cannot always separate,
  and say why in the code or the next reader deletes it as dead weight.
- **New habit: mutate every half of your own patch.** A three-part fix landed
  with two mutants; reverting the third left the selftest green because the
  fixture made that assertion unobservable. N independent parts need N mutants,
  and the one you are least worried about is the one most likely untested.
- **Discipline for controls.** Three failed this batch by exercising the wrong
  branch -- a pack id absent from the fixture, a tool run where its ROOT
  resolves to `/`, a fault the OLD branch already caught. Name the branch, and
  assert the neighbour's marker is ABSENT.
