# NOW -- An inline `#` comment on a field deletes the next field, in every backend (2026-09-05)

Found while resolving a merge conflict, of all places: a seal file named
`agent_"[]const u8".json`. A seal is named after a module, so a seal named after a type
expression meant something upstream had read one as the other.

## The defect (Refs #3243)

- an inline `#` comment on a struct field consumes the rest of the line AND the declaration that follows it
- five lines reproduce it, and `parse` and `typecheck` both ACCEPT the result
- all three backends agree and all three are wrong the same way: `a : Float  # note,` followed by `b : String,` yields the single field `pub a: Floatb:String,`, and field `b` is gone
- `specs/tri/pipeline/pipeline_parallel.t27` loses **four** declarations to one comment: `pub id: U8command:Stringargs:Stringgroup_id:U8status:JobStatus,`

## A 2x2 that killed my first hypothesis (Refs #3243)

- I thought this needed a non-primitive type name; it does not
- `u8` gives 2 fields, `u8  # note` gives **1**; `Float` gives 2, `Float  # note` gives **1**
- my first probe looked healthy only because the commented field was the LAST one, so there was nothing left to swallow -- the probe had no victim, not no defect

## The evidence was already on disk (Refs #3243)

- **11 seal files** under `.trinity/seals/` are named after wreckage rather than a module: `"[]const u8"`, `Str = "",`, `String  # phi, trinity, gematria, evolution, safety`
- each points at a spec that exists and already carries a correctly-named seal, so they are EXTRA seals minted from a parse artifact
- 223 of 1318 seal names are not plain identifiers, but most are legitimately hyphenated; the 11 are the ones carrying a character no naming scheme admits

## A new shape in `tri misread`, and a correction to how I named it (Refs #3243)

- `rust: pub a: Xb:Y,` -- a bare colon inside a type; a Rust type never has one, since a path spells its separator `::`
- I called it `RustSwallowedField` first, and the corpus answered: two of its eight hits are `pub env: Vec<str:str>`, a map type `[str: str]` the emitter cannot spell -- same footprint, different wreck
- the module's own header says to name a shape for what the OUTPUT shows, because the intent is exactly what was lost, and I broke that rule on the first attempt
- the census now reads 22 / 1 / 0 / 1 / **8**, with all five shapes firing on the control

## Not repaired here (Refs #3243)

- the fix is in the lexer or the field parser, further upstream than anything this pass has touched
- what `#` should mean inside a declaration is a language question rather than a mapping one, and the mapping questions are the ones this pass has been answering
