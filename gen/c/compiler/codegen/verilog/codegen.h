/* Auto-generated from compiler/codegen/verilog/codegen.t27 */
/* DO NOT EDIT -- regenerate with: tri gen compiler/codegen/verilog/codegen.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */

#ifndef T27_CODEGEN_VERILOG_H
#define T27_CODEGEN_VERILOG_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    const char *target_device;
    uint32_t    clock_freq_hz;
    bool        include_testbench;
    bool        include_toplevel;
} T27VerilogOptions;

typedef struct {
    uint32_t indent_level;
    uint8_t  pc_width;
    uint8_t  addr_width;
    uint8_t  data_width;
    T27VerilogOptions options;
} T27VerilogCodegen;

T27VerilogCodegen t27_verilog_codegen_new(T27VerilogOptions opts);
uint8_t t27_verilog_calculate_width(uint32_t count);

#ifdef __cplusplus
}
#endif

#endif /* T27_CODEGEN_VERILOG_H */
