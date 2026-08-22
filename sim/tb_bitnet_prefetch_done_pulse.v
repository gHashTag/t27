`timescale 1ns/1ps
// ===========================================================================
// tb_bitnet_prefetch_done_pulse -- differential harness for issue #1985
// ===========================================================================
// `weight_prefetch_ctrl` documents `prefetch_done` as a one-cycle pulse raised
// in DONE_ST. It was not one. The pre-fix emitter cleared the flag only INSIDE
// the start guard:
//
//     IDLE: if (start_prefetch) begin
//         state <= FETCH; prefetch_active <= 1'b1; prefetch_done <= 1'b0;
//         ...
//     end
//
// DONE_ST raises `prefetch_done` and drops straight back to IDLE, and nothing
// in IDLE lowers it again until the NEXT request arrives. So the flag is not a
// pulse at all -- it is a level that stays asserted for the whole idle gap,
// however long that gap happens to be.
//
// That matters because of who reads it. A requester that samples
// `prefetch_done` in the same cycle it raises `start_prefetch` -- which is
// exactly what the `multilayer_sequencer` WAIT_PF state does -- reads the
// PREVIOUS transaction's completion and concludes its own prefetch is already
// finished, skipping it.
//
// The fix retires the flag on entry to IDLE, unconditionally, ahead of the
// guard:
//
//     IDLE: begin
//         prefetch_done <= 1'b0;
//         if (start_prefetch) begin ... end
//     end
//
// This harness elaborates BOTH the pre-fix and the post-fix emitter output in
// one simulation, drives them from identical stimulus, and compares them. The
// two variants differ only in the `module_name` passed to the emitter, so the
// comparison is between two renderings of the same design, not two designs.
//
// Build (see sim/README.md for the emit step):
//   iverilog -g2005 -o tb.vvp sim/tb_bitnet_prefetch_done_pulse.v \
//            pf_old.v pf_new.v
//   vvp tb.vvp
//
// The measured property has two halves, and BOTH are asserted:
//
//   * `prefetch_done` must be observable as a pulse, not a level -- measured
//     by holding the idle gap at two different lengths and checking that the
//     number of cycles the flag stays high does not track the gap.
//   * NOTHING ELSE may change. Every other output is compared cycle by cycle
//     across the whole run and must be identical. Without that half, a
//     rendering that fixed the flag by breaking the fetch would pass.
// ===========================================================================

module tb_bitnet_prefetch_done_pulse;

  localparam integer WORDS = 2;   // BRAM writes per transaction

  reg clk = 0, rst_n = 0;
  always #5 clk = ~clk;

  integer errors = 0;

  reg         d_start = 0;
  reg  [31:0] d_src   = 32'h1000;
  reg  [15:0] d_words = WORDS;

  wire o_active, n_active, o_done, n_done;
  wire [31:0] o_araddr, n_araddr;
  wire o_arvalid, n_arvalid, o_rready, n_rready;
  wire [11:0] o_baddr, n_baddr;
  wire [53:0] o_bdata, n_bdata;
  wire o_bwe, n_bwe;

  // AXI read slave: address always accepted, data always available. Beat K
  // carries the value K, so a payload comparison is meaningful.
  reg [63:0] rbeat = 0;
  always @(posedge clk) if (!rst_n) rbeat <= 0;
                       else if (o_rready) rbeat <= rbeat + 1;

  pf_old u_old (
    .clk(clk), .rst_n(rst_n), .start_prefetch(d_start),
    .src_addr(d_src), .num_words(d_words),
    .prefetch_active(o_active), .prefetch_done(o_done),
    .axi_araddr(o_araddr), .axi_arvalid(o_arvalid), .axi_arready(1'b1),
    .axi_rdata(rbeat), .axi_rvalid(1'b1), .axi_rready(o_rready),
    .bram_addr(o_baddr), .bram_data(o_bdata), .bram_we(o_bwe)
  );

  pf_new u_new (
    .clk(clk), .rst_n(rst_n), .start_prefetch(d_start),
    .src_addr(d_src), .num_words(d_words),
    .prefetch_active(n_active), .prefetch_done(n_done),
    .axi_araddr(n_araddr), .axi_arvalid(n_arvalid), .axi_arready(1'b1),
    .axi_rdata(rbeat), .axi_rvalid(1'b1), .axi_rready(n_rready),
    .bram_addr(n_baddr), .bram_data(n_bdata), .bram_we(n_bwe)
  );

  // -----------------------------------------------------------------------
  // Observers, sampled on the NEGEDGE.
  //
  // Every output of this FSM is registered, so a posedge observer reads the
  // value from before that edge's non-blocking update and trails the design by
  // a cycle. The negedge reads the settled value of the cycle the design is
  // actually in. (A sibling harness in this directory reported a count one
  // short for exactly this reason.)
  // -----------------------------------------------------------------------
  integer o_we_count = 0, n_we_count = 0;
  integer o_done_rises = 0, n_done_rises = 0;
  integer o_done_high = 0, n_done_high = 0;
  reg     o_done_prev = 0, n_done_prev = 0;

  // What a requester sees when it samples the flag in the same cycle it raises
  // `start_prefetch`. Indexed by which request it is: t1 is the first (both
  // renderings must read 0 -- nothing has completed yet), t2 the second.
  //
  // These are captured inside `pulse_start`, in the stimulus process itself,
  // NOT by the negedge observer below. `d_start` is asserted on the negedge,
  // so an observer that also triggers on the negedge races the assignment and
  // may sample the cycle before the request. The first draft did exactly that
  // and reported t2=0 for both renderings -- an ordering artefact, not a
  // property of either. Sampling from the process that drives the signal
  // removes the race by construction.
  integer start_idx = 0;
  reg o_t1 = 1'bx, n_t1 = 1'bx, o_t2 = 1'bx, n_t2 = 1'bx;

  // Comparator over EVERY output except `prefetch_done`. `other_mismatch` must
  // stay 0: the fix is meant to be surgical. `done_differs` counts the cycles
  // on which the flag itself diverges and must be NON-zero, or the harness is
  // observing nothing.
  integer other_mismatch = 0, done_differs = 0, cmp_cycles = 0;

  always @(negedge clk) if (!rst_n) begin
    o_we_count = 0; n_we_count = 0;
    o_done_rises = 0; n_done_rises = 0;
    o_done_high = 0; n_done_high = 0;
    o_done_prev = 0; n_done_prev = 0;
    other_mismatch = 0; done_differs = 0; cmp_cycles = 0;
  end else begin
    if (o_bwe) o_we_count = o_we_count + 1;
    if (n_bwe) n_we_count = n_we_count + 1;
    if (o_done && !o_done_prev) o_done_rises = o_done_rises + 1;
    if (n_done && !n_done_prev) n_done_rises = n_done_rises + 1;
    if (o_done) o_done_high = o_done_high + 1;
    if (n_done) n_done_high = n_done_high + 1;
    o_done_prev = o_done;
    n_done_prev = n_done;

    cmp_cycles = cmp_cycles + 1;
    if (o_active  !== n_active  || o_araddr !== n_araddr ||
        o_arvalid !== n_arvalid || o_rready !== n_rready ||
        o_baddr   !== n_baddr   || o_bdata  !== n_bdata  ||
        o_bwe     !== n_bwe) begin
      if (other_mismatch == 0)
        $display("  FAIL a non-prefetch_done output diverged at compared cycle %0d: active %b/%b araddr %0h/%0h arvalid %b/%b rready %b/%b baddr %0d/%0d bdata %0h/%0h bwe %b/%b",
                 cmp_cycles, o_active, n_active, o_araddr, n_araddr,
                 o_arvalid, n_arvalid, o_rready, n_rready,
                 o_baddr, n_baddr, o_bdata, n_bdata, o_bwe, n_bwe);
      other_mismatch = other_mismatch + 1;
    end
    if (o_done !== n_done) done_differs = done_differs + 1;
  end

  task expect_eq(input [511:0] what, input integer got, input integer want);
    begin
      if (got !== want) begin
        errors = errors + 1;
        $display("  FAIL %0s: got %0d, want %0d", what, got, want);
      end
    end
  endtask

  // Stimulus changes on the NEGEDGE so `start_prefetch` is stable across
  // exactly one posedge whatever phase the caller is in. Driven from the
  // posedge it races the DUT's own sampling and the transaction can silently
  // never begin.
  // The flag is sampled here, one delta past the negedge, BEFORE `d_start` is
  // raised. That is precisely the requester's view: the value it reads on the
  // posedge at which the DUT first sees `start_prefetch` high is the value
  // settled from the previous posedge, which is what this negedge holds.
  task pulse_start;
    begin
      @(negedge clk);
      #1;
      start_idx = start_idx + 1;
      if (start_idx == 1) begin o_t1 = o_done; n_t1 = n_done; end
      if (start_idx == 2) begin o_t2 = o_done; n_t2 = n_done; end
      d_start = 1;
      @(negedge clk); d_start = 0;
    end
  endtask

  task hard_reset;
    begin
      @(negedge clk); rst_n = 0;
      repeat (4) @(posedge clk);
      @(negedge clk); rst_n = 1;
      @(posedge clk);
    end
  endtask

  // -----------------------------------------------------------------------
  // Two back-to-back transactions separated by `gap` idle cycles, with the
  // requester sampling `prefetch_done` in the same cycle it raises
  // `start_prefetch`.
  // -----------------------------------------------------------------------
  task two_transactions(input integer gap);
    begin
      hard_reset;
      start_idx = 0;
      o_t1 = 1'bx; n_t1 = 1'bx; o_t2 = 1'bx; n_t2 = 1'bx;
      pulse_start;
      repeat (WORDS + 8) @(posedge clk);   // let transaction 1 retire
      repeat (gap) @(posedge clk);         // idle gap
      pulse_start;
      repeat (WORDS + 8) @(posedge clk);   // let transaction 2 retire
      @(negedge clk); #1;                  // step past the observer's own edge
    end
  endtask

  // -----------------------------------------------------------------------
  // Stimulus
  // -----------------------------------------------------------------------
  initial begin

    // ------------------------------------------------------------------
    // CASE 1 -- the reported defect, with a short idle gap.
    // ------------------------------------------------------------------
    two_transactions(8);

    $display("== CASE 1: two %0d-word transactions, 8-cycle idle gap ==", WORDS);
    $display("    old: t1 sampled_done=%b  t2 sampled_done=%b  done_rises=%0d  done_high_cycles=%0d  we_count=%0d",
             o_t1, o_t2, o_done_rises, o_done_high, o_we_count);
    $display("    new: t1 sampled_done=%b  t2 sampled_done=%b  done_rises=%0d  done_high_cycles=%0d  we_count=%0d",
             n_t1, n_t2, n_done_rises, n_done_high, n_we_count);
    $display("    non-prefetch_done outputs: %0d mismatch(es) over %0d cycles; prefetch_done differed on %0d cycle(s)",
             other_mismatch, cmp_cycles, done_differs);

    // Liveness first. Every claim below is vacuous if the transactions did not
    // happen: both renderings must have completed both of them and written the
    // same number of BRAM words.
    expect_eq("old done rises",  o_done_rises, 2);
    expect_eq("new done rises",  n_done_rises, 2);
    expect_eq("old BRAM writes", o_we_count, 2 * WORDS);
    expect_eq("new BRAM writes", n_we_count, 2 * WORDS);

    // At the FIRST request nothing has completed yet, so both renderings must
    // read the flag low. This is what makes the t2 reading below a difference
    // in staleness rather than a constant offset between the two renderings.
    if (o_t1 !== 1'b0 || n_t1 !== 1'b0) begin
      errors = errors + 1;
      $display("  FAIL at the first request the flag should read low in both, got old=%b new=%b",
               o_t1, n_t1);
    end

    // The defect must still be present in the OLD rendering, or this harness
    // is not measuring what it claims to measure.
    if (o_t2 !== 1'b1) begin
      errors = errors + 1;
      $display("  FAIL harness: old read prefetch_done=%b at the second request, expected the stale 1 -- the defect this harness exists to demonstrate is not reproducing",
               o_t2);
    end

    // The fix: the second requester sees its own state, not the previous
    // transaction's completion.
    if (n_t2 !== 1'b0) begin
      errors = errors + 1;
      $display("  FAIL new read prefetch_done=%b at the second request, expected 0",
               n_t2);
    end

    // The fix must be surgical: the flag is the ONLY thing that may differ.
    expect_eq("non-prefetch_done outputs that diverged", other_mismatch, 0);
    // ...and it must actually differ, or the comparison above is measuring an
    // absence of change that includes the change under test.
    if (done_differs == 0) begin
      errors = errors + 1;
      $display("  FAIL prefetch_done never differed between the renderings -- the harness observed nothing");
    end

    // ------------------------------------------------------------------
    // CASE 2 -- pulse or level? Same stimulus, a much longer idle gap.
    //
    // This is the case that distinguishes the two possible readings of CASE 1.
    // A one-cycle pulse holds its high-time constant when the gap grows; a
    // level stretches with it. The old rendering's high-time must track the
    // gap and the new one's must not.
    // ------------------------------------------------------------------
    begin : case2
      integer o_high_short, n_high_short;
      o_high_short = o_done_high;
      n_high_short = n_done_high;

      two_transactions(40);

      $display("== CASE 2: same, 40-cycle idle gap (was 8) ==");
      $display("    old: done_high_cycles=%0d (8-cycle gap gave %0d)  done_rises=%0d  we_count=%0d",
               o_done_high, o_high_short, o_done_rises, o_we_count);
      $display("    new: done_high_cycles=%0d (8-cycle gap gave %0d)  done_rises=%0d  we_count=%0d",
               n_done_high, n_high_short, n_done_rises, n_we_count);
      $display("    non-prefetch_done outputs: %0d mismatch(es) over %0d cycles; prefetch_done differed on %0d cycle(s)",
               other_mismatch, cmp_cycles, done_differs);

      expect_eq("old done rises (long gap)",  o_done_rises, 2);
      expect_eq("new done rises (long gap)",  n_done_rises, 2);
      expect_eq("old BRAM writes (long gap)", o_we_count, 2 * WORDS);
      expect_eq("new BRAM writes (long gap)", n_we_count, 2 * WORDS);

      // The old flag is a level: 32 more idle cycles, 32 more cycles high.
      expect_eq("old prefetch_done high-time grew by the extra gap",
                o_done_high - o_high_short, 32);

      // The new flag is a pulse: one cycle per completion, gap-independent.
      expect_eq("new prefetch_done high-time is gap-independent",
                n_done_high - n_high_short, 0);
      expect_eq("new prefetch_done high-time is one cycle per completion",
                n_done_high, 2);

      expect_eq("non-prefetch_done outputs that diverged (long gap)",
                other_mismatch, 0);
      if (o_t2 !== 1'b1 || n_t2 !== 1'b0) begin
        errors = errors + 1;
        $display("  FAIL long-gap sampled_done: old=%b new=%b, expected old=1 new=0",
                 o_t2, n_t2);
      end
    end

    $display("");
    if (errors == 0) $display("RESULT: PASS (0 errors)");
    else             $display("RESULT: FAIL (%0d errors)", errors);
    $finish;
  end

endmodule
