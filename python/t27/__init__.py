# SPDX-License-Identifier: Apache-2.0
# t27/python/t27/__init__.py
# Python bindings for t27 Toolchain

"""
t27 Python Bindings
===================

Python interface to t27:
- Number format conversions
- GoldenFloat arithmetic
- Quantization utilities
- RTL simulation helpers
"""

__version__ = "1.0.0"
__author__ = "Trinity t27 Team"

from t27.formats import (
    GF16, GF32, GF64, GF128, GF256,
    FP8_E4M3, FP8_E5M2,
    Int4, Int8,
    NF4, Posit16,
)
from t27.conversions import convert, convert_batch
from t27.quantizers import Quantizer, Dequantizer
from t27.phi import phi, phi_ratio, optimal_phi_distance

__all__ = [
    "GF16", "GF32", "GF64", "GF128", "GF256",
    "FP8_E4M3", "FP8_E5M2",
    "Int4", "Int8",
    "NF4", "Posit16",
    "convert", "convert_batch",
    "Quantizer", "Dequantizer",
    "phi", "phi_ratio", "optimal_phi_distance",
]