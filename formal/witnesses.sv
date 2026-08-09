// ============================================================================
// Witness harnesses -- proof that the properties have teeth.
//
// Vacuity is the mirror of the inert-constraint failure in Prop. 11. There, a
// constraint did nothing and a result looked meaningful. Here, a property can
// look proved because the *interesting case never happens*: `assert (!A || B)`
// is trivially true whenever A is false, and `G |-> P` is free if G is
// unreachable. Neither shows up as a failure. Both make a green run worthless.
//
// Each module below asserts the NEGATION of a case the corresponding property
// exists to cover. CI runs each expecting **refutation** -- a counterexample is
// the witness that the case is reachable. A witness that starts PROVING means
// that case became unreachable and the property it guards has quietly lost its
// teeth.
//
// Requires `-set-assumes` (see Prop. 11) and `-flatten` (see Prop. 7).
// Expected result for every module here: REFUTED.
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`default_nettype none

module w_irq_mask_case (
    input wire       clk,
    input wire       rst_n,
    input wire       inference_done,
    input wire       dma_done,
    input wire       error,
    input wire [2:0] irq_enable,
    input wire       status_read
);

    wire [2:0] irq_status;
    wire       irq_out;

    interrupt_controller dut (
        .clk(clk), .rst_n(rst_n),
        .inference_done(inference_done), .dma_done(dma_done), .error(error),
        .irq_enable(irq_enable), .irq_status(irq_status),
        .status_read(status_read), .irq_out(irq_out)
    );

    // a_mask_suppresses is only meaningful if a fully-masked, non-empty
    // status is reachable.
    always @(posedge clk) if (rst_n)
        w: assert (!(irq_enable == 3'b000 && irq_status != 3'b000));
endmodule

module w_irq_concurrent_read (
    input wire       clk,
    input wire       rst_n,
    input wire       inference_done,
    input wire       dma_done,
    input wire       error,
    input wire [2:0] irq_enable,
    input wire       status_read
);

    wire [2:0] irq_status;
    wire       irq_out;

    interrupt_controller dut (
        .clk(clk), .rst_n(rst_n),
        .inference_done(inference_done), .dma_done(dma_done), .error(error),
        .irq_enable(irq_enable), .irq_status(irq_status),
        .status_read(status_read), .irq_out(irq_out)
    );

    // a_event_never_lost exists for the read-during-event race.
    always @(posedge clk) if (rst_n)
        w: assert (!(inference_done && status_read));
endmodule

module w_axi_outstanding_one (
    input wire clk, input wire rst_n,
    input wire [7:0] awaddr, input wire awvalid,
    input wire [31:0] wdata, input wire [3:0] wstrb, input wire wvalid,
    input wire bready,
    input wire [7:0] araddr, input wire arvalid, input wire rready,
    input wire [31:0] reg_status, input wire [31:0] reg_irq_stat,
    input wire [63:0] reg_cycles
);
    wire awready, wready, bvalid, arready, rvalid;
    wire [1:0] bresp, rresp;
    wire [1:0] s_axi_bresp_probe = bresp;
    wire [31:0] rdata, reg_ctrl, reg_irq_en, reg_num_layers, reg_neurons, reg_chunks, reg_threshold;
    wire [63:0] reg_weight_addr, reg_input_addr, reg_output_addr;
    axi_lite_slave dut (
        .clk(clk), .rst_n(rst_n),
        .s_axi_awaddr(awaddr), .s_axi_awvalid(awvalid), .s_axi_awready(awready),
        .s_axi_wdata(wdata), .s_axi_wstrb(wstrb), .s_axi_wvalid(wvalid), .s_axi_wready(wready),
        .s_axi_bresp(bresp), .s_axi_bvalid(bvalid), .s_axi_bready(bready),
        .s_axi_araddr(araddr), .s_axi_arvalid(arvalid), .s_axi_arready(arready),
        .s_axi_rdata(rdata), .s_axi_rresp(rresp), .s_axi_rvalid(rvalid), .s_axi_rready(rready),
        .reg_ctrl(reg_ctrl), .reg_status(reg_status), .reg_irq_en(reg_irq_en),
        .reg_irq_stat(reg_irq_stat), .reg_num_layers(reg_num_layers), .reg_neurons(reg_neurons),
        .reg_chunks(reg_chunks), .reg_threshold(reg_threshold),
        .reg_weight_addr(reg_weight_addr), .reg_input_addr(reg_input_addr),
        .reg_output_addr(reg_output_addr), .reg_cycles(reg_cycles)
    );

    reg [3:0] ow;
    always @(posedge clk or negedge rst_n)
        if (!rst_n) ow <= 4'd0;
        else ow <= ow + ((awvalid && wvalid && awready && wready) ? 4'd1 : 4'd0)
                      - ((bvalid && bready) ? 4'd1 : 4'd0);
    // a_one_outstanding_write is only a bound if 1 is actually reached.
    always @(posedge clk) if (rst_n) w: assert (ow != 4'd1);
endmodule

module w_axi_resp_pending (
    input wire clk, input wire rst_n,
    input wire [7:0] awaddr, input wire awvalid,
    input wire [31:0] wdata, input wire [3:0] wstrb, input wire wvalid,
    input wire bready,
    input wire [7:0] araddr, input wire arvalid, input wire rready,
    input wire [31:0] reg_status, input wire [31:0] reg_irq_stat,
    input wire [63:0] reg_cycles
);
    wire awready, wready, bvalid, arready, rvalid;
    wire [1:0] bresp, rresp;
    wire [1:0] s_axi_bresp_probe = bresp;
    wire [31:0] rdata, reg_ctrl, reg_irq_en, reg_num_layers, reg_neurons, reg_chunks, reg_threshold;
    wire [63:0] reg_weight_addr, reg_input_addr, reg_output_addr;
    axi_lite_slave dut (
        .clk(clk), .rst_n(rst_n),
        .s_axi_awaddr(awaddr), .s_axi_awvalid(awvalid), .s_axi_awready(awready),
        .s_axi_wdata(wdata), .s_axi_wstrb(wstrb), .s_axi_wvalid(wvalid), .s_axi_wready(wready),
        .s_axi_bresp(bresp), .s_axi_bvalid(bvalid), .s_axi_bready(bready),
        .s_axi_araddr(araddr), .s_axi_arvalid(arvalid), .s_axi_arready(arready),
        .s_axi_rdata(rdata), .s_axi_rresp(rresp), .s_axi_rvalid(rvalid), .s_axi_rready(rready),
        .reg_ctrl(reg_ctrl), .reg_status(reg_status), .reg_irq_en(reg_irq_en),
        .reg_irq_stat(reg_irq_stat), .reg_num_layers(reg_num_layers), .reg_neurons(reg_neurons),
        .reg_chunks(reg_chunks), .reg_threshold(reg_threshold),
        .reg_weight_addr(reg_weight_addr), .reg_input_addr(reg_input_addr),
        .reg_output_addr(reg_output_addr), .reg_cycles(reg_cycles)
    );

    // a_no_write_accept_while_pending needs a pending response to exist.
    always @(posedge clk) if (rst_n) w: assert (!(bvalid && !bready));
endmodule

module w_dma_multibeat_burst (
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
        .local_addr(local_addr), .local_wdata(local_wdata), .local_we(local_we),
        .local_rdata(local_rdata)
    );

    // a_read_burst_not_abandoned is vacuous on single-beat bursts only.
    always @(posedge clk) if (rst_n) w: assert (!(rvalid && rready && !rlast));
endmodule

module w_dma_burst_active (
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
        .local_addr(local_addr), .local_wdata(local_wdata), .local_we(local_we),
        .local_rdata(local_rdata)
    );

    reg ba;
    always @(posedge clk or negedge rst_n)
        if (!rst_n) ba <= 1'b0;
        else if (arvalid && arready) ba <= 1'b1;
        else if (ba && rvalid && rready && rlast) ba <= 1'b0;
    // a_rready_implies_burst needs a burst to be reachable at all.
    always @(posedge clk) if (rst_n) w: assert (!ba);
endmodule

module w_ls_multi_neuron (
    input wire        clk,
    input wire        rst_n,
    input wire        start,
    input wire [15:0] num_neurons,
    input wire [7:0]  num_chunks
);
    wire [15:0] neuron_id;
    wire [7:0]  chunk_id;
    wire        first_chunk, last_chunk, valid, done;

    layer_sequencer dut (
        .clk(clk), .rst_n(rst_n), .start(start),
        .num_neurons(num_neurons), .num_chunks(num_chunks),
        .neuron_id(neuron_id), .chunk_id(chunk_id),
        .first_chunk(first_chunk), .last_chunk(last_chunk),
        .valid(valid), .done(done)
    );

    // The descriptor is held stable for the duration, as the CSR block drives it.
    always @(posedge clk) if (rst_n && $past(rst_n)) assume (num_neurons == $past(num_neurons));
    always @(posedge clk) if (rst_n && $past(rst_n)) assume (num_chunks  == $past(num_chunks));

    // a_neuron_in_range only bites if more than neuron 0 is ever reached.
    always @(posedge clk) if (rst_n) w: assert (!(valid && neuron_id != 16'd0));
endmodule

module w_wp_writes_happen (
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

    weight_prefetch_ctrl dut (
        .clk(clk), .rst_n(rst_n), .start_prefetch(start_prefetch),
        .src_addr(src_addr), .num_words(num_words),
        .prefetch_active(prefetch_active), .prefetch_done(prefetch_done),
        .axi_araddr(araddr), .axi_arvalid(arvalid), .axi_arready(arready),
        .axi_rdata(rdata), .axi_rvalid(rvalid), .axi_rready(rready),
        .bram_addr(bram_addr), .bram_data(bram_data), .bram_we(bram_we)
    );

    always @(posedge clk) if (rst_n && $past(rst_n)) assume (num_words == $past(num_words));

    // a_no_overwrite only bites if writes actually occur.
    always @(posedge clk) if (rst_n) w: assert (!(prefetch_active && bram_we));
endmodule

// ============================================================================
// INTERLEAVING witnesses (Wave 606, Prop. 56).
//
// Prop. 51 probed that each module's core ACTIVITY is reachable, and stated its
// limit: a constraint removing a rare INTERLEAVING while leaving the activity
// reachable passes every one of those probes. These close that gap, and they
// are chosen to be the shapes this campaign's defects actually took --
// back-to-back transfers (Props. 31c, 32) and both transfer directions.
//
// Expected result for every module here: REFUTED.
// ============================================================================

module w_dma_back_to_back (
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
        .local_addr(local_addr), .local_wdata(local_wdata), .local_we(local_we),
        .local_rdata(local_rdata)
    );

    // Two completed transfers. Prop. 31c was state carried across exactly this
    // boundary, and nothing checks that the second transfer is reachable.
    // Synchronous, and the previous value is an explicit register: $past inside
    // an async-reset block makes async2sync reject the design outright.
    reg [1:0] n; reg done_q;
    always @(posedge clk)
        if (!rst_n) begin n <= 2'd0; done_q <= 1'b0; end
        else begin
            done_q <= done;
            if (done && !done_q && n != 2'd3) n <= n + 2'd1;
        end
    always @(posedge clk) if (rst_n) w: assert (n < 2'd2);
endmodule

module w_dma_both_directions (
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
        .local_addr(local_addr), .local_wdata(local_wdata), .local_we(local_we),
        .local_rdata(local_rdata)
    );

    // A read transfer and a write transfer both occur. `direction` is sampled
    // once at start, so a constraint pinning it would remove half the design
    // while every activity probe still passed.
    reg saw_rd, saw_wr, busy_q;
    always @(posedge clk)
        if (!rst_n) begin saw_rd <= 1'b0; saw_wr <= 1'b0; busy_q <= 1'b0; end
        else begin
            busy_q <= busy;
            if (busy && !busy_q) begin
                if (!direction) saw_rd <= 1'b1;
                else            saw_wr <= 1'b1;
            end
        end
    always @(posedge clk) if (rst_n) w: assert (!(saw_rd && saw_wr));
endmodule

module w_wp_back_to_back (
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

    weight_prefetch_ctrl dut (
        .clk(clk), .rst_n(rst_n), .start_prefetch(start_prefetch),
        .src_addr(src_addr), .num_words(num_words),
        .prefetch_active(prefetch_active), .prefetch_done(prefetch_done),
        .axi_araddr(araddr), .axi_arvalid(arvalid), .axi_arready(arready),
        .axi_rdata(rdata), .axi_rvalid(rvalid), .axi_rready(rready),
        .bram_addr(bram_addr), .bram_data(bram_data), .bram_we(bram_we)
    );

    always @(posedge clk) if (rst_n && $past(rst_n)) assume (num_words == $past(num_words));

    // Two completed prefetches. The engine issues one per layer, so a
    // constraint that allowed only the first would leave every layer after the
    // first unverified while `w_wp_writes_happen` still refuted.
    reg [1:0] n; reg pd_q;
    always @(posedge clk)
        if (!rst_n) begin n <= 2'd0; pd_q <= 1'b0; end
        else begin
            pd_q <= prefetch_done;
            if (prefetch_done && !pd_q && n != 2'd3) n <= n + 2'd1;
        end
    always @(posedge clk) if (rst_n) w: assert (n < 2'd2);
endmodule

`default_nettype wire
