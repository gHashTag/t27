# NOW — I built a command that already existed (2026-08-24)

`t27c parse-complete --show <spec>` prints every discarded token grouped by line, with the source line beside it. It has been there since W634. I did not find it, and built `parse-accounted --spans` instead — a worse version of the same thing, over two pull requests.

- **The existing output is better than mine.** It shows the line of source next to what was dropped:

  ```
  372|         invariant total > 0;
      dropped: > 0 ;
  ```

  Mine printed the tokens alone, and capped the list at 40 lines until I noticed and fixed that too — a cap the existing command never had.

- **And `parse-complete` with no arguments is the corpus census I hand-rolled in Python**, reporting the same figures to the token: 650 scanned, 430 consume all, 66 DISCARD (26,546 tokens), 154 do not parse.

- **Folded.** `parse-accounted` is removed. Its one genuinely new capability, `--bisect`, moves to `parse-complete --bisect <spec>`: `--show` says WHAT was dropped, `--bisect` says WHICH top-level construct the parser stops on. Two commands answering one question is the same defect as the two seal-naming conventions found three days ago — the second one is where the drift lives.

- **The rule I wrote and then broke twice.** §14 of the ci-gates skill says to look for the right sample in the tree before inventing a coarser one. In two days I reinvented `parse_ast_dropped_spans` (which the compiler already had) and then `parse-complete --show` (which the CLI already had). Grepping `--help` for the noun in my own commit message would have caught both.

- **A comment that drifted.** `compiler.rs:1208` says the counter covered *"one of four discard channels"*. There are five increment sites, in `skip_brace_body`, `skip_to_semicolon`, `skip_to_next_top_level`, `parse` and `recover_to_stmt_boundary`. The commit that added the fourth is named *"[GOLD-RING] 0008: the fourth channel"*, and a fifth arrived the same week. Filed rather than edited: `compiler.rs` is under the stage0 freeze and a comment is not worth spending it.

- **`dropped_spans` is capped at 20,000 in all five sites.** No file reaches it today — the largest is 1,813 — so nothing measured here is affected. Recorded because a census that grows into that cap would silently stop counting, and the count and the spans would then disagree without either being wrong.

Refs #2479
