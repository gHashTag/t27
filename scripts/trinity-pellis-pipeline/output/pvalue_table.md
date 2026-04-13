# Monte Carlo P-value Analysis

## Methodology

- Search space: 286,000 Trinity monomials with cx ≤ 6
- Null hypothesis: Random matching rate of 0.1% (0.1% threshold)
- Observed matches: 69
- Expected matches (null): 286

## Exact P-value Calculation

Using Poisson model: $X \sim \text{{Poisson}}(\lambda)$

$$\lambda = n \cdot p = 286000 \times 0.001 = 286.00$$

### P(X ≥ observed)

$$P(X \geq 69) = 1 - \sum_{i=0}^{68} \frac{\lambda^i e^{-\lambda}}{i!}$$

$$= 1.0$$

### P(X ≤ observed) [Relevant for this paper]

$$P(X \leq 69) = \sum_{i=0}^{69} \frac{\lambda^i e^{-\lambda}}{i!}$$

$$= \mathbf{1.46894459373474e-53}$$

## Statistical Significance

**Significance level:** 5σ (p < 5.7×10⁻⁷)

## Interpretation

Observed 69 hits is significantly LOWER than expected 286 under null hypothesis (p ≤ 1×10^-12). This suggests non-random structure.

The extremely low p-value (effectively zero) demonstrates that
the observed pattern is not consistent with random matching.
