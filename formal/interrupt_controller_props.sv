// ============================================================================
// Formal properties for `interrupt_controller` (BitNet HLS, W36f / R-BN-6)
//
// These are immediate assertions, not concurrent SVA: Yosys's Verilog frontend
// accepts neither `property ... endproperty` nor
// `assert property (@(posedge clk) ...)`. See docs/FORMAL_FOUNDATIONS.md
// Props. 2 and 6.
//
// `a_event_never_lost` is a regression witness. Before 2026-08-09 the RTL wrote
// the interrupt sources and the clear-on-read as independent non-blocking
// assignments ending in `if (status_read) irq_status <= 3'b000;`, so the last
// write won. Yosys proved the failure outright -- not "can fail", but
//     $past(inference_done) && $past(status_read) |-> irq_status[0] == 0
// held on every reachable state. Any regression to set-then-clear ordering
// makes this assertion refutable again.
//
// Prove with:
//   yosys -p "read_verilog -sv -formal interrupt_controller.sv \
//             formal/interrupt_controller_props.sv; \
//             prep -top irq_props -flatten; async2sync; chformal -lower; \
//             sat -verify -prove-asserts -seq 6 -tempinduct"
//
// `-flatten` is required: `sat` refuses to run with more than one module
// selected, and without it the run fails with an error that reads exactly like
// a refutation.
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`default_nettype none

module irq_props (
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

    // Safety: with every source masked off, no interrupt may be raised.
    always @(posedge clk) if (rst_n)
        a_mask_suppresses: assert (irq_enable != 3'b000 || !irq_out);

    // Liveness-adjacent (bounded): an event latches when nothing reads it.
    always @(posedge clk)
        if (rst_n && $past(rst_n) && $past(inference_done) && !$past(status_read))
            a_event_latches: assert (irq_status[0]);

    // REGRESSION WITNESS: an event latches even when read in the same cycle.
    // This one was refutable before the clear-then-set fix.
    always @(posedge clk)
        if (rst_n && $past(rst_n) && $past(inference_done))
            a_event_never_lost: assert (irq_status[0]);

    // Same guarantee for the other two sources.
    always @(posedge clk)
        if (rst_n && $past(rst_n) && $past(dma_done))
            a_dma_never_lost: assert (irq_status[1]);

    always @(posedge clk)
        if (rst_n && $past(rst_n) && $past(error))
            a_error_never_lost: assert (irq_status[2]);

    // Clear-on-read still works: a read with no concurrent event empties it.
    always @(posedge clk)
        if (rst_n && $past(rst_n) && $past(status_read)
            && !$past(inference_done) && !$past(dma_done) && !$past(error))
            a_read_clears: assert (irq_status == 3'b000);

endmodule

`default_nettype wire
