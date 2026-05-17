# SPDX-License-Identifier: Apache-2.0
# t27/python/t27/quantizers.py
# Quantization utilities for ML workloads

from typing import Union, List, Tuple, Optional
import numpy as np

from t27.formats import Int8, Int4, NF4, GF16, FP8_E4M3, FP8_E5M2


class Quantizer:
    """Base class for quantizers"""

    def __init__(self, target_format: type):
        self.target_format = target_format
        self.scale: Optional[float] = None
        self.zero_point: Optional[int] = None

    def calibrate(self, values: np.ndarray) -> 'Quantizer':
        """
        Calibrate quantizer parameters from data.

        Args:
            values: Array of float values to quantize

        Returns:
            Self for chaining
        """
        raise NotImplementedError

    def quantize(self, values: Union[float, np.ndarray]) -> Union[int, np.ndarray]:
        """Quantize value(s) to target format"""
        raise NotImplementedError

    def dequantize(self, values: Union[int, np.ndarray]) -> Union[float, np.ndarray]:
        """Dequantize value(s) from target format"""
        raise NotImplementedError


class Int8Quantizer(Quantizer):
    """Symmetric Int8 quantizer for weights/activations"""

    def __init__(self, symmetric: bool = True):
        super().__init__(Int8)
        self.symmetric = symmetric

    def calibrate(self, values: np.ndarray) -> 'Int8Quantizer':
        if self.symmetric:
            # Symmetric: scale = max(|values|) / 127
            max_val = np.max(np.abs(values))
            self.scale = max_val / 127.0
            self.zero_point = 0
        else:
            # Asymmetric: min/max based
            min_val, max_val = np.min(values), np.max(values)
            self.scale = (max_val - min_val) / 255.0
            self.zero_point = int(round(-min_val / self.scale)) - 128
        return self

    def quantize(self, values: Union[float, np.ndarray]) -> Union[int, np.ndarray]:
        if isinstance(values, (int, float)):
            return Int8.from_float(values / (self.scale or 1.0))
        return np.array([Int8.from_float(v / (self.scale or 1.0)) for v in values])

    def dequantize(self, values: Union[int, np.ndarray]) -> Union[float, np.ndarray]:
        if isinstance(values, (int, np.integer)):
            return Int8.to_float(values) * (self.scale or 1.0)
        return np.array([Int8.to_float(int(v)) * (self.scale or 1.0) for v in values])


class Int4Quantizer(Quantizer):
    """Symmetric Int4 quantizer (range: -8 to 7)"""

    def __init__(self):
        super().__init__(Int4)

    def calibrate(self, values: np.ndarray) -> 'Int4Quantizer':
        max_val = np.max(np.abs(values))
        self.scale = max_val / 7.0
        return self

    def quantize(self, values: Union[float, np.ndarray]) -> Union[int, np.ndarray]:
        if isinstance(values, (int, float)):
            return Int4.from_float(values / (self.scale or 1.0))
        return np.array([Int4.from_float(v / (self.scale or 1.0)) for v in values])

    def dequantize(self, values: Union[int, np.ndarray]) -> Union[float, np.ndarray]:
        if isinstance(values, (int, np.integer)):
            return Int4.to_float(values) * (self.scale or 1.0)
        return np.array([Int4.to_float(int(v)) * (self.scale or 1.0) for v in values])


class NF4Quantizer(Quantizer):
    """
    NormalFloat4 quantizer for QLoRA.

    NF4 uses 16 discrete values optimized for normal distribution.
    Values are symmetrically distributed around 0.
    """

    # Pre-computed NF4 quantization points (from QLoRA paper)
    NF4_VALUES = np.array([
        -1.0, -0.6962, -0.5251, -0.3949,
        -0.2124, -0.0875, 0.0,
        0.0875, 0.2124, 0.3949, 0.5251,
        0.6962, 1.0, float('inf'), float('inf')
    ])

    def __init__(self):
        super().__init__(NF4)

    def calibrate(self, values: np.ndarray) -> 'NF4Quantizer':
        # NF4 uses distribution-based scaling
        # Scale based on standard deviation
        std_val = np.std(values)
        self.scale = max(std_val, 1e-6)
        return self

    def quantize(self, values: Union[float, np.ndarray]) -> Union[int, np.ndarray]:
        if isinstance(values, (int, float)):
            scaled = values / self.scale
            # Find nearest NF4 value
            idx = np.argmin(np.abs(self.NF4_VALUES - scaled))
            return idx & 0xF
        return np.array([self.quantize(v) for v in values])

    def dequantize(self, values: Union[int, np.ndarray]) -> Union[float, np.ndarray]:
        if isinstance(values, (int, np.integer)):
            idx = values & 0xF
            return self.NF4_VALUES[idx] * self.scale
        return np.array([self.NF4_VALUES[int(v) & 0xF] * self.scale for v in values])


class GF16Quantizer(Quantizer):
    """GoldenFloat16 quantizer for high-precision weights"""

    def __init__(self):
        super().__init__(GF16)

    def calibrate(self, values: np.ndarray) -> 'GF16Quantizer':
        # GF16 doesn't need per-tensor scaling
        return self

    def quantize(self, values: Union[float, np.ndarray]) -> Union[int, np.ndarray]:
        if isinstance(values, (int, float)):
            return GF16.from_float(values).bits
        return np.array([GF16.from_float(v).bits for v in values])

    def dequantize(self, values: Union[int, np.ndarray]) -> Union[float, np.ndarray]:
        if isinstance(values, (int, np.integer)):
            return GF16(values).to_float()
        return np.array([GF16(int(v)).to_float() for v in values])


class FP8Quantizer(Quantizer):
    """FP8 quantizer (OCP standard)"""

    def __init__(self, variant: str = 'e4m3'):
        """
        Args:
            variant: 'e4m3' for higher precision, 'e5m2' for higher dynamic range
        """
        self.variant = variant
        super().__init__(FP8_E4M3 if variant == 'e4m3' else FP8_E5M2)

    def calibrate(self, values: np.ndarray) -> 'FP8Quantizer':
        # FP8 doesn't need per-tensor scaling
        return self

    def quantize(self, values: Union[float, np.ndarray]) -> Union[int, np.ndarray]:
        if self.variant == 'e4m3':
            cls = FP8_E4M3
        else:
            cls = FP8_E5M2

        if isinstance(values, (int, float)):
            return cls.from_float(values).bits
        return np.array([cls.from_float(v).bits for v in values])

    def dequantize(self, values: Union[int, np.ndarray]) -> Union[float, np.ndarray]:
        if self.variant == 'e4m3':
            cls = FP8_E4M3
        else:
            cls = FP8_E5M2

        if isinstance(values, (int, np.integer)):
            return cls(values).to_float()
        return np.array([cls(int(v)).to_float() for v in values])


# Helper functions for common quantization workflows

def quantize_tensor(
    tensor: np.ndarray,
    quantizer: Quantizer,
    calibrate: bool = True
) -> Tuple[np.ndarray, Quantizer]:
    """
    Quantize a tensor.

    Args:
        tensor: Input tensor (float32)
        quantizer: Quantizer instance
        calibrate: Whether to calibrate quantizer from tensor

    Returns:
        Tuple of (quantized tensor, calibrated quantizer)
    """
    if calibrate:
        quantizer.calibrate(tensor)

    quantized = np.zeros(tensor.shape, dtype=np.int8)
    for idx in np.ndindex(tensor.shape):
        quantized[idx] = quantizer.quantize(tensor[idx])

    return quantized, quantizer


def dequantize_tensor(
    quantized: np.ndarray,
    quantizer: Quantizer
) -> np.ndarray:
    """
    Dequantize a tensor.

    Args:
        quantized: Quantized tensor
        quantizer: Quantizer used for quantization

    Returns:
        Dequantized float tensor
    """
    dequantized = np.zeros(quantized.shape, dtype=np.float32)
    for idx in np.ndindex(quantized.shape):
        dequantized[idx] = quantizer.dequantize(quantized[idx])

    return dequantized


def quantization_error(
    original: np.ndarray,
    quantized_dequantized: np.ndarray,
    metric: str = 'mse'
) -> float:
    """
    Calculate quantization error.

    Args:
        original: Original float tensor
        quantized_dequantized: Dequantized tensor
        metric: 'mse', 'mae', 'max_abs', 'sqnr'

    Returns:
        Error value
    """
    diff = original - quantized_dequantized

    if metric == 'mse':
        return float(np.mean(diff ** 2))
    elif metric == 'mae':
        return float(np.mean(np.abs(diff)))
    elif metric == 'max_abs':
        return float(np.max(np.abs(diff)))
    elif metric == 'sqnr':
        # Signal to Quantization Noise Ratio
        signal_power = np.mean(original ** 2)
        noise_power = np.mean(diff ** 2)
        return float(10 * np.log10(signal_power / (noise_power + 1e-10)))
    else:
        raise ValueError(f"Unknown metric: {metric}")


# Common quantization configurations for different scenarios

QUANTIZATION_CONFIGS = {
    'int8_symmetric': {
        'quantizer': Int8Quantizer(symmetric=True),
        'use_case': 'General purpose inference',
    },
    'int8_asymmetric': {
        'quantizer': Int8Quantizer(symmetric=False),
        'use_case': 'Activation quantization',
    },
    'int4_weights': {
        'quantizer': Int4Quantizer(),
        'use_case': 'Weight quantization (4-bit)',
    },
    'nf4_qlora': {
        'quantizer': NF4Quantizer(),
        'use_case': 'QLoRA fine-tuning',
    },
    'fp8_e4m3_weights': {
        'quantizer': FP8Quantizer(variant='e4m3'),
        'use_case': 'Transformer weights (FP8)',
    },
    'fp8_e5m2_activations': {
        'quantizer': FP8Quantizer(variant='e5m2'),
        'use_case': 'Transformer activations (FP8)',
    },
    'gf16_high_precision': {
        'quantizer': GF16Quantizer(),
        'use_case': 'High-precision inference',
    },
}


def get_quantizer(config_name: str) -> Quantizer:
    """Get pre-configured quantizer by name"""
    return QUANTIZATION_CONFIGS[config_name]['quantizer']