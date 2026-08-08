// ============================================================================
// Formal properties for `axi_lite_slave` (BitNet HLS, W36d / R-BN-4)
//
// Immediate assertions, not concurrent SVA: Yosys's frontend accepts neither
// `property ... endproperty` nor `assert property (@(posedge clk) ...)`.
// See docs/FORMAL_FOUNDATIONS.md Props. 2, 6, 8.
//
// Prove with (note -flatten and -set-init-zero, both load-bearing):
//   yosys -p "read_verilog -sv -formal axi_lite_slave.sv \
//             formal/axi_lite_slave_props.sv; \
//             prep -top axi_props -flatten; async2sync; chformal -lower; \
//             sat -verify -prove-asserts -seq 10 -set-init-zero -set-assumes"
//
// -set-init-zero matters: under -tempinduct with an unconstrained initial
// state, `bresp == 2'b00` is refutable even though bresp is only ever assigned
// 2'b00, because induction may start from an unreachable state. That was an
// artifact, not a defect, and cross-checking against a reachable start is what
// separated it from the two real ones below.
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`default_nettype none
module axi_props (
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

    // Harness sanity. A tautology cannot be refuted; if this ever fails, the
    // run is not evaluating what it appears to. Three properties once "failed"
    // in this repo because `sat` refuses to run with more than one module
    // selected and its error reads exactly like a refutation -- `-flatten`
    // fixes it, and this assertion is what makes that visible.
    always @(posedge clk) if (rst_n)
        a_sanity: assert (s_axi_bresp_probe == s_axi_bresp_probe);

    // AXI rule: VALID must not be deasserted without a handshake.
    always @(posedge clk) if (rst_n && $past(rst_n) && $past(bvalid) && !$past(bready))
        a_bvalid_stable: assert (bvalid);
    always @(posedge clk) if (rst_n && $past(rst_n) && $past(rvalid) && !$past(rready))
        a_rvalid_stable: assert (rvalid);

    // REGRESSION WITNESSES -----------------------------------------------
    // The slave has a single bvalid/bresp register, so it can owe at most one
    // write response. Before 2026-08-09, awready and wready were set at reset
    // and never cleared, so a second write was accepted while the first
    // response was unacknowledged: two accepted writes, one B beat, and an AXI
    // master left waiting forever. Yosys refuted this from a reachable state.
    reg [3:0] outstanding_w;
    wire accept_w = awvalid && wvalid && awready && wready;
    wire done_b   = bvalid && bready;
    always @(posedge clk or negedge rst_n)
        if (!rst_n) outstanding_w <= 4'd0;
        else outstanding_w <= outstanding_w + (accept_w ? 4'd1 : 4'd0) - (done_b ? 4'd1 : 4'd0);
    always @(posedge clk) if (rst_n)
        a_one_outstanding_write: assert (outstanding_w <= 4'd1);

    // Same bound on the read channel, which had the identical defect.
    reg [3:0] outstanding_r;
    wire accept_r = arvalid && arready;
    wire done_r   = rvalid && rready;
    always @(posedge clk or negedge rst_n)
        if (!rst_n) outstanding_r <= 4'd0;
        else outstanding_r <= outstanding_r + (accept_r ? 4'd1 : 4'd0) - (done_r ? 4'd1 : 4'd0);
    always @(posedge clk) if (rst_n)
        a_one_outstanding_read: assert (outstanding_r <= 4'd1);

    // The direct backpressure rule the fix implements.
    always @(posedge clk) if (rst_n && bvalid && !bready)
        a_no_write_accept_while_pending: assert (!awready);
    always @(posedge clk) if (rst_n && rvalid && !rready)
        a_no_read_accept_while_pending: assert (!arready);

endmodule

`default_nettype wire
