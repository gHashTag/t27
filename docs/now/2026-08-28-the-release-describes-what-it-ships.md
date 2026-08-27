# NOW -- The release describes what it ships (2026-08-28)

## The release describes what it ships (Refs #2161)

- Refs #2161. Eight more t27c fixes landed after the 0.2.0 tag was cut, and the release is still an unpublished draft -- so they belong in its notes rather than in a follow-up version. CHANGELOG updated, tag re-pointed at the head that actually contains them
- Final state: version 0.2.0, health OK across six stages, 620 of 746 specs parse, 1629 unit tests pass with 6 pre-existing failures, 7 behavioural backend tests pass, RATCHET CLEAN
- Publishing stays with the owner: the release pipeline runs npm publish --access public and cargo publish on the `release: published` event. A draft does not fire it. Creating the draft is preparation; pushing packages to public registries under their name is their decision
