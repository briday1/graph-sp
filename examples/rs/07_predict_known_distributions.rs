// Example 07: Statistical Prediction — Known Analytical Distributions
//
// Demonstrates `Dag::predict()` when every node has a known, closed-form
// distribution transfer function.  The framework propagates the input prior
// analytically through the graph without any sampling.
//
// Pipeline
// --------
//     x ~ Normal(μ=10, σ=2)
//          │
//     [Amplify]     y = 3x + 5         →  N(μ=35, σ=6)
//          │
//     [Attenuate]   z = 0.5y           →  N(μ=17.5, σ=3)
//          │
//     [AddNoise]    w = z + ε, ε~N(0,1) →  N(μ=17.5, σ=√10)
//          │
//     [Clip]        out = clip(w, 0, 100) (no transfer → MC fallback)
//
// Key concepts shown:
//   - Attaching `DistTransferFn` closures via `Graph::set_dist_transfer_for()`
//   - Fully-analytical pass for the first three nodes
//   - Graceful MC fallback for the fourth node (no transfer provided)
//   - `predict_at()` for early-stop at a specific node
//   - Verifying analytical results against theory

mod benchmark_utils;

use std::collections::HashMap;
use std::sync::Arc;

use benchmark_utils::{print_header, print_section, Benchmark};
use dagex::{DistTransferFn, Distribution, Graph, GraphData, PredictTarget};

// ── Node functions ────────────────────────────────────────────────────────────

fn amplify(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let x = inputs.get("x").and_then(|d| d.as_float()).unwrap_or(0.0);
    let mut out = HashMap::new();
    out.insert("y".to_string(), GraphData::float(3.0 * x + 5.0));
    out
}

fn attenuate(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let y = inputs.get("y").and_then(|d| d.as_float()).unwrap_or(0.0);
    let mut out = HashMap::new();
    out.insert("z".to_string(), GraphData::float(0.5 * y));
    out
}

fn add_noise(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    // In production this would actually draw from a noise distribution;
    // here we just pass through (the distribution transfer handles propagation).
    let z = inputs.get("z").and_then(|d| d.as_float()).unwrap_or(0.0);
    let mut out = HashMap::new();
    out.insert("w".to_string(), GraphData::float(z)); // noise handled by dist transfer
    out
}

fn clip(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let w = inputs.get("w").and_then(|d| d.as_float()).unwrap_or(0.0);
    let mut out = HashMap::new();
    out.insert("out".to_string(), GraphData::float(w.clamp(0.0, 100.0)));
    out
}

fn main() {
    print_header("Example 07: Statistical Prediction — Known Analytical Distributions");

    println!("Story:");
    println!("   A signal processing pipeline where each node has a known linear");
    println!("   (or near-linear) distribution transfer.  predict() propagates the");
    println!("   input Normal prior analytically, yielding exact Normal outputs for");
    println!("   the first three stages.  The Clip node has no transfer so it falls");
    println!("   back to Monte Carlo sampling transparently.\n");

    // ── Distribution transfer closures  ───────────────────────────────────────
    //
    // Each closure receives the impl-var-keyed DistContext for that node and
    // returns the output DistContext.  Return None to force MC fallback.

    let amplify_dist: DistTransferFn = Arc::new(|dists| {
        let x = dists.get("x")?;
        // y = 3x + 5  →  Normal(3μ+5, 3σ)
        Some(HashMap::from([(
            "y".to_string(),
            Distribution::normal(3.0 * x.mean() + 5.0, 3.0 * x.std()),
        )]))
    });

    let attenuate_dist: DistTransferFn = Arc::new(|dists| {
        let y = dists.get("y")?;
        // z = 0.5y  →  Normal(0.5μ, 0.5σ)
        Some(HashMap::from([(
            "z".to_string(),
            Distribution::normal(0.5 * y.mean(), 0.5 * y.std()),
        )]))
    });

    let add_noise_dist: DistTransferFn = Arc::new(|dists| {
        let z = dists.get("z")?;
        // w = z + ε, ε ~ N(0,1)  →  Normal(μ_z, sqrt(σ_z² + 1))
        Some(HashMap::from([(
            "w".to_string(),
            Distribution::normal(
                z.mean(),
                (z.variance() + 1.0_f64).sqrt(),
            ),
        )]))
    });

    // ── Build graph ───────────────────────────────────────────────────────────
    print_section("Building the Graph");

    let mut graph = Graph::new();
    graph.add(amplify,   Some("Amplify"),   Some(vec![("x", "x")]),  Some(vec![("y", "y")]));
    graph.add(attenuate, Some("Attenuate"), Some(vec![("y", "y")]),  Some(vec![("z", "z")]));
    graph.add(add_noise, Some("AddNoise"),  Some(vec![("z", "z")]),  Some(vec![("w", "w")]));
    graph.add(clip,      Some("Clip"),      Some(vec![("w", "w")]),  Some(vec![("out", "out")]));

    graph.set_dist_transfer_for("Amplify",   amplify_dist);
    graph.set_dist_transfer_for("Attenuate", attenuate_dist);
    graph.set_dist_transfer_for("AddNoise",  add_noise_dist);
    // Clip has NO transfer → automatic MC fallback

    let dag = graph.build();
    println!("{}", dag.to_mermaid());

    // ── Full predict pass ─────────────────────────────────────────────────────
    print_section("Full predict() — analytical + MC fallback for Clip");

    let input_dists = HashMap::from([(
        "x".to_string(),
        Distribution::normal(10.0, 2.0),
    )]);

    let bm = Benchmark::start("predict");
    let stat = dag.predict(input_dists.clone(), 2000);
    let result = bm.finish();
    println!("  Time: {:.1} ms\n", result.duration_ms);

    // ── Verify analytical results ─────────────────────────────────────────────
    print_section("Verification — theory vs. predict()");

    let expected = [
        ("y",   35.0_f64,  6.0_f64),
        ("z",   17.5_f64,  3.0_f64),
        ("w",   17.5_f64,  (10.0_f64).sqrt()),
    ];

    for (var, exp_mean, exp_std) in &expected {
        let d = stat.get(var).expect("key missing");
        let s = d.summary();
        let mean_err = (s.mean - exp_mean).abs();
        let std_err  = (s.std  - exp_std).abs();
        let ok = mean_err < 0.001 && std_err < 0.001;
        println!(
            "  {var}  mean={:.4} (expected {exp_mean:.4}, Δ={mean_err:.2e})  \
             std={:.4} (expected {exp_std:.4}, Δ={std_err:.2e})  {}",
            s.mean, s.std,
            if ok { "✓ exact" } else { "✗ mismatch" }
        );
    }

    // Clip — MC, just print summary
    if let Some(out_dist) = stat.get("out") {
        let s = out_dist.summary();
        println!(
            "\n  out (MC fallback):  mean={:.3}  std={:.3}  [p5={:.2}, p95={:.2}]",
            s.mean, s.std, s.p5, s.p95
        );
    }

    // ── Distribution details ──────────────────────────────────────────────────
    print_section("Distribution details");

    for var in ["x", "y", "z", "w", "out"] {
        if let Some(d) = stat.get(var) {
            let s = d.summary();
            println!(
                "  {var:<5}  {d:?}",
                d = d
            );
            let _ = s;
        }
    }

    // ── Early stop at AddNoise ────────────────────────────────────────────────
    print_section("predict_at(NodeLabel=\"AddNoise\") — stop before Clip");

    let target = PredictTarget::NodeLabel("AddNoise".to_string());
    let stat_early = dag.predict_at(input_dists.clone(), 2000, Some(&target));

    println!("  'w' present:   {}", stat_early.contains("w"));
    println!("  'out' present: {}", stat_early.contains("out"));
    if let Some(s) = stat_early.summary("w") {
        println!("  w summary: mean={:.4}  std={:.4}", s.mean, s.std);
    }

    // ── Sample inspection ─────────────────────────────────────────────────────
    print_section("Sample inspection — first 5 samples of 'out' (Empirical)");

    if let Some(dagex::Distribution::Empirical { samples }) = stat.get("out") {
        for (i, &v) in samples.iter().take(5).enumerate() {
            println!("  sample[{i}] = {v:.4}");
        }
    }
}
