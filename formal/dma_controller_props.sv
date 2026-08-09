// ============================================================================
// Formal properties for `dma_controller` (BitNet HLS, W36e / R-BN-5)
//
// Immediate assertions, not concurrent SVA (Yosys frontend limits: see
// docs/FORMAL_FOUNDATIONS.md Props. 2, 6). Two of these are regression
// witnesses for real defects fixed on 2026-08-09 -- Prop. 9.
//
// Prove with:
//   yosys -p "read_verilog -sv -formal dma_controller.sv \
//             formal/dma_controller_props.sv; \
//             prep -top dma_props -flatten; async2sync; chformal -lower; \
//             sat -verify -prove-asserts -seq 12 -set-init-zero -set-assumes"
//
// -set-init-zero rather than -tempinduct: induction from an unconstrained
// initial state refutes properties that hold on every reachable state.
// See Prop. 8c -- a refutation is only evidence of a bug if the
// counterexample state is reachable.
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`default_nettype none
module dma_props (
    input wire clk, input wire rst_n, input wire start,
    input wire [63:0] src_addr, input wire [63:0] dst_addr,
    input wire [31:0] length, input wire direction,
    input wire arready, input wire [63:0] rdata, input wire rlast, input wire rvalid,
    input wire awready, input wire wready, input wire bvalid,
    input wire [63:0] local_rdata
);
    wire busy, done, arvalid, awvalid, wvalid, wlast, rready, bready, local_we;
    wire [63:0] araddr, awaddr, wdata, local_wdata;
    wire [7:0] arlen, awlen;
    wire [11:0] local_addr;
    wire dut_overflow_dma;
    dma_controller dut (
        .clk(clk), .rst_n(rst_n), .start(start),
        .src_addr(src_addr), .dst_addr(dst_addr), .length(length), .direction(direction),
        .busy(busy), .done(done),
        .m_axi_araddr(araddr), .m_axi_arlen(arlen), .m_axi_arvalid(arvalid),
        .m_axi_arready(arready), .m_axi_rdata(rdata), .m_axi_rlast(rlast),
        .m_axi_rvalid(rvalid), .m_axi_rready(rready),
        .m_axi_awaddr(awaddr), .m_axi_awlen(awlen), .m_axi_awvalid(awvalid),
        .m_axi_awready(awready), .m_axi_wdata(wdata), .m_axi_wlast(wlast),
        .m_axi_wvalid(wvalid), .m_axi_wready(wready), .m_axi_bvalid(bvalid),
        .m_axi_bready(bready),
        .local_addr(local_addr), .local_wdata(local_wdata), .local_we(local_we), .overflow(dut_overflow_dma),
        .local_rdata(local_rdata)
    );

    // Harness sanity: a tautology. If this is refuted the run is not
    // evaluating what it appears to (see the -flatten trap, Prop. 7).
    // a_sanity was removed in Wave 591. Its body was `X == X`, which the
    // optimiser folds to constant true before any signal is read: it proved
    // unconditionally and tested nothing. Worse, it still emitted a $check
    // cell, so it inflated the non-empty-property gate (Prop. 5) that exists to
    // catch exactly an all-vacuous set. See Prop. 41.

    // AXI: VALID must not drop without a handshake. These held on the
    // pre-fix RTL too and are kept to bound what the defects were *not*.
    //
    // Detection verdicts over 84 mechanical mutants (Wave 613, Prop. 64):
    //   a_arvalid_stable  SUBSUMED by a_rready_implies_burst -- kept, it states
    //                     the AXI handshake rule in the specification's own form
    //   a_awvalid_stable  BITES, and uniquely: one mutant nothing else catches.
    //                     Its read-side twin is subsumed and its write-side twin
    //                     is not, which is worth noticing -- symmetric-looking
    //                     properties need not have symmetric detection power
    //   a_wvalid_stable   detects nothing, and is INNOCENT rather than weak: its
    //                     guard is in the always header, so the 4 mutants that
    //                     could violate it instead make `$past(wvalid) &&
    //                     !$past(wready)` unreachable and it proves vacuously.
    //                     Measured, not assumed -- see Prop. 61d for the
    //                     mechanism and Prop. 64 for the sweep that confirmed it
    always @(posedge clk) if (rst_n && $past(rst_n) && $past(arvalid) && !$past(arready))
        a_arvalid_stable: assert (arvalid);
    always @(posedge clk) if (rst_n && $past(rst_n) && $past(awvalid) && !$past(awready))
        a_awvalid_stable: assert (awvalid);
    always @(posedge clk) if (rst_n && $past(rst_n) && $past(wvalid) && !$past(wready))
        a_wvalid_stable: assert (wvalid);

    // REGRESSION WITNESS 1 -- burst abandonment.
    // arlen/awlen were hardwired to 8'hFF (256 beats) for every transfer while
    // the FSM stopped once bytes_remaining fell to one beat. A short transfer
    // therefore requested 256 beats and then dropped rready mid-burst, which an
    // AXI4 master may not do. Refuted from a reachable state before the fix.
    // SUBSUMED by a_rready_implies_burst over the same 84 mutants (Wave 613,
    // Prop. 64) and kept: this is the regression witness for the defect Prop. 9
    // fixed, and a suite that deletes its regression witnesses because a newer
    // property happens to cover them loses the record of what went wrong.
    always @(posedge clk) if (rst_n && $past(rst_n) && $past(rready) && $past(rvalid) && !$past(rlast))
        a_read_burst_not_abandoned: assert (rready);

    // Minimal AXI4 read-slave model: a burst exists only after a genuine
    // address handshake. Needed for the next property to mean anything --
    // with rvalid free, a misbehaving slave is indistinguishable from a
    // master defect.
    reg burst_active;
    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) burst_active <= 1'b0;
        else if (arvalid && arready) burst_active <= 1'b1;
        else if (burst_active && rvalid && rready && rlast) burst_active <= 1'b0;
    end
    always @(posedge clk) if (rst_n) assume (!rvalid || burst_active);

    // REGRESSION WITNESS 2 -- ready without valid.
    // READ_ADDR advanced on `if (m_axi_arready)` alone, so a ready asserted
    // while arvalid was still low moved the FSM into READ_DATA having issued
    // no address: the master sat ready for a burst nobody owed it.
    always @(posedge clk) if (rst_n)
        a_rready_implies_burst: assert (!rready || burst_active);

    // A zero-length request moves no data. This held before the fix too --
    // recorded so the guard added alongside it is not mistaken for a bug fix.
    always @(posedge clk) if (rst_n && $past(rst_n) && $past(start) && $past(length) == 32'd0 && !$past(busy))
        a_zero_length_moves_nothing: assert (!local_we);

endmodule

`default_nettype wire
