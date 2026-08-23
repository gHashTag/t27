# NOW — zero, and a null result that answered the easier question (2026-08-23)

**21 gates, 0 with no control in any form.** The count that opened this campaign at 4 of 12 is at zero, across a set two-thirds larger and selected by property rather than by name.

- The last one carried sixteen `assert`s — XOR trains 4/4, held-out clears 90%, the emitted Verilog carries the ports it claims — and nothing showed any of them could go red. Three planted cases now do: a sign flip in the shared multiplier stops XOR converging, a renamed port trips the emitter assertion, and the clean tree stays green.
- **Building it corrected yesterday's null result.** That measured *"do any CI-invoked tools fail ONLY through assert?"* — zero, none invisible to the selector — and I reported it as if it settled the matter. The mutation operators score this gate **0/0, 0/0, 0/0**: its verdicts are asserts, which no operator recognises, so **every verdict here is invisible to three of the four questions.** Two different questions, and I answered the easier one.
- **The documented trap, reproduced.** The port-rename plant spelled its needle literally, so the first occurrence of that string became the control's **own source line**, and `str.replace(.., 1)` edited the harness instead of the target — reporting the gate as blind when nothing had been planted. A sibling gate carries a comment warning about exactly this. I had read it. The fix is to assemble the needle so it exists as no literal.

**Named and left:** the boundary column reads 5/31 — arithmetic internals where moving a comparison is a numerical change, not a verdict change. A different surface, and not this campaign's question.
