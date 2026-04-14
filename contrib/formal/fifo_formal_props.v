module fifo_formal_props (
    input wire clk,
    input wire rst_n,
    input wire [31:0] din,
    input wire wr_en,
    output wire full,
    output wire [31:0] dout,
    input wire rd_en,
    output wire empty
);

    default clocking fp @(posedge clk); endclocking
    default disable !rst_n;

    // P1: FIFO empty after reset
    assert property (rst_n |-> empty == 1'b1)
        else $error("P1 FAILED: FIFO not empty after reset");

    // P2: FIFO not full after reset
    assert property (rst_n |-> full == 1'b0)
        else $error("P2 FAILED: FIFO full after reset");

    // P3: Write to empty FIFO makes it non-empty
    assert property (empty && wr_en && !full |=> !empty)
        else $error("P3 FAILED: write to empty FIFO still empty");

    // P4: Read from full FIFO makes it non-full
    assert property (full && rd_en && !empty |=> !full)
        else $error("P4 FAILED: read from full FIFO still full");

    // P5: Data integrity: read returns first written value
    reg [31:0] written_data;
    always @(posedge clk) begin
        if (rst_n && wr_en && !full) begin
            written_data <= din;
        end
    end
    assert property (empty && wr_en && !full ##1 rd_en && !empty |=> dout == written_data)
        else $error("P5 FAILED: FIFO data integrity violation");

    // P6: Cannot write when full (overflow protection)
    assert property (!(full && wr_en))
        else $error("P6 FAILED: write while full");

    // P7: Cannot read when empty (underflow protection)
    assert property (!(empty && rd_en))
        else $error("P7 FAILED: read while empty");

    // P8: Cover point: FIFO becomes full
    cover property (full);

    // P9: Cover point: FIFO becomes empty after being non-empty
    cover property (!empty ##1 empty);

endmodule
