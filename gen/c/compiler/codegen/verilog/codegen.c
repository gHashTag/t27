/* Auto-generated from compiler/codegen/verilog/codegen.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/codegen/verilog/codegen.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#include "codegen.h"

T27VerilogCodegen t27_verilog_codegen_new(T27VerilogOptions opts) {
    T27VerilogCodegen gen;
    gen.indent_level = 0;
    gen.pc_width     = 8;
    gen.addr_width   = 12;
    gen.data_width   = 32;
    gen.options      = opts;
    return gen;
}

uint8_t t27_verilog_calculate_width(uint32_t count) {
    if (count <= 1) return 1;
    uint8_t w = 0;
    uint32_t n = count - 1;
    while (n > 0) { w++; n >>= 1; }
    return w;
}
