/* Auto-generated from specs/fpga/mac.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/fpga/mac.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 28 | Module: ZeroDSP_MAC */

#ifndef FPGA_MAC_H
#define FPGA_MAC_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/* ===================================================================== */
/* TernaryWord and Trit types                                             */
/* ===================================================================== */

typedef struct {
    uint32_t raw;
} TernaryWord;

typedef int8_t Trit;

#define TRIT_NEG   ((Trit)-1)
#define TRIT_ZERO  ((Trit)0)
#define TRIT_POS   ((Trit)1)

/* ===================================================================== */
/* 1. MAC Configuration                                                   */
/* ===================================================================== */

#define MAC_WIDTH         27
#define MAC_ACC_BITS      32
#define NUM_MAC_UNITS     8
#define PIPELINE_STAGES   4

#define OP_MAC_MUL   0
#define OP_MAC_MAC   1
#define OP_MAC_MACC  2
#define OP_MAC_DOT   3

#define MAC_STATUS_READY  0
#define MAC_STATUS_BUSY   1
#define MAC_STATUS_DONE   2

/* ===================================================================== */
/* 2. Ternary Multiplication LUT                                          */
/* ===================================================================== */

extern const int8_t MAC_LUT[9];

/* ===================================================================== */
/* 3. MAC Unit State                                                      */
/* ===================================================================== */

typedef struct {
    int32_t     accumulator;
    uint8_t     status;
    TernaryWord pipeline[PIPELINE_STAGES];
} MACUnit;

typedef struct {
    MACUnit units[NUM_MAC_UNITS];
} MACArray;

/* ===================================================================== */
/* API                                                                    */
/* ===================================================================== */

void        mac_array_init(MACArray *mac);

/* Trit extraction and packing */
Trit        mac_extract_trit(TernaryWord word, size_t index);
uint32_t    mac_pack_trit(Trit trit, size_t index);

/* MAC operations */
TernaryWord mac_multiply(MACArray *mac, TernaryWord a, TernaryWord b, uint8_t unit);
int32_t     mac_cycle(MACArray *mac, TernaryWord a, TernaryWord b, uint8_t unit, int32_t acc);
int32_t     mac_dot_product(MACArray *mac, const TernaryWord *a, const TernaryWord *b, size_t len, uint8_t unit);
void        mac_matrix_vector(MACArray *mac, const TernaryWord *mat, const TernaryWord *vec,
                              size_t rows, size_t cols, int32_t *result, const uint8_t *unit_assign);

/* MAC unit management */
uint8_t     mac_status_read(const MACArray *mac, uint8_t unit);
bool        mac_status_write(MACArray *mac, uint8_t unit, uint8_t status);
bool        mac_reset(MACArray *mac, uint8_t unit);
void        mac_reset_all(MACArray *mac);
int32_t     mac_get_accumulator(const MACArray *mac, uint8_t unit);
bool        mac_set_accumulator(MACArray *mac, uint8_t unit, int32_t value);

/* Parallel operations */
void        mac_parallel_multiply(MACArray *mac, const TernaryWord *a, const TernaryWord *b,
                                  TernaryWord *results, size_t count);

/* Test entry point */
void        test_fpga_mac(void);

#endif /* FPGA_MAC_H */
