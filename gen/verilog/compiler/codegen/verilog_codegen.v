// Auto-generated from compiler/codegen/verilog/codegen.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/codegen/verilog/codegen.t27
// phi^2 + phi^-2 = 3 | TRINITY

// ============================================================================
// Verilog RTL Generator Hardware Module
// ============================================================================
// Synthesizable module for hardware-assisted Verilog code generation
// Implements instruction decode ROM and data path width calculation

module t27_verilog_codegen #(
    parameter PC_WIDTH   = 8,
    parameter ADDR_WIDTH = 12,
    parameter DATA_WIDTH = 32
) (
    input  wire                  clk,
    input  wire                  rst_n,
    input  wire                  gen_start,
    input  wire [PC_WIDTH-1:0]   instr_count,
    output reg  [3:0]            calc_pc_width,
    output reg                   gen_done
);

    // Width calculator: ceil(log2(count))
    reg [2:0] state;
    reg [PC_WIDTH-1:0] shift_reg;

    localparam S_IDLE  = 3'd0;
    localparam S_CALC  = 3'd1;
    localparam S_DONE  = 3'd2;

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            state         <= S_IDLE;
            calc_pc_width <= 4'd1;
            gen_done      <= 1'b0;
            shift_reg     <= {PC_WIDTH{1'b0}};
        end else begin
            case (state)
                S_IDLE: begin
                    gen_done <= 1'b0;
                    if (gen_start) begin
                        if (instr_count <= {{(PC_WIDTH-1){1'b0}}, 1'b1}) begin
                            calc_pc_width <= 4'd1;
                            state         <= S_DONE;
                        end else begin
                            shift_reg     <= instr_count - {{(PC_WIDTH-1){1'b0}}, 1'b1};
                            calc_pc_width <= 4'd0;
                            state         <= S_CALC;
                        end
                    end
                end

                S_CALC: begin
                    if (shift_reg == {PC_WIDTH{1'b0}}) begin
                        state <= S_DONE;
                    end else begin
                        shift_reg     <= shift_reg >> 1;
                        calc_pc_width <= calc_pc_width + 4'd1;
                    end
                end

                S_DONE: begin
                    gen_done <= 1'b1;
                    state    <= S_IDLE;
                end

                default: state <= S_IDLE;
            endcase
        end
    end

endmodule
