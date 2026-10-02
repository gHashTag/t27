# NOW -- gen-ts: a seventh backend, sharing the value layer it would otherwise have copied (2026-09-21)

## gen-ts: a seventh backend, sharing the value layer it would otherwise have copied (Closes #4501)

- t27c gen-ts emits a TypeScript module of a spec's declarations: satisfies rather than an annotation so literals survive, interface + const so one import is both the type and the runtime descriptor, a frozen object + union rather than TS enum so nothing in the artifact runs
- codegen_js became the shared value layer for both backends -- reserved words, escapes, array checks and the constant walk have one home, and the difference is a Target { tag, lang } argument rather than a second file
- parity over the corpus: ok=796 refused=154, parity-breaks=0; tsc --strict over all 796 emitted modules finds 9 errors in 7 specs, each reproduced by gen-js and therefore pre-existing
