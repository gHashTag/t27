# NOW -- The compiler knew and said nothing (2026-08-29)

## Two silences on the import path, both now audible (Refs #2764)

- `resolve` already explains every refusal in a comment; codegen strips comments, so `cc` says `unknown type name 'Trit'` and the cause never leaves the compiler
- printed to stderr from all three gen commands; stdout is still exactly the generated code, so nothing downstream changes
- 14 of the 460 specs that import something now name what they could not resolve and where it was declared twice
- the second silence is worse: when the spliced source does not compile, the command falls back to the UNRESOLVED original and succeeds -- every import discarded, no signal at all
- exactly one spec does that, `specs/nn/hslm.t27`, on all three backends -- and it is the one spec whose `Trit` errors did not improve when splicing started working
- so my first explanation for hslm (blocked by the malformed typedef in #2830) was wrong: the splice never reached codegen there
- `ambiguous` was never deduped, so a name re-entering the frontier printed its refusal twice
- a prefix match on `// UNRESOLVED ` also matches prose; the refusal has two halves and the filter must check both
