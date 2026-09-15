// ============================================================================
// Zero-sized-request policy sweep (Wave 575)
//
// Three waves found the same defect shape one module at a time, reactively:
// zero neurons (Prop. 9), zero words (Prop. 10), and a claim about zero bytes
// that turned out to be wrong (Prop. 25c, corrected in Prop. 26). This file
// stops finding them one at a time and MEASURES the policy of every module
// that takes a count and reports completion.
//
// Each wrapper holds its count at zero and asserts the module NEVER completes.
//   PROVES  -> the module silently drops a zero-sized request: no work, no
//              done, no error. A host waiting on the completion IRQ hangs.
//   REFUTES -> the module completes a zero-sized request immediately.
//
// Neither answer is wrong in isolation. Four modules disagreeing is the defect:
// a host driving this engine cannot know which to expect.
//
// MEASURED, before the fix:
//
//   layer_sequencer       zero neurons  refutes  completes
//   weight_prefetch_ctrl  zero words    refutes  completes
//   multilayer_sequencer  zero layers   PROVES   DROPPED   <- host hangs
//   dma_controller        zero length   PROVES   DROPPED   <- host hangs
//
// A 2-2 split, and the dropping half is the dangerous half: a request that
// produces no work, no completion and no error is the one outcome a host
// cannot observe. Both were changed to complete.
//
// This file is now a REGRESSION GATE with an inverted polarity: every
// `*_never_completes` property must REFUTE, and every `*_no_work` /
// `*_moves_no_data` / `*_writes_nothing` property must PROVE. Completing a
// zero-sized job is only safe if completing it does not pretend to have done
// it. See docs/FORMAL_FOUNDATIONS.md Prop. 26.
//
// Immediate assertions, not concurrent SVA (Yosys frontend limits, Props 2/6).
// REQUIRES `-set-assumes` (Prop 11) and `-flatten` (Prop 7).
//
// Prove with (per wrapper):
//   yosys -p "read_verilog -sv -formal build/rtl/<dut>.sv formal/zero_size_props.sv; \
//             prep -top <wrapper> -flatten; async2sync; chformal -lower; \
//             sat -verify -prove-asserts -seq 24 -set-init-zero -set-assumes"
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`default_nettype none

// ---------------------------------------------------------------------------
// multilayer_sequencer: zero layers
// ---------------------------------------------------------------------------
module zs_multilayer (
    input wire clk, input wire rst_n, input wire start,
    input wire [5:0] num_layers, input wire layer_done, input wire prefetch_done
);
    wire [5:0] current_layer;
    wire layer_start, start_prefetch, inference_done, idle;

    multilayer_sequencer dut (
        .clk(clk), .rst_n(rst_n), .start(start), .num_layers(num_layers),
        .layer_done(layer_done), .prefetch_done(prefetch_done),
        .current_layer(current_layer), .layer_start(layer_start),
        .start_prefetch(start_prefetch), .inference_done(inference_done),
        .idle(idle)
    );

    always @(posedge clk) if (rst_n) assume (num_layers == 6'd0);

    always @(posedge clk) if (rst_n)
        a_zero_layers_never_completes: assert (!inference_done);

    // Completing must not mean pretending. A zero-layer inference may report
    // done; it may not launch a layer or a prefetch on the way there.
    always @(posedge clk) if (rst_n)
        a_zero_layers_emits_no_work: assert (!layer_start && !start_prefetch);
endmodule

// ---------------------------------------------------------------------------
// dma_controller: zero-length transfer
// ---------------------------------------------------------------------------
module zs_dma (
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

    always @(posedge clk) if (rst_n) assume (length == 32'd0);

    always @(posedge clk) if (rst_n)
        a_zero_length_never_completes: assert (!done);

    // A zero-length transfer may report done; it may not touch the bus or the
    // local memory on the way there.
    always @(posedge clk) if (rst_n)
        a_zero_length_moves_no_data: assert (!local_we && !m_axi_arvalid && !m_axi_awvalid);
endmodule

// ---------------------------------------------------------------------------
// weight_prefetch_ctrl: zero words
// ---------------------------------------------------------------------------
module zs_prefetch (
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

    always @(posedge clk) if (rst_n) assume (num_words == 16'd0);

    always @(posedge clk) if (rst_n)
        a_zero_words_never_completes: assert (!prefetch_done);

    always @(posedge clk) if (rst_n)
        a_zero_words_writes_nothing: assert (!bram_we && !axi_arvalid);
endmodule

// ---------------------------------------------------------------------------
// layer_sequencer: zero neurons and zero chunks
// ---------------------------------------------------------------------------
module zs_layer (
    input wire clk, input wire rst_n, input wire start,
    input wire [15:0] num_neurons, input wire [7:0] num_chunks
);
    wire [15:0] neuron_id;
    wire [7:0] chunk_id;
    wire first_chunk, last_chunk, valid, done;

    layer_sequencer dut (
        .clk(clk), .rst_n(rst_n), .start(start), .num_neurons(num_neurons),
        .num_chunks(num_chunks), .neuron_id(neuron_id), .chunk_id(chunk_id),
        .first_chunk(first_chunk), .last_chunk(last_chunk),
        .valid(valid), .done(done)
    );

    always @(posedge clk) if (rst_n) assume (num_neurons == 16'd0);

    // DEAD over 12 mutants (Wave 614, Prop. 65) -- the only DEAD verdict in the
    // campaign so far, and reported with its denominator because 12 is a weak
    // one. This is an EXPECTED REFUTATION: it records that a zero-neuron job
    // does report done. "Detection" for an inverted property means a mutant made
    // it PROVE, i.e. removed the completion, and no mutation of this 23-line
    // sequencer does that -- `state <= DONE_ST` is reached from the zero guard
    // by a path no single-token edit diverts.
    //
    // Kept. Its job is documentary: it pins a completion policy that Prop. 26
    // decided deliberately, and its sibling below is what makes that policy
    // safe. A property whose value is the record it leaves does not have to
    // earn its place by detection -- but the verdict is written here so nobody
    // has to re-derive that.
    always @(posedge clk) if (rst_n)
        a_zero_neurons_never_completes: assert (!done);

    // A zero-sized job must never emit work, whichever completion policy wins.
    // BITES (2, both uniquely) -- this is the half that carries the safety.
    always @(posedge clk) if (rst_n)
        a_zero_neurons_emits_no_work: assert (!valid);
endmodule

`default_nettype wire
