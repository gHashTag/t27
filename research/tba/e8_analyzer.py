#!/usr/bin/env python3
"""
E₈ Y-System Results Analyzer

Analyzes and visualizes optimization results from mass deformation scans.

Status: Week 4 Development
Date: 2026-04-06
"""

from __future__ import annotations
import numpy as np
import json
from pathlib import Path
from typing import List, Dict, Any, Tuple
import matplotlib.pyplot as plt


class E8ResultsAnalyzer:
    """Analyzer for E₈ Y-system optimization results."""

    def __init__(self, results_path: Path):
        self.results_path = results_path
        self.data = self._load_results()

    def _load_results(self) -> Dict[str, Any]:
        """Load JSON results file."""
        with open(self.results_path) as f:
            return json.load(f)

    def find_phi_power_matches(self, tolerance: float = 0.01) -> List[Dict]:
        """
        Find parameter sets that produce φ-power mass ratios.

        Returns: List of parameter sets with their φ-power matches.
        """
        phi = 1.618033988749895
        phi_powers = [phi**i for i in range(1, 6)]  # φ¹ to φ⁵

        matches = []

        # Scan through results (assuming results format)
        if "all_results" in self.data:
            for result in self.data["all_results"]:
                ratios = result.get("predicted_ratios", [])
                phi_matches = self._count_phi_matches(ratios, phi_powers, tolerance)

                if phi_matches > 0:
                    match_info = {
                        "params": result.get("params", {}),
                        "phi_matches": phi_matches,
                        "total_matches": result.get("n_matches", 0),
                        "ratios": ratios,
                    }
                    matches.append(match_info)

        return matches

    def _count_phi_matches(
        self,
        ratios: List[float],
        phi_powers: List[float],
        tolerance: float
    ) -> int:
        """Count how many ratios match φ powers."""
        matches = 0
        for ratio in ratios:
            for power_idx, phi_power in enumerate(phi_powers):
                if abs(ratio - phi_power) / phi_power < tolerance:
                    matches += 1
                    break
        return matches

    def find_sm_matches(self, target: float = 206.76, tolerance: float = 0.01) -> List[Dict]:
        """
        Find parameter sets that produce m_μ/m_e ≈ 206.76.

        Returns: List of matching parameter sets.
        """
        matches = []

        if "all_results" in self.data:
            for result in self.data["all_results"]:
                ratios = result.get("predicted_ratios", [])
                if ratios and abs(ratios[0] - target) / target < tolerance:
                    match_info = {
                        "params": result.get("params", {}),
                        "m_mu_me": ratios[0],
                        "error": abs(ratios[0] - target) / target * 100,
                        "all_ratios": ratios,
                    }
                    matches.append(match_info)

        return matches

    def analyze_parameter_distribution(self) -> Dict[str, Any]:
        """Analyze distribution of parameters in results."""
        if "all_results" not in self.data:
            return {}

        all_params = [r["params"] for r in self.data["all_results"]]
        param_names = [f"mu_{i}" for i in range(8)]

        distribution = {}
        for i, name in enumerate(param_names):
            values = [p[name] for p in all_params if name in p]
            if values:
                distribution[name] = {
                    "mean": np.mean(values),
                    "std": np.std(values),
                    "min": np.min(values),
                    "max": np.max(values),
                    "median": np.median(values),
                }

        return distribution

    def plot_error_landscape(self, save_path: Path = None) -> None:
        """
        Plot error landscape over parameter space.

        Simplified: shows error vs first two parameters.
        """
        if "all_results" not in self.data:
            print("No results to plot")
            return

        # Extract errors and first two parameters
        errors = [r["weighted_error"] for r in self.data["all_results"]]
        mu_0 = [r["params"]["mu_0"] for r in self.data["all_results"]]
        mu_1 = [r["params"]["mu_1"] for r in self.data["all_results"]]

        # Create scatter plot
        fig, ax = plt.subplots(figsize=(10, 8))
        scatter = ax.scatter(mu_0, mu_1, c=errors, cmap='viridis', s=20)
        plt.colorbar(scatter, ax=ax, label='Weighted Error')

        ax.set_xlabel('μ₀ (first mass parameter)')
        ax.set_ylabel('μ₁ (second mass parameter)')
        ax.set_title('E₈ Y-System Error Landscape')
        ax.grid(True, alpha=0.3)

        if save_path:
            plt.savefig(save_path, dpi=150, bbox_inches='tight')
            print(f"Plot saved to: {save_path}")
        else:
            plt.show()

    def plot_ratio_correlation(self, save_path: Path = None) -> None:
        """Plot correlation between predicted ratios."""
        if "all_results" not in self.data:
            print("No results to plot")
            return

        ratios = np.array([r["predicted_ratios"] for r in self.data["all_results"]])

        # Correlation matrix
        corr_matrix = np.corrcoef(ratios.T)

        # Plot heatmap
        fig, ax = plt.subplots(figsize=(10, 8))
        im = ax.imshow(corr_matrix, cmap='coolwarm', vmin=-1, vmax=1)
        plt.colorbar(im, ax=ax)

        ratio_names = ['m_μ/m_e', 'm_τ/m_e', 'm_c/m_e', 'm_b/m_e', 'm_t/m_e', 'm_s/m_e']
        ax.set_xticks(range(len(ratio_names)))
        ax.set_yticks(range(len(ratio_names)))
        ax.set_xticklabels(ratio_names)
        ax.set_yticklabels(ratio_names)
        ax.set_title('Mass Ratio Correlations')

        if save_path:
            plt.savefig(save_path, dpi=150, bbox_inches='tight')
            print(f"Plot saved to: {save_path}")
        else:
            plt.show()

    def plot_parameter_pareto(self, save_path: Path = None) -> None:
        """
        Plot Pareto front of optimization results.

        Shows trade-off between different objectives.
        """
        if "all_results" not in self.data:
            print("No results to plot")
            return

        # Extract: matches vs error
        n_matches = [r["n_matches"] for r in self.data["all_results"]]
        errors = [r["weighted_error"] for r in self.data["all_results"]]

        # Create scatter plot
        fig, ax = plt.subplots(figsize=(10, 8))
        scatter = ax.scatter(errors, n_matches, alpha=0.5, s=20)
        ax.set_xlabel('Weighted Error')
        ax.set_ylabel('Number of Matches (within 1%)')
        ax.set_title('Pareto Front: Error vs. SM Matches')
        ax.grid(True, alpha=0.3)

        # Highlight Pareto front
        # Sort by error, then keep points that improve matches
        sorted_results = sorted(self.data["all_results"], key=lambda r: r["weighted_error"])
        pareto = []
        best_matches = 0
        for r in sorted_results:
            if r["n_matches"] > best_matches:
                pareto.append(r)
                best_matches = r["n_matches"]

        if pareto:
            pareto_errors = [r["weighted_error"] for r in pareto]
            pareto_matches = [r["n_matches"] for r in pareto]
            ax.scatter(pareto_errors, pareto_matches, color='red', s=50, label='Pareto Front')
            ax.legend()

        if save_path:
            plt.savefig(save_path, dpi=150, bbox_inches='tight')
            print(f"Plot saved to: {save_path}")
        else:
            plt.show()

    def generate_report(self, output_path: Path) -> None:
        """Generate comprehensive analysis report."""
        report = []
        report.append("=" * 60)
        report.append("E₈ Y-System Results Analysis Report")
        report.append("=" * 60)
        report.append("")

        # φ-power matches
        phi_matches = self.find_phi_power_matches()
        report.append(f"φ-Power Matches (tolerance 1%): {len(phi_matches)}")
        for i, match in enumerate(phi_matches[:5]):  # Top 5
            report.append(f"  {i+1}. φ-matches: {match['phi_matches']}, total: {match['total_matches']}")
        report.append("")

        # SM matches
        sm_matches = self.find_sm_matches()
        report.append(f"m_μ/m_e ≈ 206.76 Matches: {len(sm_matches)}")
        for i, match in enumerate(sm_matches[:5]):  # Top 5
            report.append(f"  {i+1}. m_μ/m_e = {match['m_mu_me']:.2f} (error: {match['error']:.2f}%)")
        report.append("")

        # Parameter distribution
        dist = self.analyze_parameter_distribution()
        if dist:
            report.append("Parameter Distribution:")
            for name, stats in dist.items():
                report.append(f"  {name}: μ={stats['mean']:.4f}±{stats['std']:.4f} [{stats['min']:.2f}, {stats['max']:.2f}]")
            report.append("")

        # Best result
        if "best_result" in self.data:
            best = self.data["best_result"]
            report.append("Best Result:")
            report.append(f"  Weighted Error: {best['weighted_error']:.6e}")
            report.append(f"  Total Matches: {best['n_matches']}/6")
            report.append(f"  Parameters:")
            for i in range(8):
                param_name = f"mu_{i}"
                param_value = best['params'].get(param_name, 0)
                report.append(f"    μ_{i} = {param_value:.6f}")
            report.append(f"  Predicted Ratios:")
            names = ['m_μ/m_e', 'm_τ/m_e', 'm_c/m_e', 'm_b/m_e', 'm_t/m_e', 'm_s/m_e']
            for i, name in enumerate(names):
                report.append(f"    {name}: {best['predicted_ratios'][i]:.6f}")
        report.append("")
        report.append("=" * 60)

        # Save report
        with open(output_path, "w") as f:
            f.write("\n".join(report))

        print(f"Report saved to: {output_path}")
        print("\n" + "\n".join(report))


def analyze_random_results():
    """Analyze random search results."""
    analyzer = E8ResultsAnalyzer(Path("research/tba/e8_mass_random_results.json"))
    analyzer.generate_report(Path("research/tba/random_search_report.txt"))

    # Generate plots
    analyzer.plot_error_landscape(Path("research/tba/error_landscape.png"))
    analyzer.plot_parameter_pareto(Path("research/tba/pareto_front.png"))


def analyze_annealing_results():
    """Analyze simulated annealing results."""
    analyzer = E8ResultsAnalyzer(Path("research/tba/e8_mass_annealing_results.json"))
    analyzer.generate_report(Path("research/tba/annealing_report.txt"))

    # Generate plots
    analyzer.plot_ratio_correlation(Path("research/tba/ratio_correlations.png"))


if __name__ == "__main__":
    print("=" * 60)
    print("E₈ Y-System Results Analyzer")
    print("=" * 60)
    print("\nChoose analysis mode:")
    print("  1. Analyze random search results")
    print("  2. Analyze simulated annealing results")
    print("  3. Both")
    print()

    # For automation, run both
    print("Running both analyses...")
    print()

    analyze_random_results()
    print()
    analyze_annealing_results()
