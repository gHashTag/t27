# NOW — 26,546 tokens never reach codegen (2026-08-24)

Yesterday's `parse-accounted --bisect` named a construct by removing items one at a time. The compiler had already recorded the answer directly and nothing exposed it.

- **`parse_ast_dropped_spans` was there all along**, returning every discarded token as `(line, lexeme)`. `--spans` prints it. The bisection I wrote is the indirect answer to a question the compiler could already answer exactly — my own skill says to look for the right sample in the tree before inventing a coarser one, and I did not.

- **It refuted the issue I filed yesterday.** #2479 says the braceless `given`-style `test` block loses its body. For `power_analysis.t27` the discarded tokens are `> 0 ;` — `invariant total` **is** consumed and the parser stops at the operator. For the largest file it is not a `test` block at all:

  ```
  ternary_inference.t27: 1813 discarded, on 195 lines
    139: forall input : InferenceInput
    140: input . activations . len == 4 == >
    141: ternary_inference_identity input . outputs == input . activations
  ```

  Those are quantified properties. Attributing them to a test-block syntax would have sent a reader to the wrong part of the parser.

- **Corpus census.** 650 specs scanned, 154 do not parse at all (another phase's business), **66 discard tokens, 26,546 in total**. By the first dropped token on a line: `forall` 415, `:` 212, `assert` 71, `and` 67, `var` 54, `then` 52, `given` 50, `const` 49, `when` 35. By directory: `specs/igla/` 20,644, `specs/vsa/` 1,753, `specs/queen/` 870, `specs/memory/` 807, `specs/nn/` 721.

- **What that means, stated carefully.** These are top-level tokens the parser reached EOF without consuming. They are authored specification text that never reaches codegen, and the largest group is `forall`-quantified properties concentrated in `specs/igla/`. This is a count of tokens and lines, not a claim about how many distinct grammar rules are missing — that needs the parser read, and the parser is under the stage0 freeze.

- **Three over-generalisations killed in two days, all mine, all by measurement.** "Invariant inside a test is discarded" — a passing spec has fifteen. "It is the expression form" — seven forms behave identically. "It is the braceless test block" — the biggest contributor is a `forall` property. Each was plausible enough to write down, and two of them I did write down.

Refs #2474, #2479
