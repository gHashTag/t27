#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
# t27/python/t27/benchmarks.py
# Format conversion performance benchmarking utilities

import time
import statistics
from typing import Callable, List, Tuple
import numpy as np

from .formats import GF16, FP8_E4M3, Int8
from .conversions import convert, convert_batch
from .quantizers import Int8Quantizer


class BenchmarkResult:
    """Stores benchmark results for a single operation."""

    def __init__(self, name: str, times: List[float]):
        self.name = name
        self.times = times
        self.mean = statistics.mean(times)
        self.median = statistics.median(times)
        self.stddev = statistics.stdev(times) if len(times) > 1 else 0
        self.min = min(times)
        self.max = max(times)
        self.ops_per_sec = 1000.0 / self.mean  # ops/second for 1000 iter

    def __str__(self):
        return (f"{self.name}:\n"
                f"  Mean:   {self.mean*1000:.3f} ms\n"
                f"  Median: {self.median*1000:.3f} ms\n"
                f"  StdDev: {self.stddev*1000:.3f} ms\n"
                f"  Min:    {self.min*1000:.3f} ms\n"
                f"  Max:    {self.max*1000:.3f} ms\n"
                f"  Ops/s:  {self.ops_per_sec:,.0f}")


def benchmark(fn: Callable, iterations: int = 1000) -> List[float]:
    """Benchmark a function and return timing results."""
    times = []
    # Warmup
    for _ in range(10):
        fn()
    # Actual benchmark
    for _ in range(iterations):
        start = time.perf_counter()
        fn()
        end = time.perf_counter()
        times.append(end - start)
    return times


def benchmark_conversion(iterations: int = 1000) -> List[BenchmarkResult]:
    """Benchmark all format conversions."""
    results = []

    # GF16 to FP8
    gf16_val = GF16.from_float(1.5)
    def gf16_to_fp8():
        convert(gf16_val, FP8_E4M3)
    times = benchmark(gf16_to_fp8, iterations)
    results.append(BenchmarkResult("GF16 -> FP8_E4M3", times))

    # FP8 to GF16
    fp8_val = FP8_E4M3.from_float(1.5)
    def fp8_to_gf16():
        convert(fp8_val, GF16)
    times = benchmark(fp8_to_gf16, iterations)
    results.append(BenchmarkResult("FP8_E4M3 -> GF16", times))

    # Batch conversion (1000 elements)
    values = [float(i) / 100.0 for i in range(1000)]
    def batch_gf16():
        convert_batch(values, GF16)
    times = benchmark(batch_gf16, iterations // 10)  # Fewer for batch
    results.append(BenchmarkResult("Batch GF16 (1000 elems)", times))

    # Int8 quantization
    tensor = np.random.randn(100, 100).astype(np.float32)
    quantizer = Int8Quantizer(symmetric=True)
    quantizer.calibrate(tensor)
    def int8_quantize():
        quantizer.quantize(tensor)
    times = benchmark(int8_quantize, iterations // 10)
    results.append(BenchmarkResult("Int8 Quantize (10k elems)", times))

    return results


def run_all_benchmarks() -> None:
    """Run all benchmarks and print results."""
    print("=" * 50)
    print("Trinity t27 Format Conversion Benchmarks")
    print("=" * 50)
    print()

    results = benchmark_conversion()

    for r in results:
        print(r)
        print()

    # Summary table
    print("=" * 50)
    print("Summary")
    print("=" * 50)
    print(f"{'Operation':<25} {'Time (ms)':>12} {'Ops/sec':>12}")
    print("-" * 50)
    for r in results:
        print(f"{r.name:<25} {r.mean*1000:>11.3f} {r.ops_per_sec:>11,.0f}")


if __name__ == "__main__":
    run_all_benchmarks()