`timescale 1ns/1ps
// ===========================================================================
// tb_bitnet_dma_we_default -- THREE-WAY differential harness for issue #2006
// ===========================================================================
// `local_we` is a one-cycle write strobe, but the pre-fix `dma_controller`
// drove it only from the arms that happened to think about it. The fix
// defaults it low ahead of the case, so a state that never mentions it leaves
// it low:
//
//     end else begin
//         local_we <= 1'b0;          // <-- added
//         case (state)
//         ...
//
// THE CLAIM THIS HARNESS MAKES IS A NULL RESULT, AND THAT NEEDS SAYING OUT
// LOUD: the pre-fix and post-fix renderings are observationally IDENTICAL.
// The fix is behaviourally latent. Reading the pre-fix emitter output, every
// reachable path already drives the strobe --
//
//     reset      : local_we <= 1'b0;
//     READ_DATA  : local_we <= 1'b1;  ... end else local_we <= 1'b0;
//     DONE_ST    : local_we <= 1'b0;
//
// -- and the states that do NOT drive it (IDLE, READ_ADDR, WRITE_ADDR,
// WRITE_DATA, default) are never entered with it high, because READ_DATA's
// only exit is DONE_ST and DONE_ST clears the strobe before returning to IDLE.
// The default-low is defence in depth against a future arm, not a repair of an
// observable defect.
//
// A harness that "passes" by finding no difference proves nothing on its own:
// a harness wired to the wrong ports, or one whose comparator never ran, finds
// no difference either. So this harness is THREE-WAY. It elaborates three
// renderings of the same design and runs one comparator over two pairs:
//
//     A = pre-#2006                       (PR #2344 base)
//     B = #2006 applied                   (PR #2344 head)
//     C = #2006 + #2003 write-address fix (PR #2345 head)
//
// B and C are consecutive revisions of the same file: PR #2344's head
// rendering is byte-identical to PR #2345's base rendering, so A, B and C are
// a linear chain and the same comparator sees all three.
//
//     A vs B  must be IDENTICAL  -- the null result under test
//     B vs C  must DIFFER        -- the anti-vacuity control
//
// The control is not decoration and it is not a separate test. It is the SAME
// comparator, on the SAME stimulus, in the SAME simulation, over the same
// signal set. If the comparator is blind -- misconnected ports, a disabled
// enable, a vector that omits `local_we`, a stimulus that never starts a
// transfer -- then B vs C reports "identical" too, and the run FAILS. Only
// when the instrument has demonstrated on C that it can see a difference does
// "A equals B" carry any information.
//
// Build (see sim/README.md for the emit step):
//   iverilog -g2005 -o tb.vvp sim/tb_bitnet_dma_we_default.v \
//            dma_a.v dma_b.v dma_c.v
//   vvp tb.vvp
// ===========================================================================

module tb_bitnet_dma_we_default;

  localparam integer CAP_WORDS = 4096;

  reg clk = 0, rst_n = 0;
  always #5 clk = ~clk;

  integer errors = 0;

  reg         d_start   = 0;
  reg         d_dir     = 0;
  reg  [31:0] d_length  = 0;
  reg         d_arready = 1;   // held low to stretch READ_ADDR
  reg         d_rvalid  = 0;
  reg         d_rlast   = 0;
  reg         d_wready  = 1;

  // ------------------------------------------------------------------------
  // Three renderings, identical stimulus.
  // ------------------------------------------------------------------------
  wire        a_busy, b_busy, c_busy, a_done, b_done, c_done;
  wire [63:0] a_araddr, b_araddr, c_araddr;
  wire [7:0]  a_arlen, b_arlen, c_arlen;
  wire        a_arvalid, b_arvalid, c_arvalid;
  wire        a_rready, b_rready, c_rready;
  wire [63:0] a_awaddr, b_awaddr, c_awaddr;
  wire [7:0]  a_awlen, b_awlen, c_awlen;
  wire        a_awvalid, b_awvalid, c_awvalid;
  wire [63:0] a_wdata_axi, b_wdata_axi, c_wdata_axi;
  wire        a_wlast, b_wlast, c_wlast, a_wvalid, b_wvalid, c_wvalid;
  wire        a_bready, b_bready, c_bready;
  wire [11:0] a_addr, b_addr, c_addr;
  wire [63:0] a_wdata, b_wdata, c_wdata;
  wire        a_we, b_we, c_we;

  // Each DUT gets its own beat counter driven by its own `rready`, so the
  // payload a rendering captures is its own beat index and the three never
  // share a counter. Beat K carries the value K.
  reg beats_clr = 0;
  reg [63:0] a_beats = 0, b_beats = 0, c_beats = 0;
  always @(posedge clk) if (!rst_n || beats_clr) a_beats <= 0; else if (a_rready && d_rvalid) a_beats <= a_beats + 1;
  always @(posedge clk) if (!rst_n || beats_clr) b_beats <= 0; else if (b_rready && d_rvalid) b_beats <= b_beats + 1;
  always @(posedge clk) if (!rst_n || beats_clr) c_beats <= 0; else if (c_rready && d_rvalid) c_beats <= c_beats + 1;

  dma_a u_a (
    .clk(clk), .rst_n(rst_n),
    .start(d_start), .src_addr(64'h2000), .dst_addr(64'h3000),
    .length(d_length), .direction(d_dir), .busy(a_busy), .done(a_done),
    .m_axi_araddr(a_araddr), .m_axi_arlen(a_arlen), .m_axi_arvalid(a_arvalid),
    .m_axi_arready(d_arready),
    .m_axi_rdata(a_beats), .m_axi_rlast(d_rlast), .m_axi_rvalid(d_rvalid),
    .m_axi_rready(a_rready),
    .m_axi_awaddr(a_awaddr), .m_axi_awlen(a_awlen), .m_axi_awvalid(a_awvalid),
    .m_axi_awready(1'b1),
    .m_axi_wdata(a_wdata_axi), .m_axi_wlast(a_wlast), .m_axi_wvalid(a_wvalid),
    .m_axi_wready(d_wready), .m_axi_bvalid(1'b1), .m_axi_bready(a_bready),
    .local_addr(a_addr), .local_wdata(a_wdata), .local_we(a_we),
    .local_rdata(64'hA5A5_0000_0000_5A5A)
  );

  dma_b u_b (
    .clk(clk), .rst_n(rst_n),
    .start(d_start), .src_addr(64'h2000), .dst_addr(64'h3000),
    .length(d_length), .direction(d_dir), .busy(b_busy), .done(b_done),
    .m_axi_araddr(b_araddr), .m_axi_arlen(b_arlen), .m_axi_arvalid(b_arvalid),
    .m_axi_arready(d_arready),
    .m_axi_rdata(b_beats), .m_axi_rlast(d_rlast), .m_axi_rvalid(d_rvalid),
    .m_axi_rready(b_rready),
    .m_axi_awaddr(b_awaddr), .m_axi_awlen(b_awlen), .m_axi_awvalid(b_awvalid),
    .m_axi_awready(1'b1),
    .m_axi_wdata(b_wdata_axi), .m_axi_wlast(b_wlast), .m_axi_wvalid(b_wvalid),
    .m_axi_wready(d_wready), .m_axi_bvalid(1'b1), .m_axi_bready(b_bready),
    .local_addr(b_addr), .local_wdata(b_wdata), .local_we(b_we),
    .local_rdata(64'hA5A5_0000_0000_5A5A)
  );

  dma_c u_c (
    .clk(clk), .rst_n(rst_n),
    .start(d_start), .src_addr(64'h2000), .dst_addr(64'h3000),
    .length(d_length), .direction(d_dir), .busy(c_busy), .done(c_done),
    .m_axi_araddr(c_araddr), .m_axi_arlen(c_arlen), .m_axi_arvalid(c_arvalid),
    .m_axi_arready(d_arready),
    .m_axi_rdata(c_beats), .m_axi_rlast(d_rlast), .m_axi_rvalid(d_rvalid),
    .m_axi_rready(c_rready),
    .m_axi_awaddr(c_awaddr), .m_axi_awlen(c_awlen), .m_axi_awvalid(c_awvalid),
    .m_axi_awready(1'b1),
    .m_axi_wdata(c_wdata_axi), .m_axi_wlast(c_wlast), .m_axi_wvalid(c_wvalid),
    .m_axi_wready(d_wready), .m_axi_bvalid(1'b1), .m_axi_bready(c_bready),
    .local_addr(c_addr), .local_wdata(c_wdata), .local_we(c_we),
    .local_rdata(64'hA5A5_0000_0000_5A5A)
  );

  // ------------------------------------------------------------------------
  // The observation vector. EVERY output port of the module is in it -- if a
  // signal is not here it is not being compared, and the null result would be
  // correspondingly weaker.
  // ------------------------------------------------------------------------
  wire [292:0] a_vec = {a_busy, a_done, a_araddr, a_arlen, a_arvalid, a_rready,
                        a_awaddr, a_awlen, a_awvalid, a_wdata_axi, a_wlast,
                        a_wvalid, a_bready, a_addr, a_wdata, a_we};
  wire [292:0] b_vec = {b_busy, b_done, b_araddr, b_arlen, b_arvalid, b_rready,
                        b_awaddr, b_awlen, b_awvalid, b_wdata_axi, b_wlast,
                        b_wvalid, b_bready, b_addr, b_wdata, b_we};
  wire [292:0] c_vec = {c_busy, c_done, c_araddr, c_arlen, c_arvalid, c_rready,
                        c_awaddr, c_awlen, c_awvalid, c_wdata_axi, c_wlast,
                        c_wvalid, c_bready, c_addr, c_wdata, c_we};

  // ------------------------------------------------------------------------
  // Observers and the shared comparator, sampled on the NEGEDGE.
  //
  // Every output of this module is registered (or a function of registered
  // state), so a posedge observer reads values from before that edge's
  // non-blocking update. The negedge holds the settled value of the cycle the
  // design is actually in.
  // ------------------------------------------------------------------------
  integer cmp_cycles = 0;
  integer ab_mismatch = 0, bc_mismatch = 0;
  integer ab_first = -1, bc_first = -1;
  integer a_we_high = 0, b_we_high = 0, c_we_high = 0;
  integer a_writes = 0, b_writes = 0, c_writes = 0;
  integer a_wbeats = 0, b_wbeats = 0, c_wbeats = 0;
  integer a_dones = 0, b_dones = 0, c_dones = 0;
  reg a_done_prev = 0, b_done_prev = 0, c_done_prev = 0;

  always @(negedge clk) if (!rst_n) begin
    a_done_prev = 0; b_done_prev = 0; c_done_prev = 0;
  end else begin
    cmp_cycles = cmp_cycles + 1;

    if (a_vec !== b_vec) begin
      if (ab_mismatch == 0) begin
        ab_first = cmp_cycles;
        $display("  FAIL A/B diverged at compared cycle %0d", cmp_cycles);
        $display("        A: busy=%b done=%b arvalid=%b rready=%b wvalid=%b local_addr=%0d local_wdata=%0h local_we=%b",
                 a_busy, a_done, a_arvalid, a_rready, a_wvalid, a_addr, a_wdata, a_we);
        $display("        B: busy=%b done=%b arvalid=%b rready=%b wvalid=%b local_addr=%0d local_wdata=%0h local_we=%b",
                 b_busy, b_done, b_arvalid, b_rready, b_wvalid, b_addr, b_wdata, b_we);
      end
      ab_mismatch = ab_mismatch + 1;
    end

    if (b_vec !== c_vec) begin
      if (bc_mismatch == 0) bc_first = cmp_cycles;
      bc_mismatch = bc_mismatch + 1;
    end

    // Liveness accounting. `local_we` high-cycles and write counts must match
    // between A and B for the null result to mean "both did the same work"
    // rather than "neither did any".
    if (a_we) begin a_we_high = a_we_high + 1; a_writes = a_writes + 1; end
    if (b_we) begin b_we_high = b_we_high + 1; b_writes = b_writes + 1; end
    if (c_we) begin c_we_high = c_we_high + 1; c_writes = c_writes + 1; end
    if (a_wvalid) a_wbeats = a_wbeats + 1;
    if (b_wvalid) b_wbeats = b_wbeats + 1;
    if (c_wvalid) c_wbeats = c_wbeats + 1;
    if (a_done && !a_done_prev) a_dones = a_dones + 1;
    if (b_done && !b_done_prev) b_dones = b_dones + 1;
    if (c_done && !c_done_prev) c_dones = c_dones + 1;
    a_done_prev = a_done; b_done_prev = b_done; c_done_prev = c_done;
  end

  task expect_eq(input [511:0] what, input integer got, input integer want);
    begin
      if (got !== want) begin
        errors = errors + 1;
        $display("  FAIL %0s: got %0d, want %0d", what, got, want);
      end
    end
  endtask

  // Stimulus changes on the NEGEDGE so `start` is stable across exactly one
  // posedge whatever phase the caller is in. Driven from the posedge it races
  // the DUTs' own sampling and the transfer can silently never begin.
  task pulse_start;
    begin
      @(negedge clk); d_start = 1;
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
  // Stimulus. Every phase runs against all three renderings at once and the
  // comparator is live throughout -- there is no per-phase enable that could
  // be left off.
  // -----------------------------------------------------------------------
  initial begin
    hard_reset;

    // PHASE 1 -- a plain 4-beat read. This is the phase that makes B and C
    // differ, because #2003 changed the READ_DATA address arm.
    d_rvalid = 1; d_rlast = 0; d_arready = 1; d_dir = 0;
    d_length = 32;
    pulse_start;
    repeat (40) @(posedge clk);

    // PHASE 2 -- a second read with no reset between, exercising the IDLE
    // re-arm path.
    @(negedge clk); beats_clr = 1; @(negedge clk); beats_clr = 0;
    d_length = 32;
    pulse_start;
    repeat (40) @(posedge clk);

    // PHASE 3 -- READ_ADDR stretched. `arready` is held low for 6 cycles, so
    // the FSM sits in READ_ADDR -- one of the states the pre-fix rendering
    // never drives `local_we` from. If a stale strobe could survive into it,
    // this is where it would show.
    d_arready = 0;
    d_length  = 32;
    pulse_start;
    repeat (6) @(posedge clk);
    @(negedge clk); d_arready = 1;
    repeat (40) @(posedge clk);

    // PHASE 4 -- write direction. IDLE -> WRITE_ADDR -> WRITE_DATA -> DONE_ST,
    // and `local_we` is never driven by any of them in the pre-fix rendering.
    d_dir = 1; d_length = 32;
    pulse_start;
    repeat (40) @(posedge clk);

    // PHASE 5 -- write with `wready` throttled, stretching WRITE_DATA.
    d_wready = 0;
    d_dir = 1; d_length = 32;
    pulse_start;
    repeat (5) @(posedge clk);
    @(negedge clk); d_wready = 1;
    repeat (40) @(posedge clk);

    // PHASE 6 -- READ_DATA throttled. `rvalid` is toggled so the FSM sits in
    // READ_DATA on cycles where no beat arrives. That is the ONLY place the
    // pre-fix rendering's `end else local_we <= 1'b0;` does any work, and
    // therefore the only place the #2006 default-low has a live competitor.
    // Without this phase the strobe would never be seen falling inside a
    // transfer and the null result would be resting on untested ground.
    d_dir = 0; d_length = 32;
    d_rvalid = 0;
    pulse_start;
    repeat (3) @(posedge clk);
    @(negedge clk); d_rvalid = 1;
    repeat (2) @(posedge clk);
    @(negedge clk); d_rvalid = 0;
    repeat (4) @(posedge clk);
    @(negedge clk); d_rvalid = 1;
    repeat (40) @(posedge clk);

    // PHASE 7 -- back to a plain read, then a long idle tail with no request
    // at all, so IDLE is held for many cycles with nothing driving the strobe.
    @(negedge clk); beats_clr = 1; @(negedge clk); beats_clr = 0;
    d_dir = 0; d_length = 16;
    pulse_start;
    repeat (40) @(posedge clk);
    repeat (60) @(posedge clk);

    @(negedge clk); #1;

    // ------------------------------------------------------------------
    // Report
    // ------------------------------------------------------------------
    $display("== THREE-WAY: A=pre-#2006  B=#2006  C=#2006+#2003 ==");
    $display("    compared %0d cycles across 7 stimulus phases", cmp_cycles);
    $display("    A: local_we_high=%0d local_writes=%0d axi_write_beats=%0d done_pulses=%0d",
             a_we_high, a_writes, a_wbeats, a_dones);
    $display("    B: local_we_high=%0d local_writes=%0d axi_write_beats=%0d done_pulses=%0d",
             b_we_high, b_writes, b_wbeats, b_dones);
    $display("    C: local_we_high=%0d local_writes=%0d axi_write_beats=%0d done_pulses=%0d",
             c_we_high, c_writes, c_wbeats, c_dones);
    $display("    A vs B: %0d mismatching cycle(s)%0s", ab_mismatch,
             ab_mismatch ? "" : "   <- the null result under test");
    $display("    B vs C: %0d mismatching cycle(s), first at %0d   <- anti-vacuity control",
             bc_mismatch, bc_first);

    // ------------------------------------------------------------------
    // LIVENESS. Everything below is vacuous if the stimulus did nothing.
    // ------------------------------------------------------------------
    if (cmp_cycles < 200) begin
      errors = errors + 1;
      $display("  FAIL harness: only %0d cycles compared -- the stimulus did not run",
               cmp_cycles);
    end
    if (a_writes == 0 || b_writes == 0 || c_writes == 0) begin
      errors = errors + 1;
      $display("  FAIL harness: a rendering produced no local writes (A=%0d B=%0d C=%0d) -- the read path was never exercised",
               a_writes, b_writes, c_writes);
    end
    if (a_wbeats == 0 || b_wbeats == 0 || c_wbeats == 0) begin
      errors = errors + 1;
      $display("  FAIL harness: a rendering produced no AXI write beats (A=%0d B=%0d C=%0d) -- the write path was never exercised",
               a_wbeats, b_wbeats, c_wbeats);
    end
    if (a_dones < 7 || b_dones < 7 || c_dones < 7) begin
      errors = errors + 1;
      $display("  FAIL harness: not every phase completed (done pulses A=%0d B=%0d C=%0d, want 7 each)",
               a_dones, b_dones, c_dones);
    end

    // ------------------------------------------------------------------
    // THE ANTI-VACUITY CONTROL, asserted BEFORE the null result.
    //
    // The comparator must be shown capable of reporting a difference on this
    // very stimulus, over this very signal set, in this very run. Until it
    // has, "A equals B" is not evidence of anything.
    // ------------------------------------------------------------------
    if (bc_mismatch == 0) begin
      errors = errors + 1;
      $display("  FAIL anti-vacuity: B and C compared equal. #2003 changes the READ_DATA address arm, so a working comparator MUST see a difference here. Since it does not, the A/B null result below is meaningless and this run proves nothing.");
    end

    // ------------------------------------------------------------------
    // THE NULL RESULT. The #2006 default-low is behaviourally latent: every
    // reachable path already drove the strobe, so A and B are indistinguishable
    // at the ports.
    // ------------------------------------------------------------------
    if (ab_mismatch !== 0) begin
      errors = errors + 1;
      $display("  FAIL A and B differ on %0d cycle(s), first at %0d -- #2006 was expected to be behaviourally latent",
               ab_mismatch, ab_first);
    end

    // Same work, not merely the same silence.
    expect_eq("A vs B local_we high-cycles", b_we_high, a_we_high);
    expect_eq("A vs B local writes",         b_writes,  a_writes);
    expect_eq("A vs B AXI write beats",      b_wbeats,  a_wbeats);
    expect_eq("A vs B done pulses",          b_dones,   a_dones);

    $display("");
    if (errors == 0) $display("RESULT: PASS (0 errors)");
    else             $display("RESULT: FAIL (%0d errors)", errors);
    $finish;
  end

endmodule
