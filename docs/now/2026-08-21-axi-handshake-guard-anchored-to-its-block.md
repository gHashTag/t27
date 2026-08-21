# AXI handshake guard re-anchored to its own block

The regression guard `axi_handshake_dropbacks_present` was loosened in #2331 and no longer
noticed the defect it names. Restored by anchoring the assertion inside the handshake block's
span. #2331's functional fix is untouched.

- `contains("s_axi_bvalid <= 1'b0;")` over the whole module is satisfied by the **reset block** —
  the emitter writes `s_axi_awready <= 1'b1; s_axi_wready <= 1'b1; s_axi_bvalid <= 1'b0;` there,
  and `axi_reset_initializes_all_outputs` lists that very string as a reset line. Same for
  `s_axi_rvalid <= 1'b0;`.
- Demonstrated, not argued: with both clears deleted from **inside** the handshake blocks in
  `bootstrap/src/bitnet_axi.rs` — an RTL defect that latches BVALID/RVALID high forever — the
  merged assertions passed 5/5 in the bin harness and 3/3 in the integration harness.
- Fixed by `stmt_in_block()`, which slices the block from its header to the `end` at the header's
  own indentation and searches only that slice. The reset block falls outside the span.
- `axi_clear_precedes_accept_no_deadlock` now orders the clear **statement** against the accept
  instead of the block **header**; the header form survived deletion of everything inside it.
- Re-run with the mutant still planted: both harnesses FAIL. B-channel and R-channel anchors
  proved to bite independently. Mutant reverted: green again.
- Not reverted to the pre-#2331 single-line string — the emitter now emits a multi-line
  `begin`/`end` block, so that string matches nothing.
