# NOW -- imported structs resolve, and mac's phantom TernaryWord is declared (2026-08-22)

## imported structs join struct_decls; mac.t27 declares the word it always assumed (Refs #2275)

- `word.raw` on a struct-typed fn param emitted the unbound identifier `word_raw`:
  the part-select branch consults `struct_decls`, which held only module-local
  structs. Mirroring the imported-enums pass, `use_resolve::imported_structs`
  now loads every direct dependency's struct decls and they merge into
  `struct_decls` without ever shadowing a local declaration. M5 performed.
- The deeper find: NO file in the corpus declares `TernaryWord{raw}` -- the
  `base::ternary_memory` struct of that name is a different shape
  (trits/state/checksum). mac.t27 referenced a phantom type and got the
  fallback width by coincidence. The spec now declares its own
  `struct TernaryWord { raw : u32 }`; `word.raw` lowers to `word[0 +: 32]`,
  parse and typecheck stay clean, the full 32-module smoke set lints 32/32.
- Still open in #2275: the `mac_units` array-of-structs state (nested array
  field) -- its DECLARATION emits `reg [31:0]` plus a "not yet lowered" TODO,
  so element-field access has nothing to bind to. That is an emitter feature,
  not a reference fix.
- Two verilog unit tests fail on clean master identically (keyword-escape
  local array, for-range) -- pre-existing, measured before blame.
