// Example 08: Statistical Prediction — Monte Carlo Learning
//
// Demonstrates `Dag::predict()` when no analytical distribution transfers are
// provided.  The framework learns the output distribution by forward-sampling
// through the deterministic node functions (Monte Carlo).
//
// Pipeline
// --------
//     x ~ Normal(μ=0, σ=1)
//          │
//     [NonLinear]   y = sin(x) · exp(−x²/4)    (no closed form)
//          │
//     [Rectify]     z = max(0, y)               (no closed form)
//          │
//     [Scale]       w = 3z + 1                   (linear — attach transfer)
//          │
//     [Square]      out = w²                      (no closed form)
//
// Key concepts shown:
//   - Fully MC pass with zero transfers → all Empirical outputs
//   - Mixed-mode: analytical Scale, MC for everything else
//   - Effect of n_samples on estimate accuracy
//   - at_node early-stop for cheap intermediate inspection

mod benchmark_utils;

use std::collections::HashMap;
use std::sync::Arc;

use benchmark_utils::{print_header, print_section, Benchmark};
use dagex::{DistContext, DistTransferFn, Distribution, Graph, GraphData, PredictTarget};

// ── Node functions ────────────────────────────────────────────────────────────

fn non_linear(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let x = inputs.get("x").and_then(|d| d.as_float()).unwrap_or(0.0);
    let y = x.sin() * (-(x * x) / 4.0).exp();
    let mut out = HashMap::new();
    out.insert("y".to_string(), GraphData::float(y));
    out
}

fn rectify(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let y = inputs.get("y").and_then(|d| d.as_float()).unwrap_or(0.0);
    let mut out = HashMap::new();
    out.insert("z".to_string(), GraphData::float(y.max(0.0)));
    out
}

fn scale(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let z = inputs.get("z").and_then(|d| d.as_float()).unwrap_or(0.0);
    let mut out = HashMap::new();
    out.insert("w".to_string(), GraphData::float(z * 3.0 + 1.0));
    out
}

fn square(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let w = inputs.get("w").and_then(|d| d.as_float()).unwrap_or(0.0);
    let mut out = HashMap::new();
    out.insert("out".to_string(), GraphData::float(w * w));
    out
}

fn main() {
    print_header("Example 08: Statistical Prediction — Monte Carlo Learning");

    println!("Story:");
    println!("   A non-linear signal processing pipeline: input noise passes through");
    println!("   a Mexican-hat nonlinearity, half-wave rectification, a linear Scale,");
    println!("   and squaring.  Only Scale has a known analytical distribution transfer.");
    println!("   All other nodes are learned via MC sampling.\n");
    println!("   This demonstrates that predict() works correctly without any domain");
    println!("   knowledge — just provide the input prior and let the framework sample.\n");

    let x_prior: DistContext = HashMap::from([(
        "x".to_string(),
        Distribution::normal(0.0, 1.0),
    )]);

    // ── Build graph (pure MC — no transfers) ──────────────────────────────────
    print_section("Pure Monte Carlo (no transfers, n_samples=500)");

    let mut graph_plain = Graph::new();
    graph_plain.add(non_linear, Some("NonLinear"), Some(vec![("x", "x")]), Some(vec![("y", "y")]));
    graph_plain.add(rectify,    Some("Rectify"),   Some(vec![("y", "y")]), Some(vec![("z", "z")]));
    graph_plain.add(scale,      Some("Scale"),     Some(vec![("z", "z")]), Some(vec![("w", "w")]));
    graph_plain.add(square,     Some("Square"),    Some(vec![("w", "w")]), Some(vec![("out", "out")]));
    let dag_plain = graph_plain.build();

    let bm = Benchmark::start("MC n=500");
    let stat_plain = dag_plain.predict_at(x_prior.clone(), Some(500), None);
    let r = bm.finish();
    let s = stat_plain.summary("out").unwrap();
    println!("  out: mean={:.4}  std={:.4}  ({:.0} ms)", s.mean, s.std, r.duration_ms);

    // ── Build graph (mixed: Scale analytical + MC rest) ───────────────────────
    print_section("Mixed-mode (Scale analytical + MC elsewhere)");

    let scale_dist: DistTransferFn = Arc::new(|dists| {
        let z = dists.get("z")?;
        // w = 3z + 1  →  Normal(3μ+1, 3σ)
        Some(HashMap::from([(
            "w".to_string(),
            Distribution::normal(3.0 * z.mean() + 1.0, 3.0 * z.std()),
        )]))
    });

    let mut graph = Graph::new();
    graph.add(non_linear, Some("NonLinear"), Some(vec![("x", "x")]), Some(vec![("y", "y")]));
    graph.add(rectify,    Some("Rectify"),   Some(vec![("y", "y")]), Some(vec![("z", "z")]));
    graph.add(scale,      Some("Scale"),     Some(vec![("z", "z")]), Some(vec![("w", "w")]));
    graph.add(square,     Some("Square"),    Some(vec![("w", "w")]), Some(vec![("out", "out")]));
    graph.set_dist_transfer_for("Scale", scale_dist);
    let dag = graph.build();

    for n in [200_usize, 1000, 5000] {
        let bm = Benchmark::start("predict_at");
        let stat = dag.predict_at(x_prior.clone(), Some(n), None);
        let r = bm.finish();
        let s_out = stat.summary("out").unwrap();
        let s_w   = stat.summary("w").unwrap();
        println!(
            "  n={n:5}  out: mean={:.4}  std={:.4}  [p5={:.3}, p95={:.3}]  ({:.0} ms)",
            s_out.mean, s_out.std, s_out.p5, s_out.p95, r.duration_ms
        );
        if n == 5000 {
            println!(
                "         w (analytical Normal):  mean={:.4}  std={:.4}",
                s_w.mean, s_w.std
            );
        }
    }

    println!("\n  Note: `w` is Normal (analytical Scale transfer).");
    println!("  All other outputs are Empirical (MC samples).\n");

    // ── Inspect intermediate distributions ────────────────────────────────────
    print_section("Inspecting all intermediate distributions (n_samples=3000)");

    let stat = dag.predict_at(x_prior.clone(), Some(3000), None);

    for var in ["x", "y", "z", "w", "out"] {
        if let Some(d) = stat.get(var) {
            let s = d.summary();
            let kind = match d {
                Distribution::Normal { .. }      => "Normal",
                Distribution::Empirical { .. }   => "Empirical",
                Distribution::Deterministic(_)   => "Deterministic",
                _                                => "Other",
            };
            println!(
                "  {var:<5}  type={kind:<12}  mean={:.4}  std={:.4}  [p5={:.3}, p95={:.3}]",
                s.mean, s.std, s.p5, s.p95
            );
        }
    }

    println!();
    // Rectify clips to ≥0 so positive fraction of z should be high
    if let Some(Distribution::Empirical { samples }) = stat.get("z") {
        let positive = samples.iter().filter(|&&v| v > 0.0).count() as f64 / samples.len() as f64;
        println!("  z > 0: {:.1}%  (> 50% because rectify clips negative values to 0)", positive * 100.0);
    }

    // ── Early stop at Rectify ─────────────────────────────────────────────────
    print_section("predict_at(NodeLabel=\"Rectify\") — early stop");

    let target = PredictTarget::NodeLabel("Rectify".to_string());
    let stat_early = dag.predict_at(x_prior.clone(), Some(2000), Some(&target));

    println!("  'z' present:   {}", stat_early.contains("z"));
    println!("  'w' present:   {}", stat_early.contains("w"));
    println!("  'out' present: {}", stat_early.contains("out"));
    if let Some(s) = stat_early.summary("z") {
        println!("  z summary: mean={:.4}  std={:.4}", s.mean, s.std);
    }

    // ── n_samples convergence ─────────────────────────────────────────────────
    print_section("Effect of n_samples on output mean estimate");

    println!("  n_samples   out.mean   out.std");
    println!("  {}", "-".repeat(34));
    for n in [50_usize, 200, 1000, 5000] {
        let s = dag.predict_at(x_prior.clone(), Some(n), None).summary("out").unwrap();
        println!("  {n:8}   {:.4}   {:.4}", s.mean, s.std);
    }
    println!("\n  Estimates converge as n_samples increases.");
}
