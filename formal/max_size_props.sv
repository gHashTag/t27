// ============================================================================
// Maximum-sized-request sweep (Wave 578)
//
// The zero end of every count was swept in Wave 575 (Prop. 26) and found two
// real defects. This is the other end, which had never been examined.
//
// The shape looked for: a count wider than the thing it indexes. Both
// transfer engines walk a 12-bit address (4096 entries) under a much wider
// count -- `num_words` is 16 bits, `length` is 32 bits. Past 4096 the address
// wraps to zero and the transfer keeps writing, overwriting data it already
// fetched, then reports success.
//
// Each wrapper asserts the address is monotonically non-decreasing while the
// transfer is active. A wrap makes it decrease.
//   PROVES  -> the address never wraps during a transfer.
//   REFUTES -> it wraps: silent corruption of already-written data.
//
// Immediate assertions, not concurrent SVA (Yosys frontend limits, Props 2/6).
// REQUIRES `-set-assumes` (Prop 11) and `-flatten` (Prop 7).
//
// Prove with (per wrapper):
//   yosys -p "read_verilog -sv -formal build/rtl/<dut>.sv formal/max_size_props.sv; \
//             prep -top <wrapper> -flatten; async2sync; chformal -lower; \
//             sat -verify -prove-asserts -seq 24 -set-init-zero -set-assumes"
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`default_nettype none

// ---------------------------------------------------------------------------
// weight_prefetch_ctrl: num_words is 16 bits, bram_addr is 12
// ---------------------------------------------------------------------------
module ms_prefetch (
    input wire clk, input wire rst_n, input wire start_prefetch,
    input wire [31:0] src_addr, input wire [15:0] num_words,
    input wire axi_arready, input wire [63:0] axi_rdata, input wire axi_rvalid
);
    wire prefetch_active, prefetch_done, axi_arvalid, axi_rready, bram_we;
    wire [31:0] axi_araddr;
    wire [11:0] bram_addr;
    wire [53:0] bram_data;

    wire dut_overflow_wei;

    weight_prefetch_ctrl dut (
        .clk(clk), .rst_n(rst_n), .start_prefetch(start_prefetch),
        .src_addr(src_addr), .num_words(num_words),
        .prefetch_active(prefetch_active), .prefetch_done(prefetch_done),
        .axi_araddr(axi_araddr), .axi_arvalid(axi_arvalid),
        .axi_arready(axi_arready), .axi_rdata(axi_rdata),
        .axi_rvalid(axi_rvalid), .axi_rready(axi_rready),
        .bram_addr(bram_addr), .bram_data(bram_data), .bram_we(bram_we), .overflow(dut_overflow_wei)
    );

    // No WRITE may go backwards inside one prefetch. Asserting monotonicity of
    // the register itself is too strong: the counter increments once more after
    // the final write, and that last increment wraps harmlessly because nothing
    // is written at the wrapped address. The claim is about writes.
    // Transfers do not overlap: the sequencer pulses start_prefetch and waits.
    // Without this the tracker below compares an address from a NEW transfer
    // against the last address of the PREVIOUS one and refutes for a reason
    // that has nothing to do with wrapping.
    always @(posedge clk) if (rst_n) assume (!(start_prefetch && prefetch_active));

    reg        fv_wrote;
    reg [11:0] fv_last;
    always @(posedge clk)
        if (!rst_n || !prefetch_active) begin fv_wrote <= 1'b0; fv_last <= 12'd0; end
        else if (bram_we)                begin fv_wrote <= 1'b1; fv_last <= bram_addr; end

    // SUBSUMED by a_bram_writes_contiguous over 41 mutants (Wave 614,
    // Prop. 65). Expected: "strictly increasing" is implied by "increases by
    // exactly one", so the pair is a weaker and a stronger form of the same
    // claim and only the stronger can detect anything the weaker does not.
    // Kept: the weaker form is the one that states the ANTI-WRAP property in
    // the words the defect was described in, and it is the property that would
    // survive if the contiguity requirement were ever relaxed.
    always @(posedge clk)
        if (rst_n && prefetch_active && bram_we && fv_wrote)
            a_bram_addr_never_wraps: assert (bram_addr > fv_last);

    // Stronger, and the property that would have caught Prop. 29d directly:
    // writes must land on 0,1,2,... with no gap and no repeat. Monotonicity
    // alone permits skipping slot 0, which is exactly what the defect did.
    reg [11:0] fv_next;
    always @(posedge clk)
        if (!rst_n || !prefetch_active) fv_next <= 12'd0;
        else if (bram_we)               fv_next <= fv_next + 12'd1;

    always @(posedge clk)
        if (rst_n && prefetch_active && bram_we)
            a_bram_writes_contiguous: assert (bram_addr == fv_next);
endmodule

// ---------------------------------------------------------------------------
// dma_controller: length is 32 bits, local_addr is 12
// ---------------------------------------------------------------------------
module ms_dma (
    input wire clk, input wire rst_n, input wire start,
    input wire [63:0] src_addr, input wire [63:0] dst_addr,
    input wire [31:0] length, input wire direction,
    input wire m_axi_arready, input wire [63:0] m_axi_rdata,
    input wire m_axi_rlast, input wire m_axi_rvalid,
    input wire m_axi_awready, input wire m_axi_wready, input wire m_axi_bvalid,
    input wire [63:0] local_rdata
);
    wire busy, done, m_axi_arvalid, m_axi_rready, m_axi_awvalid;
    wire m_axi_wlast, m_axi_wvalid, m_axi_bready, local_we;
    wire [63:0] m_axi_araddr, m_axi_awaddr, m_axi_wdata, local_wdata;
    wire [7:0] m_axi_arlen, m_axi_awlen;
    wire [11:0] local_addr;
    wire dut_overflow_dma;

    dma_controller dut (
        .clk(clk), .rst_n(rst_n), .start(start), .src_addr(src_addr),
        .dst_addr(dst_addr), .length(length), .direction(direction),
        .busy(busy), .done(done),
        .m_axi_araddr(m_axi_araddr), .m_axi_arlen(m_axi_arlen),
        .m_axi_arvalid(m_axi_arvalid), .m_axi_arready(m_axi_arready),
        .m_axi_rdata(m_axi_rdata), .m_axi_rlast(m_axi_rlast),
        .m_axi_rvalid(m_axi_rvalid), .m_axi_rready(m_axi_rready),
        .m_axi_awaddr(m_axi_awaddr), .m_axi_awlen(m_axi_awlen),
        .m_axi_awvalid(m_axi_awvalid), .m_axi_awready(m_axi_awready),
        .m_axi_wdata(m_axi_wdata), .m_axi_wlast(m_axi_wlast),
        .m_axi_wvalid(m_axi_wvalid), .m_axi_wready(m_axi_wready),
        .m_axi_bvalid(m_axi_bvalid), .m_axi_bready(m_axi_bready),
        .local_addr(local_addr), .local_wdata(local_wdata),
        .local_we(local_we), .local_rdata(local_rdata), .overflow(dut_overflow_dma)
    );

    always @(posedge clk) if (rst_n) assume (!(start && busy));

    // A compliant slave ends every burst it accepts. Left free, the solver
    // plays a slave that never asserts rlast, so the DMA consumes beats past
    // the clamp and the address wraps -- a real hazard, but one about AXI
    // compliance rather than about sizing. Restricted here to single-beat
    // bursts (rlast on every beat), which is the weakest environment that
    // makes the sizing question answerable. Prop. 10's gate carries the full
    // slave model.
    always @(posedge clk) if (rst_n) assume (m_axi_rlast == m_axi_rvalid);

    // Direction is a mode register the host writes before it writes the start
    // bit; it does not change under a running transfer. Left free, the solver
    // flips it every cycle, and since local_addr's two roles are selected by
    // the FSM path taken at start, a mid-transfer flip makes the write and read
    // pointers interleave on one register. That is an environment fault, not a
    // design defect -- Prop. 29f.
    always @(posedge clk) if (rst_n && busy) assume (direction == $past(direction));

    reg        fv_wrote;
    reg [11:0] fv_last;
    always @(posedge clk)
        if (!rst_n || !busy)  begin fv_wrote <= 1'b0; fv_last <= 12'd0; end
        else if (local_we)    begin fv_wrote <= 1'b1; fv_last <= local_addr; end

    // SUBSUMED by a_local_writes_contiguous over 84 mutants (Wave 614,
    // Prop. 65), for the same reason as its prefetch twin above: strictly
    // increasing is implied by increases-by-one. Kept for the same reason.
    always @(posedge clk)
        if (rst_n && busy && local_we && fv_wrote)
            a_local_addr_never_wraps: assert (local_addr > fv_last);

    reg [11:0] fv_next;
    always @(posedge clk)
        if (!rst_n || !busy) fv_next <= 12'd0;
        else if (local_we)   fv_next <= fv_next + 12'd1;

    always @(posedge clk)
        if (rst_n && busy && local_we)
            a_local_writes_contiguous: assert (local_addr == fv_next);
endmodule

`default_nettype wire
