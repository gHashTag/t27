/**
 * GoldenFloat: Phi-structured floating-point formats for ML and scientific computing
 *
 * GoldenFloat formats are phi-optimal floating-point representations that
 * provide better precision for constants like φ (golden ratio) compared to
 * standard IEEE 754 formats.
 */

/**
 * GF16: 16-bit phi-structured floating-point format
 */
export class GF16 {
  private value: number;

  constructor(value: number);
  toFloat(): number;
  bits(): number;
  isZero(): boolean;
  isInf(): boolean;
  isNan(): boolean;
}

/**
 * GF32: 32-bit phi-structured floating-point format
 */
export class GF32 {
  private value: number;

  constructor(value: number);
  toFloat(): number;
  bits(): number;
  isZero(): boolean;
  isInf(): boolean;
  isNan(): boolean;
}

/**
 * Get the constant phi (golden ratio) as GF16
 */
export function phiGF16(): number;

/**
 * Get the constant phi (golden ratio) as GF32
 */
export function phiGF32(): number;

/**
 * Check if a value represents positive zero
 */
export function isPositiveZero(value: number): boolean;

/**
 * Check if a value represents negative zero
 */
export function isNegativeZero(value: number): boolean;
