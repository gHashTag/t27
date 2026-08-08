// ============================================================================
// Minimal AXI4 read-slave model, for master-side formal properties.
//
// Why this exists: with `rvalid` left as a free input, a prover can have the
// slave deliver beats nobody requested, and a misbehaving environment is
// indistinguishable from a defect in the master. Every master-side property is
// really a property about the slave's contract too. This file makes that
// contract explicit and reviewable rather than implicit in one harness.
//
// It assumes ONLY what AXI4 requires of a compliant read slave:
//   1. beats appear only while a burst is outstanding
//   2. rlast lands exactly on the (arlen+1)-th beat of that burst
//   3. rvalid, once asserted, holds until rready (slave-side VALID stability)
//
// It deliberately does NOT constrain `arready`, which a slave may stall
// arbitrarily, and does NOT assume the master behaves.
//
// PRECONDITION, asserted not assumed: this model tracks one burst at a time,
// which is only faithful if the master issues one at a time. That is checked
// by `a_model_precondition_single_burst` below rather than assumed away --
// assuming it would let the model hide exactly the kind of defect it exists to
// expose.
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`default_nettype none

module axi4_read_slave_model (
    input wire       clk,
    input wire       rst_n,
    // Master -> slave
    input wire       arvalid,
    input wire [7:0] arlen,
    input wire       rready,
    // Slave -> master (free inputs at the top level, constrained here)
    input wire       arready,
    input wire       rvalid,
    input wire       rlast
);

    reg       burst_active;
    reg [8:0] beats_left;   // beats still owed in the current burst

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            burst_active <= 1'b0;
            beats_left   <= 9'd0;
        end else if (arvalid && arready) begin
            burst_active <= 1'b1;
            beats_left   <= {1'b0, arlen} + 9'd1;
        end else if (burst_active && rvalid && rready) begin
            // Clear on rlast rather than on the local count. The two agree by
            // assumption (2), but keying the clear off the master-visible
            // signal keeps the model from latching burst_active forever if the
            // count and the burst ever disagree -- which would refute the
            // precondition below for a reason that is the model's fault, not
            // the master's.
            if (rlast) burst_active <= 1'b0;
            beats_left <= beats_left - 9'd1;
        end
    end

    // (1) No unsolicited beats.
    always @(posedge clk) if (rst_n)
        assume (!rvalid || burst_active);

    // (2) rlast exactly on the final beat of the burst.
    always @(posedge clk) if (rst_n && burst_active && rvalid)
        assume (rlast == (beats_left == 9'd1));

    // (3) Slave-side VALID stability.
    always @(posedge clk) if (rst_n && $past(rst_n) && $past(rvalid) && !$past(rready))
        assume (rvalid);

    // Precondition on the master, asserted so the model cannot silently
    // over-constrain: no new address handshake while a burst is in flight.
    always @(posedge clk) if (rst_n && burst_active)
        a_model_precondition_single_burst: assert (!(arvalid && arready));

endmodule

`default_nettype wire
