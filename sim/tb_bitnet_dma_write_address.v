`timescale 1ns/1ps
// ===========================================================================
// tb_bitnet_dma_write_address -- differential harness for issue #2003
// ===========================================================================
// `dma_controller` presented the local write address one beat ahead of the
// data it belonged to. In READ_DATA the pre-fix emitter raised the write
// strobe and post-incremented the address in the SAME non-blocking group:
//
//     local_wdata     <= m_axi_rdata;
//     local_we        <= 1'b1;
//     local_addr      <= local_addr + 12'd1;   // <-- lands with the strobe
//
// so beat 0's data was written at address 1, beat 1's at address 2, and
// address 0 was never written at all. A four-beat transfer filled 1..4
// instead of 0..3: slot 0 kept whatever it held before, and slot 4 -- which
// belongs to the next word -- was clobbered.
//
// The fix gives the beat being captured its own index register and drives
// `local_addr` from it, so address, data and enable are registered from one
// stage:
//
//     local_addr      <= beat_index;
//     beat_index      <= beat_index + 12'd1;
//
// A later wave renamed that register `word_index` and gave the WRITE_DATA arm
// its own use of it. Nothing here depends on the name: every check below is
// made at the module PORTS, so this harness runs unmodified against the
// as-merged rendering and against current master. Both are recorded in
// sim/README.md.
//
// This harness elaborates BOTH the pre-fix and the post-fix emitter output in
// one simulation, drives them from identical stimulus, and compares them. The
// two variants differ only in the `module_name` passed to the emitter, so the
// comparison is between two renderings of the same design, not two designs.
//
// Build (see sim/README.md for the emit step):
//   iverilog -g2005 -o tb.vvp sim/tb_bitnet_dma_write_address.v \
//            dma_old.v dma_new.v
//   vvp tb.vvp
//
// The measured property is "beat N is written at local address N, and no
// address outside 0..N-1 is written". It is checked on the ADDRESS AND THE
// DATA together: an address-only check passes for a controller that writes
// the right slots in the wrong order.
//
// The AXI read slave holds `rlast` low throughout and streams beats forever,
// so termination is decided purely by the DUT's own byte counter. That
// isolates the counter-to-address path under test; `arlen` being hardwired to
// 8'hFF regardless of `length` is issue #1970 and is not exercised here.
// ===========================================================================

module tb_bitnet_dma_write_address;

  // Local memory geometry, from the emitter's own port widths.
  localparam integer CAP_WORDS = 4096;   // 12-bit address
  localparam integer BEATS     = 4;      // beats in the transfer under test
  localparam integer XFER_BYTES = BEATS * 8;

  reg clk = 0, rst_n = 0;
  always #5 clk = ~clk;

  integer errors = 0;

  reg         d_start = 0;
  reg         d_dir   = 0;   // 0 = read (DDR->local), 1 = write
  reg  [31:0] d_length = 0;

  wire [11:0] o_addr, n_addr;
  wire [63:0] o_wdata, n_wdata;
  wire        o_we, n_we, o_busy, n_busy, o_done, n_done;
  wire [63:0] o_araddr, n_araddr, o_awaddr, n_awaddr;
  wire [7:0]  o_arlen, n_arlen, o_awlen, n_awlen;
  wire        o_arvalid, n_arvalid, o_rready, n_rready;
  wire        o_awvalid, n_awvalid, o_wlast, n_wlast, o_wvalid, n_wvalid;
  wire [63:0] o_wdata_axi, n_wdata_axi;

  // Each DUT gets its own beat counter, so the payload it captures is its own
  // beat index and the two never share a counter. Beat K carries the value K.
  //
  // `beats_clr` restarts the numbering between transfers, and is pulsed off the
  // NEGEDGE on purpose: driven from the posedge it races the sampling
  // always-block at the same timestep, so the stimulus can deassert it before
  // the counter sees it and the clear is silently lost.
  reg beats_clr = 0;
  reg [63:0] o_beats = 0, n_beats = 0;
  always @(posedge clk) if (!rst_n || beats_clr) o_beats <= 0; else if (o_rready) o_beats <= o_beats + 1;
  always @(posedge clk) if (!rst_n || beats_clr) n_beats <= 0; else if (n_rready) n_beats <= n_beats + 1;

  dma_old u_dma_old (
    .clk(clk), .rst_n(rst_n),
    .start(d_start), .src_addr(64'd0), .dst_addr(64'd0),
    .length(d_length), .direction(d_dir),
    .busy(o_busy), .done(o_done),
    .m_axi_araddr(o_araddr), .m_axi_arlen(o_arlen), .m_axi_arvalid(o_arvalid),
    .m_axi_arready(1'b1),
    .m_axi_rdata(o_beats), .m_axi_rlast(1'b0), .m_axi_rvalid(rst_n),
    .m_axi_rready(o_rready),
    .m_axi_awaddr(o_awaddr), .m_axi_awlen(o_awlen), .m_axi_awvalid(o_awvalid),
    .m_axi_awready(1'b1),
    .m_axi_wdata(o_wdata_axi), .m_axi_wlast(o_wlast), .m_axi_wvalid(o_wvalid),
    .m_axi_wready(1'b1), .m_axi_bvalid(1'b1), .m_axi_bready(),
    .local_addr(o_addr), .local_wdata(o_wdata), .local_we(o_we),
    .local_rdata(64'd0)
  );

  dma_new u_dma_new (
    .clk(clk), .rst_n(rst_n),
    .start(d_start), .src_addr(64'd0), .dst_addr(64'd0),
    .length(d_length), .direction(d_dir),
    .busy(n_busy), .done(n_done),
    .m_axi_araddr(n_araddr), .m_axi_arlen(n_arlen), .m_axi_arvalid(n_arvalid),
    .m_axi_arready(1'b1),
    .m_axi_rdata(n_beats), .m_axi_rlast(1'b0), .m_axi_rvalid(rst_n),
    .m_axi_rready(n_rready),
    .m_axi_awaddr(n_awaddr), .m_axi_awlen(n_awlen), .m_axi_awvalid(n_awvalid),
    .m_axi_awready(1'b1),
    .m_axi_wdata(n_wdata_axi), .m_axi_wlast(n_wlast), .m_axi_wvalid(n_wvalid),
    .m_axi_wready(1'b1), .m_axi_bvalid(1'b1), .m_axi_bready(),
    .local_addr(n_addr), .local_wdata(n_wdata), .local_we(n_we),
    .local_rdata(64'd0)
  );

  // -----------------------------------------------------------------------
  // Local memory models.
  //
  // `local_addr`, `local_wdata` and `local_we` are all registered outputs, so
  // sampling them at posedge is what a synchronous local memory actually
  // sees. `mem` records the payload, `cnt` how many times each slot was
  // written -- an unwritten slot must stay distinguishable from one written
  // with the value 0, hence the separate counter.
  // -----------------------------------------------------------------------
  reg [63:0] o_mem [0:CAP_WORDS-1];
  reg [63:0] n_mem [0:CAP_WORDS-1];
  integer    o_cnt [0:CAP_WORDS-1];
  integer    n_cnt [0:CAP_WORDS-1];

  integer o_w, n_w;          // total writes
  integer o_lo, o_hi;        // lowest / highest address written, -1 = none
  integer n_lo, n_hi;
  integer i;

  task reset_memories;
    begin
      for (i = 0; i < CAP_WORDS; i = i + 1) begin
        o_cnt[i] = 0; n_cnt[i] = 0;
        o_mem[i] = 64'hDEAD_BEEF_DEAD_BEEF;
        n_mem[i] = 64'hDEAD_BEEF_DEAD_BEEF;
      end
      o_w = 0; n_w = 0;
      o_lo = -1; o_hi = -1; n_lo = -1; n_hi = -1;
    end
  endtask

  // `local_addr` is an unsigned 12-bit port and `o_hi` an integer seeded to
  // -1. Comparing them directly promotes the whole expression to unsigned, so
  // `o_addr > -1` is FALSE and the high-water mark never advances. Copy the
  // port into an integer first so the comparison stays signed.
  integer o_a, n_a;
  always @(posedge clk) if (rst_n && o_we) begin
    o_a = o_addr;
    o_mem[o_a] = o_wdata;
    o_cnt[o_a] = o_cnt[o_a] + 1;
    o_w = o_w + 1;
    if (o_lo < 0 || o_a < o_lo) o_lo = o_a;
    if (o_a > o_hi) o_hi = o_a;
  end
  always @(posedge clk) if (rst_n && n_we) begin
    n_a = n_addr;
    n_mem[n_a] = n_wdata;
    n_cnt[n_a] = n_cnt[n_a] + 1;
    n_w = n_w + 1;
    if (n_lo < 0 || n_a < n_lo) n_lo = n_a;
    if (n_a > n_hi) n_hi = n_a;
  end

  // AXI write-side beat counters, used by the control case: on the write path
  // the DUT never raises `local_we`, so a local-write comparison there would
  // be vacuous (0 == 0). Count what the write path actually produces.
  integer o_wb = 0, n_wb = 0;
  always @(posedge clk) if (!rst_n) o_wb = 0; else if (o_wvalid) o_wb = o_wb + 1;
  always @(posedge clk) if (!rst_n) n_wb = 0; else if (n_wvalid) n_wb = n_wb + 1;

  task expect_eq(input [511:0] what, input integer got, input integer want);
    begin
      if (got !== want) begin
        errors = errors + 1;
        $display("  FAIL %0s: got %0d, want %0d", what, got, want);
      end
    end
  endtask

  // Print the written address set compactly, e.g. "1,2,3,4".
  task show_writes(input [511:0] tag, input integer which);
    integer k, first;
    begin
      $write("    %0s wrote:", tag);
      first = 1;
      for (k = 0; k < 16; k = k + 1) begin
        if ((which == 0 ? o_cnt[k] : n_cnt[k]) != 0) begin
          if (first) begin $write(" "); first = 0; end else $write(",");
          $write("%0d(=%0d)", k, (which == 0 ? o_mem[k] : n_mem[k]));
        end
      end
      if (first) $write(" nothing");
      $write("\n");
    end
  endtask

  // All stimulus changes on the NEGEDGE, so `start` is stable across exactly
  // one posedge no matter what phase the caller happens to be in. Driving it
  // from the posedge instead makes the pulse race the DUT's own sampling: the
  // stimulus can clear `start` at the same timestep the DUT reads it, and the
  // transfer silently never begins.
  task run_transfer(input integer settle);
    begin
      @(negedge clk);
      d_start = 1;
      @(negedge clk);
      d_start = 0;
      repeat (settle) @(posedge clk);
    end
  endtask

  // -----------------------------------------------------------------------
  // Stimulus
  // -----------------------------------------------------------------------
  initial begin
    reset_memories;
    #20 rst_n = 1;
    @(posedge clk);

    // ------------------------------------------------------------------
    // CASE 1 -- the reported defect: a four-beat read transfer
    // ------------------------------------------------------------------
    d_length = XFER_BYTES;
    run_transfer(60);

    $display("== CASE 1: %0d-beat read transfer (length=%0d bytes) ==",
             BEATS, XFER_BYTES);
    $display("    old: writes=%0d addr_range=%0d..%0d done=%b",
             o_w, o_lo, o_hi, o_done);
    show_writes("old", 0);
    $display("    new: writes=%0d addr_range=%0d..%0d done=%b",
             n_w, n_lo, n_hi, n_done);
    show_writes("new", 1);

    // Both renderings must actually have run. Every "address 0 untouched"
    // claim below is vacuous if the transfer never happened.
    expect_eq("old beats written", o_w, BEATS);
    expect_eq("new beats written", n_w, BEATS);
    if (o_done !== 1'b1) begin
      errors = errors + 1;
      $display("  FAIL old never reported done -- harness did not complete a transfer");
    end
    if (n_done !== 1'b1) begin
      errors = errors + 1;
      $display("  FAIL new never reported done -- harness did not complete a transfer");
    end

    // The defect must still be present in the OLD rendering, or this harness
    // is not measuring what it claims to measure.
    if (o_cnt[0] !== 0) begin
      errors = errors + 1;
      $display("  FAIL harness: old wrote address 0 -- the off-by-one this "
             , "harness exists to demonstrate is not reproducing");
    end
    if (o_lo !== 1 || o_hi !== BEATS) begin
      errors = errors + 1;
      $display("  FAIL harness: old wrote %0d..%0d, expected the shifted window %0d..%0d",
               o_lo, o_hi, 1, BEATS);
    end

    // The fix: beat K lands at address K, and nothing outside 0..BEATS-1 is
    // touched. Address AND payload, so a right-slots-wrong-order controller
    // cannot pass.
    expect_eq("new lowest address written",  n_lo, 0);
    expect_eq("new highest address written", n_hi, BEATS - 1);
    for (i = 0; i < BEATS; i = i + 1) begin
      if (n_cnt[i] !== 1) begin
        errors = errors + 1;
        $display("  FAIL new address %0d written %0d times, want exactly 1", i, n_cnt[i]);
      end
      if (n_mem[i] !== i) begin
        errors = errors + 1;
        $display("  FAIL new address %0d holds %0d, want beat %0d", i, n_mem[i], i);
      end
    end
    // Slot BEATS belongs to the next word. Old clobbers it; new must not.
    if (n_cnt[BEATS] !== 0) begin
      errors = errors + 1;
      $display("  FAIL new wrote address %0d, one past the transfer", BEATS);
    end
    if (o_cnt[BEATS] === 0) begin
      errors = errors + 1;
      $display("  FAIL harness: old did not clobber address %0d -- expected overrun absent",
               BEATS);
    end

    // ------------------------------------------------------------------
    // CASE 2 -- re-arm: a SECOND transfer with no reset in between.
    //
    // The fix clears `beat_index` in the IDLE start branch as well as at
    // reset. Without that clear the index survives the first transfer and the
    // second one starts writing where the first stopped. `local_addr` is
    // cleared in IDLE by both renderings, so this case is blind to the
    // READ_DATA hunk and sees only the re-arm.
    // ------------------------------------------------------------------
    reset_memories;
    // Restart the payload numbering too, so beat K of THIS transfer again
    // carries the value K. Without this the tb's own counters run on from the
    // first transfer and the expected payloads would be 4..7 -- an artefact of
    // the harness, not of either rendering.
    @(negedge clk); beats_clr = 1; @(negedge clk); beats_clr = 0;
    d_length = XFER_BYTES;
    run_transfer(60);

    $display("== CASE 2: second transfer, no reset between ==");
    $display("    old: writes=%0d addr_range=%0d..%0d", o_w, o_lo, o_hi);
    show_writes("old", 0);
    $display("    new: writes=%0d addr_range=%0d..%0d", n_w, n_lo, n_hi);
    show_writes("new", 1);

    expect_eq("new writes on second transfer",          n_w,  BEATS);
    expect_eq("new lowest address on second transfer",  n_lo, 0);
    expect_eq("new highest address on second transfer", n_hi, BEATS - 1);
    for (i = 0; i < BEATS; i = i + 1) begin
      if (n_mem[i] !== i) begin
        errors = errors + 1;
        $display("  FAIL second transfer: new address %0d holds %0d, want beat %0d",
                 i, n_mem[i], i);
      end
    end

    // ------------------------------------------------------------------
    // CASE 3 -- control: the change must not reach the write path.
    //
    // With direction=1 the FSM goes IDLE -> WRITE_ADDR -> WRITE_DATA and never
    // enters READ_DATA, the only arm PR #2345 touched. Old and new must be
    // observationally identical here. `local_we` is never raised on this path,
    // so asserting "same local writes" alone would be 0 == 0 and prove
    // nothing; the AXI write beats and the local READ pointer are compared
    // instead, and the zero-write claim is stated separately.
    // ------------------------------------------------------------------
    rst_n = 0; repeat (4) @(posedge clk);
    reset_memories;
    rst_n = 1; @(posedge clk);
    @(negedge clk); beats_clr = 1; @(negedge clk); beats_clr = 0;

    d_dir    = 1;
    d_length = XFER_BYTES;
    run_transfer(60);

    $display("== CASE 3: control -- write direction, READ_DATA never entered ==");
    $display("    old: axi_write_beats=%0d local_writes=%0d final_local_addr=%0d done=%b",
             o_wb, o_w, o_addr, o_done);
    $display("    new: axi_write_beats=%0d local_writes=%0d final_local_addr=%0d done=%b",
             n_wb, n_w, n_addr, n_done);

    // The control must actually have exercised the write path.
    if (o_wb == 0 || n_wb == 0) begin
      errors = errors + 1;
      $display("  FAIL harness: control produced no AXI write beats -- the "
             , "write path was never entered and the control proves nothing");
    end
    expect_eq("control: same AXI write beats",   n_wb, o_wb);
    expect_eq("control: old local writes",       o_w,  0);
    expect_eq("control: new local writes",       n_w,  0);
    if (o_addr !== n_addr) begin
      errors = errors + 1;
      $display("  FAIL control: local read pointer diverged, old=%0d new=%0d",
               o_addr, n_addr);
    end
    if (o_done !== n_done) begin
      errors = errors + 1;
      $display("  FAIL control: done diverged, old=%b new=%b", o_done, n_done);
    end

    $display("");
    if (errors == 0) $display("RESULT: PASS (0 errors)");
    else             $display("RESULT: FAIL (%0d errors)", errors);
    $finish;
  end

endmodule
