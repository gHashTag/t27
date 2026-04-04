/* Auto-generated from specs/isa/registers.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/isa/registers.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 28 | Module: ISARegisters */

#include "registers.h"
#include <assert.h>
#include <string.h>

/* ===================================================================== */
/* 2. Coptic Alphabet Encoding                                            */
/* ===================================================================== */

const uint32_t COPTIC_ALPHABET[27] = {
    0x03B1,  /* alpha   (R0)  */
    0x03B2,  /* bita    (R1)  */
    0x03B3,  /* gamma   (R2)  */
    0x03B4,  /* dalda   (R3)  */
    0x03B5,  /* ei      (R4)  */
    0x03C6,  /* sima    (R5)  */
    0x03B6,  /* zata    (R6)  */
    0x03B7,  /* ita     (R7)  */
    0x03B8,  /* thita   (R8)  */
    0x03B9,  /* iota    (R9)  */
    0x03BA,  /* kappa   (R10) */
    0x03BB,  /* lauda   (R11) */
    0x03BC,  /* mi      (R12) */
    0x03BD,  /* ni      (R13) */
    0x03BE,  /* ksi     (R14) */
    0x03C0,  /* pi      (R15) */
    0x03C1,  /* ro      (R16) */
    0x03C3,  /* sigma   (R17) */
    0x03C4,  /* tau     (R18) */
    0x03C5,  /* upsilon (R19) */
    0x03C6,  /* fi      (R20) */
    0x03C7,  /* khi     (R21) */
    0x03C8,  /* psi     (R22) */
    0x03C9,  /* ou      (R23) */
    0x0417,  /* sampi   (R24) */
    0x0418,  /* koppa   (R25) */
    0x0419,  /* shei    (R26) */
};

/* ===================================================================== */
/* 3. Register File Init                                                  */
/* ===================================================================== */

void rf_init(RegisterFile *rf) {
    size_t i;
    for (i = 0; i < NUM_REGISTERS; i++) {
        rf->regs[i].raw = 0;
    }
}

/* ===================================================================== */
/* 4. Register Read/Write                                                 */
/* ===================================================================== */

TernaryWord rf_reg_read(const RegisterFile *rf, uint8_t reg) {
    TernaryWord zero_word = { .raw = 0 };

    /* R0 always returns 0 */
    if (reg == R0) {
        return zero_word;
    }

    /* Invalid register */
    if (reg > R26) {
        return zero_word;
    }

    return rf->regs[reg];
}

bool rf_reg_write(RegisterFile *rf, uint8_t reg, TernaryWord value) {
    /* R0 writes are ignored */
    if (reg == R0) {
        return false;
    }

    /* Invalid register */
    if (reg > R26) {
        return false;
    }

    rf->regs[reg] = value;
    return true;
}

/* ===================================================================== */
/* 5. Coptic Encoding Conversion                                          */
/* ===================================================================== */

uint32_t rf_reg_to_coptic(uint8_t reg) {
    if (reg > R26) {
        return 0;
    }
    return COPTIC_ALPHABET[reg];
}

uint8_t rf_coptic_to_reg(uint32_t cp) {
    uint8_t i;
    for (i = 0; i < 27; i++) {
        if (COPTIC_ALPHABET[i] == cp) {
            return i;
        }
    }
    return 0xFF;  /* Error sentinel */
}

/* ===================================================================== */
/* 6. Status Register Operations                                          */
/* ===================================================================== */

bool rf_status_read(const RegisterFile *rf, uint8_t flag) {
    TernaryWord status_val = rf_reg_read(rf, R20);
    uint32_t mask = (uint32_t)1 << flag;
    return (status_val.raw & mask) != 0;
}

bool rf_status_write(RegisterFile *rf, uint8_t flag, bool value) {
    if (flag > 5) {
        return false;
    }

    TernaryWord status_val = rf_reg_read(rf, R20);
    uint32_t mask = (uint32_t)1 << flag;

    if (value) {
        status_val.raw = status_val.raw | mask;
    } else {
        status_val.raw = status_val.raw & ~mask;
    }

    return rf_reg_write(rf, R20, status_val);
}

/* ===================================================================== */
/* 7. Stack Operations                                                    */
/* ===================================================================== */

bool rf_push_reg(RegisterFile *rf, uint8_t reg) {
    TernaryWord sp;

    (void)rf_reg_read(rf, reg);  /* Read value (would store to memory) */
    sp = rf_reg_read(rf, R17);
    sp.raw = sp.raw - 4;

    if (!rf_reg_write(rf, R17, sp)) {
        return false;
    }
    return true;
}

bool rf_pop_reg(RegisterFile *rf, uint8_t reg) {
    TernaryWord sp = rf_reg_read(rf, R17);
    TernaryWord value = { .raw = 0 };  /* Placeholder */

    sp.raw = sp.raw + 4;

    if (!rf_reg_write(rf, R17, sp)) {
        return false;
    }

    return rf_reg_write(rf, reg, value);
}

/* ===================================================================== */
/* 8. Context Save/Restore                                                */
/* ===================================================================== */

void rf_save_context(RegisterFile *rf) {
    rf_push_reg(rf, R5);
    rf_push_reg(rf, R6);
    rf_push_reg(rf, R7);
    rf_push_reg(rf, R8);
    rf_push_reg(rf, R9);
}

void rf_restore_context(RegisterFile *rf) {
    rf_pop_reg(rf, R9);
    rf_pop_reg(rf, R8);
    rf_pop_reg(rf, R7);
    rf_pop_reg(rf, R6);
    rf_pop_reg(rf, R5);
}

/* ===================================================================== */
/* Tests                                                                  */
/* ===================================================================== */

void test_isa_registers(void) {
    RegisterFile rf;

    /* test isa_r0_always_zero */
    {
        TernaryWord word = { .raw = 0x123456 };
        rf_init(&rf);
        rf_reg_write(&rf, R0, word);
        TernaryWord result = rf_reg_read(&rf, R0);
        assert(result.raw == 0);
    }

    /* test isa_r0_write_ignored */
    {
        rf_init(&rf);
        TernaryWord before = rf_reg_read(&rf, R0);
        bool success = rf_reg_write(&rf, R0, (TernaryWord){ .raw = 0xDEADBEEF });
        TernaryWord after = rf_reg_read(&rf, R0);
        assert(success == false);
        assert(before.raw == after.raw);
        assert(after.raw == 0);
    }

    /* test isa_register_count_27 */
    assert(NUM_REGISTERS == 27);

    /* test isa_valid_register_write_succeeds */
    {
        rf_init(&rf);
        TernaryWord value = { .raw = 0xABCDEF };
        bool success = rf_reg_write(&rf, R10, value);
        assert(success == true);
    }

    /* test isa_valid_register_write_read_roundtrip */
    {
        rf_init(&rf);
        TernaryWord value = { .raw = 0x123456 };
        rf_reg_write(&rf, R10, value);
        TernaryWord result = rf_reg_read(&rf, R10);
        assert(result.raw == value.raw);
    }

    /* test isa_invalid_register_write_fails */
    {
        rf_init(&rf);
        TernaryWord value = { .raw = 0x123456 };
        bool success = rf_reg_write(&rf, 27, value);
        assert(success == false);
    }

    /* test isa_invalid_register_read_returns_zero */
    {
        rf_init(&rf);
        TernaryWord result = rf_reg_read(&rf, 27);
        assert(result.raw == 0);
    }

    /* test isa_reg_to_coptic_r0 */
    assert(rf_reg_to_coptic(R0) == 0x03B1);

    /* test isa_reg_to_coptic_r10 */
    assert(rf_reg_to_coptic(R10) == 0x03BA);

    /* test isa_reg_to_coptic_r26 */
    assert(rf_reg_to_coptic(R26) == 0x0419);

    /* test isa_coptic_to_reg_alpha */
    assert(rf_coptic_to_reg(0x03B1) == R0);

    /* test isa_coptic_to_reg_kappa */
    assert(rf_coptic_to_reg(0x03BA) == R10);

    /* test isa_coptic_to_reg_invalid */
    assert(rf_coptic_to_reg(0x0041) == 0xFF);

    /* test isa_coptic_roundtrip_r0 */
    {
        uint32_t original_cp = rf_reg_to_coptic(R0);
        uint8_t recovered_reg = rf_coptic_to_reg(original_cp);
        uint32_t recovered_cp = rf_reg_to_coptic(recovered_reg);
        assert(original_cp == recovered_cp);
    }

    /* test isa_status_read_initial_false */
    {
        rf_init(&rf);
        assert(rf_status_read(&rf, FLAG_ZERO) == false);
    }

    /* test isa_status_write_set */
    {
        rf_init(&rf);
        rf_status_write(&rf, FLAG_ZERO, true);
        assert(rf_status_read(&rf, FLAG_ZERO) == true);
    }

    /* test isa_status_write_clear */
    {
        rf_init(&rf);
        rf_status_write(&rf, FLAG_ZERO, true);
        rf_status_write(&rf, FLAG_ZERO, false);
        assert(rf_status_read(&rf, FLAG_ZERO) == false);
    }

    /* test isa_status_flags_independent */
    {
        rf_init(&rf);
        rf_status_write(&rf, FLAG_ZERO, true);
        rf_status_write(&rf, FLAG_NEG, true);
        assert(rf_status_read(&rf, FLAG_ZERO) == true);
        assert(rf_status_read(&rf, FLAG_NEG) == true);
    }

    /* test isa_status_invalid_flag */
    {
        rf_init(&rf);
        bool success = rf_status_write(&rf, 10, true);
        assert(success == false);
    }

    /* test isa_register_aliases_match */
    assert(ARG0 == R1);
    assert(ARG1 == R2);
    assert(SP_REG == R17);
    assert(FP_REG == R16);

    /* test isa_coptic_alphabet_size_27 */
    assert(sizeof(COPTIC_ALPHABET) / sizeof(COPTIC_ALPHABET[0]) == 27);
}
