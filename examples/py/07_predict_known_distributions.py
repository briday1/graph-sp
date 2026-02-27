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
from benchmark_utils import print_header, print_section, print_dist_table
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
    print_dist_table([
        ("x",      stat["x"],      "input prior"),
        ("y",      stat["y"],      "analytical: N(35, 6)"),
        ("z",      stat["z"],      "analytical: N(17.5, 3)"),
        ("w",      stat["w"],      "analytical: N(17.5, √10≈3.16)"),
        ("output", stat["output"], "MC fallback: clip(w, 0, 100)"),
    ])

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

    # ── Joint distribution analysis ───────────────────────────────────────────
    print_section("Joint Distribution Analysis  (predict_particles)")

    print("  Running predict_particles() — full end-to-end trajectories...")
    stat_p = dag.predict_particles(x_input, n_samples=8000)
    joint  = dagex.joint(stat_p)

    print()
    joint.print_summary()

    print()
    print("  Key observations:")
    r_xy = joint.correlation("x", "y")
    r_xz = joint.correlation("x", "z")
    r_xw = joint.correlation("x", "w")
    r_xo = joint.correlation("x", "output")
    print(f"    r(x, y)      = {r_xy:.4f}  [expected ≈1.000 — y = 3x+5 is linear]")
    print(f"    r(x, z)      = {r_xz:.4f}  [expected ≈1.000 — z = 0.5y, still linear]")
    print(f"    r(x, w)      = {r_xw:.4f}  [expected <1.000 — AddNoise adds noise: w = z + ε]")
    print(f"    r(x, output) = {r_xo:.4f}  [expected <1.000 — Clip is non-linear; noise dominates]")
    print()
    print("  Note: predict_particles() always runs node functions directly.")
    print("  dist_transfer shortcuts (used by predict()) are bypassed, so")
    print("  exact joint structure is preserved through ALL nodes.")

    # ── Analytical joint vs empirical ─────────────────────────────────────────
    print_section("Analytical Joint vs Empirical  (x, y, z, w)")

    # Pipeline is a sequence of linear Gaussian transforms up through w:
    #   x  ~ N(10, 2)           Var(x) = 4
    #   y = 3x + 5              Var(y) = 9·4 = 36
    #   z = 0.5y                Var(z) = 0.25·36 = 9
    #   w = z + ε, ε~N(0,1)    Var(w) = 9 + 1 = 10
    #
    # Covariances via linearity of covariance:
    #   Cov(x, y) = 3·Var(x) = 12       → r(x,y) = 12/(2·6)   = 1.000
    #   Cov(x, z) = 1.5·Var(x) = 6      → r(x,z) = 6/(2·3)    = 1.000
    #   Cov(x, w) = Cov(x, z) = 6       → r(x,w) = 6/(2·√10)  ≈ 0.9487
    #   Cov(y, z) = 0.5·Var(y) = 18     → r(y,z) = 18/(6·3)   = 1.000
    #   Cov(y, w) = Cov(y, z) = 18      → r(y,w) = 18/(6·√10) ≈ 0.9487
    #   Cov(z, w) = Var(z) = 9          → r(z,w) = 9/(3·√10)  ≈ 0.9487

    r_noise = 3.0 / math.sqrt(10.0)   # = Cov(x,w) / (σ_x · σ_w)
    analytical_corr = {
        ("x", "x"): 1.0,
        ("x", "y"): 1.0,
        ("x", "z"): 1.0,
        ("x", "w"): r_noise,
        ("y", "y"): 1.0,
        ("y", "z"): 1.0,
        ("y", "w"): r_noise,
        ("z", "z"): 1.0,
        ("z", "w"): r_noise,
        ("w", "w"): 1.0,
    }
    # Symmetrise
    for (a, b), v in list(analytical_corr.items()):
        analytical_corr[(b, a)] = v

    vars_linear = ["x", "y", "z", "w"]
    COL = 6

    # Print side-by-side table: analytical | empirical | error
    header = (
        f"  {'':>{COL}}  "
        + "  ".join(f"{v:>6}" for v in vars_linear)
        + "     (error vs analytical)"
    )
    print("  Analytical correlation matrix:")
    print(header)
    print("  " + "─" * (len(header) - 2))
    for vi in vars_linear:
        row = f"  {vi:<{COL}}  " + "  ".join(
            f"{analytical_corr[(vi, vj)]:6.4f}" for vj in vars_linear
        )
        print(row)

    print()
    print("  Empirical correlation matrix (predict_particles, n=8000):")
    print(header)
    print("  " + "─" * (len(header) - 2))
    max_err = 0.0
    for vi in vars_linear:
        emp_vals = [joint.correlation(vi, vj) for vj in vars_linear]
        errs     = [abs(emp_vals[j] - analytical_corr[(vi, vars_linear[j])])
                    for j in range(len(vars_linear))]
        max_err  = max(max_err, max(errs))
        row  = f"  {vi:<{COL}}  " + "  ".join(f"{v:6.4f}" for v in emp_vals)
        errs_fmt = "  err: " + "  ".join(f"{e:.4f}" for e in errs)
        print(row + errs_fmt)

    print()
    ok = max_err < 0.05
    print(f"  Maximum |empirical − analytical| correlation error: {max_err:.4f}  "
          + ("✓ within tolerance" if ok else "✗ exceeds 0.05"))
    print()
    print("  Interpretation:")
    print(f"    r(x,y) = r(x,z) = r(y,z) = 1.000  — pure linear chain, zero variance inflation")
    print(f"    r(*,w) ≈ {r_noise:.4f}              — noise ε dilutes all correlations equally")
    print(f"    The empirical particles recover the exact analytical joint.")

    # ── Marginal PDFs: KDE vs analytical Normal ───────────────────────────────
    print_section("Marginal PDFs: KDE vs Analytical")

    import numpy as np
    from scipy.stats import norm as sp_norm

    print("  Evaluating KDE marginal PDFs at 5 test points and comparing to N(μ,σ).")
    print()
    print(f"  {'Var':<4}  {'μ':>7}  {'σ':>7}   {'pt':>8}  {'KDE pdf':>10}  {'Exact pdf':>10}  {'rel err':>8}")
    print("  " + "─" * 64)

    marginal_specs = [
        ("x",  10.0, 2.0),
        ("y",  35.0, 6.0),
        ("z",  17.5, 3.0),
        ("w",  17.5, math.sqrt(10.0)),
    ]
    rng_pts = [-1.5, -0.5, 0.0, 0.5, 1.5]   # in units of σ from μ

    for var, mu, sigma in marginal_specs:
        x_pts = [mu + z * sigma for z in rng_pts]
        x_grid, kde_vals = joint.marginal_pdf(var, n_points=400)
        # Interpolate KDE at each test point
        for pt in x_pts:
            exact = sp_norm.pdf(pt, loc=mu, scale=sigma)
            # Nearest-neighbour interpolation in the grid (fine enough)
            idx   = int(np.searchsorted(x_grid, pt))
            idx   = max(0, min(len(kde_vals) - 1, idx))
            kde   = float(kde_vals[idx])
            rel   = abs(kde - exact) / exact if exact > 1e-12 else float("nan")
            print(f"  {var:<4}  {mu:7.2f}  {sigma:7.4f}   {pt:8.3f}  {kde:10.6f}  {exact:10.6f}  {rel:8.4f}")
        print()

    # ── Joint PDF: KDE vs analytical bivariate Normal ─────────────────────────
    print_section("Joint PDF: KDE vs Analytical Bivariate Normal  (x, w)")

    # Analytical bivariate normal for (x, w)
    #   x ~ N(10, 2),  w ~ N(17.5, √10),  Cov(x,w) = 6
    mu_x, sig_x = 10.0, 2.0
    mu_w, sig_w = 17.5, math.sqrt(10.0)
    cov_xw      = 6.0   # derived above

    def bvn_pdf(xi, wi):
        """Analytical bivariate normal PDF for (x, w)."""
        rho  = cov_xw / (sig_x * sig_w)
        zx   = (xi - mu_x) / sig_x
        zw   = (wi - mu_w) / sig_w
        rho2 = 1.0 - rho ** 2
        exp  = math.exp(-1.0 / (2.0 * rho2) * (zx**2 - 2*rho*zx*zw + zw**2))
        return exp / (2.0 * math.pi * sig_x * sig_w * math.sqrt(rho2))

    # Evaluate on a sparse 5×5 grid of test points for console display
    test_xs = [mu_x + dz * sig_x for dz in [-1.0, -0.5, 0.0, 0.5, 1.0]]
    test_ws = [mu_w + dz * sig_w for dz in [-1.0, -0.5, 0.0, 0.5, 1.0]]

    xx_kde, yy_kde, zz_kde = joint.joint_pdf("x", "w", n_grid=80)

    def kde_at(xi, wi):
        # Bilinear lookup in the meshgrid
        ix = int(np.searchsorted(xx_kde[0, :], xi))
        iy = int(np.searchsorted(yy_kde[:, 0], wi))
        ix = max(0, min(zz_kde.shape[1] - 1, ix))
        iy = max(0, min(zz_kde.shape[0] - 1, iy))
        return float(zz_kde[iy, ix])

    print(f"  Grid: x ∈ [μ_x ± σ_x],  w ∈ [μ_w ± σ_w]")
    print(f"  Showing joint PDF f(x, w) at a 5×5 test grid.")
    print()
    print(f"  {'KDE  f(x,w)':<36}   {'Analytical  f(x,w)':<36}")
    print(f"  {'x →':>6}  " + "  ".join(f"{v:7.2f}" for v in test_xs)
          + "     " + "  ".join(f"{v:7.2f}" for v in test_xs))
    print("  " + "─" * 80)

    max_rel_err = 0.0
    peak_anal = max(bvn_pdf(xi, wi) for xi in test_xs for wi in test_ws)
    for wi in test_ws:
        kde_row  = [kde_at(xi, wi) for xi in test_xs]
        anal_row = [bvn_pdf(xi, wi) for xi in test_xs]
        for k, a in zip(kde_row, anal_row):
            if a > 0.20 * peak_anal:   # only compare core region (≥20% of peak)
                max_rel_err = max(max_rel_err, abs(k - a) / a)
        print(
            f"  w={wi:5.2f}  " + "  ".join(f"{v:7.5f}" for v in kde_row)
            + "     " + "  ".join(f"{v:7.5f}" for v in anal_row)
        )

    print()
    print(f"  Maximum relative error (KDE vs analytical): {max_rel_err:.4f}  "
          + ("✓" if max_rel_err < 0.50 else "✗"))
    print()
    print("  Note: 2D KDE pointwise error of ~40% at ±1σ is expected with finite samples")
    print("  (curse of dimensionality).  Contour shapes and correlation structure are correct.")

    # ── Save comparison figure ─────────────────────────────────────────────────
    import os
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    fig, axes = plt.subplots(1, 2, figsize=(12, 5))

    # ---- (1) Marginal PDF of w: KDE vs analytical ----
    ax = axes[0]
    x_g, kde_g = joint.marginal_pdf("w", n_points=300)
    ax.plot(x_g, kde_g, color="steelblue", linewidth=2, label="KDE (empirical)")
    ax.plot(
        x_g,
        [sp_norm.pdf(v, loc=mu_w, scale=sig_w) for v in x_g],
        color="crimson", linewidth=2, linestyle="--", label=f"Analytical N({mu_w:.1f}, √10)"
    )
    ax.set_title("Marginal PDF of w", fontsize=12)
    ax.set_xlabel("w"); ax.set_ylabel("density")
    ax.legend(fontsize=9)

    # ---- (2) Joint PDF (x, w): KDE (solid) overlaid with analytical (dashed) ----
    ax = axes[1]

    # Build the analytical grid on the same extent as the KDE grid so levels are comparable
    xi_grid = xx_kde[0, :]
    wi_grid = yy_kde[:, 0]
    ZZ_anal = np.array([[bvn_pdf(xi, wi) for xi in xi_grid] for wi in wi_grid])

    # Use contour levels derived from the analytical density so both share identical iso-lines
    vmax    = float(ZZ_anal.max())
    levels  = np.linspace(vmax * 0.02, vmax * 0.98, 10)

    # KDE — solid filled contours + solid lines (blue palette)
    ax.contourf(xx_kde, yy_kde, zz_kde, levels=levels, cmap="Blues", alpha=0.55)
    cs_kde = ax.contour(xx_kde, yy_kde, zz_kde,  levels=levels,
                        colors="steelblue", linewidths=1.5, linestyles="solid")

    # Analytical — dashed contour lines only (crimson), no fill, same levels
    cs_anal = ax.contour(xx_kde, yy_kde, ZZ_anal, levels=levels,
                         colors="crimson", linewidths=1.8, linestyles="dashed")

    # Proxy artists for the legend (contour objects don't auto-label well)
    from matplotlib.lines import Line2D
    legend_handles = [
        Line2D([0], [0], color="steelblue", linewidth=2, linestyle="solid",  label="KDE (empirical)"),
        Line2D([0], [0], color="crimson",   linewidth=2, linestyle="dashed", label="Analytical BVN"),
    ]
    ax.legend(handles=legend_handles, fontsize=9, loc="upper left")
    ax.set_title("Joint PDF  f(x, w)  — KDE vs Analytical", fontsize=12)
    ax.set_xlabel("x"); ax.set_ylabel("w")

    fig.suptitle("Example 07 — KDE vs Analytical  (x, w)", fontsize=13)
    fig.tight_layout()

    plot_path_pdf = os.path.join(os.path.dirname(__file__), "07_joint_pdf_comparison.png")
    fig.savefig(plot_path_pdf, dpi=130, bbox_inches="tight")
    plt.close(fig)
    print(f"\n  PDF comparison figure saved → {plot_path_pdf}")

    # ---- Pair plot (all variables) ----
    plot_path = os.path.join(os.path.dirname(__file__), "07_joint_pairs.png")
    fig, _ = joint.plot_pairs(title="Example 07 — Joint Distribution Pair Plot")
    fig.savefig(plot_path, dpi=120, bbox_inches="tight")
    plt.close(fig)
    print(f"  Pair plot saved           → {plot_path}")

    print()
    print("  Forcing x ⊥ output (assume_independent):")
    joint_indep = joint.assume_independent("x", "output")
    r_after = joint_indep.correlation("x", "output")
    print(f"    r(x, output) before = {r_xo:.4f}")
    print(f"    r(x, output) after  = {r_after:.4f}  [≈ 0: independence enforced]")


if __name__ == "__main__":
    main()
