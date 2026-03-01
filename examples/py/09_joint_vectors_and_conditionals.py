"""Example 09: Joint Distributions — Vector Outputs, iid Pooling, and Conditionals

Demonstrates the advanced joint-distribution API: vector node outputs,
iid element pooling for better density estimation, conditional (posterior)
distributions via particle slicing, and a full correlation heatmap.

Pipeline
--------
    x ~ Normal(0, 1)                          (latent signal)
         │
    [Nonlinear]    a = tanh(x)                (compresses to (-1, 1))
         │
    [SensorArray]  obs = [a + N(0, σ), ...]   (N_SENSORS correlated sensors)
                   returns a Python list → auto-flattened to obs[0]…obs[N-1]
         │
    [Fuse]         fused = mean(obs)           (sensor-fusion estimate of a)

Key concepts shown
------------------
  1. Vector output from a node: returns a Python list → stored as obs[0], obs[1], …
  2. assume_iid("obs") — pool all 6 × n_samples points for smoother marginal KDE
  3. conditional({"fused": v}) — approximate posterior p(x | fused ≈ v) via
     particle rejection; demonstrates ABC-style Bayesian inference in the graph
  4. covariance_matrix() — plotted as an annotated colour heatmap
  5. Plot: pair plot of (x, a, obs[0], fused)

Produced figures
----------------
  09_iid_kde_comparison.png      — KDE quality without vs with iid pooling
  09_conditional_posterior.png   — posterior over x and a given fused ≈ v
  09_correlation_heatmap.png     — full Pearson r correlation matrix
  09_joint_pairs.png             — pair scatter grid: (x, a, obs[0], fused)
"""

import sys, math, random
sys.path.insert(0, '/home/runner/work/graph-sp/graph-sp/examples/py')

from benchmark_utils import print_header, print_section, print_dist_table
import dagex

SIGMA_NOISE = 0.30
N_SENSORS   = 6
N_SAMPLES   = 6_000


# ── Node functions ────────────────────────────────────────────────────────────

def nonlinear(inputs):
    """a = tanh(x)  — smooth saturating nonlinearity."""
    x = inputs.get("x", 0.0)
    return {"a": math.tanh(x)}


def sensor_array(inputs):
    """Return a list of N_SENSORS noisy observations of a.

    Returning a plain Python list auto-converts to GraphData::FloatVec and is
    stored as obs[0], obs[1], … in the joint particle table.
    """
    a = inputs.get("a", 0.0)
    return {"obs": [a + random.gauss(0.0, SIGMA_NOISE) for _ in range(N_SENSORS)]}


def fuse(inputs):
    """fused = mean(obs)  — arithmetic sensor fusion.

    inputs["obs"] arrives as a Python list (reconstructed from the FloatVec
    particle entries by predict()).
    """
    obs = inputs.get("obs") or [0.0]
    return {"fused": sum(obs) / len(obs)}


# ── Main ──────────────────────────────────────────────────────────────────────

def main():
    print_header("Example 09: Vector Outputs, iid Pooling, and Conditional Posteriors")

    print("📖 Story:")
    print("   A hidden signal x drives a saturating sensor bank.  Each of the")
    print(f"   {N_SENSORS} sensors reports a + noise.  Their average 'fused' is our")
    print("   best estimate.  We ask: what can we say about x given fused ≈ v?\n")

    # ── Build graph ───────────────────────────────────────────────────────────
    print_section("Building the graph")
    graph = dagex.Graph()
    graph.add(nonlinear,    label="Nonlinear",   inputs=[("x",   "x")],   outputs=[("a",     "a")])
    graph.add(sensor_array, label="SensorArray", inputs=[("a",   "a")],   outputs=[("obs",   "obs")])
    graph.add(fuse,         label="Fuse",        inputs=[("obs", "obs")], outputs=[("fused", "fused")])
    dag = graph.build()
    print(dag.to_mermaid())

    # ── Particle forward pass ─────────────────────────────────────────────────
    print_section(f"Particle forward pass  (n = {N_SAMPLES:,})")
    x_input = {"x": dagex.normal(0.0, 1.0)}
    stat    = dag.predict(x_input, n_samples=N_SAMPLES)
    jd      = dagex.joint(stat)

    obs_vars = [f"obs[{k}]" for k in range(N_SENSORS)]
    all_vars = ["x", "a"] + obs_vars + ["fused"]
    print(f"  Scalar dimensions captured: {len(jd.variables)}")
    print(f"  Variables: {jd.variables}\n")

    # ── Summary with iid annotation ───────────────────────────────────────────
    print_section("Joint summary  (sensor elements marked as iid)")
    jd_iid = jd.assume_iid("obs")
    jd_iid.print_summary(variables=all_vars)

    # ── Print analytical expectations ─────────────────────────────────────────
    # E[a] = E[tanh(x)] ≈ 0 (x symmetric)
    # Var(a) ≈ 0.429  →  std(a) ≈ 0.655
    # E[obs[k]] = E[a], Var(obs[k]) = Var(a) + σ_noise²
    # E[fused]  = E[a], Var(fused) = Var(a) + σ_noise²/N
    var_a = jd.covariance("a", "a")
    var_f = jd.covariance("fused", "fused")
    r_a_obs  = jd.correlation("a", "obs[0]")
    r_oi_oj  = jd.correlation("obs[0]", "obs[1]")
    r_x_fuse = jd.correlation("x", "fused")
    print_section("Key correlations")
    print(f"  r(x,     a)       = {jd.correlation('x',     'a'):+.4f}   (strong nonlinear 1-to-1 map)")
    print(f"  r(a,     obs[0])  = {r_a_obs:+.4f}   (signal-to-noise ratio)")
    print(f"  r(obs[0],obs[1])  = {r_oi_oj:+.4f}   (shared latent a; expect σ_a²/(σ_a²+σ²))")
    print(f"  r(x,     fused)   = {r_x_fuse:+.4f}   (fusion recovers x)")
    snr_r_a_obs = math.sqrt(var_a / (var_a + SIGMA_NOISE ** 2))
    snr_r_oi_oj = var_a / (var_a + SIGMA_NOISE ** 2)
    print(f"\n  Theoretical r(a, obs[k]) = sqrt(σ_a²/(σ_a²+σ²)) = {snr_r_a_obs:.4f}  (empirical: {r_a_obs:.4f})")
    print(f"  Theoretical r(obs_i, obs_j)  = σ_a²/(σ_a²+σ²)   = {snr_r_oi_oj:.4f}  (empirical: {r_oi_oj:.4f})")

    # ── Figures ───────────────────────────────────────────────────────────────
    _plot_iid_kde(jd, jd_iid)
    _plot_conditional_posterior(jd)
    _plot_correlation_heatmap(jd, all_vars)
    _plot_pairs(jd)


# ── Plot helpers ──────────────────────────────────────────────────────────────

def _plot_iid_kde(jd, jd_iid):
    print_section("Figure 1: iid KDE quality comparison")
    import numpy as np
    import matplotlib.pyplot as plt

    var = "obs[0]"
    s   = np.array(jd.samples(var))
    x_range = (float(s.mean() - 4.0 * s.std()), float(s.mean() + 4.0 * s.std()))

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(11, 4), sharey=True, sharex=True)

    for ax, (jd_use, label, pts) in zip(
        (ax1, ax2),
        [
            (jd,     "Without iid pooling",    N_SAMPLES),
            (jd_iid, "With iid pooling",        N_SENSORS * N_SAMPLES),
        ],
    ):
        s_hist = np.array(jd.samples(var))         # histogram always 1-element
        x_g, pdf = jd_use.marginal_pdf(var, x_range=x_range, n_points=350)
        ax.hist(s_hist, bins=60, density=True, alpha=0.35,
                color="steelblue", label=f"histogram  (1 element,  n={N_SAMPLES:,})")
        ax.plot(x_g, pdf, color="steelblue", linewidth=2.2,
                label=f"KDE  ({pts:,} pts)")
        ax.set_title(f"{label}\n({pts:,} points used for KDE)", fontsize=10)
        ax.set_xlabel("obs[0]")
        ax.legend(fontsize=8)

    ax1.set_ylabel("density")
    fig.suptitle(
        f"Marginal PDF of obs[0]   (tanh signal + Gaussian noise, σ={SIGMA_NOISE})",
        fontsize=11,
    )
    fig.tight_layout()
    path = "examples/py/09_iid_kde_comparison.png"
    fig.savefig(path, dpi=130, bbox_inches="tight")
    plt.close(fig)
    print(f"  Saved → {path}")


def _plot_conditional_posterior(jd):
    print_section("Figure 2: Conditional posteriors  p(x | fused ≈ v)")
    import numpy as np
    import matplotlib.pyplot as plt

    targets = [-0.60, 0.00, 0.60]
    colors  = ["royalblue", "seagreen", "crimson"]

    sx = np.array(jd.samples("x"))
    sa = np.array(jd.samples("a"))
    x_lo, x_hi = float(sx.mean() - 3.8 * sx.std()), float(sx.mean() + 3.8 * sx.std())
    a_lo, a_hi = float(sa.mean() - 3.5 * sa.std()), float(sa.mean() + 3.5 * sa.std())

    fig, (ax_x, ax_a) = plt.subplots(1, 2, figsize=(11, 4.5))

    # Overlay the prior
    xg_prior, px_prior = jd.marginal_pdf("x", x_range=(x_lo, x_hi), n_points=300)
    ag_prior, pa_prior = jd.marginal_pdf("a", x_range=(a_lo, a_hi), n_points=300)
    ax_x.plot(xg_prior, px_prior, color="black", linewidth=1.5,
              linestyle="--", alpha=0.7, label="prior  p(x)  [unconditional]")
    ax_a.plot(ag_prior, pa_prior, color="black", linewidth=1.5,
              linestyle="--", alpha=0.7, label="prior  p(a)")

    rows = []
    for tgt, col in zip(targets, colors):
        try:
            jd_c = jd.conditional({"fused": tgt})
            n    = jd_c.n_samples
            xg, px = jd_c.marginal_pdf("x", x_range=(x_lo, x_hi), n_points=300)
            ag, pa = jd_c.marginal_pdf("a", x_range=(a_lo, a_hi), n_points=300)
            ax_x.fill_between(xg, px, alpha=0.18, color=col)
            ax_x.plot(xg, px, color=col, linewidth=2.0,
                      label=f"fused≈{tgt:+.2f}  (n={n:,})")
            ax_a.fill_between(ag, pa, alpha=0.18, color=col)
            ax_a.plot(ag, pa, color=col, linewidth=2.0,
                      label=f"fused≈{tgt:+.2f}  (n={n:,})")
            rows.append((tgt, n, jd_c.mean("x"), jd_c.std("x"),
                                   jd_c.mean("a"), jd_c.std("a")))
        except ValueError as e:
            print(f"  Warning: {e}")

    for ax, xlabel, title in zip(
        (ax_x, ax_a),
        ["x  (latent signal)", "a = tanh(x)"],
        ["Posterior  p(x | fused ≈ v)", "Posterior  p(a | fused ≈ v)"],
    ):
        ax.set_xlabel(xlabel)
        ax.set_ylabel("density")
        ax.set_title(title, fontsize=10)
        ax.legend(fontsize=8)

    arctanh = lambda v: 0.5 * math.log((1 + v) / (1 - v)) if abs(v) < 1.0 else float("nan")
    fig.suptitle(
        "Approximate posteriors via particle conditioning\n"
        "(particle rejection = ABC sampler: keep particles where |fused − v| ≤ ½σ_fused)",
        fontsize=10,
    )
    fig.tight_layout()
    path = "examples/py/09_conditional_posterior.png"
    fig.savefig(path, dpi=130, bbox_inches="tight")
    plt.close(fig)
    print(f"  Saved → {path}")

    # Console table
    print(f"\n  {'fused target':>12}  {'n kept':>7}  "
          f"{'E[x|·]':>8}  {'σ[x|·]':>8}  "
          f"{'E[a|·]':>8}  {'σ[a|·]':>8}  {'arctanh(v)':>10}")
    print("  " + "─" * 72)
    for tgt, n, ex, sx_, ea, sa_ in rows:
        theory_x = arctanh(tgt)
        print(f"  {tgt:>12.2f}  {n:>7,}  "
              f"{ex:>8.4f}  {sx_:>8.4f}  "
              f"{ea:>8.4f}  {sa_:>8.4f}  {theory_x:>10.4f}")
    print("  (arctanh(v) is where population concentrates; posterior mean is wider due"
          " to the slice window |fused-v| ≤ ½σ)")


def _plot_correlation_heatmap(jd, all_vars):
    print_section("Figure 3: Correlation matrix heatmap")
    import numpy as np
    import matplotlib.pyplot as plt

    labels, mat = jd.correlation_matrix(variables=all_vars)
    mat = np.array(mat)

    fig, ax = plt.subplots(figsize=(9, 8))
    im = ax.imshow(mat, vmin=-1, vmax=1, cmap="RdBu_r", aspect="auto")
    for i in range(len(labels)):
        for j in range(len(labels)):
            v = mat[i, j]
            txt_col = "white" if abs(v) > 0.65 else "black"
            ax.text(j, i, f"{v:.2f}", ha="center", va="center",
                    fontsize=8, color=txt_col)
    ax.set_xticks(range(len(labels)))
    ax.set_yticks(range(len(labels)))
    ax.set_xticklabels(labels, rotation=45, ha="right", fontsize=9)
    ax.set_yticklabels(labels, fontsize=9)
    fig.colorbar(im, ax=ax, fraction=0.03, pad=0.02, label="Pearson r")
    ax.set_title(
        "Pearson correlation matrix\n"
        r"$x\!\to\!a=\tanh(x)\!\to\!\mathrm{obs}[k]=a+\varepsilon_k"
        r"\!\to\!\mathrm{fused}=\overline{\mathrm{obs}}$",
        fontsize=10,
    )
    fig.tight_layout()
    path = "examples/py/09_correlation_heatmap.png"
    fig.savefig(path, dpi=130, bbox_inches="tight")
    plt.close(fig)
    print(f"  Saved → {path}")

    # Expected block structure
    print("\n  Expected block structure of the correlation matrix:")
    print("  ┌─────────────────────────────────────────────────────────────┐")
    print("  │     x    │  a   │    obs[0..5]    │  fused  │")
    print("  │──────────│──────│─────────────────│─────────│")
    print("  │ x  = 1.0 │ high │      high       │  high   │")
    print("  │ a  = high│ 1.0  │ SNR/(SNR+1)     │  high   │")
    print("  │ obs high │ SNR  │ 1.0  off-diag≈r │  high   │")
    print("  │ fused    │ high │      high       │  1.0    │")
    print("  └─────────────────────────────────────────────────────────────┘")
    print(f"  (Theoretical off-diagonal r(obs_i, obs_j) ≈ "
          f"{jd.covariance('a','a')/(jd.covariance('a','a') + SIGMA_NOISE**2):.3f})")


def _plot_pairs(jd):
    print_section("Figure 4: Pair plot  (x, a, obs[0], fused)")
    import matplotlib.pyplot as plt

    fig, _ = jd.plot_pairs(
        variables=["x", "a", "obs[0]", "obs[1]", "fused"],
        title="Pair plot: latent → sensors → fusion",
    )
    path = "examples/py/09_joint_pairs.png"
    fig.savefig(path, dpi=120, bbox_inches="tight")
    plt.close(fig)
    print(f"  Saved → {path}")


if __name__ == "__main__":
    main()
