/* Auto-generated from specs/isa/registers.t27 */
/* DO NOT EDIT -- regenerate with: tri gen specs/isa/registers.t27 */
/* phi^2 + phi^-2 = 3 | TRINITY */
/* Ring: 28 | Module: ISARegisters */

#ifndef ISA_REGISTERS_H
#define ISA_REGISTERS_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

/* ===================================================================== */
/* TernaryWord: 27 trits packed into a u32                                */
/* ===================================================================== */

typedef struct {
    uint32_t raw;
} TernaryWord;

/* ===================================================================== */
/* 1. Register Constants                                                  */
/* ===================================================================== */

#define NUM_REGISTERS   27
#define REG_WIDTH       27
#define COPIC_BASE      27

/* Register identifiers */
#define R0   0    /* Zero register (always reads as 0) */
#define R1   1    /* Argument/return register */
#define R2   2    /* Argument/return register */
#define R3   3    /* Argument register */
#define R4   4    /* Argument register */
#define R5   5    /* Temporary register */
#define R6   6    /* Temporary register */
#define R7   7    /* Temporary register */
#define R8   8    /* Temporary register */
#define R9   9    /* Temporary register */
#define R10  10   /* Saved register S0 */
#define R11  11   /* Saved register S1 */
#define R12  12   /* Saved register S2 */
#define R13  13   /* Saved register S3 */
#define R14  14   /* Saved register S4 */
#define R15  15   /* Saved register S5 */
#define R16  16   /* Frame pointer */
#define R17  17   /* Stack pointer */
#define R18  18   /* Link register */
#define R19  19   /* Program counter */
#define R20  20   /* Status register */
#define R21  21   /* Exception handler */
#define R22  22   /* Kernel stack pointer */
#define R23  23   /* Thread pointer */
#define R24  24   /* Reserved */
#define R25  25   /* Reserved */
#define R26  26   /* Reserved */

/* Status register flags (R20) */
#define FLAG_ZERO       0
#define FLAG_NEG        1
#define FLAG_CARRY      2
#define FLAG_OVERFLOW   3
#define FLAG_TRAP       4
#define FLAG_INTERRUPT  5

/* Register aliases */
#define ARG0   R1
#define ARG1   R2
#define ARG2   R3
#define ARG3   R4
#define TMP0   R5
#define TMP1   R6
#define TMP2   R7
#define TMP3   R8
#define TMP4   R9
#define SAVED0 R10
#define SAVED1 R11
#define SAVED2 R12
#define SAVED3 R13
#define SAVED4 R14
#define SAVED5 R15
#define FP_REG R16
#define SP_REG R17
#define LR_REG R18
#define PC_REG R19
#define STATUS_REG R20
#define EH_REG R21
#define KSP_REG R22
#define TP_REG R23

/* ===================================================================== */
/* Coptic Alphabet                                                        */
/* ===================================================================== */

extern const uint32_t COPTIC_ALPHABET[27];

/* ===================================================================== */
/* Register File API                                                      */
/* ===================================================================== */

typedef struct {
    TernaryWord regs[NUM_REGISTERS];
} RegisterFile;

void         rf_init(RegisterFile *rf);
TernaryWord  rf_reg_read(const RegisterFile *rf, uint8_t reg);
bool         rf_reg_write(RegisterFile *rf, uint8_t reg, TernaryWord value);

uint32_t     rf_reg_to_coptic(uint8_t reg);
uint8_t      rf_coptic_to_reg(uint32_t cp);

bool         rf_status_read(const RegisterFile *rf, uint8_t flag);
bool         rf_status_write(RegisterFile *rf, uint8_t flag, bool value);

bool         rf_push_reg(RegisterFile *rf, uint8_t reg);
bool         rf_pop_reg(RegisterFile *rf, uint8_t reg);
void         rf_save_context(RegisterFile *rf);
void         rf_restore_context(RegisterFile *rf);

/* Test entry point */
void         test_isa_registers(void);

#endif /* ISA_REGISTERS_H */
