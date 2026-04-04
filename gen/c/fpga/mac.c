/* Auto-generated from specs/fpga/mac.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/fpga/mac.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 28 | Module: ZeroDSP_MAC */

#include "mac.h"
#include <assert.h>
#include <string.h>

/* ===================================================================== */
/* 2. Ternary Multiplication LUT                                          */
/* ===================================================================== */

const int8_t MAC_LUT[9] = {
     1,   /* (-1,-1) -> +1 */
     0,   /* (-1, 0) ->  0 */
    -1,   /* (-1,+1) -> -1 */
     0,   /* ( 0,-1) ->  0 */
     0,   /* ( 0, 0) ->  0 */
     0,   /* ( 0,+1) ->  0 */
    -1,   /* (+1,-1) -> -1 */
     0,   /* (+1, 0) ->  0 */
     1,   /* (+1,+1) -> +1 */
};

/* ===================================================================== */
/* 3. Init                                                                */
/* ===================================================================== */

void mac_array_init(MACArray *mac) {
    size_t i, j;
    for (i = 0; i < NUM_MAC_UNITS; i++) {
        mac->units[i].accumulator = 0;
        mac->units[i].status = MAC_STATUS_READY;
        for (j = 0; j < PIPELINE_STAGES; j++) {
            mac->units[i].pipeline[j].raw = 0;
        }
    }
}

/* ===================================================================== */
/* 4. Trit Extraction                                                     */
/* ===================================================================== */

Trit mac_extract_trit(TernaryWord word, size_t index) {
    uint32_t bit_pos = (uint32_t)(index * 2);
    uint32_t encoded = (word.raw >> bit_pos) & 3;
    if (encoded == 2) return TRIT_NEG;
    if (encoded == 1) return TRIT_POS;
    return TRIT_ZERO;
}

uint32_t mac_pack_trit(Trit trit, size_t index) {
    uint32_t bit_pos = (uint32_t)(index * 2);
    uint32_t encoded;
    if (trit == TRIT_NEG) encoded = 2;
    else if (trit == TRIT_POS) encoded = 1;
    else encoded = 0;
    return encoded << bit_pos;
}

/* ===================================================================== */
/* 5. MAC Operations                                                      */
/* ===================================================================== */

TernaryWord mac_multiply(MACArray *mac, TernaryWord a, TernaryWord b, uint8_t unit) {
    TernaryWord zero_word = { .raw = 0 };
    size_t i;

    if (unit >= NUM_MAC_UNITS) {
        return zero_word;
    }

    mac->units[unit].status = MAC_STATUS_BUSY;
    uint32_t result = 0;

    for (i = 0; i < MAC_WIDTH; i++) {
        Trit a_trit = mac_extract_trit(a, i);
        Trit b_trit = mac_extract_trit(b, i);
        int a_idx = (int)a_trit + 1;
        int b_idx = (int)b_trit + 1;
        int lut_idx = a_idx * 3 + b_idx;
        int8_t product = MAC_LUT[lut_idx];

        if (product == 1) {
            result |= mac_pack_trit(TRIT_POS, i);
        } else if (product == -1) {
            result |= mac_pack_trit(TRIT_NEG, i);
        }
    }

    mac->units[unit].status = MAC_STATUS_DONE;
    TernaryWord res = { .raw = result };
    return res;
}

int32_t mac_cycle(MACArray *mac, TernaryWord a, TernaryWord b, uint8_t unit, int32_t acc) {
    size_t i;

    if (unit >= NUM_MAC_UNITS) {
        return 0;
    }

    mac->units[unit].status = MAC_STATUS_BUSY;
    int32_t dot = 0;

    for (i = 0; i < MAC_WIDTH; i++) {
        Trit a_trit = mac_extract_trit(a, i);
        Trit b_trit = mac_extract_trit(b, i);
        dot += (int32_t)a_trit * (int32_t)b_trit;
    }

    mac->units[unit].accumulator = acc + dot;
    mac->units[unit].status = MAC_STATUS_DONE;
    return mac->units[unit].accumulator;
}

int32_t mac_dot_product(MACArray *mac, const TernaryWord *a, const TernaryWord *b, size_t len, uint8_t unit) {
    size_t i;

    if (unit >= NUM_MAC_UNITS) {
        return 0;
    }

    mac->units[unit].status = MAC_STATUS_BUSY;
    mac->units[unit].accumulator = 0;

    for (i = 0; i < len; i++) {
        mac->units[unit].accumulator = mac_cycle(
            mac, a[i], b[i], unit, mac->units[unit].accumulator
        );
    }

    mac->units[unit].status = MAC_STATUS_DONE;
    return mac->units[unit].accumulator;
}

void mac_matrix_vector(MACArray *mac, const TernaryWord *mat, const TernaryWord *vec,
                       size_t rows, size_t cols, int32_t *result, const uint8_t *unit_assign) {
    size_t row, col;

    for (row = 0; row < rows; row++) {
        uint8_t unit = unit_assign[row];
        mac->units[unit].accumulator = 0;

        for (col = 0; col < cols; col++) {
            size_t mat_idx = row * cols + col;
            mac->units[unit].accumulator = mac_cycle(
                mac, mat[mat_idx], vec[col], unit, mac->units[unit].accumulator
            );
        }

        result[row] = mac->units[unit].accumulator;
    }
}

/* ===================================================================== */
/* 6. MAC Unit Management                                                 */
/* ===================================================================== */

uint8_t mac_status_read(const MACArray *mac, uint8_t unit) {
    if (unit >= NUM_MAC_UNITS) return 0xFF;
    return mac->units[unit].status;
}

bool mac_status_write(MACArray *mac, uint8_t unit, uint8_t status) {
    if (unit >= NUM_MAC_UNITS) return false;
    mac->units[unit].status = status;
    return true;
}

bool mac_reset(MACArray *mac, uint8_t unit) {
    size_t j;
    if (unit >= NUM_MAC_UNITS) return false;
    mac->units[unit].accumulator = 0;
    mac->units[unit].status = MAC_STATUS_READY;
    for (j = 0; j < PIPELINE_STAGES; j++) {
        mac->units[unit].pipeline[j].raw = 0;
    }
    return true;
}

void mac_reset_all(MACArray *mac) {
    size_t i;
    for (i = 0; i < NUM_MAC_UNITS; i++) {
        mac_reset(mac, (uint8_t)i);
    }
}

int32_t mac_get_accumulator(const MACArray *mac, uint8_t unit) {
    if (unit >= NUM_MAC_UNITS) return 0;
    return mac->units[unit].accumulator;
}

bool mac_set_accumulator(MACArray *mac, uint8_t unit, int32_t value) {
    if (unit >= NUM_MAC_UNITS) return false;
    mac->units[unit].accumulator = value;
    return true;
}

/* ===================================================================== */
/* 7. Parallel MAC Operations                                             */
/* ===================================================================== */

void mac_parallel_multiply(MACArray *mac, const TernaryWord *a, const TernaryWord *b,
                           TernaryWord *results, size_t count) {
    size_t i;
    for (i = 0; i < count; i++) {
        uint8_t unit = (uint8_t)(i % NUM_MAC_UNITS);
        results[i] = mac_multiply(mac, a[i], b[i], unit);
    }
}

/* ===================================================================== */
/* Tests                                                                  */
/* ===================================================================== */

void test_fpga_mac(void) {
    MACArray mac;

    /* test mac_lut_multiply_pos_pos */
    {
        mac_array_init(&mac);
        uint32_t set_trit = mac_pack_trit(TRIT_POS, 0);
        TernaryWord a = { .raw = set_trit };
        TernaryWord b = { .raw = set_trit };
        TernaryWord result = mac_multiply(&mac, a, b, 0);
        Trit result_trit = mac_extract_trit(result, 0);
        assert(result_trit == TRIT_POS);
    }

    /* test mac_lut_multiply_neg_neg */
    {
        mac_array_init(&mac);
        uint32_t set_trit = mac_pack_trit(TRIT_NEG, 0);
        TernaryWord a = { .raw = set_trit };
        TernaryWord b = { .raw = set_trit };
        TernaryWord result = mac_multiply(&mac, a, b, 0);
        Trit result_trit = mac_extract_trit(result, 0);
        assert(result_trit == TRIT_POS);
    }

    /* test mac_lut_multiply_pos_neg */
    {
        mac_array_init(&mac);
        TernaryWord a = { .raw = mac_pack_trit(TRIT_POS, 0) };
        TernaryWord b = { .raw = mac_pack_trit(TRIT_NEG, 0) };
        TernaryWord result = mac_multiply(&mac, a, b, 0);
        Trit result_trit = mac_extract_trit(result, 0);
        assert(result_trit == TRIT_NEG);
    }

    /* test mac_lut_multiply_with_zero */
    {
        mac_array_init(&mac);
        TernaryWord a = { .raw = mac_pack_trit(TRIT_POS, 0) };
        TernaryWord b = { .raw = mac_pack_trit(TRIT_ZERO, 0) };
        TernaryWord result = mac_multiply(&mac, a, b, 0);
        Trit result_trit = mac_extract_trit(result, 0);
        assert(result_trit == TRIT_ZERO);
    }

    /* test mac_lut_size_9 */
    assert(sizeof(MAC_LUT) / sizeof(MAC_LUT[0]) == 9);

    /* test mac_num_units_8 */
    assert(NUM_MAC_UNITS == 8);

    /* test mac_width_27 */
    assert(MAC_WIDTH == 27);

    /* test mac_pipeline_stages_4 */
    assert(PIPELINE_STAGES == 4);

    /* test mac_cycle_with_zero_accumulator */
    {
        mac_array_init(&mac);
        TernaryWord a = { .raw = mac_pack_trit(TRIT_POS, 0) | mac_pack_trit(TRIT_POS, 1) };
        TernaryWord b = { .raw = mac_pack_trit(TRIT_POS, 0) | mac_pack_trit(TRIT_POS, 1) };
        int32_t result = mac_cycle(&mac, a, b, 0, 0);
        assert(result == 2);
    }

    /* test mac_cycle_with_initial_accumulator */
    {
        mac_array_init(&mac);
        TernaryWord a = { .raw = mac_pack_trit(TRIT_POS, 0) };
        TernaryWord b = { .raw = mac_pack_trit(TRIT_POS, 0) };
        int32_t result = mac_cycle(&mac, a, b, 0, 5);
        assert(result == 6);
    }

    /* test mac_dot_product_simple */
    {
        mac_array_init(&mac);
        TernaryWord a[] = {
            { .raw = mac_pack_trit(TRIT_POS, 0) },
            { .raw = mac_pack_trit(TRIT_POS, 0) }
        };
        TernaryWord b[] = {
            { .raw = mac_pack_trit(TRIT_POS, 0) },
            { .raw = mac_pack_trit(TRIT_POS, 0) }
        };
        int32_t result = mac_dot_product(&mac, a, b, 2, 0);
        assert(result == 2);
    }

    /* test mac_dot_product_with_negatives */
    {
        mac_array_init(&mac);
        TernaryWord a[] = {
            { .raw = mac_pack_trit(TRIT_POS, 0) },
            { .raw = mac_pack_trit(TRIT_NEG, 0) }
        };
        TernaryWord b[] = {
            { .raw = mac_pack_trit(TRIT_POS, 0) },
            { .raw = mac_pack_trit(TRIT_POS, 0) }
        };
        int32_t result = mac_dot_product(&mac, a, b, 2, 0);
        assert(result == 0);
    }

    /* test mac_status_initially_ready */
    {
        mac_array_init(&mac);
        assert(mac_status_read(&mac, 0) == MAC_STATUS_READY);
    }

    /* test mac_status_after_operation_is_done */
    {
        mac_array_init(&mac);
        TernaryWord a = { .raw = mac_pack_trit(TRIT_POS, 0) };
        TernaryWord b = { .raw = mac_pack_trit(TRIT_POS, 0) };
        mac_multiply(&mac, a, b, 0);
        assert(mac_status_read(&mac, 0) == MAC_STATUS_DONE);
    }

    /* test mac_reset_clears_accumulator */
    {
        mac_array_init(&mac);
        TernaryWord a = { .raw = mac_pack_trit(TRIT_POS, 0) };
        TernaryWord b = { .raw = mac_pack_trit(TRIT_POS, 0) };
        mac_cycle(&mac, a, b, 0, 0);
        mac_reset(&mac, 0);
        assert(mac_get_accumulator(&mac, 0) == 0);
    }

    /* test mac_reset_clears_status */
    {
        mac_array_init(&mac);
        TernaryWord a = { .raw = mac_pack_trit(TRIT_POS, 0) };
        TernaryWord b = { .raw = mac_pack_trit(TRIT_POS, 0) };
        mac_multiply(&mac, a, b, 0);
        mac_reset(&mac, 0);
        assert(mac_status_read(&mac, 0) == MAC_STATUS_READY);
    }

    /* test mac_invalid_unit_returns_zero */
    {
        mac_array_init(&mac);
        TernaryWord result = mac_multiply(&mac, (TernaryWord){.raw = 0}, (TernaryWord){.raw = 0}, 99);
        assert(result.raw == 0);
    }

    /* test mac_extract_trit_zero */
    {
        TernaryWord word = { .raw = 0 };
        assert(mac_extract_trit(word, 0) == TRIT_ZERO);
    }

    /* test mac_extract_trit_pos */
    {
        TernaryWord word = { .raw = 1 };  /* 0b01 */
        assert(mac_extract_trit(word, 0) == TRIT_POS);
    }

    /* test mac_extract_trit_neg */
    {
        TernaryWord word = { .raw = 2 };  /* 0b10 */
        assert(mac_extract_trit(word, 0) == TRIT_NEG);
    }

    /* test mac_pack_trit_roundtrip */
    {
        Trit original = TRIT_POS;
        uint32_t packed = mac_pack_trit(original, 0);
        TernaryWord word = { .raw = packed };
        Trit extracted = mac_extract_trit(word, 0);
        assert(extracted == original);
    }

    /* test mac_matrix_vector_2x2 */
    {
        mac_array_init(&mac);
        TernaryWord mat[] = {
            { .raw = mac_pack_trit(TRIT_POS, 0) },
            { .raw = mac_pack_trit(TRIT_ZERO, 0) },
            { .raw = mac_pack_trit(TRIT_ZERO, 0) },
            { .raw = mac_pack_trit(TRIT_POS, 0) }
        };
        TernaryWord vec[] = {
            { .raw = mac_pack_trit(TRIT_POS, 0) },
            { .raw = mac_pack_trit(TRIT_POS, 0) }
        };
        int32_t result[2] = {0, 0};
        uint8_t units[] = {0, 1};
        mac_matrix_vector(&mac, mat, vec, 2, 2, result, units);
        assert(result[0] == 1);
        assert(result[1] == 1);
    }
}
