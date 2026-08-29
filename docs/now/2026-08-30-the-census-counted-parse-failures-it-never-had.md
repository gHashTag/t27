# NOW -- The census counted parse failures it never had (2026-08-30)

## The census counted parse failures it never had (Refs #2864)

- Of 97 specs t27c refused, only 79 fail at PARSE. Thirteen parse perfectly and fail TYPE CHECKING, four die in the lexer on an unterminated string, one is a semantic refusal. Reading a construct off the failing line of a type error is nonsense: gf8 stops at 'exp = exp + 1;', which compiles, rejected as 'cannot assign F64 to F32'.
- The stage split is checked both ways: no typecheck output contains a parse word, no parse output contains 'Typecheck'.
- Five constructs added, each with probe and counter: anonymous array literal, ';;', a stray closing brace, 'pub use NAME;', an anonymous fn literal. Blind spot 32 -> 10, named-and-probed 43 -> 53, 16 of 16 counters compile.
- A prose row was built, measured and REMOVED. Prose acceptance is position-dependent -- sdk_contract.t27 parses while carrying a paragraph after a body-less signature -- so no line-level rule is faithful. The loosest fired on 8925 lines inside parsing specs; the tightest still fired on 42 and lost 2 of 5 real cases.
- A switch-prong row was NOT added: the form copied verbatim from a parsing spec is rejected in isolation, so no honest probe exists. The census stays silent rather than guessing.
