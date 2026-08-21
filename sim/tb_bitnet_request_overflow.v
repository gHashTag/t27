`timescale 1ns/1ps
// ===========================================================================
// tb_bitnet_request_overflow -- differential harness for issue #2002
// ===========================================================================
// Maximum-sized requests silently truncate: a count wider than the address it
// drives wraps the address counter and overwrites data already transferred,
// then reports success.
//
//   dma_controller       length[31:0]     -> local_addr[11:0]
//   weight_prefetch_ctrl num_words[15:0]  -> bram_addr[11:0]
//
// This harness elaborates BOTH the pre-fix and the post-fix emitter output in
// one simulation, drives them from identical stimulus, and compares them. The
// two variants differ only in the `module_name` passed to the emitter, so the
// comparison is between two renderings of the same design, not two designs.
//
// Build (see sim/README.md for the emit step):
//   iverilog -g2005 -o tb.vvp sim/tb_bitnet_request_overflow.v \
//            dma_old.v dma_new.v pf_old.v pf_new.v
//   vvp tb.vvp
//
// The measured property is "no local address is written twice within one
// transfer". That is deliberately independent of the separate off-by-one in
// weight_prefetch_ctrl (word N lands at address N+1), which this change does
// NOT fix and which shifts the address set without duplicating it.
//
// The AXI read slave holds `rlast` low throughout and streams beats forever,
// so termination is decided purely by the DUT's own byte/word counter. That
// isolates the counter-to-address path under test; the separate question of
// `arlen` being hardwired to 8'hFF regardless of `length` is issue #1970 and
// is not exercised here.
// ===========================================================================

module tb_bitnet_request_overflow;

  // Local memory geometry, from the emitters' own port widths.
  localparam integer CAP_WORDS = 4096;          // 12-bit address
  localparam integer CAP_BYTES = CAP_WORDS * 8; // 8 bytes per DMA beat

  reg clk = 0, rst_n = 0;
  always #5 clk = ~clk;

  integer errors = 0;

  // -----------------------------------------------------------------------
  // DMA controller: old and new
  // -----------------------------------------------------------------------
  reg         d_start = 0;
  reg  [31:0] d_length = 0;

  wire [11:0] o_addr, n_addr;
  wire [63:0] o_wdata, n_wdata;
  wire        o_we, n_we, o_busy, n_busy, o_done, n_done;
  wire        n_overflow;
  wire [63:0] o_araddr, n_araddr, o_awaddr, n_awaddr;
  wire [7:0]  o_arlen, n_arlen, o_awlen, n_awlen;
  wire        o_arvalid, n_arvalid, o_rready, n_rready;
  wire        o_awvalid, n_awvalid, o_wlast, n_wlast, o_wvalid, n_wvalid;
  wire [63:0] o_wdata_axi, n_wdata_axi;

  // Each DUT gets its own beat counter so the data it captures is its own
  // beat index; the two never share a counter.
  reg [63:0] o_beats = 0, n_beats = 0;
  always @(posedge clk) if (!rst_n) o_beats <= 0; else if (o_rready) o_beats <= o_beats + 1;
  always @(posedge clk) if (!rst_n) n_beats <= 0; else if (n_rready) n_beats <= n_beats + 1;

  dma_old u_dma_old (
    .clk(clk), .rst_n(rst_n),
    .start(d_start), .src_addr(64'd0), .dst_addr(64'd0),
    .length(d_length), .direction(1'b0),
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
    .length(d_length), .direction(1'b0),
    .busy(n_busy), .done(n_done), .overflow(n_overflow),
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
  // Weight prefetch controller: old and new
  // -----------------------------------------------------------------------
  reg         p_start = 0;
  reg  [15:0] p_words = 0;

  wire [11:0] po_addr, pn_addr;
  wire [53:0] po_data, pn_data;
  wire        po_we, pn_we, po_active, pn_active, po_done, pn_done;
  wire        pn_overflow;
  wire [31:0] po_araddr, pn_araddr;
  wire        po_arvalid, pn_arvalid, po_rready, pn_rready;

  pf_old u_pf_old (
    .clk(clk), .rst_n(rst_n),
    .start_prefetch(p_start), .src_addr(32'd0), .num_words(p_words),
    .prefetch_active(po_active), .prefetch_done(po_done),
    .axi_araddr(po_araddr), .axi_arvalid(po_arvalid), .axi_arready(1'b1),
    .axi_rdata(64'hDEAD_0000_0000_0000), .axi_rvalid(rst_n), .axi_rready(po_rready),
    .bram_addr(po_addr), .bram_data(po_data), .bram_we(po_we)
  );

  pf_new u_pf_new (
    .clk(clk), .rst_n(rst_n),
    .start_prefetch(p_start), .src_addr(32'd0), .num_words(p_words),
    .prefetch_active(pn_active), .prefetch_done(pn_done), .overflow(pn_overflow),
    .axi_araddr(pn_araddr), .axi_arvalid(pn_arvalid), .axi_arready(1'b1),
    .axi_rdata(64'hDEAD_0000_0000_0000), .axi_rvalid(rst_n), .axi_rready(pn_rready),
    .bram_addr(pn_addr), .bram_data(pn_data), .bram_we(pn_we)
  );

  // -----------------------------------------------------------------------
  // Per-address write counters
  // -----------------------------------------------------------------------
  integer oc [0:4095];
  integer nc [0:4095];
  integer poc [0:4095];
  integer pnc [0:4095];

  integer o_w, n_w, po_w, pn_w;          // total writes
  integer o_max, n_max, po_max, pn_max;  // max writes to any one address
  integer o_dup, n_dup, po_dup, pn_dup;  // first duplicated address, -1 = none
  integer i;

  task reset_counters;
    begin
      for (i = 0; i < CAP_WORDS; i = i + 1) begin
        oc[i] = 0; nc[i] = 0; poc[i] = 0; pnc[i] = 0;
      end
      o_w = 0; n_w = 0; po_w = 0; pn_w = 0;
      o_max = 0; n_max = 0; po_max = 0; pn_max = 0;
      o_dup = -1; n_dup = -1; po_dup = -1; pn_dup = -1;
    end
  endtask

  always @(posedge clk) if (rst_n && o_we) begin
    oc[o_addr] = oc[o_addr] + 1; o_w = o_w + 1;
    if (oc[o_addr] > o_max) o_max = oc[o_addr];
    if (oc[o_addr] == 2 && o_dup < 0) o_dup = o_addr;
  end
  always @(posedge clk) if (rst_n && n_we) begin
    nc[n_addr] = nc[n_addr] + 1; n_w = n_w + 1;
    if (nc[n_addr] > n_max) n_max = nc[n_addr];
    if (nc[n_addr] == 2 && n_dup < 0) n_dup = n_addr;
  end
  always @(posedge clk) if (rst_n && po_we) begin
    poc[po_addr] = poc[po_addr] + 1; po_w = po_w + 1;
    if (poc[po_addr] > po_max) po_max = poc[po_addr];
    if (poc[po_addr] == 2 && po_dup < 0) po_dup = po_addr;
  end
  always @(posedge clk) if (rst_n && pn_we) begin
    pnc[pn_addr] = pnc[pn_addr] + 1; pn_w = pn_w + 1;
    if (pnc[pn_addr] > pn_max) pn_max = pnc[pn_addr];
    if (pnc[pn_addr] == 2 && pn_dup < 0) pn_dup = pn_addr;
  end

  task expect_eq(input [255:0] what, input integer got, input integer want);
    begin
      if (got !== want) begin
        errors = errors + 1;
        $display("  FAIL %0s: got %0d, want %0d", what, got, want);
      end
    end
  endtask

  // -----------------------------------------------------------------------
  // Stimulus
  // -----------------------------------------------------------------------
  initial begin
    reset_counters;
    #20 rst_n = 1;
    @(posedge clk);

    // ------------------------------------------------------------------
    // CASE 1 -- oversized: capacity + 1
    // ------------------------------------------------------------------
    d_length = CAP_BYTES + 8;   // 32776 bytes = 4097 beats
    p_words  = CAP_WORDS + 1;   // 4097 words
    d_start = 1; p_start = 1;
    @(posedge clk);
    d_start = 0; p_start = 0;
    repeat (9000) @(posedge clk);

    $display("== CASE 1: oversized request (capacity + 1) ==");
    $display("  DMA  length=%0d bytes (%0d beats), capacity %0d beats",
             d_length, d_length/8, CAP_WORDS);
    $display("    old: writes=%0d max_writes_per_addr=%0d first_dup_addr=%0d done=%b",
             o_w, o_max, o_dup, o_done);
    $display("    new: writes=%0d max_writes_per_addr=%0d first_dup_addr=%0d done=%b overflow=%b",
             n_w, n_max, n_dup, n_done, n_overflow);
    $display("  PREFETCH num_words=%0d, capacity %0d words", p_words, CAP_WORDS);
    $display("    old: writes=%0d max_writes_per_addr=%0d first_dup_addr=%0d",
             po_w, po_max, po_dup);
    $display("    new: writes=%0d max_writes_per_addr=%0d first_dup_addr=%0d overflow=%b",
             pn_w, pn_max, pn_dup, pn_overflow);

    // The defect must still be present in the old emitter, or this harness
    // is not measuring what it claims to measure.
    if (o_max < 2) begin
      errors = errors + 1;
      $display("  FAIL harness: old DMA did not overwrite any address -- the "
             , "harness never reached the wrap and proves nothing");
    end
    if (po_max < 2) begin
      errors = errors + 1;
      $display("  FAIL harness: old prefetch did not overwrite any address -- the "
             , "harness never reached the wrap and proves nothing");
    end

    // The fix: bounded AND observable.
    expect_eq("new DMA max writes per address",      n_max,  1);
    expect_eq("new DMA writes",                      n_w,    CAP_WORDS);
    expect_eq("new prefetch max writes per address", pn_max, 1);
    expect_eq("new prefetch writes",                 pn_w,   CAP_WORDS);
    if (n_overflow !== 1'b1) begin
      errors = errors + 1;
      $display("  FAIL new DMA clamped but did not raise overflow: a short "
             , "transfer that reports done=1 is still silent");
    end
    if (pn_overflow !== 1'b1) begin
      errors = errors + 1;
      $display("  FAIL new prefetch clamped but did not raise overflow: a short "
             , "transfer is still silent");
    end

    // ------------------------------------------------------------------
    // CASE 2 -- control: an ordinary in-range request must be unchanged
    // ------------------------------------------------------------------
    rst_n = 0; repeat (4) @(posedge clk);
    reset_counters;
    rst_n = 1; @(posedge clk);

    d_length = 32'd256;   // 32 beats
    p_words  = 16'd32;    // 32 words
    d_start = 1; p_start = 1;
    @(posedge clk);
    d_start = 0; p_start = 0;
    repeat (200) @(posedge clk);

    $display("== CASE 2: control, in-range request ==");
    $display("  DMA      old writes=%0d done=%b   new writes=%0d done=%b overflow=%b",
             o_w, o_done, n_w, n_done, n_overflow);
    $display("  PREFETCH old writes=%0d          new writes=%0d overflow=%b",
             po_w, pn_w, pn_overflow);

    expect_eq("control DMA writes old vs new",      n_w,  o_w);
    expect_eq("control prefetch writes old vs new", pn_w, po_w);
    for (i = 0; i < CAP_WORDS; i = i + 1) begin
      if (oc[i] !== nc[i]) begin
        errors = errors + 1;
        $display("  FAIL control DMA address %0d: old written %0d times, new %0d",
                 i, oc[i], nc[i]);
      end
      if (poc[i] !== pnc[i]) begin
        errors = errors + 1;
        $display("  FAIL control prefetch address %0d: old written %0d times, new %0d",
                 i, poc[i], pnc[i]);
      end
    end
    if (n_overflow !== 1'b0) begin
      errors = errors + 1;
      $display("  FAIL in-range request raised overflow");
    end
    if (pn_overflow !== 1'b0) begin
      errors = errors + 1;
      $display("  FAIL in-range prefetch raised overflow");
    end

    // ------------------------------------------------------------------
    // CASE 3 -- boundary: exactly capacity must NOT be flagged
    // ------------------------------------------------------------------
    rst_n = 0; repeat (4) @(posedge clk);
    reset_counters;
    rst_n = 1; @(posedge clk);

    d_length = CAP_BYTES;    // exactly 4096 beats
    p_words  = CAP_WORDS;    // exactly 4096 words
    d_start = 1; p_start = 1;
    @(posedge clk);
    d_start = 0; p_start = 0;
    repeat (9000) @(posedge clk);

    $display("== CASE 3: boundary, exactly capacity ==");
    $display("  DMA      new writes=%0d max_per_addr=%0d overflow=%b",
             n_w, n_max, n_overflow);
    $display("  PREFETCH new writes=%0d max_per_addr=%0d overflow=%b",
             pn_w, pn_max, pn_overflow);
    expect_eq("boundary DMA writes",      n_w,  CAP_WORDS);
    expect_eq("boundary prefetch writes", pn_w, CAP_WORDS);
    if (n_overflow !== 1'b0) begin
      errors = errors + 1;
      $display("  FAIL exactly-capacity DMA request flagged as overflow");
    end
    if (pn_overflow !== 1'b0) begin
      errors = errors + 1;
      $display("  FAIL exactly-capacity prefetch request flagged as overflow");
    end

    $display("");
    if (errors == 0) $display("RESULT: PASS (0 errors)");
    else             $display("RESULT: FAIL (%0d errors)", errors);
    $finish;
  end

endmodule
