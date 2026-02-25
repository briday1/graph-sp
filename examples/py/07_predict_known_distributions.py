"""Example 07: Statistical Prediction with Known Distributions

Demonstrates predict() when the input→output distribution transform is known
in closed form.  Analytical dist_transfer functions are attached to each node
so that predict() propagates distributions exactly — no sampling needed.

Pipeline
--------
    x ~ Normal(μ=10, σ=2)
        │
    [Amplify]   y = 3x + 5        (linear)   → y ~ Normal(35, 6)
        │
    [Attenuate] z = y * 0.5       (linear)   → z ~ Normal(17.5, 3)
        │
    [AddNoise]  w = z + ε,  ε~N(0,1)        → w ~ Normal(17.5, √10 ≈ 3.162)
        │
    [Clip]      clip to [0, 100] via MC only  (non-linear, no closed form)

The first three nodes use exact analytical transfers.  The final Clip node has
no transfer attached, so predict() automatically falls back to Monte Carlo for
that step only — demonstrating the mixed-mode behaviour.
"""

import sys
sys.path.insert(0, '/home/runner/work/graph-sp/graph-sp/examples/py')

import math
from benchmark_utils import print_header, print_section
import dagex


# ── Deterministic node functions ──────────────────────────────────────────────

def amplify(inputs):
    x = inputs.get("x", 0)
    return {"y": x * 3.0 + 5.0}


def attenuate(inputs):
    y = inputs.get("y", 0)
    return {"z": y * 0.5}


def add_noise(inputs):
    import random
    z = inputs.get("z", 0)
    return {"w": z + random.gauss(0.0, 1.0)}


def clip(inputs):
    w = inputs.get("w", 0)
    return {"output": max(0.0, min(100.0, w))}


# ── Analytical distribution transfers (same port names as the node functions) ─

def amplify_dist(dists):
    """y = 3x + 5  →  N(3μ+5, 3σ)  exactly."""
    x = dists["x"]
    return {"y": dagex.normal(x.mean * 3.0 + 5.0, x.std * 3.0)}


def attenuate_dist(dists):
    """z = 0.5y  →  N(0.5μ, 0.5σ)  exactly."""
    y = dists["y"]
    return {"z": dagex.normal(y.mean * 0.5, y.std * 0.5)}


def add_noise_dist(dists):
    """w = z + ε,  ε~N(0,1)  →  N(μ_z, sqrt(σ_z² + 1))  (sum of indep. normals)."""
    z = dists["z"]
    combined_std = math.sqrt(z.variance + 1.0)
    return {"w": dagex.normal(z.mean, combined_std)}

# Clip has no closed form — no transfer attached; will use MC.


def print_dist(name, dist, analytical_note=""):
    note = f"  [{analytical_note}]" if analytical_note else ""
    print(f"  {name:10s}  {dist.summary()}{note}")


def main():
    print_header("Example 07: Statistical Prediction — Known Distributions")

    print("📖 Story:")
    print("   A signal passes through an amplifier, attenuator, noise injection")
    print("   and a clipping stage.  The first three transforms are linear so we")
    print("   know their exact distribution transfer in closed form.  The clip is")
    print("   non-linear — predict() falls back to Monte Carlo for that node.\n")

    # ── Build graph ───────────────────────────────────────────────────────────
    print_section("Building the Graph")

    graph = dagex.Graph()
    graph.add(amplify,   label="Amplify",   inputs=[("x",  "x")], outputs=[("y", "y")])
    graph.add(attenuate, label="Attenuate", inputs=[("y",  "y")], outputs=[("z", "z")])
    graph.add(add_noise, label="AddNoise",  inputs=[("z",  "z")], outputs=[("w", "w")])
    graph.add(clip,      label="Clip",      inputs=[("w",  "w")], outputs=[("output", "output")])

    # Attach analytical transfers to the three linear nodes
    graph.set_dist_transfer("Amplify",   amplify_dist)
    graph.set_dist_transfer("Attenuate", attenuate_dist)
    graph.set_dist_transfer("AddNoise",  add_noise_dist)
    # Clip intentionally left without a transfer → MC fallback

    dag = graph.build()

    print(dag.to_mermaid())
    print("Node labels:", dag.node_labels())

    # ── Statistical forward pass ──────────────────────────────────────────────
    print_section("Analytical prediction (mixed-mode)")

    x_input = {"x": dagex.normal(mean=10.0, std=2.0)}
    stat = dag.predict(x_input, n_samples=5000)

    print("  Input: x ~ Normal(μ=10, σ=2)\n")
    print("  Variable  Mean±Std summary (p5 / p50 / p95)")
    print("  " + "-" * 62)
    print_dist("x",      stat["x"],      "input prior")
    print_dist("y",      stat["y"],      "analytical: N(35, 6)")
    print_dist("z",      stat["z"],      "analytical: N(17.5, 3)")
    print_dist("w",      stat["w"],      "analytical: N(17.5, √10≈3.16)")
    print_dist("output", stat["output"], "MC fallback: clip(w, 0, 100)")

    # ── Verify analytical results agree with theory ───────────────────────────
    print_section("Verification against theory")

    checks = [
        ("y",  35.0,         6.0,          "N(3·10+5, 3·2)"),
        ("z",  17.5,         3.0,          "N(35/2, 6/2)"),
        ("w",  17.5,         math.sqrt(10),"N(17.5, √(9+1))"),
    ]
    all_ok = True
    for var, exp_mean, exp_std, formula in checks:
        d = stat[var]
        mean_ok = abs(d.mean - exp_mean) < 0.01
        std_ok  = abs(d.std  - exp_std)  < 0.01
        status  = "✓" if (mean_ok and std_ok) else "✗"
        all_ok  = all_ok and mean_ok and std_ok
        print(f"  {status}  {var}: expected {formula}")
        print(f"       got mean={d.mean:.4f} (exp {exp_mean:.4f}), "
              f"std={d.std:.4f} (exp {exp_std:.4f})")

    print()
    print("  All analytical results exact:", "✓ YES" if all_ok else "✗ NO")

    # ── Targeted prediction (early stop) ─────────────────────────────────────
    print_section("Targeted prediction — stop at AddNoise")

    stat_partial = dag.predict(x_input, n_samples=1000, at_node="AddNoise")
    print("  Keys after stopping at AddNoise:", stat_partial.keys())
    print("  'output' present:", stat_partial.get("output") is not None)
    print("  w:", stat_partial["w"].summary())

    # ── Sample a few values from the final distribution for inspection ────────
    print_section("10 samples from output distribution")
    samples = stat["output"].sample_n(10)
    print("  ", [f"{s:.2f}" for s in samples])


if __name__ == "__main__":
    main()
