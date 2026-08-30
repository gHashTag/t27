# NOW -- C needs a type before its use (2026-08-30)

## C needs a type before its use (Closes #2942)

- Zig and Rust require no ordering between declarations, so nothing upstream forces one; C requires the complete type at a by-value member
- two defects: struct-against-struct (source-order emission) and constant-against-struct (the Constants section precedes Structs)
- the sections cannot swap -- a `[T; N]` value struct may size itself from a const name -- so the constants SPLIT: primitive-typed stay, struct-typed move below
- specs with an `unknown type name` family **140 -> 115**; `cc accepts` **257 -> 264**
- a self-referencing struct is NOT fixed by any sort: that is a cycle of length one and needs a forward-declaration block
- my scan promised 36 and delivered 14: it matched any `} Name;` line, including typedefs the struct sort does not control
