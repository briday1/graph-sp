"""Example 08: Statistical Prediction — Learned / Monte Carlo

Demonstrates predict() when no analytical distribution transfer is known.
The framework automatically learns the output distribution by sampling through
the node functions (Monte Carlo forward pass).

Pipeline
--------
    x ~ Normal(μ=0, σ=1)
        │
    [NonLinear]   y = sin(x) · exp(−x²/4)            (no closed form)
        │
    [Rectify]     z = max(0, y)  (half-wave rectifier) (no closed form)
        │
    [Scale]       w = 3z + 1                            (linear — attach transfer)
        │
    [Square]      out = w²                              (no closed form)

Key concepts shown:
  - Full MC pass with no transfers  → all Empirical outputs
  - Mixed-mode: analytical Scale, MC for the rest
  - Effect of n_samples on accuracy
  - Accessing output distributions: mean, std, percentiles, raw samples
  - at_node early stop to inspect intermediate variables cheaply
"""

import sys
sys.path.insert(0, '/home/runner/work/graph-sp/graph-sp/examples/py')

import math
from benchmark_utils import print_header, print_section, print_dist_table, Benchmark
import dagex


# ── Node functions ────────────────────────────────────────────────────────────

def non_linear(inputs):
    """y = sin(x) * exp(-x²/4)  — a Mexican-hat-like pulse."""
    import math
    x = inputs.get("x", 0)
    return {"y": math.sin(x) * math.exp(-(x ** 2) / 4.0)}


def rectify(inputs):
    """z = max(0, y)  — half-wave rectification (keeps positive part only)."""
    y = inputs.get("y", 0)
    return {"z": max(0.0, y)}


def scale(inputs):
    """w = 3z + 1  — linear scaling."""
    z = inputs.get("z", 0)
    return {"w": z * 3.0 + 1.0}


def square(inputs):
    """out = w²."""
    w = inputs.get("w", 0)
    return {"out": w ** 2}


def main():
    print_header("Example 08: Statistical Prediction — Monte Carlo Learning")

    print("📖 Story:")
    print("   A non-linear signal processing pipeline: input noise passes through")
    print("   a Mexican-hat nonlinearity, half-wave rectification, a linear scale,")
    print("   and squaring.  Only the linear Scale node has a known analytical")
    print("   distribution transfer.  All others are learned via Monte Carlo.\n")
    print("   This shows how predict() works transparently even without any domain")
    print("   knowledge — just supply the prior on the input and let it sample.\n")

    # ── Build graph ───────────────────────────────────────────────────────────
    print_section("Building the Graph")

    graph = dagex.Graph()
    graph.add(non_linear, label="NonLinear", inputs=[("x",  "x")], outputs=[("y", "y")])
    graph.add(rectify,    label="Rectify",   inputs=[("y",  "y")], outputs=[("z", "z")])
    graph.add(scale,      label="Scale",     inputs=[("z",  "z")], outputs=[("w", "w")])
    graph.add(square,     label="Square",    inputs=[("w",  "w")], outputs=[("out", "out")])

    # Only Scale has a known distribution transfer
    def scale_dist(dists):
        z = dists["z"]
        return {"w": dagex.normal(z.mean * 3.0 + 1.0, z.std * 3.0)}

    graph.set_dist_transfer("Scale", scale_dist)

    dag = graph.build()
    print(dag.to_mermaid())

    x_prior = {"x": dagex.normal(0.0, 1.0)}

    # ── Full MC pass — no transfers at all first ──────────────────────────────
    print_section("Full Monte Carlo (no transfers, n_samples=500)")

    graph_plain = dagex.Graph()
    graph_plain.add(non_linear, label="NonLinear", inputs=[("x", "x")], outputs=[("y", "y")])
    graph_plain.add(rectify,    label="Rectify",   inputs=[("y", "y")], outputs=[("z", "z")])
    graph_plain.add(scale,      label="Scale",     inputs=[("z", "z")], outputs=[("w", "w")])
    graph_plain.add(square,     label="Square",    inputs=[("w", "w")], outputs=[("out", "out")])
    dag_plain = graph_plain.build()

    with Benchmark("MC n=500") as bm:
        stat_plain = dag_plain.predict(x_prior, n_samples=500)
    print(f"  Time: {bm.result.duration_ms:.1f} ms")
    print(f"  out:  {stat_plain['out'].summary()}")

    # ── Mixed-mode (Scale analytical, rest MC) ────────────────────────────────
    print_section("Mixed-mode (Scale analytical + MC elsewhere)")

    for n in [200, 1000, 5000]:
        with Benchmark(f"n={n}") as bm:
            stat = dag.predict(x_prior, n_samples=n)
        d_out = stat["out"]
        d_w   = stat["w"]
        print(f"  n={n:5d}  out: mean={d_out.mean:.4f}  std={d_out.std:.4f}"
              f"  [p5={d_out.p5:.3f}, p95={d_out.p95:.3f}]"
              f"  ({bm.result.duration_ms:.0f} ms)")
        if n == 5000:
            print(f"         w (analytical Scale input):  {d_w}")

    print()
    print("  Note: w comes from the analytical Scale transfer (Normal).")
    print("  Upstream/downstream nodes are Empirical (MC samples).\n")

    # ── Inspecting individual distributions ───────────────────────────────────
    print_section("Inspecting intermediate distributions (n_samples=3000)")

    stat = dag.predict(x_prior, n_samples=3000)

    rows = []
    for var in ["x", "y", "z", "w", "out"]:
        d = stat[var]
        type_str = repr(d).split("(")[0]
        rows.append((var, d, "", type_str))
    print_dist_table(rows, show_type=True)

    print()
    print("  Empirical types can be histogrammed:")
    z_samples = stat["z"].samples
    if z_samples:
        positive = sum(1 for s in z_samples if s > 0) / len(z_samples)
        print(f"  z > 0: {positive:.1%}  (expected ≈50% before rectification,")
        print(f"         actual >{positive:.0%} because rectify clips z to ≥0)")

    # ── Early stop: only compute up to Rectify ───────────────────────────────
    print_section("at_node='Rectify' — early stop before Scale and Square")

    stat_early = dag.predict(x_prior, n_samples=2000, at_node="Rectify")
    print("  Keys:", stat_early.keys())
    print("  'w' present:   ", stat_early.get("w")   is not None)
    print("  'out' present: ", stat_early.get("out") is not None)
    print("  z (Rectify output):", stat_early["z"].summary())

    # ── Compare distributions across sample counts ────────────────────────────
    print_section("Effect of n_samples on output mean estimate")

    print("  n_samples   out.mean   out.std")
    print("  " + "-" * 35)
    for n in [50, 200, 1000, 5000]:
        s = dag.predict(x_prior, n_samples=n)
        d = s["out"]
        print(f"  {n:8d}   {d.mean:8.4f}   {d.std:8.4f}")

    print()
    print("  As n_samples increases, estimates converge.")

    # ── Joint distribution analysis ───────────────────────────────────────────
    print_section("Joint Distribution Analysis  (predict_particles)")

    print("  Running predict_particles() — preserves joint structure...")
    stat_p  = dag.predict_particles(x_prior, n_samples=3000)
    joint   = dagex.joint(stat_p)

    print()
    joint.print_summary()

    print()
    print("  Key observations:")
    r_xy = joint.correlation("x", "y")
    r_yz = joint.correlation("y", "z")
    r_zw = joint.correlation("z", "w")
    r_wo = joint.correlation("w", "out")
    print(f"    r(x, y) = {r_xy:.4f}  [x→NonLinear→y: non-monotone Mexican hat, expect moderate r]")
    print(f"    r(y, z) = {r_yz:.4f}  [y→Rectify→z: z=max(0,y), expect positive r]")
    print(f"    r(z, w) = {r_zw:.4f}  [z→Scale→w: w = 3z+1 is linear, expect r = 1.000]")
    print(f"    r(w,out)= {r_wo:.4f}  [w→Square→out: out=w², expect positive r]")
    print()
    print("  Note: predict_particles() runs node functions directly (bypasses dist_transfer),")
    print("  so the analytical Scale node contributes exact correlation just like any other node.")

    # Demonstrate assume_independent: force x ⊥ out
    joint_indep = joint.assume_independent("x", "out")
    r_before = joint.correlation("x", "out")
    r_after  = joint_indep.correlation("x", "out")
    print()
    print("  Forcing x ⊥ out (assume_independent):")
    print(f"    r(x, out) before = {r_before:.4f}")
    print(f"    r(x, out) after  = {r_after:.4f}  [≈ 0: independence enforced]")

    # Save pair plot
    import os
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    plot_path = os.path.join(os.path.dirname(__file__), "08_joint_pairs.png")
    fig, _ = joint.plot_pairs(title="Example 08 — Joint Distribution (predict_particles)")
    fig.savefig(plot_path, dpi=120, bbox_inches="tight")
    plt.close(fig)
    print(f"\n  Pair plot saved → {plot_path}")


if __name__ == "__main__":
    main()
