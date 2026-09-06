// ============================================================================
// weight_bram properties.
//
// Wave 626. Prop. 76 classified this module INDIRECT: constrained only through
// the engine's integration properties, at one remove. It is the memory the
// weight prefetch fills and the compute stage reads, and Prop. 34's DEPTH
// scaling and the `memory_map` pass in every engine proof both exist because of
// it -- yet nothing stated what it is supposed to do.
//
// The property below is the memory axiom, over a SYMBOLIC address held constant
// by assumption: a read returns the last value written to that address. Stating
// it for an arbitrary address rather than a fixed one is what makes it cover
// non-interference too -- if a write to any other address disturbed this one,
// the shadow would disagree.
//
// Collision semantics matter here and are load-bearing: `rd_data <= mem[rd_addr]`
// and `mem[wr_addr] <= wr_data` are both non-blocking, so a read concurrent with
// a write to the same address returns the OLD value. The shadow is compared as
// of the read cycle, before that cycle's write, which is what `$past(fv_mem)`
// expresses.
//
// REQUIRES `-set-assumes` (Prop. 11) and `-flatten` (Prop. 7). DEPTH is scaled
// down by `chparam` in CI, as every engine proof does (Prop. 34).
//
// phi^2 + 1/phi^2 = 3 | TRINITY
// ============================================================================

`default_nettype none

module wb_props #(
    parameter DEPTH = 4096,
    parameter ADDR_WIDTH = 12
) (
    input wire                    clk,
    input wire                    rst_n,
    input wire [ADDR_WIDTH-1:0]   rd_addr,
    input wire [ADDR_WIDTH-1:0]   wr_addr,
    input wire [53:0]             wr_data,
    input wire                    wr_en,
    input wire [ADDR_WIDTH-1:0]   fv_addr
);

    wire [53:0] rd_data;

    weight_bram dut (
        .clk(clk),
        .rd_addr(rd_addr), .rd_data(rd_data),
        .wr_addr(wr_addr), .wr_data(wr_data), .wr_en(wr_en)
    );

    // Addresses are in range. At the REAL depth this assumption is vacuous --
    // DEPTH is 4096 and ADDR_WIDTH is 12, so every representable address is
    // legal. It exists only because CI scales DEPTH down for tractability
    // (Prop. 34), which makes most 12-bit addresses out of bounds and lets the
    // solver write to mem[2048] of a four-entry array. That is a scaling
    // artifact, not a design behaviour, and the counterexample said so
    // literally before this was added.
    always @(posedge clk) begin
        assume (rd_addr < DEPTH);
        assume (wr_addr < DEPTH);
        assume (fv_addr < DEPTH);
    end

    // The address under observation is arbitrary but fixed. Without this the
    // solver may move it every cycle, and the shadow would be tracking a
    // different location than the one being read.
    always @(posedge clk) if (rst_n && $past(rst_n))
        assume (fv_addr == $past(fv_addr));

    // Shadow of mem[fv_addr]. Non-blocking, so at cycle t it holds the value
    // BEFORE that cycle's write -- which is exactly what a concurrent read sees.
    reg [53:0] fv_mem;
    reg        fv_written;
    always @(posedge clk) begin
        if (!rst_n) begin
            fv_written <= 1'b0;
        end else if (wr_en && wr_addr == fv_addr) begin
            fv_mem     <= wr_data;
            fv_written <= 1'b1;
        end
    end

    // The memory axiom. A read of the observed address returns the last value
    // written to it -- and since the address is symbolic, this also says that
    // writes to every OTHER address leave it alone.
    always @(posedge clk) if (rst_n && $past(rst_n) && $past(fv_written)
                              && $past(rd_addr) == fv_addr)
        a_read_returns_last_write: assert (rd_data == $past(fv_mem));

endmodule

`default_nettype wire
