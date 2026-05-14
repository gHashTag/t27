// JTAG-to-SPI proxy bridge for Xilinx XC7A100T-FGG676 (QMTech core board)
//
// Ported to plain Verilog from the Migen Python source in
// openocd/contrib/loaders/flash/fpga/xilinx_bscan_spi.py
// (https://github.com/openocd-org/openocd/blob/master/contrib/loaders/flash/fpga/xilinx_bscan_spi.py)
//
// Behaviour mirrors the quartiq/bscan_spi_bitstreams family:
//   * USER1 (BSCANE2 JTAG_CHAIN=1) is the SPI gateway.
//   * Each transaction starts with a single marker bit "1" while DRCK rises.
//   * After the marker, the host shifts a 32-bit big-endian length, then
//     "length" bits of SPI data, with the bridge driving CS_N low for the
//     entire data phase.
//   * Bits are sampled on DRCK rising edge (TDI -> MOSI). MISO is sampled
//     on the falling edge of CCLK and presented on TDO.
//   * CCLK comes from STARTUPE2 (USRCCLKO), the dedicated config clock pin.

`timescale 1ns / 1ps
`default_nettype none

module bscan_spi_qmtech (
    inout  wire cs_n,
    inout  wire mosi,
    inout  wire miso
);

    // ------------------------------------------------------------------
    // BSCANE2 - USER1 instance (JTAG_CHAIN=1 == USER1 IR opcode)
    // ------------------------------------------------------------------
    wire jtag_capture;
    wire jtag_drck;
    wire jtag_reset;
    wire jtag_runtest;
    wire jtag_sel;
    wire jtag_shift;
    wire jtag_tck;
    wire jtag_tdi;
    wire jtag_update;
    reg  jtag_tdo;

    BSCANE2 #(
        .JTAG_CHAIN(1)
    ) bscan_i (
        .CAPTURE (jtag_capture),
        .DRCK    (jtag_drck),
        .RESET   (jtag_reset),
        .RUNTEST (jtag_runtest),
        .SEL     (jtag_sel),
        .SHIFT   (jtag_shift),
        .TCK     (jtag_tck),
        .TDI     (jtag_tdi),
        .TDO     (jtag_tdo),
        .TMS     (),
        .UPDATE  (jtag_update)
    );

    // ------------------------------------------------------------------
    // STARTUPE2 - dedicated CCLK driver. USRCCLKO is the SPI CLK source.
    // Drive USRCCLKO from DRCK so the flash gets a 1:1 JTAG-derived clock.
    // (USRCCLKTS=0 keeps the buffer enabled; CFGCLK / CFGMCLK / EOS unused.)
    // ------------------------------------------------------------------
    wire cclk;
    assign cclk = jtag_drck;

    STARTUPE2 #(
        .PROG_USR("FALSE"),
        .SIM_CCLK_FREQ(0.0)
    ) startup_i (
        .CFGCLK    (),
        .CFGMCLK   (),
        .EOS       (),
        .PREQ      (),
        .CLK       (1'b0),
        .GSR       (1'b0),
        .GTS       (1'b0),
        .KEYCLEARB (1'b1),
        .PACK      (1'b0),
        .USRCCLKO  (cclk),
        .USRCCLKTS (1'b0),
        .USRDONEO  (1'b1),
        .USRDONETS (1'b1)
    );

    // ------------------------------------------------------------------
    // State machine:
    //   IDLE   -> wait for SEL & SHIFT; first TDI=1 == marker
    //   LENGTH -> 32 big-endian bits load the data-phase counter
    //   DATA   -> stream "remaining" data bits with CS_N low
    // ------------------------------------------------------------------
    localparam [1:0] S_IDLE   = 2'b00;
    localparam [1:0] S_LENGTH = 2'b01;
    localparam [1:0] S_DATA   = 2'b10;

    reg [1:0]  state;
    reg [5:0]  len_cnt;     // 0..32 length-shift counter
    reg [31:0] remaining;   // remaining data bits

    // CS_N is asserted low only during S_DATA. Tristate everything else
    // so that other configuration users of the dedicated pins still work.
    reg cs_n_oe;
    reg cs_n_d;

    // MOSI is driven from TDI sampled on DRCK rising edge while in S_DATA.
    reg mosi_d;
    reg mosi_oe;

    // MISO is sampled on falling edge of CCLK (i.e. DRCK falling edge).
    reg miso_capture;

    // -------- rising-edge logic on DRCK / jtag_tck --------
    always @(posedge jtag_drck or posedge jtag_reset) begin
        if (jtag_reset) begin
            state     <= S_IDLE;
            len_cnt   <= 6'd0;
            remaining <= 32'd0;
            mosi_d    <= 1'b0;
            mosi_oe   <= 1'b0;
            cs_n_d    <= 1'b1;
            cs_n_oe   <= 1'b0;
        end else if (jtag_sel && jtag_shift) begin
            case (state)
                S_IDLE: begin
                    cs_n_d  <= 1'b1;
                    cs_n_oe <= 1'b0;
                    mosi_oe <= 1'b0;
                    if (jtag_tdi) begin
                        state   <= S_LENGTH;
                        len_cnt <= 6'd0;
                    end
                end

                S_LENGTH: begin
                    // big-endian: MSB first
                    remaining <= {remaining[30:0], jtag_tdi};
                    if (len_cnt == 6'd31) begin
                        len_cnt <= 6'd0;
                        // Begin SPI data phase only if length is non-zero.
                        if ({remaining[30:0], jtag_tdi} != 32'd0) begin
                            state   <= S_DATA;
                            cs_n_d  <= 1'b0;
                            cs_n_oe <= 1'b1;
                            mosi_oe <= 1'b1;
                        end else begin
                            state <= S_IDLE;
                        end
                    end else begin
                        len_cnt <= len_cnt + 6'd1;
                    end
                end

                S_DATA: begin
                    // Drive MOSI from the current TDI bit.
                    mosi_d <= jtag_tdi;
                    if (remaining == 32'd1) begin
                        // last bit -- release CS_N on the next falling edge
                        state     <= S_IDLE;
                        remaining <= 32'd0;
                    end else begin
                        remaining <= remaining - 32'd1;
                    end
                end

                default: state <= S_IDLE;
            endcase
        end else if (jtag_update || !jtag_sel) begin
            // Exit shift -- park outputs.
            state   <= S_IDLE;
            mosi_oe <= 1'b0;
            cs_n_oe <= 1'b0;
            cs_n_d  <= 1'b1;
        end
    end

    // -------- falling-edge logic: sample MISO, drive TDO --------
    always @(negedge jtag_drck or posedge jtag_reset) begin
        if (jtag_reset) begin
            miso_capture <= 1'b0;
            jtag_tdo     <= 1'b0;
        end else begin
            miso_capture <= miso;
            jtag_tdo     <= miso_capture;
        end
    end

    // -------- tri-state IO buffers for dedicated config pins --------
    assign cs_n = cs_n_oe ? cs_n_d : 1'bz;
    assign mosi = mosi_oe ? mosi_d : 1'bz;
    // MISO is always an input.

endmodule

`default_nettype wire
