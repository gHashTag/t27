# NOW -- Zig-subset campaign, wave 2: four more gaps, seventeen specs total (2026-08-22)

## Zig-subset campaign, wave 2: four more gaps, seventeen specs total (Closes #2426)

- Keyword-named enum MEMBERS, nested const X = enum inside a struct body, payload capture in if, and Zig's while continue expression. compiler/codegen/verilog/codegen.t27 joins lexer.t27 as generating.
- 541 generate to 558, ledger 171 to 154. The nested-type gap is the third appearance of one shape: a nested brace group ending the construct that contains it, after W577 for methods and the anonymous return type in wave 1.
