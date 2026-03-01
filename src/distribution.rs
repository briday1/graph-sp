//! Statistical distributions and distribution-transfer types for `predict()`
//!
//! ## Design
//!
//! `DistContext` is a parallel to `ExecutionContext`: instead of holding concrete values
//! it holds a `Distribution` per named port.  Nodes can optionally carry a `DistTransferFn`
//! that transforms input `DistContext` → output `DistContext` analytically.  When absent,
//! `Dag::predict()` falls back to Monte Carlo sampling through the deterministic function.
//!
//! Port key conventions match the execution layer:
//!   - regular nodes use bare broadcast variable names
//!   - branch nodes use `__branch_{id}__{var}` prefixed keys (same as `ExecutionContext`)

use std::collections::HashMap;
use std::sync::Arc;

use rand::Rng;
use rand_distr::{
    Beta as RandBeta, Distribution as RandDist, Gamma as RandGamma, LogNormal as RandLogNormal,
    Normal as RandNormal, Uniform as RandUniform,
};

// ─── Public type aliases ─────────────────────────────────────────────────────

/// A mapping from port name → Distribution, mirroring `ExecutionContext`.
pub type DistContext = HashMap<String, Distribution>;

/// Optional analytical distribution transfer for a node.
///
/// Receives distributions keyed by **impl_var** names (the same keys the node function sees).
/// Returns distributions keyed by **impl_var** output names (the same keys the node function
/// returns), or `None` to signal that Monte Carlo fallback should be used for this node.
pub type DistTransferFn = Arc<dyn Fn(&DistContext) -> Option<DistContext> + Send + Sync>;

// ─── Distribution enum ───────────────────────────────────────────────────────

/// Parametric and empirical probability distributions.
///
/// All variants expose a common moment interface (`.mean()`, `.std()`, `.variance()`,
/// `.percentile(p)`) so that downstream analytical transfers can be written generically.
/// Every variant also supports `.sample_n(n)` for use in the Monte Carlo fallback.
#[derive(Clone, Debug)]
pub enum Distribution {
    /// Point mass — a fixed, known value.  Variance is zero.
    Deterministic(f64),

    /// Normal / Gaussian: N(mean, σ).
    Normal { mean: f64, std: f64 },

    /// Continuous uniform: U(low, high).
    Uniform { low: f64, high: f64 },

    /// Beta distribution: Beta(α, β).  Support [0, 1].
    Beta { alpha: f64, beta: f64 },

    /// Gamma distribution: Γ(shape, rate).  Mean = shape/rate.
    /// Note: parameterised by **rate** (= 1/scale), not scale.
    Gamma { shape: f64, rate: f64 },

    /// Log-normal: exp(N(μ, σ)).  `mu` and `sigma` are the parameters of the
    /// underlying normal, not the mean/std of the log-normal itself.
    LogNormal { mu: f64, sigma: f64 },

    /// Empirical distribution defined by a vector of observed samples.
    /// This is the universal output of the Monte Carlo fallback path.
    Empirical { samples: Arc<Vec<f64>> },
}

impl Distribution {
    // ── Constructors ─────────────────────────────────────────────────────────

    /// `Deterministic(value)`.
    pub fn deterministic(value: f64) -> Self {
        Distribution::Deterministic(value)
    }

    /// `Normal { mean, std }`.
    pub fn normal(mean: f64, std: f64) -> Self {
        Distribution::Normal { mean, std }
    }

    /// `Uniform { low, high }`.
    pub fn uniform(low: f64, high: f64) -> Self {
        Distribution::Uniform { low, high }
    }

    /// `Beta { alpha, beta }`.
    pub fn beta(alpha: f64, beta: f64) -> Self {
        Distribution::Beta { alpha, beta }
    }

    /// `Gamma { shape, rate }`.  rate = 1/scale.
    pub fn gamma(shape: f64, rate: f64) -> Self {
        Distribution::Gamma { shape, rate }
    }

    /// `LogNormal { mu, sigma }`.  mu/sigma are the underlying normal parameters.
    pub fn lognormal(mu: f64, sigma: f64) -> Self {
        Distribution::LogNormal { mu, sigma }
    }

    /// `Empirical` from an owned `Vec<f64>`.
    pub fn empirical(samples: Vec<f64>) -> Self {
        Distribution::Empirical {
            samples: Arc::new(samples),
        }
    }

    // ── Moment accessors ─────────────────────────────────────────────────────

    /// Expected value E[X].
    pub fn mean(&self) -> f64 {
        match self {
            Distribution::Deterministic(v) => *v,
            Distribution::Normal { mean, .. } => *mean,
            Distribution::Uniform { low, high } => (low + high) / 2.0,
            Distribution::Beta { alpha, beta } => alpha / (alpha + beta),
            Distribution::Gamma { shape, rate } => shape / rate,
            Distribution::LogNormal { mu, sigma } => (mu + sigma * sigma / 2.0).exp(),
            Distribution::Empirical { samples } => {
                if samples.is_empty() {
                    f64::NAN
                } else {
                    samples.iter().sum::<f64>() / samples.len() as f64
                }
            }
        }
    }

    /// Variance Var[X].
    pub fn variance(&self) -> f64 {
        match self {
            Distribution::Deterministic(_) => 0.0,
            Distribution::Normal { std, .. } => std * std,
            Distribution::Uniform { low, high } => (high - low).powi(2) / 12.0,
            Distribution::Beta { alpha, beta } => {
                let s = alpha + beta;
                (alpha * beta) / (s * s * (s + 1.0))
            }
            Distribution::Gamma { shape, rate } => shape / (rate * rate),
            Distribution::LogNormal { mu, sigma } => {
                let s2 = sigma * sigma;
                s2.exp_m1() * (2.0 * mu + s2).exp()
            }
            Distribution::Empirical { samples } => {
                if samples.len() < 2 {
                    0.0
                } else {
                    let m = self.mean();
                    samples.iter().map(|x| (x - m).powi(2)).sum::<f64>()
                        / (samples.len() - 1) as f64
                }
            }
        }
    }

    /// Standard deviation √Var[X].
    pub fn std(&self) -> f64 {
        self.variance().sqrt()
    }

    /// Compute the p-th percentile (0.0 ≤ p ≤ 1.0) via sampling.
    ///
    /// For `Empirical` this is exact (order-statistic).
    /// For parametric families this draws 4096 samples and uses the order statistic —
    /// accurate enough for visualisation and summaries without implementing exact inverse CDFs.
    pub fn percentile(&self, p: f64) -> f64 {
        let p = p.clamp(0.0, 1.0);
        match self {
            Distribution::Deterministic(v) => *v,
            Distribution::Empirical { samples } => {
                if samples.is_empty() {
                    return f64::NAN;
                }
                let mut sorted = samples.as_ref().clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let idx = ((p * (sorted.len() - 1) as f64).round() as usize)
                    .min(sorted.len() - 1);
                sorted[idx]
            }
            other => {
                // Parametric: sample and take order statistic
                let mut s = other.sample_n(4096);
                s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let idx = ((p * (s.len() - 1) as f64).round() as usize).min(s.len() - 1);
                s[idx]
            }
        }
    }

    // ── Sampling ─────────────────────────────────────────────────────────────

    /// Draw `n` independent samples.
    pub fn sample_n(&self, n: usize) -> Vec<f64> {
        let mut rng = rand::thread_rng();
        self.sample_n_with_rng(n, &mut rng)
    }

    /// Draw one sample.
    pub fn sample_one(&self) -> f64 {
        self.sample_n(1).into_iter().next().unwrap_or(f64::NAN)
    }

    /// Draw `n` samples using the provided RNG (useful when a caller wants a seeded RNG).
    pub fn sample_n_with_rng<R: Rng>(&self, n: usize, rng: &mut R) -> Vec<f64> {
        match self {
            Distribution::Deterministic(v) => vec![*v; n],
            Distribution::Normal { mean, std } => {
                match RandNormal::new(*mean, *std) {
                    Ok(d) => (0..n).map(|_| d.sample(rng)).collect(),
                    Err(_) => vec![*mean; n],
                }
            }
            Distribution::Uniform { low, high } => {
                // rand::distributions::Uniform::new panics on bad input but returns directly (not Result)
                let d = RandUniform::new(*low, *high);
                (0..n).map(|_| d.sample(rng)).collect()
            }
            Distribution::Beta { alpha, beta } => {
                match RandBeta::new(*alpha, *beta) {
                    Ok(d) => (0..n).map(|_| d.sample(rng)).collect(),
                    Err(_) => vec![alpha / (alpha + beta); n],
                }
            }
            Distribution::Gamma { shape, rate } => {
                // rand_distr::Gamma uses shape + scale (scale = 1/rate)
                match RandGamma::new(*shape, 1.0 / rate) {
                    Ok(d) => (0..n).map(|_| d.sample(rng)).collect(),
                    Err(_) => vec![shape / rate; n],
                }
            }
            Distribution::LogNormal { mu, sigma } => {
                match RandLogNormal::new(*mu, *sigma) {
                    Ok(d) => (0..n).map(|_| d.sample(rng)).collect(),
                    Err(_) => vec![mu.exp(); n],
                }
            }
            Distribution::Empirical { samples } => {
                if samples.is_empty() {
                    return vec![0.0; n];
                }
                // Sample with replacement
                (0..n)
                    .map(|_| {
                        let idx = rng.gen_range(0..samples.len());
                        samples[idx]
                    })
                    .collect()
            }
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    /// Returns `true` if this is a `Deterministic` point mass.
    pub fn is_deterministic(&self) -> bool {
        matches!(self, Distribution::Deterministic(_))
    }

    /// Returns `true` if this is an `Empirical` sample-based distribution.
    pub fn is_empirical(&self) -> bool {
        matches!(self, Distribution::Empirical { .. })
    }

    /// If `Deterministic`, return the value; otherwise `None`.
    pub fn as_deterministic(&self) -> Option<f64> {
        match self {
            Distribution::Deterministic(v) => Some(*v),
            _ => None,
        }
    }

    /// If `Empirical`, return a reference to the sample slice; otherwise `None`.
    pub fn as_samples(&self) -> Option<&[f64]> {
        match self {
            Distribution::Empirical { samples } => Some(samples.as_slice()),
            _ => None,
        }
    }

    /// Compute a `PortSummary` from this distribution.
    pub fn summary(&self) -> PortSummary {
        PortSummary {
            mean: self.mean(),
            std: self.std(),
            variance: self.variance(),
            p5: self.percentile(0.05),
            p50: self.percentile(0.50),
            p95: self.percentile(0.95),
        }
    }
}

impl std::fmt::Display for Distribution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Distribution::Deterministic(v) => write!(f, "Deterministic({v})"),
            Distribution::Normal { mean, std } => write!(f, "Normal(μ={mean}, σ={std})"),
            Distribution::Uniform { low, high } => write!(f, "Uniform({low}, {high})"),
            Distribution::Beta { alpha, beta } => write!(f, "Beta(α={alpha}, β={beta})"),
            Distribution::Gamma { shape, rate } => write!(f, "Gamma(k={shape}, λ={rate})"),
            Distribution::LogNormal { mu, sigma } => write!(f, "LogNormal(μ={mu}, σ={sigma})"),
            Distribution::Empirical { samples } => {
                write!(f, "Empirical(n={}, μ≈{:.4})", samples.len(), self.mean())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Constructors & helpers ────────────────────────────────────────────────

    #[test]
    fn test_deterministic_moments() {
        let d = Distribution::deterministic(5.0);
        assert_eq!(d.mean(), 5.0);
        assert_eq!(d.variance(), 0.0);
        assert_eq!(d.std(), 0.0);
        assert_eq!(d.percentile(0.5), 5.0);
        assert!(d.is_deterministic());
        assert!(!d.is_empirical());
        assert_eq!(d.as_deterministic(), Some(5.0));
        assert!(d.as_samples().is_none());
    }

    #[test]
    fn test_normal_moments() {
        let d = Distribution::normal(2.0, 3.0);
        assert_eq!(d.mean(), 2.0);
        assert!((d.variance() - 9.0).abs() < 1e-10);
        assert!((d.std() - 3.0).abs() < 1e-10);
        assert!(!d.is_deterministic());
    }

    #[test]
    fn test_uniform_moments() {
        let d = Distribution::uniform(0.0, 4.0);
        assert_eq!(d.mean(), 2.0);
        // Var = (high - low)^2 / 12
        assert!((d.variance() - 16.0 / 12.0).abs() < 1e-10);
    }

    #[test]
    fn test_beta_moments() {
        // Beta(2, 2): mean = 0.5, var = 2*2 / (4^2 * 5) = 4/80 = 0.05
        let d = Distribution::beta(2.0, 2.0);
        assert!((d.mean() - 0.5).abs() < 1e-10);
        assert!((d.variance() - 0.05).abs() < 1e-10);
    }

    #[test]
    fn test_gamma_moments() {
        // Gamma(shape=2, rate=0.5): mean = 4, var = 2 / 0.25 = 8
        let d = Distribution::gamma(2.0, 0.5);
        assert!((d.mean() - 4.0).abs() < 1e-10);
        assert!((d.variance() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_lognormal_moments() {
        // LogNormal(mu=0, sigma=0): mean = e^0 = 1, var = 0
        let d = Distribution::lognormal(0.0, 0.0);
        assert!((d.mean() - 1.0).abs() < 1e-10);
        assert_eq!(d.variance(), 0.0);
    }

    #[test]
    fn test_empirical_moments() {
        let samples = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let d = Distribution::empirical(samples);
        assert!((d.mean() - 3.0).abs() < 1e-10);
        // Sample variance of [1,2,3,4,5] = 2.5
        assert!((d.variance() - 2.5).abs() < 1e-10);
        assert!(!d.is_deterministic());
        assert!(d.is_empirical());
        assert_eq!(d.as_samples().unwrap().len(), 5);
    }

    #[test]
    fn test_empirical_empty() {
        let d = Distribution::empirical(vec![]);
        assert!(d.mean().is_nan());
        assert_eq!(d.variance(), 0.0);
        assert_eq!(d.percentile(0.5).is_nan(), true);
    }

    // ── Sampling ──────────────────────────────────────────────────────────────

    #[test]
    fn test_sample_n_count() {
        for d in [
            Distribution::deterministic(1.0),
            Distribution::normal(0.0, 1.0),
            Distribution::uniform(0.0, 1.0),
            Distribution::beta(1.0, 1.0),
            Distribution::gamma(1.0, 1.0),
            Distribution::lognormal(0.0, 1.0),
            Distribution::empirical(vec![1.0, 2.0, 3.0]),
        ] {
            assert_eq!(d.sample_n(20).len(), 20);
        }
    }

    #[test]
    fn test_deterministic_sample_is_constant() {
        let d = Distribution::deterministic(42.0);
        let s = d.sample_n(50);
        assert!(s.iter().all(|&v| v == 42.0));
    }

    #[test]
    fn test_sample_one() {
        let d = Distribution::normal(5.0, 0.0001);
        let v = d.sample_one();
        assert!((v - 5.0).abs() < 1.0); // just must be finite
    }

    // ── Percentile ───────────────────────────────────────────────────────────

    #[test]
    fn test_empirical_percentile() {
        // [0,1,2,...,99], p50 should be 49 or 50
        let d = Distribution::empirical((0..100).map(|x| x as f64).collect());
        let p50 = d.percentile(0.5);
        assert!((p50 - 49.5).abs() <= 1.0);
        let p0 = d.percentile(0.0);
        assert_eq!(p0, 0.0);
        let p100 = d.percentile(1.0);
        assert_eq!(p100, 99.0);
    }

    // ── Display ───────────────────────────────────────────────────────────────

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Distribution::deterministic(1.0)), "Deterministic(1)");
        assert!(format!("{}", Distribution::normal(0.0, 1.0)).contains("Normal"));
        assert!(format!("{}", Distribution::uniform(0.0, 1.0)).contains("Uniform"));
        assert!(format!("{}", Distribution::beta(1.0, 1.0)).contains("Beta"));
        assert!(format!("{}", Distribution::gamma(1.0, 1.0)).contains("Gamma"));
        assert!(format!("{}", Distribution::lognormal(0.0, 1.0)).contains("LogNormal"));
        let emp = Distribution::empirical(vec![1.0, 2.0]);
        assert!(format!("{}", emp).contains("Empirical"));
    }

    // ── PortSummary ───────────────────────────────────────────────────────────

    #[test]
    fn test_port_summary_fields() {
        let d = Distribution::deterministic(7.0);
        let s = d.summary();
        assert_eq!(s.mean, 7.0);
        assert_eq!(s.std, 0.0);
        assert_eq!(s.variance, 0.0);
        assert_eq!(s.p5, 7.0);
        assert_eq!(s.p50, 7.0);
        assert_eq!(s.p95, 7.0);
        assert!(format!("{}", s).contains("mean=7"));
    }
}

// ─── Port summary ─────────────────────────────────────────────────────────────

/// Summary statistics for a single scalar output port.
#[derive(Debug, Clone)]
pub struct PortSummary {
    /// E[X]
    pub mean: f64,
    /// √Var[X]
    pub std: f64,
    /// Var[X]
    pub variance: f64,
    /// 5th percentile
    pub p5: f64,
    /// Median
    pub p50: f64,
    /// 95th percentile
    pub p95: f64,
}

impl std::fmt::Display for PortSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "mean={:.4}  std={:.4}  [p5={:.4}, p50={:.4}, p95={:.4}]",
            self.mean, self.std, self.p5, self.p50, self.p95
        )
    }
}
