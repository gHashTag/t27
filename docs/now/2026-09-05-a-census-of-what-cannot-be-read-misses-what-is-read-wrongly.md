# NOW -- A census of what cannot be read misses what is read wrongly (2026-09-05)

Went looking for the 18 specs emitting an empty field type, expecting a front-end
question with two answers -- the parser loses the type, or the spec omits it. It was
neither, and the class it uncovered is invisible by construction.

## What every stage accepts (Refs #3225)

- a four-line spec with `bad : 0,` -- an integer literal in TYPE position -- is accepted by `parse` AND by `typecheck`
- the Rust backend writes `pub bad: 0,`, the C backend writes `0 bad;`, and the Zig backend drops the field entirely
- two backends emit something their language cannot parse; the third silently produces a type missing a member, which is worse because nothing downstream can notice
- **14 specs** emit an empty field type `pub f: ,`; **all 14** pass both `parse` and `typecheck`; only **3** appear in the debt ledger, for other reasons, so **11 are tracked by nothing at all**

## Where it comes from (Refs #3225)

- fourteen specs use a list-valued declaration the parser does not implement, `variants :` followed by `- success : 0,` items on later lines
- the intent is an enum with named discriminants; what the parser builds is a struct whose fields are `variants` with an empty type, then `success` with type `0`, `command_error` with type `1`
- recovery is what hides it: the items become fields, so the spec comes out the other side looking well-formed

## Why no instrument sees it (Refs #3225)

- `tri unparsed` ranks the constructs that stop the parser, each row backed by a live probe, and its population is defined by **the compiler refused**
- anything the compiler accepted is outside that census by construction, however wrong the result
- the cheapest detector for the second class is not a parser change but a shape check on the GENERATED output: `pub f: ,` and `0 bad;` are trivially greppable, and they find exactly the population no phase covers

## The proposal and its price, not shipped (Refs #3225)

- a typecheck rule refusing an empty field type catches all 14 and needs no judgement, since a field must have a type
- but it turns 14 passing specs into failures while `suite_expectations.json` sits at **152 entries against a cap of 152**, and `max_entries` re-blesses at `min(previous, observed)`, so absorbing them takes a deliberate hand edit raising the cap
- narrowing the rule to "literal in type position" is unambiguous but fires on **1 spec** with 8 fields -- a rule priced at almost nothing
- having the emitters refuse instead moves `generates` from 581 to 567, the same number moving down in a more visible place
- implementing the list form is a language feature, not a repair
