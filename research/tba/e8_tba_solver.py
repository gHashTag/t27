#!/usr/bin/env python3
"""
E₈ TBA (Thermodynamic Bethe Ansatz) Solver
============================================
PROJECT KEPLER→NEWTON

Solves the TBA integral equations for the E₈ affine Toda field theory
to compute the exact (non-perturbative) ground state energy and mass spectrum.

The TBA equations for E₈ (8 coupled particles, purely elastic scattering):

  εₐ(θ) = mₐR cosh(θ) - Σ_b φₐᵦ * Lᵦ(θ)

where:
  εₐ(θ) = pseudo-energy of particle a at rapidity θ
  mₐ = mass of particle a (Zamolodchikov spectrum)
  R = system size (inverse temperature)
  φₐᵦ(θ) = scattering kernel (derivative of phase shift)
  Lᵦ(θ) = log(1 + exp(-εᵦ(θ)))
  * denotes convolution: (f*g)(θ) = ∫ f(θ-θ')g(θ') dθ'/2π

The scattering matrix S_{ab}(θ) for E₈ Toda is known exactly
(Braden-Corrigan-Dorey-Sasaki 1990, Christe-Mussardo 1990).

Ground state energy:
  E₀(R) = -Σₐ mₐ/(2π) ∫ cosh(θ) Lₐ(θ) dθ

Effective central charge:
  c_eff(R) = 6R²/(π) E₀(R)  → should give c = 1/2 (Ising) as R → 0

References:
  - Al.B. Zamolodchikov, NPB 342 (1990) 695 — original TBA
  - Christe & Mussardo, NPB 330 (1990) 465 — E₈ S-matrix
  - Dorey, NPB 358 (1991) 654 — E₈ mass ratios from root systems
  - Klassen & Melzer, NPB 338 (1990) 485 — numerical TBA for E₈
"""

import numpy as np
from scipy.integrate import quad, simpson
from scipy.interpolate import interp1d
import math
import json
import os

PHI = (1 + math.sqrt(5)) / 2
PI = math.pi

# ═══════════════════════════════════════════════════════════════
# E₈ Mass Spectrum (exact Zamolodchikov values, normalized to m₁=1)
# ═══════════════════════════════════════════════════════════════

def zamolodchikov_masses():
    """Exact E₈ mass ratios from affine Toda S-matrix bootstrap"""
    return np.array([
        1.0,
        2 * math.cos(PI/5),                              # φ
        2 * math.cos(PI/30),
        4 * math.cos(PI/5) * math.cos(7*PI/30),
        4 * math.cos(PI/5) * math.cos(2*PI/15),
        4 * math.cos(PI/5) * math.cos(PI/30),
        8 * math.cos(PI/5)**2 * math.cos(7*PI/30),
        8 * math.cos(PI/5)**2 * math.cos(2*PI/15),
    ])

# ═══════════════════════════════════════════════════════════════
# E₈ S-matrix kernels
# ═══════════════════════════════════════════════════════════════

# The E₈ S-matrix is built from "building blocks" {x}:
# {x}(θ) = sinh(θ/2 + iπx/60) / sinh(θ/2 - iπx/60)
#
# The kernel φₐᵦ(θ) = -i d/dθ log(Sₐᵦ(θ))
# For the Fourier transform: φ̃ₐᵦ(k) = ∫ φₐᵦ(θ) e^{ikθ} dθ
#
# For block {x}: φ̃{x}(k) = e^{-πk|x|/30} / cosh(πk/2)
# (up to normalization)

# The E₈ S-matrix elements Sₐᵦ are products of blocks.
# Following Christe-Mussardo (1990) / Dorey (1991):

# For simplicity, use the DIAGONAL approximation first:
# In the E₈ Toda theory, the S-matrix is purely elastic (diagonal).
# The ADE kernel for simply-laced E₈:
# φₐᵦ(θ) related to the E₈ incidence matrix Iₐᵦ

# E₈ incidence matrix (adjacency of Dynkin diagram)
E8_INCIDENCE = np.array([
    [0, 1, 0, 0, 0, 0, 0, 0],
    [1, 0, 1, 0, 0, 0, 0, 0],
    [0, 1, 0, 1, 0, 0, 0, 0],
    [0, 0, 1, 0, 1, 0, 0, 0],
    [0, 0, 0, 1, 0, 1, 0, 1],
    [0, 0, 0, 0, 1, 0, 1, 0],
    [0, 0, 0, 0, 0, 1, 0, 0],
    [0, 0, 0, 0, 1, 0, 0, 0],
], dtype=float)

def kernel_universal(theta, h=30):
    """Universal ADE kernel: φ(θ) = h/(2 cosh(hθ/2))
    This is the leading-order kernel for ADE Toda theories."""
    return h / (2 * np.cosh(h * theta / 2) + 1e-300)

def kernel_matrix(theta):
    """Full kernel matrix φₐᵦ(θ) for E₈
    Leading order: φₐᵦ(θ) = Iₐᵦ × φ_universal(θ)
    where Iₐᵦ is the incidence matrix."""
    phi_univ = 1.0 / (2 * np.cosh(theta) + 1e-300)
    return E8_INCIDENCE * phi_univ

# ═══════════════════════════════════════════════════════════════
# TBA Solver (Iterative method, following Zamolodchikov 1990)
# ═══════════════════════════════════════════════════════════════

class E8TBASolver:
    def __init__(self, R=1.0, N_theta=200, theta_max=15.0):
        """
        R: system size (dimensionless, in units of 1/m₁)
        N_theta: number of rapidity grid points
        theta_max: maximum rapidity
        """
        self.R = R
        self.masses = zamolodchikov_masses()
        self.N_particles = 8
        
        # Rapidity grid
        self.N_theta = N_theta
        self.theta_max = theta_max
        self.theta = np.linspace(-theta_max, theta_max, N_theta)
        self.dtheta = self.theta[1] - self.theta[0]
        
        # Initialize pseudo-energies: εₐ(θ) = mₐR cosh(θ) (free theory)
        self.epsilon = np.zeros((self.N_particles, N_theta))
        for a in range(self.N_particles):
            self.epsilon[a] = self.masses[a] * R * np.cosh(self.theta)
    
    def L(self, epsilon):
        """L(ε) = log(1 + exp(-ε))"""
        # Numerically stable version
        return np.where(epsilon > 50, np.exp(-epsilon),
               np.where(epsilon < -50, -epsilon,
                        np.log(1 + np.exp(-epsilon))))
    
    def convolve(self, kernel_row, f):
        """Compute convolution: (kernel * f)(θ) = ∫ kernel(θ-θ')f(θ') dθ'/2π"""
        result = np.zeros_like(self.theta)
        for i in range(self.N_theta):
            integrand = np.zeros(self.N_theta)
            for j in range(self.N_theta):
                dt = self.theta[i] - self.theta[j]
                integrand[j] = kernel_row(dt) * f[j]
            result[i] = simpson(integrand, x=self.theta) / (2 * PI)
        return result
    
    def iterate(self, max_iter=100, tol=1e-10):
        """Solve TBA equations by iteration"""
        for iteration in range(max_iter):
            epsilon_new = np.zeros_like(self.epsilon)
            
            for a in range(self.N_particles):
                # Driving term
                epsilon_new[a] = self.masses[a] * self.R * np.cosh(self.theta)
                
                # Interaction term: -Σ_b I_{ab} * (φ * L_b)
                for b in range(self.N_particles):
                    if E8_INCIDENCE[a, b] > 0:
                        Lb = self.L(self.epsilon[b])
                        # Simplified convolution using universal kernel
                        conv = np.convolve(
                            1.0 / (2 * np.cosh(self.theta) + 1e-300),
                            Lb, mode='same') * self.dtheta / (2 * PI)
                        epsilon_new[a] -= E8_INCIDENCE[a, b] * conv
            
            # Check convergence
            diff = np.max(np.abs(epsilon_new - self.epsilon))
            self.epsilon = epsilon_new
            
            if diff < tol:
                print(f"  TBA converged after {iteration+1} iterations (diff={diff:.2e})")
                return True
        
        print(f"  TBA did not converge after {max_iter} iterations (diff={diff:.2e})")
        return False
    
    def ground_state_energy(self):
        """E₀(R) = -Σₐ mₐ/(2π) ∫ cosh(θ) Lₐ(θ) dθ"""
        E0 = 0.0
        for a in range(self.N_particles):
            La = self.L(self.epsilon[a])
            integrand = np.cosh(self.theta) * La
            E0 -= self.masses[a] / (2 * PI) * simpson(integrand, x=self.theta)
        return E0
    
    def effective_central_charge(self):
        """c_eff = -6R/(π) E₀(R) → should give c=1/2 for Ising as R→0"""
        E0 = self.ground_state_energy()
        return -6 * self.R / PI * E0

# ═══════════════════════════════════════════════════════════════
# MAIN COMPUTATION
# ═══════════════════════════════════════════════════════════════

if __name__ == "__main__":
    print("=" * 80)
    print("E₈ TBA SOLVER — PROJECT KEPLER→NEWTON")
    print("=" * 80)
    
    masses = zamolodchikov_masses()
    print(f"\nZamolodchikov masses (m_i/m_1):")
    for i, m in enumerate(masses):
        phi_note = " = φ" if abs(m - PHI) < 1e-10 else ""
        print(f"  m_{i+1} = {m:.10f}{phi_note}")
    
    print(f"\nGolden ratio checks:")
    print(f"  m₂/m₁ = {masses[1]/masses[0]:.15f} = φ")
    print(f"  m₆/m₃ = {masses[5]/masses[2]:.15f} = φ")
    print(f"  m₇/m₄ = {masses[6]/masses[3]:.15f} = φ")
    print(f"  m₈/m₅ = {masses[7]/masses[4]:.15f} = φ")
    
    # Solve TBA for several values of R
    print(f"\n{'='*80}")
    print(f"Solving TBA equations for various R values")
    print(f"{'='*80}")
    
    results = []
    for R in [0.01, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0]:
        print(f"\n  R = {R}:")
        solver = E8TBASolver(R=R, N_theta=150, theta_max=12.0)
        converged = solver.iterate(max_iter=50, tol=1e-8)
        E0 = solver.ground_state_energy()
        c_eff = solver.effective_central_charge()
        
        results.append({
            'R': R,
            'E0': float(E0),
            'c_eff': float(c_eff),
            'converged': converged,
        })
        
        print(f"    E₀(R) = {E0:.10f}")
        print(f"    c_eff = {c_eff:.6f} (target: c = 0.5 for Ising CFT as R→0)")
    
    print(f"\n{'='*80}")
    print(f"CENTRAL CHARGE FLOW")
    print(f"{'='*80}")
    print(f"\n  R → 0 limit should give c = 1/2 (Ising model)")
    print(f"  R → ∞ limit should give c = 0 (massive theory)")
    print(f"\n  {'R':>8} {'c_eff':>12}")
    print(f"  {'─'*22}")
    for r in results:
        print(f"  {r['R']:>8.2f} {r['c_eff']:>12.6f}")
    
    # Save results
    output = {
        'masses': masses.tolist(),
        'tba_results': results,
        'description': 'E8 TBA ground state energy and effective central charge',
    }
    out_path = os.path.join(os.path.dirname(__file__), "e8_tba_results.json")
    with open(out_path, "w") as f:
        json.dump(output, f, indent=2)
    
    print(f"\nResults saved to research/tba/e8_tba_results.json")
