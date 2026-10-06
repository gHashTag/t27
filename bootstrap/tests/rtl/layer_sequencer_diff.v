`timescale 1ns/1ps
// ===========================================================================
// tb_bitnet_sequencer_zero_count -- differential harness for issue #1977
// ===========================================================================
// `layer_sequencer` never left the RUN state when it was asked for zero work.
// Both of the FSM's terminators are `index == count-1` compares against an
// unsigned input port:
//
//     last_chunk <= (chunk_id == num_chunks-1);
//     if(chunk_id==num_chunks-1) begin chunk_id<=0;
//         if(neuron_id==num_neurons-1) state<=DONE_ST; else neuron_id<=neuron_id+1;
//     end else chunk_id<=chunk_id+1;
//
// `num_neurons` is a 16-bit port and `num_chunks` an 8-bit one, but the bare
// literal `1` makes each subtraction 32 bits wide. A zero count therefore
// BORROWS rather than saturating: `0 - 1` is 32'hFFFFFFFF, while the index on
// the left zero-extends into the same 32 bits. No value a 16-bit `neuron_id`
// or an 8-bit `chunk_id` can hold ever equals 32'hFFFFFFFF, so the compare can
// never fire, the FSM never reaches DONE_ST, `done` never pulses, and `valid`
// is asserted forever for work nobody requested.
//
// The fix retires a zero count straight to DONE_ST with `valid` low:
//
//     if(num_neurons==0 || num_chunks==0) begin valid<=0; state<=DONE_ST; end
//     else begin ...original body... end
//
// This harness elaborates BOTH the pre-fix and the post-fix emitter output in
// one simulation, drives them from identical stimulus, and compares them. The
// two variants differ only in the `module_name` passed to the emitter, so the
// comparison is between two renderings of the same design, not two designs.
//
// Build (see sim/README.md for the emit step):
//   iverilog -g2005 -o tb.vvp sim/tb_bitnet_sequencer_zero_count.v \
//            seq_old.v seq_new.v
//   vvp tb.vvp
//
// The measured property is "a request for zero work terminates, and a request
// for non-zero work is unchanged". The second half is not decoration: a guard
// that retired EVERY request to DONE_ST would satisfy the first half alone.
// The six non-zero controls below are what stop that, and they compare the
// full output vector on EVERY cycle rather than just the final state -- a
// controller that reaches the same endpoint by a different path is a
// behavioural change and must be caught.
// ===========================================================================

module tb_bitnet_sequencer_zero_count;

  // Long enough that "still running" is not a matter of opinion. The pre-fix
  // rendering is asked for zero neurons and is still counting at cycle
  // 200,000; with num_chunks=4 that is exactly 50,000 spurious neurons.
  localparam integer HANG_CYCLES = 200000;

  reg clk = 0, rst_n = 0;
  always #5 clk = ~clk;

  integer errors = 0;

  reg         d_start = 0;
  reg  [15:0] d_neurons = 0;
  reg  [7:0]  d_chunks  = 0;

  wire [15:0] o_nid, n_nid;
  wire [7:0]  o_cid, n_cid;
  wire        o_first, n_first, o_last, n_last, o_valid, n_valid, o_done, n_done;

  seq_old u_old (
    .clk(clk), .rst_n(rst_n), .start(d_start),
    .num_neurons(d_neurons), .num_chunks(d_chunks),
    .neuron_id(o_nid), .chunk_id(o_cid),
    .first_chunk(o_first), .last_chunk(o_last),
    .valid(o_valid), .done(o_done)
  );

  seq_new u_new (
    .clk(clk), .rst_n(rst_n), .start(d_start),
    .num_neurons(d_neurons), .num_chunks(d_chunks),
    .neuron_id(n_nid), .chunk_id(n_cid),
    .first_chunk(n_first), .last_chunk(n_last),
    .valid(n_valid), .done(n_done)
  );

  // -----------------------------------------------------------------------
  // Observers.
  //
  // Sampled on the NEGEDGE, deliberately. Every output of this FSM is a
  // registered (non-blocking) output, so an `always @(posedge clk)` observer
  // reads the value from BEFORE that edge's update and its high-water mark
  // trails the design by exactly one cycle. Sampling at the negedge reads the
  // settled value of the cycle the design is actually in. The first draft of
  // this harness sampled at the posedge and reported max_nid=49999 for a run
  // whose port plainly read 50000 -- a phase error in the instrument, not a
  // property of either rendering.
  //
  // `done` is a single-cycle pulse, so it must be latched: sampling it once at
  // the end of a case would miss it entirely and report every rendering as
  // hung. `valid_seen` is latched for the same reason and is what makes the
  // zero-work claim about `valid` meaningful.
  //
  // The high-water marks copy the port into an integer BEFORE comparing: a
  // direct `o_nid > max` promotes the whole expression to unsigned, which is
  // harmless while max >= 0 but is the trap that silently froze the
  // high-water mark in a sibling harness. Keep it signed by construction
  // rather than by luck.
  //
  // No separate clear signal: `hard_reset` holds `rst_n` low across several
  // negedges, which is what zeroes these.
  // -----------------------------------------------------------------------
  reg  o_done_seen = 0, n_done_seen = 0;
  reg  o_valid_seen = 0, n_valid_seen = 0;
  integer o_max_nid = 0, n_max_nid = 0;
  integer o_max_cid = 0, n_max_cid = 0;
  integer o_done_pulses = 0, n_done_pulses = 0;
  integer o_a, n_a;

  always @(negedge clk) if (!rst_n) begin
    o_done_seen = 0; n_done_seen = 0;
    o_valid_seen = 0; n_valid_seen = 0;
    o_max_nid = 0; n_max_nid = 0;
    o_max_cid = 0; n_max_cid = 0;
    o_done_pulses = 0; n_done_pulses = 0;
  end else begin
    if (o_done) begin o_done_seen = 1; o_done_pulses = o_done_pulses + 1; end
    if (n_done) begin n_done_seen = 1; n_done_pulses = n_done_pulses + 1; end
    if (o_valid) o_valid_seen = 1;
    if (n_valid) n_valid_seen = 1;
    o_a = o_nid; if (o_a > o_max_nid) o_max_nid = o_a;
    n_a = n_nid; if (n_a > n_max_nid) n_max_nid = n_a;
    o_a = o_cid; if (o_a > o_max_cid) o_max_cid = o_a;
    n_a = n_cid; if (n_a > n_max_cid) n_max_cid = n_a;
  end

  // -----------------------------------------------------------------------
  // Cycle-by-cycle equality comparator, used by the non-zero controls.
  //
  // Every observable output is compared on every cycle while `cmp_en` is high,
  // not merely the final state. Two renderings that arrive at the same place
  // by different routes are NOT the same rendering, and a control that only
  // checked the endpoint would let that through.
  //
  // `first_chunk` and `last_chunk` are compared only while `valid` is high.
  // That is not a convenience: NEITHER rendering resets them. The emitted
  // reset block is
  //
  //     state<=IDLE; neuron_id<=0; chunk_id<=0; valid<=0; done<=0;
  //
  // with no mention of `first_chunk`/`last_chunk`, so both sit at X from
  // power-up until the first RUN cycle assigns them, and X !== X. That is a
  // property of the design, identical in old and new, and unrelated to the
  // zero-count guard under test. Qualifying on `valid` -- the strobe these two
  // flags accompany -- compares them exactly when they carry meaning.
  //
  // `qual_cycles` counts how often that qualified comparison actually ran, and
  // the caller asserts it equals the number of work cycles. Without that
  // counter the qualification could silently disable the check.
  // -----------------------------------------------------------------------
  reg cmp_en = 0;
  integer cmp_cycles = 0, cmp_mismatch = 0, qual_cycles = 0;
  integer first_bad_cycle = -1;
  reg vec_bad, qual_bad;

  always @(negedge clk) if (rst_n && cmp_en) begin
    cmp_cycles = cmp_cycles + 1;
    vec_bad = (o_nid   !== n_nid  ) || (o_cid   !== n_cid  ) ||
              (o_valid !== n_valid) || (o_done  !== n_done );
    qual_bad = 1'b0;
    if (o_valid || n_valid) begin
      qual_cycles = qual_cycles + 1;
      qual_bad = (o_first !== n_first) || (o_last !== n_last);
    end
    if (vec_bad || qual_bad) begin
      if (cmp_mismatch == 0) begin
        first_bad_cycle = cmp_cycles;
        $display("  FAIL control diverged at compared cycle %0d:", cmp_cycles);
        $display("        old nid=%0d cid=%0d first=%b last=%b valid=%b done=%b",
                 o_nid, o_cid, o_first, o_last, o_valid, o_done);
        $display("        new nid=%0d cid=%0d first=%b last=%b valid=%b done=%b",
                 n_nid, n_cid, n_first, n_last, n_valid, n_done);
      end
      cmp_mismatch = cmp_mismatch + 1;
    end
  end

  task expect_eq(input [511:0] what, input integer got, input integer want);
    begin
      if (got !== want) begin
        errors = errors + 1;
        $display("  FAIL %0s: got %0d, want %0d", what, got, want);
      end
    end
  endtask

  // All stimulus changes on the NEGEDGE, so `start` is stable across exactly
  // one posedge no matter what phase the caller is in. Driving it from the
  // posedge instead makes the pulse race the DUT's own sampling: the stimulus
  // can clear `start` at the same timestep the DUT reads it, and the run
  // silently never begins.
  task pulse_start;
    begin
      @(negedge clk); d_start = 1;
      @(negedge clk); d_start = 0;
    end
  endtask

  // `cmp_en` is dropped here so the comparator never sees the reset window.
  // `rst_n` is held low across several negedges, which is what clears the
  // negedge-sampled observers above.
  task hard_reset;
    begin
      @(posedge clk); cmp_en = 0;
      @(negedge clk); rst_n = 0;
      repeat (4) @(posedge clk);
      @(negedge clk); rst_n = 1;
      @(posedge clk);
    end
  endtask

  // -----------------------------------------------------------------------
  // One non-zero control: identical stimulus, full-vector comparison, and a
  // liveness requirement so that "identical" cannot mean "both did nothing".
  // -----------------------------------------------------------------------
  integer ctl_idx = 0;
  task run_control(input [15:0] neurons, input [7:0] chunks);
    integer budget;
    begin
      ctl_idx = ctl_idx + 1;
      hard_reset;   // leaves us on a posedge, opposite phase to the comparator
      cmp_cycles = 0; cmp_mismatch = 0; qual_cycles = 0; first_bad_cycle = -1;
      d_neurons = neurons;
      d_chunks  = chunks;
      cmp_en = 1;
      pulse_start;
      // neurons*chunks RUN cycles, plus DONE_ST, plus slack.
      budget = neurons * chunks + 32;
      repeat (budget) @(posedge clk);
      // Settle on the negedge the comparator uses, then step past it before
      // reading its counters, so the read cannot race the final comparison.
      @(negedge clk); #1;
      cmp_en = 0;

      $display("  control %0d: num_neurons=%0d num_chunks=%0d -> compared %0d cycles (%0d qualified), %0d mismatches; old(done=%0d valid_seen=%b) new(done=%0d valid_seen=%b)",
               ctl_idx, neurons, chunks, cmp_cycles, qual_cycles, cmp_mismatch,
               o_done_pulses, o_valid_seen, n_done_pulses, n_valid_seen);

      if (cmp_mismatch !== 0) begin
        errors = errors + 1;
        $display("  FAIL control %0d: %0d cycle(s) differ, first at %0d",
                 ctl_idx, cmp_mismatch, first_bad_cycle);
      end

      // Anti-vacuity: an "identical" verdict from two renderings that never
      // ran is worth nothing. Both must have completed real work.
      if (!o_valid_seen || !n_valid_seen) begin
        errors = errors + 1;
        $display("  FAIL control %0d: valid never asserted (old=%b new=%b) -- the control did no work and proves nothing",
                 ctl_idx, o_valid_seen, n_valid_seen);
      end
      if (o_done_pulses !== 1 || n_done_pulses !== 1) begin
        errors = errors + 1;
        $display("  FAIL control %0d: expected exactly one done pulse each, got old=%0d new=%0d",
                 ctl_idx, o_done_pulses, n_done_pulses);
      end
      if (cmp_cycles < neurons * chunks) begin
        errors = errors + 1;
        $display("  FAIL control %0d: only %0d cycles compared, fewer than the %0d work cycles",
                 ctl_idx, cmp_cycles, neurons * chunks);
      end
      // The `first_chunk`/`last_chunk` comparison is qualified on `valid`.
      // Pin how often it actually ran, so the qualification cannot quietly
      // reduce that half of the vector to a no-op: exactly one qualified
      // cycle per (neuron, chunk) pair.
      if (qual_cycles !== neurons * chunks) begin
        errors = errors + 1;
        $display("  FAIL control %0d: first/last compared on %0d cycles, want %0d -- the valid-qualified half of the vector is not being exercised",
                 ctl_idx, qual_cycles, neurons * chunks);
      end
    end
  endtask

  // -----------------------------------------------------------------------
  // Stimulus
  // -----------------------------------------------------------------------
  initial begin
    hard_reset;

    // ------------------------------------------------------------------
    // CASE 1 -- the reported defect: zero neurons, non-zero chunks.
    //
    // num_chunks is deliberately non-zero so the chunk terminator still
    // fires normally. That isolates the neuron compare: the only reason the
    // FSM cannot finish is `neuron_id == num_neurons-1` against 32'hFFFFFFFF.
    // ------------------------------------------------------------------
    d_neurons = 16'd0;
    d_chunks  = 8'd4;
    pulse_start;
    repeat (HANG_CYCLES) @(posedge clk);
    @(negedge clk); #1;   // step past the observer's own sampling edge

    $display("== CASE 1: zero neurons (num_neurons=0, num_chunks=4), %0d cycles ==",
             HANG_CYCLES);
    $display("    old: done_seen=%b done_pulses=%0d valid_seen=%b max_nid=%0d final_nid=%0d",
             o_done_seen, o_done_pulses, o_valid_seen, o_max_nid, o_nid);
    $display("    new: done_seen=%b done_pulses=%0d valid_seen=%b max_nid=%0d final_nid=%0d",
             n_done_seen, n_done_pulses, n_valid_seen, n_max_nid, n_nid);

    // The defect must still be present in the OLD rendering, or this harness
    // is not measuring what it claims to measure.
    if (o_done_seen !== 1'b0) begin
      errors = errors + 1;
      $display("  FAIL harness: old reported done for a zero-neuron request -- "
             , "the non-termination this harness exists to demonstrate is not "
             , "reproducing");
    end
    // Not merely "no done": still actively counting. A rendering that had
    // wedged with all outputs frozen would also show done=0, and that is a
    // different defect from the one under test.
    if (o_max_nid < 1000) begin
      errors = errors + 1;
      $display("  FAIL harness: old only reached neuron_id=%0d in %0d cycles -- expected it to keep counting, not to freeze",
               o_max_nid, HANG_CYCLES);
    end
    // num_chunks=4 advances neuron_id once every 4 RUN cycles, so after
    // HANG_CYCLES posedges in RUN the index is exactly HANG_CYCLES/4 and has
    // not yet wrapped its 16 bits (50000 < 65536). Pin the arithmetic: an
    // off-by-a-lot here would mean the FSM is not doing what the analysis says.
    expect_eq("old neuron_id after the hang", o_max_nid, HANG_CYCLES / 4);
    if (o_valid_seen !== 1'b1) begin
      errors = errors + 1;
      $display("  FAIL harness: old never asserted valid -- expected it to emit "
             , "work strobes for a request of zero neurons");
    end

    // The fix: terminate, and do not emit a single work strobe on the way.
    if (n_done_seen !== 1'b1) begin
      errors = errors + 1;
      $display("  FAIL new never reported done for a zero-neuron request");
    end
    expect_eq("new neuron_id after the run", n_max_nid, 0);
    if (n_valid_seen !== 1'b0) begin
      errors = errors + 1;
      $display("  FAIL new asserted valid for a request of zero neurons");
    end

    // ------------------------------------------------------------------
    // CASE 2 -- the other zero: zero chunks, non-zero neurons.
    //
    // The chunk terminator has its own borrow, and a guard that tested only
    // num_neurons would leave this half hanging. Separate case, separate
    // claim.
    // ------------------------------------------------------------------
    hard_reset;
    d_neurons = 16'd8;
    d_chunks  = 8'd0;
    pulse_start;
    repeat (HANG_CYCLES) @(posedge clk);
    @(negedge clk); #1;

    $display("== CASE 2: zero chunks (num_neurons=8, num_chunks=0), %0d cycles ==",
             HANG_CYCLES);
    $display("    old: done_seen=%b valid_seen=%b max_nid=%0d max_cid=%0d final_cid=%0d",
             o_done_seen, o_valid_seen, o_max_nid, o_max_cid, o_cid);
    $display("    new: done_seen=%b valid_seen=%b max_nid=%0d max_cid=%0d final_cid=%0d",
             n_done_seen, n_valid_seen, n_max_nid, n_max_cid, n_cid);

    if (o_done_seen !== 1'b0) begin
      errors = errors + 1;
      $display("  FAIL harness: old reported done for a zero-chunk request -- "
             , "the zero-chunk half of the defect is not reproducing");
    end
    if (o_valid_seen !== 1'b1) begin
      errors = errors + 1;
      $display("  FAIL harness: old never asserted valid on the zero-chunk path");
    end
    // Pin the mechanism, not just the symptom. With num_chunks=0 the CHUNK
    // compare is the one that can never match, so chunk_id free-runs and wraps
    // its 8 bits while neuron_id never advances at all. Both halves are
    // asserted: a rendering that hung with chunk_id frozen, or one that
    // advanced neuron_id anyway, would be a different defect.
    expect_eq("old chunk_id high-water on the zero-chunk path", o_max_cid, 255);
    expect_eq("old neuron_id on the zero-chunk path",           o_max_nid, 0);
    // HANG_CYCLES posedges in RUN leave chunk_id at HANG_CYCLES mod 256.
    expect_eq("old chunk_id after the hang", o_cid, HANG_CYCLES % 256);
    if (n_done_seen !== 1'b1) begin
      errors = errors + 1;
      $display("  FAIL new never reported done for a zero-chunk request");
    end
    if (n_valid_seen !== 1'b0) begin
      errors = errors + 1;
      $display("  FAIL new asserted valid for a request of zero chunks");
    end

    // ------------------------------------------------------------------
    // CASES 3..8 -- six non-zero controls.
    //
    // These are what stop the guard from being a licence to retire everything:
    // a rendering that jumped to DONE_ST unconditionally would pass CASE 1 and
    // CASE 2 perfectly. Every observable output is compared on every cycle.
    //
    // The set spans the shapes where the terminators behave differently:
    // num_chunks=1 makes `chunk_id==num_chunks-1` true at chunk_id=0 so the
    // chunk loop never iterates; num_neurons=1 exercises the single-neuron
    // exit; 8'd255 is the largest value the 8-bit chunk port can carry, which
    // is the boundary the widened subtraction would disturb if the guard were
    // implemented by narrowing the compare instead.
    // ------------------------------------------------------------------
    $display("== CASES 3..8: six non-zero controls, full-vector cycle-by-cycle ==");
    run_control(16'd1,   8'd1);
    run_control(16'd1,   8'd4);
    run_control(16'd4,   8'd1);
    run_control(16'd3,   8'd5);
    run_control(16'd7,   8'd2);
    run_control(16'd2,   8'd255);

    $display("");
    if (errors == 0) $display("RESULT: PASS (0 errors)");
    else             $display("RESULT: FAIL (%0d errors)", errors);
    $finish;
  end

endmodule
