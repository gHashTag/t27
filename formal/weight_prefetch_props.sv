// ============================================================================
// Formal properties for `weight_prefetch_ctrl` (BitNet HLS, W36c / R-BN-3)
//
// Immediate assertions (Props 2/6). REQUIRES `-set-assumes` (Prop 11) and
// `-flatten` (Prop 7).
//
// a_no_overwrite is a regression witness for a real defect fixed 2026-08-09:
// with num_words == 0, words_remaining underflowed to 16'hFFFF on the first
// beat, the `words_remaining == 1` terminator never matched, and the
// controller wrote BRAM indefinitely -- past the 4096-entry buffer and past
// anything the caller asked for. See FORMAL_FOUNDATIONS Prop. 13.
//
// Prove with:
//   yosys -p "read_verilog -sv -formal weight_prefetch_ctrl.sv \
//             formal/weight_prefetch_props.sv; \
//             prep -top wp_props -flatten; async2sync; chformal -lower; \
//             sat -verify -prove-asserts -seq 14 -set-init-zero -set-assumes"
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`default_nettype none

module wp_props (
    input wire        clk,
    input wire        rst_n,
    input wire        start_prefetch,
    input wire [31:0] src_addr,
    input wire [15:0] num_words,
    input wire        arready,
    input wire [63:0] rdata,
    input wire        rvalid
);
    wire        prefetch_active, prefetch_done, arvalid, rready, bram_we;
    wire [31:0] araddr;
    wire [11:0] bram_addr;
    wire [53:0] bram_data;

    wire dut_overflow_wei;

    weight_prefetch_ctrl dut (
        .clk(clk), .rst_n(rst_n), .start_prefetch(start_prefetch),
        .src_addr(src_addr), .num_words(num_words),
        .prefetch_active(prefetch_active), .prefetch_done(prefetch_done),
        .axi_araddr(araddr), .axi_arvalid(arvalid), .axi_arready(arready),
        .axi_rdata(rdata), .axi_rvalid(rvalid), .axi_rready(rready),
        .bram_addr(bram_addr), .bram_data(bram_data), .bram_we(bram_we), .overflow(dut_overflow_wei)
    );

    always @(posedge clk) if (rst_n && $past(rst_n)) assume (num_words == $past(num_words));

    // a_sanity was removed in Wave 591. Its body was `X == X`, which the
    // optimiser folds to constant true before any signal is read: it proved
    // unconditionally and tested nothing. Worse, it still emitted a $check
    // cell, so it inflated the non-empty-property gate (Prop. 5) that exists to
    // catch exactly an all-vacuous set. See Prop. 41.

    // REGRESSION WITNESS: never write more words than were requested.
    // A balance, not a shape check -- the per-beat logic was individually
    // correct, and what went wrong was that the beats never stopped.
    reg [16:0] writes;
    always @(posedge clk or negedge rst_n)
        if (!rst_n)               writes <= 17'd0;
        else if (!prefetch_active) writes <= 17'd0;
        else if (bram_we)          writes <= writes + 17'd1;

    always @(posedge clk) if (rst_n && prefetch_active)
        a_no_overwrite: assert (writes <= {1'b0, num_words});

    // A cheaper decomposition was attempted and withdrawn: relate `writes` to
    // `bram_addr` locally, so the solver need not carry a 17-bit counter across
    // the unrolling, and lean on max_size_props for the address never wrapping.
    // The idea is sound; the alignment is not established. `writes` is
    // registered off `bram_we` while `bram_addr` is assigned from `word_index`
    // on the same edge, and the relation between them at the sampling point was
    // guessed twice and refuted both times. Recorded rather than guessed a
    // third time -- see Prop. 35c.

    // rready is derived from the FETCH state, so it may only be high while
    // the controller is active.
    always @(posedge clk) if (rst_n)
        a_rready_implies_active: assert (!rready || prefetch_active);


    // A CONSERVATION property was attempted here and did not land (Wave 600):
    //   word_index + words_remaining == the clamped request
    // Two counters tracking one quantity by different routes -- the shape behind
    // every defect in this campaign (Prop. 48c), so worth asserting.
    //
    // It refuted in two formulations. Against the live `num_words` it refuted
    // because this file's stability assumption is guarded by `$past(rst_n)` and
    // does not cover the cycle where the DUT loads it. Against a latched copy it
    // still refuted, on a timing mismatch between the capture point
    // (`start_prefetch && !prefetch_active`) and the DUT's own load (which also
    // requires `state == IDLE`) that is NOT established.
    //
    // Strengthening the environment was tried and reverted: without an `rst_n`
    // guard, `$past` at cycle 0 pins the input to zero forever and two vacuity
    // witnesses stopped refuting -- an over-constraint that would have silently
    // weakened the vacuity gate. Recorded rather than patched a third time.
    // See Prop. 50.

    // The address channel runs ahead of the data channel, never behind it:
    // an address is issued before its beat returns.
    always @(posedge clk)
        if (rst_n && prefetch_active)
            a_addr_ahead_of_data: assert (dut.word_index <= {4'd0, bram_addr} + 12'd1);
endmodule

`default_nettype wire
