"""Example 10: TDOA Localization Performance Analysis

Uses predict() to characterise the actual position-error distribution
of a TDOA-based localizer and compare it to the Cramér-Rao Lower Bound (CRLB).

The CRLB assumes an efficient, unbiased estimator and linearized geometry.
The particle simulation runs the *actual* iterative-LS algorithm and captures:
  - Estimator bias (shifts the ellipse centre away from the truth)
  - Super-CRLB variance near geometry singularities (hyperbola tangencies)
  - Non-Gaussian tails when the algorithm gets pulled to local minima
  - The full 2×2 MSE matrix P = Cov + bias·biasᵀ (not just trace)

Geometry
--------
  5 receivers on a ~1 km baseline; true source at a known fixed position.
  Noise model: RDOA (range-difference-of-arrival) ~ N(0, σ_r²) per pair,
  where σ_r = SIGMA_R metres (timing noise × c collapsed into distance units).

Pipeline
--------
    input dists:  rdoa_k ~ N(true_rdoa_k, σ_r)   for k = 0 … M-2
                  (measurement uncertainty — centres on the true geometry,
                  spread by the noise power σ_r²)
         │
    [LocalizeTLS]  Taylor-series iterative LS — takes scalar rdoa_k inputs,
                   solves for (x_est, y_est)

    The graph is a standalone algorithm.  predict() propagates the
    input distribution through it to obtain the output error distribution.
    The CRLB is the theoretical lower bound on that output covariance.

Produced figures
----------------
  10_error_ellipses.png     — error scatter + 1σ ellipses (empirical vs CRLB)
  10_marginal_errors.png    — marginal PDFs of x-error and y-error
  10_efficiency_heatmap.png — grid: σ_pos_emp / σ_pos_crlb over the scene
  10_tdoa_pairs.png         — pair scatter of rdoa[0..1] and (x_est, y_est)
"""

import sys, math, random
sys.path.insert(0, '/home/runner/work/graph-sp/graph-sp/examples/py')

import numpy as np
from benchmark_utils import print_header, print_section, print_dist_table
import dagex

# ── Scenario constants ────────────────────────────────────────────────────────

SIGMA_R  = 5.0   # RDOA noise 1-sigma [metres]
N_SAMPLES = 8_000

# Receiver positions [m].  Index 0 is the reference.
RECEIVERS = np.array([
    [   0.0,    0.0],   # ref
    [ 900.0,   80.0],
    [ 820.0,  900.0],
    [  60.0,  950.0],
    [-120.0,  430.0],
], dtype=float)

# True source location used for the main particle analysis
X_TRUE, Y_TRUE = 450.0, 380.0


# ── Helper: compute RDOA Jacobian and CRLB at a position ─────────────────────

def rdoa_jacobian(px, py, receivers=RECEIVERS):
    """(M-1)×2 matrix  ∂RDOA_k/∂[x,y]  at (px, py)."""
    r0  = receivers[0]
    d0  = math.hypot(px - r0[0], py - r0[1])
    rows = []
    for si in receivers[1:]:
        di = math.hypot(px - si[0], py - si[1])
        rows.append([
            (px - si[0]) / di - (px - r0[0]) / d0,
            (py - si[1]) / di - (py - r0[1]) / d0,
        ])
    return np.array(rows)   # (M-1, 2)


def crlb_covariance(px, py, sigma_r=None, receivers=RECEIVERS):
    """2×2 CRLB position error covariance at (px, py)."""
    if sigma_r is None:
        sigma_r = SIGMA_R
    H = rdoa_jacobian(px, py, receivers)
    fim = (H.T @ H) / sigma_r**2
    try:
        return np.linalg.inv(fim)
    except np.linalg.LinAlgError:
        return np.full((2, 2), np.inf)


def error_ellipse_params(cov2, n_sigma=1.0):
    """Semi-axes (a, b) [m] and rotation angle [deg] from a 2×2 covariance."""
    vals, vecs = np.linalg.eigh(cov2)
    vals = np.maximum(vals, 0.0)
    a  = n_sigma * math.sqrt(vals[1])
    b  = n_sigma * math.sqrt(vals[0])
    angle = math.degrees(math.atan2(vecs[1, 1], vecs[0, 1]))
    return a, b, angle


def draw_ellipse(ax, cov2, center, n_sigma=1.0, n_pts=200, **kw):
    theta = np.linspace(0, 2 * math.pi, n_pts)
    vals, vecs = np.linalg.eigh(cov2)
    vals = np.maximum(vals, 0.0)
    radii = n_sigma * np.sqrt(vals)
    pts   = vecs @ (radii[:, None] * np.vstack([np.cos(theta), np.sin(theta)]))
    ax.plot(pts[0] + center[0], pts[1] + center[1], **kw)


# ── Helper: true RDOAs from geometry (no noise) ──────────────────────────────

def true_rdoas(x_src, y_src, receivers=RECEIVERS):
    """Exact range-difference-of-arrival values for a source at (x_src, y_src).

    These become the *means* of the input distributions passed to
    predict — the noise power goes into the distribution width, not
    into the graph itself.
    """
    r0 = receivers[0]
    d0 = math.hypot(x_src - r0[0], y_src - r0[1])
    return [math.hypot(x_src - ri[0], y_src - ri[1]) - d0 for ri in receivers[1:]]


# ── Node functions ────────────────────────────────────────────────────────────

def localize_tls_node(inputs):
    """Taylor-series iterative LS localization from scalar RDOA inputs.

    Each RDOA channel arrives as a separate scalar input ``rdoa_k``.  The graph
    is a pure algorithm — it knows nothing about source geometry or noise power.
    Those live in the input distributions supplied to predict().
    """
    M = len(RECEIVERS) - 1
    rdoa_meas = [inputs.get(f"rdoa_{k}", 0.0) for k in range(M)]

    r0   = RECEIVERS[0]
    rest = RECEIVERS[1:]

    # Initialise at the centroid of the receiver array
    x, y = float(RECEIVERS[:, 0].mean()), float(RECEIVERS[:, 1].mean())

    for _ in range(40):
        d0 = math.hypot(x - r0[0], y - r0[1])
        H_rows, res = [], []
        for si, rdoa_i in zip(rest, rdoa_meas):
            di = math.hypot(x - si[0], y - si[1])
            res.append(rdoa_i - (di - d0))
            H_rows.append([
                (x - si[0]) / di - (x - r0[0]) / d0,
                (y - si[1]) / di - (y - r0[1]) / d0,
            ])
        H   = np.array(H_rows)
        b   = np.array(res)
        delta, _, _, _ = np.linalg.lstsq(H, b, rcond=None)
        x  += float(delta[0])
        y  += float(delta[1])
        if math.hypot(delta[0], delta[1]) < 1e-7:
            break

    return {"x_est": float(x), "y_est": float(y)}


# ── Main ──────────────────────────────────────────────────────────────────────

def build_and_run(x_true=X_TRUE, y_true=Y_TRUE, n_samples=N_SAMPLES, sigma_r=SIGMA_R):
    """Build the localizer graph, run predict, return JointDistribution.

    The graph is a standalone algorithm (LocalizeTLS only).  The measurement
    uncertainty is expressed entirely in the input distributions:

        rdoa_k ~ N(true_rdoa_k, sigma_r)   for k = 0 … M-2

    predict() propagates those M input distributions through the
    localizer to obtain the joint distribution over (rdoa_0…rdoa_{M-2},
    x_est, y_est), from which we read off the empirical error covariance.

    sigma_r can be varied freely — the graph itself is rebuilt each call but
    the algorithm (LocalizeTLS) is identical; only the input distributions change.
    """
    M = len(RECEIVERS) - 1
    rdoa_means = true_rdoas(x_true, y_true)          # noise-free geometry
    input_dists = {
        f"rdoa_{k}": dagex.normal(rdoa_means[k], sigma_r)
        for k in range(M)
    }

    graph = dagex.Graph()
    graph.add(localize_tls_node, label="LocalizeTLS",
              inputs=[(f"rdoa_{k}", f"rdoa_{k}") for k in range(M)],
              outputs=[("x_est", "x_est"), ("y_est", "y_est")])
    dag = graph.build()
    stat = dag.predict(input_dists, n_samples=n_samples)
    return dagex.joint(stat)


def empirical_mse_matrix(jd, x_true, y_true):
    """Full 2×2 MSE matrix  P = Cov(x_est, y_est) + bias·biasᵀ."""
    xs = np.array(jd.samples("x_est"))
    ys = np.array(jd.samples("y_est"))
    ex, ey = xs.mean() - x_true, ys.mean() - y_true
    cov  = np.cov(np.vstack([xs, ys]))
    mse  = cov + np.outer([ex, ey], [ex, ey])
    return mse, np.array([ex, ey])


def main():
    print_header("Example 10: TDOA Localization — Empirical vs CRLB Performance")

    print("  Receivers:")
    for k, (rx, ry) in enumerate(RECEIVERS):
        tag = " (ref)" if k == 0 else ""
        print(f"    R{k}: ({rx:7.1f}, {ry:7.1f}) m{tag}")
    print(f"  True source: ({X_TRUE}, {Y_TRUE}) m")
    print(f"  RDOA noise σ = {SIGMA_R} m   ({SIGMA_R/343*1e6:.1f} µs at 343 m/s)\n")

    # ── Main particle analysis ────────────────────────────────────────────────
    print_section(f"Particle analysis  (n = {N_SAMPLES:,} at true position)")

    jd = build_and_run()
    P_emp, bias = empirical_mse_matrix(jd, X_TRUE, Y_TRUE)
    P_crlb      = crlb_covariance(X_TRUE, Y_TRUE)

    a_emp,  b_emp,  ang_emp  = error_ellipse_params(P_emp)
    a_crlb, b_crlb, ang_crlb = error_ellipse_params(P_crlb)

    xs = np.array(jd.samples("x_est"))
    ys = np.array(jd.samples("y_est"))
    rms_emp  = math.sqrt(float(np.trace(P_emp)))
    rms_crlb = math.sqrt(float(np.trace(P_crlb)))

    print(f"\n  {'':30s}  {'Empirical':>12}  {'CRLB':>10}")
    print("  " + "─" * 58)
    print(f"  {'RMS position error [m]':<30s}  {rms_emp:12.4f}  {rms_crlb:10.4f}")
    print(f"  {'1σ ellipse semi-major [m]':<30s}  {a_emp:12.4f}  {a_crlb:10.4f}")
    print(f"  {'1σ ellipse semi-minor [m]':<30s}  {b_emp:12.4f}  {b_crlb:10.4f}")
    print(f"  {'Ellipse azimuth [deg]':<30s}  {ang_emp:12.2f}  {ang_crlb:10.2f}")
    print(f"  {'Bias [x, y] [m]':<30s}  [{bias[0]:+.4f}, {bias[1]:+.4f}]")
    print(f"  {'RMS degradation (emp/CRLB)²':<30s}  {(rms_emp/rms_crlb)**2:12.4f}")

    eff = (rms_emp / rms_crlb)**2
    verdict = "✓ near-optimal" if eff < 1.25 else ("△ moderately sub-optimal" if eff < 2.0 else "✗ poor")
    print(f"\n  Sub-optimality at ({X_TRUE:.0f}, {Y_TRUE:.0f}) m:  {eff:.3f}  {verdict}")

    # Print joint summary for the particle variables
    # rdoa_k inputs are not iid (different means) — summarise individually
    rdoa_vars = [f"rdoa_{k}" for k in range(len(RECEIVERS) - 1)]
    jd.print_summary(variables=rdoa_vars + ["x_est", "y_est"])

    # ── Figures ───────────────────────────────────────────────────────────────
    _plot_error_ellipses(jd, P_emp, P_crlb, bias)
    _plot_marginal_errors(jd)
    _plot_efficiency_heatmap()
    _plot_pairs(jd)
    _plot_sigma_sweep()

    print()
    print_section("Summary")
    print("  The particle simulation captures the full error distribution of the")
    print("  actual TLS algorithm, including:")
    print("  • Any estimator bias not visible in the CRLB")
    print("  • Heavier tails near geometry singularities (hyperbola tangencies)")
    print("  • Non-elliptical contours when the linearisation is inaccurate")
    print("  • The sub-optimality ratio (emp/CRLB)² = tr(P_emp) / tr(CRLB) — how much")
    print("    the algorithm exceeds the theoretical minimum; 1 = achieves the bound")
    print("  This is always a tighter / more honest bound than the CRLB alone.")


# ── Plot helpers ──────────────────────────────────────────────────────────────

def _plot_sigma_sweep(
    sigmas=None,
    n_samples=3_000,
    x_true=X_TRUE,
    y_true=Y_TRUE,
):
    """Sweep sigma_r and plot empirical RMS vs CRLB RMS, plus (emp/CRLB)² ratio.

    Each point in the sweep is a fresh call to build_and_run() with a different
    input distribution width.  The graph (LocalizeTLS) is identical every time;
    only the N(true_rdoa_k, sigma_r) input distributions change.
    """
    print_section("Figure 5: Estimator variance vs CRLB across noise power")
    import matplotlib.pyplot as plt

    if sigmas is None:
        sigmas = np.logspace(np.log10(0.5), 4, 30)  # 0.5 m → 10 km

    emp_rms_vals   = []
    crlb_rms_vals  = []
    ratio_vals     = []

    for s in sigmas:
        jd = build_and_run(x_true=x_true, y_true=y_true,
                           n_samples=n_samples, sigma_r=s)
        P_emp, _  = empirical_mse_matrix(jd, x_true, y_true)
        P_crlb    = crlb_covariance(x_true, y_true, sigma_r=s)
        rms_e  = math.sqrt(max(float(np.trace(P_emp)),  0.0))
        rms_c  = math.sqrt(max(float(np.trace(P_crlb)), 0.0))
        emp_rms_vals.append(rms_e)
        crlb_rms_vals.append(rms_c)
        ratio_vals.append(rms_e / rms_c if rms_c > 0 else float("nan"))
        print(f"    σ_r = {s:6.2f} m   emp = {rms_e:.3f} m   CRLB = {rms_c:.3f} m   "
              f"(emp/CRLB)² = {(rms_e/rms_c)**2:.3f}")

    emp_rms_vals  = np.array(emp_rms_vals)
    crlb_rms_vals = np.array(crlb_rms_vals)
    ratio_vals    = np.array(ratio_vals)

    # Points where TLS diverges (emp >> CRLB) — physically meaningful breakdown
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 5))

    # ── Left: absolute RMS ────────────────────────────────────────────────────
    ax1.plot(sigmas, crlb_rms_vals, color="crimson", linewidth=2.5,
             linestyle="--", label="CRLB  (theoretical minimum)")
    ax1.plot(sigmas, emp_rms_vals, color="steelblue", linewidth=2.0,
             marker="o", markersize=4, label="Empirical  (particle sim)")
    ax1.axvline(SIGMA_R, color="gray", linewidth=1.0, linestyle=":",
                label=f"nominal σ_r = {SIGMA_R} m")
    ax1.set_xscale("log")
    ax1.set_yscale("log")
    ax1.set_xlabel("RDOA noise  σ_r  [m]")
    ax1.set_ylabel("RMS position error  [m]")
    ax1.set_title("Absolute error vs noise power")
    ax1.legend(fontsize=9)
    ax1.grid(True, which="both", alpha=0.3)

    # ── Right: (emp/CRLB)² ratio ──────────────────────────────────────────────
    ax2.plot(sigmas, ratio_vals**2, color="darkorange", linewidth=2.0,
             marker="o", markersize=4, label="(emp/CRLB)²")
    ax2.axhline(1.0, color="crimson", linewidth=1.5, linestyle="--",
                label="CRLB limit  (ratio = 1)")
    ax2.axvline(SIGMA_R, color="gray", linewidth=1.0, linestyle=":")
    ax2.set_xscale("log")
    ax2.set_yscale("log")
    ax2.set_xlabel("RDOA noise  σ_r  [m]")
    ax2.set_ylabel("(emp RMS / CRLB RMS)²")
    ax2.set_title("Sub-optimality ratio vs noise power")
    ax2.legend(fontsize=9)
    ax2.grid(True, which="both", alpha=0.3)

    fig.suptitle(
        f"TDOA TLS localizer — noise power sweep  "
        f"({len(RECEIVERS)} receivers, source at ({x_true:.0f}, {y_true:.0f}) m)\n"
        f"n_samples = {n_samples:,} per point,  {len(sigmas)} σ_r values",
        fontsize=9,
    )
    fig.tight_layout()
    path = "examples/py/10_sigma_sweep.png"
    fig.savefig(path, dpi=130, bbox_inches="tight")
    plt.close(fig)
    print(f"  Saved → {path}")


def _plot_error_ellipses(jd, P_emp, P_crlb, bias):
    print_section("Figure 1: Error scatter and 1σ ellipses")
    import matplotlib.pyplot as plt

    xs = np.array(jd.samples("x_est")) - X_TRUE
    ys = np.array(jd.samples("y_est")) - Y_TRUE

    fig, ax = plt.subplots(figsize=(7, 6))
    ax.scatter(xs, ys, s=1.5, alpha=0.15, color="steelblue", label=f"particles  (n={N_SAMPLES:,})")
    ax.scatter(*bias, s=90, color="steelblue", marker="x",
               linewidths=2.0, zorder=5, label=f"empirical mean  ({bias[0]:+.2f}, {bias[1]:+.2f}) m")
    ax.scatter(0, 0, s=80, color="crimson", marker="+",
               linewidths=2.5, zorder=5, label="true position")

    draw_ellipse(ax, P_emp,  bias, n_sigma=1.0,
                 color="steelblue", linewidth=2.0, linestyle="-",
                 label="empirical 1σ")
    draw_ellipse(ax, P_crlb, (0, 0), n_sigma=1.0,
                 color="crimson",   linewidth=2.0, linestyle="--",
                 label="CRLB 1σ  (centred on truth)")

    # 2σ empirical in lighter style
    draw_ellipse(ax, P_emp,  bias, n_sigma=2.0,
                 color="steelblue", linewidth=1.0, linestyle=":",
                 label="empirical 2σ", alpha=0.6)
    draw_ellipse(ax, P_crlb, (0, 0), n_sigma=2.0,
                 color="crimson",   linewidth=1.0, linestyle=":",
                 label="CRLB 2σ", alpha=0.6)

    # CRLB and empirical RMS annotations
    rms_emp  = math.sqrt(float(np.trace(P_emp)))
    rms_crlb = math.sqrt(float(np.trace(P_crlb)))
    ax.set_xlabel("x error  [m]")
    ax.set_ylabel("y error  [m]")
    ax.set_title(
        f"TDOA localization error  ({len(RECEIVERS)}-receiver 2D array)\n"
        f"True source: ({X_TRUE:.0f}, {Y_TRUE:.0f}) m  —  σ_RDOA = {SIGMA_R} m\n"
        f"RMS emp = {rms_emp:.2f} m    CRLB = {rms_crlb:.2f} m    "
        f"(emp/CRLB)² = {(rms_emp/rms_crlb)**2:.3f}",
        fontsize=9,
    )
    ax.set_aspect("equal")
    ax.legend(fontsize=8, loc="upper right")
    ax.axhline(0, color="gray", linewidth=0.5)
    ax.axvline(0, color="gray", linewidth=0.5)
    fig.tight_layout()
    path = "examples/py/10_error_ellipses.png"
    fig.savefig(path, dpi=140, bbox_inches="tight")
    plt.close(fig)
    print(f"  Saved → {path}")


def _plot_marginal_errors(jd):
    print_section("Figure 2: Marginal error PDFs vs Gaussian CRLB")
    import matplotlib.pyplot as plt
    from scipy.stats import norm as scipy_norm

    P_crlb = crlb_covariance(X_TRUE, Y_TRUE)
    xs = np.array(jd.samples("x_est")) - X_TRUE
    ys = np.array(jd.samples("y_est")) - Y_TRUE

    fig, (ax_x, ax_y) = plt.subplots(1, 2, figsize=(11, 4.5))

    for ax, errs, lbl, crlb_std in zip(
        (ax_x, ax_y),
        (xs, ys),
        ("x error  [m]", "y error  [m]"),
        (math.sqrt(P_crlb[0, 0]), math.sqrt(P_crlb[1, 1])),
    ):
        ax.hist(errs, bins=60, density=True, alpha=0.38, color="steelblue")
        xg = np.linspace(errs.min(), errs.max(), 300)
        # CRLB Gaussian (centred on 0)
        ax.plot(xg, scipy_norm.pdf(xg, 0, crlb_std), color="crimson",
                linewidth=2.5, linestyle="--",
                label=f"CRLB  N(0, {crlb_std:.2f}²)")
        # Empirical KDE
        xg_k, pdf_k = jd.marginal_pdf("x_est" if "x" in lbl else "y_est",
                                       n_points=300)
        x_shift = X_TRUE if "x" in lbl else Y_TRUE
        ax.plot(xg_k - x_shift, pdf_k, color="steelblue",
                linewidth=2.0, label="empirical KDE")
        ax.axvline(errs.mean(), color="steelblue", linestyle=":", linewidth=1.5,
                   label=f"emp mean  {errs.mean():+.3f} m")
        ax.axvline(0, color="crimson", linestyle=":", linewidth=1.2, alpha=0.6)
        ax.set_xlabel(lbl)
        ax.set_ylabel("density")
        ax.legend(fontsize=8)

    ax_x.set_title("Marginal x-error: empirical vs CRLB Gaussian")
    ax_y.set_title("Marginal y-error: empirical vs CRLB Gaussian")
    fig.tight_layout()
    path = "examples/py/10_marginal_errors.png"
    fig.savefig(path, dpi=130, bbox_inches="tight")
    plt.close(fig)
    print(f"  Saved → {path}")


def _plot_efficiency_heatmap(grid_n=22, mc_per_point=600):
    """Compute and plot σ_emp / σ_CRLB over a position grid.

    Done in pure numpy (bypassing the dagex graph) for speed.
    The dagex pipeline is the same computation — this is just a fast batch.
    """
    print_section(f"Figure 3: Algorithm efficiency heatmap  ({grid_n}×{grid_n} grid, {mc_per_point} MC/point)")
    import matplotlib.pyplot as plt
    import warnings

    margin = 150.0
    x_min, x_max = RECEIVERS[:, 0].min() - margin, RECEIVERS[:, 0].max() + margin
    y_min, y_max = RECEIVERS[:, 1].min() - margin, RECEIVERS[:, 1].max() + margin
    xs_grid = np.linspace(x_min, x_max, grid_n)
    ys_grid = np.linspace(y_min, y_max, grid_n)

    crlb_rms  = np.full((grid_n, grid_n), np.nan)
    emp_rms   = np.full((grid_n, grid_n), np.nan)
    efficiency = np.full((grid_n, grid_n), np.nan)

    rng = np.random.default_rng(42)

    def _np_localize(rdoa_meas, n_iter=30):
        """Pure-numpy TLS localizer (same algorithm, faster for batch)."""
        r0   = RECEIVERS[0]
        rest = RECEIVERS[1:]
        x, y = float(RECEIVERS[:, 0].mean()), float(RECEIVERS[:, 1].mean())
        for _ in range(n_iter):
            d0     = math.hypot(x - r0[0], y - r0[1])
            dists  = np.hypot(x - rest[:, 0], y - rest[:, 1])
            H      = np.column_stack([
                (x - rest[:, 0]) / dists - (x - r0[0]) / d0,
                (y - rest[:, 1]) / dists - (y - r0[1]) / d0,
            ])
            res    = rdoa_meas - (dists - d0)
            delta, _, _, _ = np.linalg.lstsq(H, res, rcond=None)
            x += delta[0]; y += delta[1]
            if np.hypot(delta[0], delta[1]) < 1e-7:
                break
        return x, y

    n_print = max(1, grid_n * grid_n // 4)
    count   = 0
    for i, xt in enumerate(xs_grid):
        for j, yt in enumerate(ys_grid):
            count += 1
            try:
                P_c = crlb_covariance(xt, yt)
                if not np.all(np.isfinite(P_c)):
                    continue
                crlb_rms[j, i] = math.sqrt(max(float(np.trace(P_c)), 0.0))

                # Monte-Carlo
                r0 = RECEIVERS[0]
                d0 = np.hypot(xt - r0[0], yt - r0[1])
                true_rdoas = np.hypot(xt - RECEIVERS[1:, 0], yt - RECEIVERS[1:, 1]) - d0
                noise      = rng.normal(0, SIGMA_R, size=(mc_per_point, len(RECEIVERS) - 1))
                meas_batch = true_rdoas + noise   # (mc_per_point, M-1)

                x_ests, y_ests = [], []
                for k in range(mc_per_point):
                    xe, ye = _np_localize(meas_batch[k])
                    x_ests.append(xe); y_ests.append(ye)

                x_ests = np.array(x_ests)
                y_ests = np.array(y_ests)
                err_x  = x_ests - xt
                err_y  = y_ests - yt
                mse    = np.mean(err_x**2) + np.mean(err_y**2)
                emp_rms[j, i] = math.sqrt(max(mse, 0.0))

                if crlb_rms[j, i] > 0:
                    efficiency[j, i] = emp_rms[j, i] / crlb_rms[j, i]

            except Exception:
                pass

        if count % n_print == 0:
            print(f"    grid {count}/{grid_n*grid_n} done …")

    # ── Plot ──────────────────────────────────────────────────────────────────
    fig, axes = plt.subplots(1, 3, figsize=(16, 5.5))

    for ax, data, title, cmap, clim in zip(
        axes,
        [crlb_rms, emp_rms, efficiency],
        ["CRLB σ_pos [m]", "Empirical σ_pos [m]", "Sub-optimality  σ_emp / σ_CRLB"],
        ["viridis", "viridis", "RdYlGn_r"],
        [(0, np.nanpercentile(crlb_rms, 95)),
         (0, np.nanpercentile(emp_rms, 95)),
         (1.0, max(2.0, np.nanpercentile(efficiency, 95)))],  # type: ignore[arg-type]
    ):
        im = ax.imshow(
            data,
            extent=[x_min, x_max, y_min, y_max],
            origin="lower", aspect="equal",
            cmap=cmap, vmin=clim[0], vmax=clim[1],
        )
        fig.colorbar(im, ax=ax, fraction=0.046, pad=0.04)
        ax.scatter(RECEIVERS[:, 0], RECEIVERS[:, 1], c="white",
                   edgecolors="black", s=60, zorder=5, label="receivers")
        ax.scatter(RECEIVERS[0, 0], RECEIVERS[0, 1], c="white",
                   edgecolors="red", s=90, marker="D", zorder=6, label="ref")
        ax.scatter(X_TRUE, Y_TRUE, c="cyan", edgecolors="black",
                   s=100, marker="*", zorder=7, label="true pos")
        ax.set_title(title, fontsize=10)
        ax.set_xlabel("x [m]")
        ax.set_ylabel("y [m]")
    axes[0].legend(fontsize=7, loc="lower right")

    fig.suptitle(
        f"TDOA localizer performance map   ({len(RECEIVERS)} receivers,  σ_RDOA = {SIGMA_R} m)\n"
        f"Sub-optimality = 1 → algorithm achieves CRLB;  > 1 → excess error vs bound",
        fontsize=9,
    )
    fig.tight_layout()
    path = "examples/py/10_efficiency_heatmap.png"
    fig.savefig(path, dpi=130, bbox_inches="tight")
    plt.close(fig)
    print(f"  Saved → {path}")


def _plot_pairs(jd):
    print_section("Figure 4: Pair plot  (rdoa_0..2, x_est, y_est)")
    import matplotlib.pyplot as plt

    fig, _ = jd.plot_pairs(
        variables=["rdoa_0", "rdoa_1", "rdoa_2", "x_est", "y_est"],
        title="TDOA particle scatter: measurements → position estimates",
    )
    path = "examples/py/10_tdoa_pairs.png"
    fig.savefig(path, dpi=110, bbox_inches="tight")
    plt.close(fig)
    print(f"  Saved → {path}")


if __name__ == "__main__":
    main()
