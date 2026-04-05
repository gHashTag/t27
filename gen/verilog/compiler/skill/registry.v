// Auto-generated from compiler/skill/registry.t27
// DO NOT EDIT -- regenerate with: tri gen compiler/skill/registry.t27
// phi^2 + phi^-2 = 3 | TRINITY

/* verilator lint_off UNUSEDPARAM */
/* verilator lint_off UNUSEDSIGNAL */
/* verilator lint_off WIDTHTRUNC */
/* verilator lint_off WIDTHEXPAND */
/* verilator lint_off DECLFILENAME */
/* verilator lint_off BLKSEQ */
/* verilator lint_off INFINITELOOP */
/* verilator lint_off UNDRIVEN */
/* verilator lint_off PINCONNECTEMPTY */


// ============================================================================
// Skill Status/Kind/Verdict Encoding (Hardware)
// ============================================================================

`define SKILL_ACTIVE    3'd0
`define SKILL_SEALED    3'd1
`define SKILL_PAUSED    3'd2
`define SKILL_BLOCKED   3'd3
`define SKILL_COMPLETED 3'd4

`define KIND_FEATURE  3'd0
`define KIND_BUGFIX   3'd1
`define KIND_HOTFIX   3'd2
`define KIND_RECOVERY 3'd3
`define KIND_REFACTOR 3'd4

`define VERDICT_NOT_TOXIC 1'd0
`define VERDICT_TOXIC     1'd1

// ============================================================================
// Skill Status Register
// ============================================================================

module t27_skill_status_reg (
    input  wire       clk,
    input  wire       rst_n,
    input  wire       wr_en,
    input  wire [2:0] status_in,
    input  wire [2:0] kind_in,
    input  wire       verdict_in,
    output reg  [2:0] status_out,
    output reg  [2:0] kind_out,
    output reg        verdict_out
);

    always @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            status_out  <= `SKILL_ACTIVE;
            kind_out    <= `KIND_FEATURE;
            verdict_out <= `VERDICT_NOT_TOXIC;
        end else if (wr_en) begin
            status_out  <= status_in;
            kind_out    <= kind_in;
            verdict_out <= verdict_in;
        end
    end

endmodule
