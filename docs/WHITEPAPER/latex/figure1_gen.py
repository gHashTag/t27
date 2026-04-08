#!/usr/bin/env python3
"""
Figure 1 generator for NeurIPS 2026 GoldenFloat paper.

Panels:
    A. Number line comparison (IEEE754 vs Posit vs GF)
    B. Precision-range trade-off curve
    C. ML benchmark result (accuracy vs bits)

Requirements:
    - Vector PDF output
    - Computer Modern Roman font
    - 9pt size
    - No rainbow colormap
"""

import matplotlib.pyplot as plt
import matplotlib as mpl
import numpy as np
from pathlib import Path

# Publication settings
plt.rcParams.update({
    'font.family': 'serif',
    'font.serif': ['Computer Modern Roman'],
    'font.size': 9,
    'axes.labelsize': 9,
    'axes.titlesize': 9,
    'xtick.labelsize': 8,
    'ytick.labelsize': 8,
    'legend.fontsize': 8,
    'figure.dpi': 300,
    'savefig.dpi': 300,
    'savefig.format': 'pdf',
    'savefig.bbox': 'tight',
    'axes.unicode_minus': False,
})

# Colors - no rainbow, use professional palette
COLORS = {
    'ieee754': '#1f77b4',   # blue
    'posit': '#ff7f0e',     # orange
    'golden': '#2ca02c',    # green
    'baseline': '#7f7f7f',  # gray
    'highlight': '#d62728', # red
}

# Golden ratio
PHI = (np.sqrt(5) + 1) / 2
PHI_SQ = PHI ** 2

# ============================================================================
# Panel A: Number line comparison
# ============================================================================

def panel_a_number_line():
    """Compare representable density on number line for IEEE754, Posit, GF16."""
    fig, ax = plt.subplots(1, 1, figsize=(4.5, 1.5))

    # Range: [-2, 2] with focus on dense region near zero
    x = np.linspace(-0.5, 0.5, 1000)

    # Simulate representable points (simplified for visualization)
    # IEEE754 FP16: 5 exp, 10 mantissa -> uniform spacing in binade
    ieee_points = np.array([-0.25, -0.125, -0.0625, -0.03125, 0, 0.03125, 0.0625, 0.125, 0.25])

    # Posit16: tapered precision, denser near zero
    posit_points = np.array([-0.3, -0.15, -0.07, -0.03, -0.01, 0, 0.01, 0.03, 0.07, 0.15, 0.3])

    # GF16: 6 exp, 9 mantissa -> similar to IEEE but shifted
    gf_points = np.array([-0.28, -0.14, -0.07, -0.035, 0, 0.035, 0.07, 0.14, 0.28])

    # Plot number line
    ax.axhline(0, color='black', linewidth=0.5)

    # IEEE754
    ax.scatter(ieee_points, np.zeros_like(ieee_points) + 0.3,
               c=COLORS['ieee754'], s=30, alpha=0.7, label='IEEE FP16', marker='o')

    # Posit
    ax.scatter(posit_points, np.zeros_like(posit_points) + 0.15,
               c=COLORS['posit'], s=30, alpha=0.7, label='Posit16', marker='s')

    # GF16
    ax.scatter(gf_points, np.zeros_like(gf_points) + 0,
               c=COLORS['golden'], s=40, alpha=0.9, label='GF16', marker='^',
               edgecolors='black', linewidth=0.5)

    # Formatting
    ax.set_xlim(-0.4, 0.4)
    ax.set_ylim(-0.1, 0.45)
    ax.set_yticks([0, 0.15, 0.3])
    ax.set_yticklabels(['GF16', 'Posit16', 'IEEE FP16'])
    ax.set_xlabel('Value (normalized)')
    ax.set_title('(A) Representable point density near zero', fontsize=9, fontweight='bold')

    # Remove top/right spines
    ax.spines['top'].set_visible(False)
    ax.spines['right'].set_visible(False)
    ax.spines['left'].set_position('zero')

    return fig

# ============================================================================
# Panel B: Precision-range trade-off
# ============================================================================

def panel_b_precision_range():
    """Precision-range trade-off curve for different format families."""
    fig, ax = plt.subplots(1, 1, figsize=(4.5, 3))

    # Bits range
    bits = np.array([4, 8, 12, 16, 20, 24, 32])

    # Calculate precision (mantissa bits) and range (exponent bits) for each format
    # GF formats
    gf_e = np.round((bits - 1) / PHI_SQ).astype(int)
    gf_m = bits - 1 - gf_e - 1  # -1 for sign bit
    gf_precision = 2 ** gf_m  # representable values per binade
    gf_range = 2 ** (2 ** gf_e - 1)  # dynamic range (simplified)

    # IEEE-like (fixed ratio e/m ≈ 0.5)
    ieee_e = (bits * 0.3).astype(int)
    ieee_m = bits - 1 - ieee_e - 1
    ieee_precision = 2 ** ieee_m
    ieee_range = 2 ** (2 ** ieee_e - 1)

    # Posit-like (more exponent early, variable mantissa)
    posit_e = (bits * 0.4).astype(int)  # More exponent allocation
    posit_m = bits - 1 - posit_e - 1
    posit_precision = 2 ** posit_m
    posit_range = 2 ** (2 ** posit_e - 1)

    # Plot precision vs range (log-log scale)
    ax.loglog(gf_range, gf_precision, 'o-', color=COLORS['golden'],
              linewidth=2, markersize=6, label='GoldenFloat')
    ax.loglog(ieee_range, ieee_precision, 's--', color=COLORS['ieee754'],
              linewidth=1.5, markersize=5, label='IEEE-like')
    ax.loglog(posit_range, posit_precision, '^:', color=COLORS['posit'],
              linewidth=1.5, markersize=5, label='Posit-like')

    # Annotate GF16 point
    gf16_idx = 3  # GF16 is at index 3
    ax.annotate('GF16', xy=(gf_range[gf16_idx], gf_precision[gf16_idx]),
                xytext=(gf_range[gf16_idx]*2, gf_precision[gf16_idx]*0.5),
                arrowprops=dict(arrowstyle='->', color=COLORS['highlight']),
                fontsize=8, color=COLORS['highlight'])

    # Formatting
    ax.set_xlabel('Dynamic Range (max representable value)')
    ax.set_ylabel('Precision (values per binade)')
    ax.set_title('(B) Precision-Range Trade-off', fontsize=9, fontweight='bold')
    ax.legend(loc='lower right', framealpha=0.9)
    ax.grid(True, alpha=0.3, which='both')

    return fig

# ============================================================================
# Panel C: ML benchmark (simulated)
# ============================================================================

def panel_c_ml_benchmark():
    """ML benchmark: accuracy vs bits (simulated data for illustration)."""
    fig, ax = plt.subplots(1, 1, figsize=(4.5, 3))

    # Bits
    bits = np.array([4, 8, 12, 16, 20, 24, 32])

    # Simulated accuracy curves (declining with fewer bits, plateau at high bits)
    # Baseline FP32 = 100%
    baseline = np.full_like(bits, 100.0)

    # IEEE-like: sharp dropoff below 16 bits
    ieee_acc = np.array([45, 72, 88, 96, 98.5, 99.2, 99.8])

    # Posit-like: better at low bits due to tapered precision
    posit_acc = np.array([52, 78, 92, 97, 98.8, 99.4, 99.9])

    # GF (hypothesized): competitive due to φ-optimized allocation
    gf_acc = np.array([50, 76, 90, 97.5, 99.0, 99.5, 99.9])

    # Plot
    ax.plot(bits, baseline, 'k--', linewidth=1, label='FP32 baseline', alpha=0.5)
    ax.plot(bits, ieee_acc, 's-', color=COLORS['ieee754'],
            linewidth=2, markersize=6, label='IEEE-like')
    ax.plot(bits, posit_acc, '^-', color=COLORS['posit'],
            linewidth=2, markersize=6, label='Posit-like')
    ax.plot(bits, gf_acc, 'o-', color=COLORS['golden'],
            linewidth=2.5, markersize=7, label='GoldenFloat')

    # Highlight GF16
    gf16_acc_idx = 3
    ax.scatter([bits[gf16_acc_idx]], [gf_acc[gf16_acc_idx]],
               s=150, facecolors='none', edgecolors=COLORS['highlight'],
               linewidth=2, zorder=10)
    ax.annotate(f'GF16: {gf_acc[gf16_acc_idx]}%',
                xy=(bits[gf16_acc_idx], gf_acc[gf16_acc_idx]),
                xytext=(bits[gf16_acc_idx]-3, gf_acc[gf16_acc_idx]+3),
                fontsize=8, color=COLORS['highlight'],
                arrowprops=dict(arrowstyle='->', color=COLORS['highlight']))

    # Formatting
    ax.set_xlabel('Total Bits')
    ax.set_ylabel('Accuracy (%)')
    ax.set_title('(C) Simulated: Accuracy vs Bit Width', fontsize=9, fontweight='bold')
    ax.set_ylim(40, 102)
    ax.legend(loc='lower right', framealpha=0.9, ncol=2)
    ax.grid(True, alpha=0.3, axis='y')

    # Add note about simulation
    ax.text(0.02, 0.98, '* Simulated data; empirical validation pending',
            transform=ax.transAxes, fontsize=6, verticalalignment='top',
            style='italic', color='gray')

    return fig

# ============================================================================
# Combined Figure 1
# ============================================================================

def create_figure1():
    """Create combined Figure 1 with all three panels."""
    fig = plt.figure(figsize=(9, 4.5))

    # Create subplots: 2 columns, 2 rows (A spans top, B and C on bottom)
    # Actually, let's do 3 panels in a row for better twitter visibility
    gs = fig.add_gridspec(1, 3, hspace=0.3, wspace=0.4)

    # Panel A
    ax1 = fig.add_subplot(gs[0, 0])
    # Panel B
    ax2 = fig.add_subplot(gs[0, 1])
    # Panel C
    ax3 = fig.add_subplot(gs[0, 2])

    # Panel A: Number line
    x = np.linspace(-0.5, 0.5, 1000)
    ieee_points = np.array([-0.25, -0.125, -0.0625, -0.03125, 0, 0.03125, 0.0625, 0.125, 0.25])
    posit_points = np.array([-0.3, -0.15, -0.07, -0.03, -0.01, 0, 0.01, 0.03, 0.07, 0.15, 0.3])
    gf_points = np.array([-0.28, -0.14, -0.07, -0.035, 0, 0.035, 0.07, 0.14, 0.28])

    ax1.axhline(0, color='black', linewidth=0.5)
    ax1.scatter(ieee_points, np.zeros_like(ieee_points) + 0.3,
                c=COLORS['ieee754'], s=30, alpha=0.7, marker='o')
    ax1.scatter(posit_points, np.zeros_like(posit_points) + 0.15,
                c=COLORS['posit'], s=30, alpha=0.7, marker='s')
    ax1.scatter(gf_points, np.zeros_like(gf_points) + 0,
                c=COLORS['golden'], s=40, alpha=0.9, marker='^',
                edgecolors='black', linewidth=0.5, label='GF16', zorder=10)

    ax1.set_xlim(-0.4, 0.4)
    ax1.set_ylim(-0.1, 0.45)
    ax1.set_yticks([0, 0.15, 0.3])
    ax1.set_yticklabels(['GF16', 'Posit16', 'IEEE FP16'], fontsize=7)
    ax1.set_xlabel('Value', fontsize=8)
    ax1.set_title('(A) Point Density', fontsize=9, fontweight='bold')
    ax1.spines['top'].set_visible(False)
    ax1.spines['right'].set_visible(False)
    ax1.spines['left'].set_position('zero')
    ax1.tick_params(axis='both', labelsize=7)

    # Panel B: Precision-Range
    bits = np.array([4, 8, 12, 16, 20, 24, 32])
    gf_e = np.round((bits - 1) / PHI_SQ).astype(int)
    gf_m = bits - 1 - gf_e - 1
    gf_precision = 2 ** np.clip(gf_m, 0, 10)  # Clip for visualization
    gf_range = 2 ** np.clip(2 ** gf_e / 8, 1, 8)  # Simplified range metric

    ieee_e = (bits * 0.3).astype(int)
    ieee_m = np.maximum(bits - 1 - ieee_e - 1, 1)
    ieee_precision = 2 ** np.clip(ieee_m, 0, 10)
    ieee_range = 2 ** np.clip(2 ** ieee_e / 8, 1, 8)

    posit_e = (bits * 0.4).astype(int)
    posit_m = np.maximum(bits - 1 - posit_e - 1, 1)
    posit_precision = 2 ** np.clip(posit_m, 0, 10)
    posit_range = 2 ** np.clip(2 ** posit_e / 8, 1, 8)

    ax2.loglog(gf_range, gf_precision, 'o-', color=COLORS['golden'],
               linewidth=2, markersize=5, label='GF', zorder=10)
    ax2.loglog(ieee_range, ieee_precision, 's--', color=COLORS['ieee754'],
               linewidth=1.5, markersize=4, label='IEEE')
    ax2.loglog(posit_range, posit_precision, '^:', color=COLORS['posit'],
               linewidth=1.5, markersize=4, label='Posit')

    ax2.set_xlabel('Range', fontsize=8)
    ax2.set_ylabel('Precision', fontsize=8)
    ax2.set_title('(B) Precision-Range Trade-off', fontsize=9, fontweight='bold')
    ax2.legend(fontsize=7, loc='lower right', framealpha=0.9)
    ax2.grid(True, alpha=0.2, which='both')
    ax2.tick_params(axis='both', labelsize=7)

    # Panel C: ML Benchmark
    ieee_acc = np.array([45, 72, 88, 96, 98.5, 99.2, 99.8])
    posit_acc = np.array([52, 78, 92, 97, 98.8, 99.4, 99.9])
    gf_acc = np.array([50, 76, 90, 97.5, 99.0, 99.5, 99.9])

    ax3.plot(bits, ieee_acc, 's-', color=COLORS['ieee754'],
             linewidth=1.5, markersize=4, label='IEEE')
    ax3.plot(bits, posit_acc, '^-', color=COLORS['posit'],
             linewidth=1.5, markersize=4, label='Posit')
    ax3.plot(bits, gf_acc, 'o-', color=COLORS['golden'],
             linewidth=2, markersize=5, label='GF', zorder=10)

    ax3.scatter([bits[3]], [gf_acc[3]], s=120, facecolors='none',
                edgecolors=COLORS['highlight'], linewidth=2, zorder=15)
    ax3.annotate('GF16', xy=(bits[3], gf_acc[3]), xytext=(bits[3]-2.5, gf_acc[3]+2.5),
                 fontsize=7, color=COLORS['highlight'],
                 arrowprops=dict(arrowstyle='->', color=COLORS['highlight'], lw=1))

    ax3.set_xlabel('Bits', fontsize=8)
    ax3.set_ylabel('Accuracy (%)', fontsize=8)
    ax3.set_title('(C) Accuracy vs Bits*', fontsize=9, fontweight='bold')
    ax3.legend(fontsize=7, loc='lower right', framealpha=0.9, ncol=1)
    ax3.set_ylim(40, 101)
    ax3.grid(True, alpha=0.2, axis='y')
    ax3.tick_params(axis='both', labelsize=7)
    ax3.text(0.02, 0.02, '*Simulated', transform=ax3.transAxes,
             fontsize=6, style='italic', color='gray')

    plt.savefig('figure1.pdf', format='pdf', dpi=300, bbox_inches='tight')
    print("Figure 1 saved to: figure1.pdf")

    return fig


if __name__ == '__main__':
    output_dir = Path('/Users/playra/t27/docs/WHITEPAPER/latex')
    import os
    os.chdir(output_dir)

    # Create combined Figure 1
    fig = create_figure1()
    plt.close(fig)

    print("✓ Figure 1 generated successfully")
    print("  Output: /Users/playra/t27/docs/WHITEPAPER/latex/figure1.pdf")
