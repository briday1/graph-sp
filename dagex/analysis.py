"""dagex.analysis — joint distribution analysis for StatResult particles.

Works with the output of ``dag.predict()``, which stores full
per-sample trajectories so that all variables at index ``i`` come from the
**same** random draw through the graph.

Typical usage::

    stat  = dag.predict({"x": dagex.normal(0.0, 1.0)}, n_samples=3000)
    joint = dagex.joint(stat)          # JointDistribution over all variables

    joint.print_summary()
    joint.correlation("x", "out")

    joint.plot_marginal("out")
    joint.plot_joint("x", "out")
    joint.plot_pairs()

    # Assert two variables are independent (shuffles one sample array)
    ji = joint.assume_independent("x", "out")
    ji.plot_joint("x", "out")
"""

from __future__ import annotations

import math
import random
from typing import Dict, List, Optional, Sequence, Tuple, Union


# ─── JointDistribution ────────────────────────────────────────────────────────

class JointDistribution:
    """Joint distribution over a set of co-sampled variables.

    Each variable's sample at index ``i`` comes from the same full-graph
    trajectory, so correlation structure across variables is preserved.

    Do not construct directly — use :func:`dagex.joint` (= ``joint_from_stat``)
    or pass a list of particle dicts.
    """

    def __init__(
        self,
        particles: List[Dict[str, float]],
        variables: Optional[List[str]] = None,
    ) -> None:
        self._particles = particles
        self._n = len(particles)

        if variables is None:
            all_keys: set = set()
            for p in particles:
                all_keys.update(k for k in p if not k.startswith("__"))
            self._variables = sorted(all_keys)
        else:
            self._variables = list(variables)

        # Cache aligned sample arrays (one list per variable)
        self._samples: Dict[str, List[float]] = {}
        for var in self._variables:
            self._samples[var] = [p[var] for p in particles if var in p]

        # iid group metadata: prefix → [element var names]
        # Populated by assume_iid(); affects marginal stats and KDE estimation.
        self._iid_groups: Dict[str, List[str]] = {}

    # ── Properties ────────────────────────────────────────────────────────────

    @property
    def variables(self) -> List[str]:
        """Ordered list of variable names."""
        return list(self._variables)

    @property
    def n_samples(self) -> int:
        """Number of particles (sample trajectories)."""
        return self._n

    # ── Sample access ─────────────────────────────────────────────────────────

    def samples(self, var: str) -> List[float]:
        """Return the raw aligned sample vector for *var*."""
        return self._samples.get(var, [])

    def _pooled(self, var: str) -> List[float]:
        """Return samples for *var*, pooling across iid siblings if applicable.

        When ``assume_iid(prefix)`` has been called and *var* is one of the
        tagged elements, all sibling element samples are concatenated.  This
        gives ``k × n_samples`` training points for KDE, dramatically improving
        density estimation for vector outputs with many iid elements.
        """
        for elements in self._iid_groups.values():
            if var in elements:
                pooled: List[float] = []
                for elem in elements:
                    pooled.extend(self._samples.get(elem, []))
                return pooled
        return self._samples.get(var, [])

    @property
    def iid_groups(self) -> Dict[str, List[str]]:
        """Currently registered iid groups: ``{prefix: [var names]}``."""
        return dict(self._iid_groups)

    # ── Scalar statistics ─────────────────────────────────────────────────────

    def mean(self, var: str) -> float:
        s = self._pooled(var)
        return sum(s) / len(s) if s else float("nan")

    def std(self, var: str) -> float:
        s = self._pooled(var)
        if len(s) < 2:
            return 0.0
        m = sum(s) / len(s)
        return math.sqrt(sum((x - m) ** 2 for x in s) / (len(s) - 1))

    def percentile(self, var: str, p: float) -> float:
        """Return the *p*-th percentile (0 ≤ p ≤ 1) for *var*."""
        s = sorted(self._pooled(var))
        if not s:
            return float("nan")
        idx = max(0, min(len(s) - 1, round(p * (len(s) - 1))))
        return s[idx]

    # ── Pairwise statistics ───────────────────────────────────────────────────

    def covariance(self, var1: str, var2: str) -> float:
        """Sample covariance between *var1* and *var2*."""
        x = self._samples.get(var1, [])
        y = self._samples.get(var2, [])
        n = min(len(x), len(y))
        if n < 2:
            return float("nan")
        mx, my = sum(x[:n]) / n, sum(y[:n]) / n
        return sum((x[i] - mx) * (y[i] - my) for i in range(n)) / (n - 1)

    def correlation(self, var1: str, var2: str) -> float:
        """Pearson correlation coefficient between *var1* and *var2*.

        Returns NaN when either variable is (near-)constant.
        """
        x = self._samples.get(var1, [])
        y = self._samples.get(var2, [])
        n = min(len(x), len(y))
        if n < 2:
            return float("nan")
        mx, my = sum(x[:n]) / n, sum(y[:n]) / n
        cov = sum((x[i] - mx) * (y[i] - my) for i in range(n)) / (n - 1)
        sx = math.sqrt(sum((x[i] - mx) ** 2 for i in range(n)) / (n - 1))
        sy = math.sqrt(sum((y[i] - my) ** 2 for i in range(n)) / (n - 1))
        if sx * sy < 1e-12:
            return float("nan")
        return cov / (sx * sy)

    def correlation_matrix(
        self, variables: Optional[List[str]] = None
    ) -> Tuple[List[str], List[List[float]]]:
        """Return ``(labels, matrix)`` where *matrix[i][j]* is the Pearson r
        between variables[i] and variables[j]."""
        vars_ = variables or self._variables
        mat = [
            [self.correlation(vi, vj) for vj in vars_]
            for vi in vars_
        ]
        return vars_, mat

    def covariance_matrix(
        self, variables: Optional[List[str]] = None
    ) -> Tuple[List[str], "numpy.ndarray"]:  # type: ignore[name-defined]
        """Return ``(labels, matrix)`` where *matrix* is a numpy ndarray with
        shape ``(n_vars, n_vars)`` containing sample covariances."""
        import numpy as np
        vars_ = variables or self._variables
        mat = np.array(
            [[self.covariance(vi, vj) for vj in vars_] for vi in vars_]
        )
        return vars_, mat

    # ── iid element grouping ────────────────────────────────────────────────

    def assume_iid(
        self,
        prefix: str,
        elements: Optional[List[str]] = None,
    ) -> "JointDistribution":
        """Return a new :class:`JointDistribution` where vector elements with
        *prefix* are treated as **iid** draws from the same 1-D distribution.

        Effect on estimation
        ~~~~~~~~~~~~~~~~~~~~
        When marginal statistics or KDE are requested for any element
        ``prefix[k]``, samples from **all** elements are pooled together,
        giving ``len(elements) × n_samples`` training points.  This can
        substantially reduce KDE noise when the vector is long.

        Cross-element correlations (via :meth:`correlation`) are unaffected —
        they still use the per-element aligned samples.

        Parameters
        ----------
        prefix:
            Base variable name, e.g. ``"weights"`` for elements
            ``"weights[0]"``, ``"weights[1]"``, …
        elements:
            Explicit list of element names.  Auto-detected from
            ``prefix[0]``, ``prefix[1]``, … when omitted.
        """
        if elements is None:
            elements = sorted(
                [v for v in self._variables if v.startswith(f"{prefix}[")],
                key=lambda v: int(v[len(prefix) + 1:-1]),
            )
        if not elements:
            raise ValueError(
                f"No variables found matching '{prefix}[*]'. "
                f"Available variables: {self._variables}"
            )
        new_jd = JointDistribution(self._particles, self._variables)
        new_jd._iid_groups = {**self._iid_groups, prefix: list(elements)}
        return new_jd

    # ── Conditional slicing ──────────────────────────────────────────────────

    def conditional(
        self,
        given: Dict[str, float],
        tolerance: Optional[Union[float, Dict[str, float]]] = None,
        min_particles: int = 30,
    ) -> "JointDistribution":
        """Slice the joint by conditioning on specific variable values.

        Keeps only particles where every variable in *given* is within
        *tolerance* of its target value.  The conditioned variables are
        excluded from the returned distribution (they are near-constant).

        Parameters
        ----------
        given:
            ``{var_name: target_value, …}``
        tolerance:
            Acceptable deviation from each target.  Can be:

            * ``None`` (default) — ``0.5 × std(var)`` per variable (keeps
              ~38 % of particles for a Gaussian marginal).
            * A single ``float`` applied uniformly to all conditioned vars.
            * A ``dict`` mapping variable names to individual tolerances.
        min_particles:
            Raise :class:`ValueError` if fewer particles pass the filter.
        """
        # Resolve per-variable tolerances
        tols: Dict[str, float] = {}
        for var in given:
            if isinstance(tolerance, dict):
                tols[var] = tolerance.get(var, 0.5 * self.std(var))
            elif tolerance is not None:
                tols[var] = float(tolerance)
            else:
                tols[var] = 0.5 * self.std(var)

        filtered = [
            p for p in self._particles
            if all(
                abs(p.get(var, float("nan")) - target) <= tols[var]
                for var, target in given.items()
            )
        ]

        if len(filtered) < min_particles:
            raise ValueError(
                f"Only {len(filtered)} particles remain after conditioning "
                f"(minimum {min_particles}).  Increase n_samples or widen "
                f"tolerance.  Tolerances used: "
                + ", ".join(f"{v}±{t:.4g}" for v, t in tols.items())
            )

        remaining_vars = [v for v in self._variables if v not in given]
        new_jd = JointDistribution(filtered, remaining_vars)
        new_jd._iid_groups = dict(self._iid_groups)
        return new_jd

    # ── Independence forcing ──────────────────────────────────────────────────

    def assume_independent(self, var1: str, var2: str) -> "JointDistribution":
        """Return a **new** JointDistribution where *var1* and *var2* are
        treated as statistically independent.

        Implemented by shuffling the sample array for *var2*, breaking its
        alignment with *var1* (and all other variables) while preserving its
        marginal distribution exactly.
        """
        new_samples = {k: list(v) for k, v in self._samples.items()}
        shuffled = list(new_samples[var2])
        random.shuffle(shuffled)
        new_samples[var2] = shuffled
        # Reconstruct particle dicts
        n = min(len(v) for v in new_samples.values())
        new_particles = [
            {var: new_samples[var][i] for var in self._variables if var in new_samples}
            for i in range(n)
        ]
        return JointDistribution(new_particles, self._variables)

    # ── Density estimation ────────────────────────────────────────────────────

    def marginal_pdf(
        self,
        var: str,
        x_range: Optional[Tuple[float, float]] = None,
        n_points: int = 200,
        bandwidth=None,
    ) -> Tuple["numpy.ndarray", "numpy.ndarray"]:  # type: ignore[name-defined]
        """KDE-estimated marginal PDF for *var*.

        Returns ``(x_grid, pdf_values)`` as numpy arrays.
        """
        import numpy as np
        from scipy.stats import gaussian_kde

        s = np.array(self._pooled(var), dtype=float)
        kde = gaussian_kde(s, bw_method=bandwidth)
        if x_range is None:
            span = s.max() - s.min() if s.std() > 0 else 1.0
            margin = span * 0.25
            x_range = (float(s.min() - margin), float(s.max() + margin))
        x = np.linspace(x_range[0], x_range[1], n_points)
        return x, kde(x)

    def joint_pdf(
        self,
        var1: str,
        var2: str,
        n_grid: int = 60,
        bandwidth=None,
    ) -> Tuple["numpy.ndarray", "numpy.ndarray", "numpy.ndarray"]:  # type: ignore[name-defined]
        """2-D KDE joint PDF for *(var1, var2)*.

        Returns ``(xx, yy, zz)`` meshgrid arrays suitable for ``ax.contour``.
        """
        import numpy as np
        from scipy.stats import gaussian_kde

        x = np.array(self._samples.get(var1, []), dtype=float)
        y = np.array(self._samples.get(var2, []), dtype=float)
        n = min(len(x), len(y))
        xy = np.vstack([x[:n], y[:n]])
        kde = gaussian_kde(xy, bw_method=bandwidth)

        def _range(a: "numpy.ndarray") -> Tuple[float, float]:
            span = a.max() - a.min()
            pad = span * 0.20 if span > 0 else 1.0
            return float(a.min() - pad), float(a.max() + pad)

        xi = np.linspace(*_range(x), n_grid)
        yi = np.linspace(*_range(y), n_grid)
        xx, yy = np.meshgrid(xi, yi)
        pos = np.vstack([xx.ravel(), yy.ravel()])
        zz = kde(pos).reshape(n_grid, n_grid)
        return xx, yy, zz

    # ── Plotting ──────────────────────────────────────────────────────────────

    def plot_marginal(
        self,
        var: str,
        ax=None,
        color: str = "steelblue",
        bins: int = 40,
        show_kde: bool = True,
        title: Optional[str] = None,
        **kwargs,
    ):
        """Histogram + KDE overlay for the marginal distribution of *var*.

        Returns ``(fig, ax)``.
        """
        import matplotlib.pyplot as plt
        import numpy as np

        if ax is None:
            fig, ax = plt.subplots(figsize=(7, 4))
        else:
            fig = ax.figure

        s = np.array(self._samples.get(var, []), dtype=float)
        ax.hist(s, bins=bins, density=True, alpha=0.40, color=color, label="histogram")
        if show_kde:
            try:
                x, y = self.marginal_pdf(var)
                ax.plot(x, y, color=color, linewidth=2.0, label="KDE")
            except Exception:
                pass
        ax.axvline(
            float(s.mean()),
            color="crimson",
            linestyle="--",
            linewidth=1.5,
            label=f"mean = {s.mean():.3f}",
        )
        ax.set_xlabel(var, fontsize=11)
        ax.set_ylabel("density", fontsize=11)
        ax.set_title(title or f"Marginal  —  {var}", fontsize=12)
        ax.legend(fontsize=9)
        fig.tight_layout()
        return fig, ax

    def plot_joint(
        self,
        var1: str,
        var2: str,
        ax=None,
        scatter_alpha: float = 0.25,
        scatter_color: str = "steelblue",
        contour_color: str = "navy",
        n_contours: int = 7,
        title: Optional[str] = None,
        **kwargs,
    ):
        """Scatter plot + KDE contour lines for the joint distribution of
        *(var1, var2)*.

        Returns ``(fig, ax)``.
        """
        import matplotlib.pyplot as plt
        import numpy as np

        if ax is None:
            fig, ax = plt.subplots(figsize=(6, 5))
        else:
            fig = ax.figure

        x = np.array(self._samples.get(var1, []), dtype=float)
        y_ = np.array(self._samples.get(var2, []), dtype=float)
        n = min(len(x), len(y_))

        ax.scatter(
            x[:n], y_[:n],
            alpha=scatter_alpha, s=6, c=scatter_color, label="samples",
            **kwargs,
        )
        try:
            xx, yy, zz = self.joint_pdf(var1, var2)
            ax.contour(xx, yy, zz, levels=n_contours, colors=contour_color, linewidths=1.0, alpha=0.75)
        except Exception:
            pass

        r = self.correlation(var1, var2)
        ax.set_xlabel(var1, fontsize=11)
        ax.set_ylabel(var2, fontsize=11)
        ax.set_title(title or f"{var1}  vs  {var2}   (r = {r:.3f})", fontsize=12)
        fig.tight_layout()
        return fig, ax

    def plot_pairs(
        self,
        variables: Optional[List[str]] = None,
        figsize: Optional[Tuple[float, float]] = None,
        title: Optional[str] = "Pair Plot",
    ):
        """Grid of pairwise plots.

        * **Diagonal**: marginal histogram + KDE
        * **Lower triangle**: scatter plot with KDE contours
        * **Upper triangle**: Pearson r value

        Returns ``(fig, axes)``.
        """
        import matplotlib.pyplot as plt
        import numpy as np

        vars_ = variables or self._variables
        nv = len(vars_)
        if nv < 2:
            raise ValueError("Need at least 2 variables for a pair plot.")

        if figsize is None:
            figsize = (2.8 * nv, 2.8 * nv)

        fig, axes = plt.subplots(nv, nv, figsize=figsize)
        if nv == 1:
            axes = [[axes]]

        cycle = plt.rcParams["axes.prop_cycle"].by_key()["color"]

        for i, vi in enumerate(vars_):
            for j, vj in enumerate(vars_):
                ax = axes[i][j]
                c = cycle[i % len(cycle)]

                if i == j:
                    # Diagonal — marginal
                    s = np.array(self._samples.get(vi, []), dtype=float)
                    ax.hist(s, bins=25, density=True, alpha=0.50, color=c)
                    try:
                        xg, yg = self.marginal_pdf(vi)
                        ax.plot(xg, yg, color=c, linewidth=1.5)
                    except Exception:
                        pass
                    ax.set_yticks([])

                elif i > j:
                    # Lower triangle — scatter
                    x = np.array(self._samples.get(vj, []), dtype=float)
                    y_ = np.array(self._samples.get(vi, []), dtype=float)
                    nn = min(len(x), len(y_))
                    ax.scatter(x[:nn], y_[:nn], alpha=0.20, s=3, c=c)
                    try:
                        xx, yy, zz = self.joint_pdf(vj, vi, n_grid=40)
                        ax.contour(xx, yy, zz, levels=5, colors="navy", linewidths=0.8, alpha=0.6)
                    except Exception:
                        pass

                else:
                    # Upper triangle — correlation text
                    r = self.correlation(vi, vj)
                    color_ = "navy" if abs(r) > 0.5 else ("darkorange" if abs(r) > 0.2 else "gray")
                    ax.text(
                        0.5, 0.5,
                        f"r = {r:.3f}",
                        ha="center", va="center",
                        fontsize=11, color=color_,
                        transform=ax.transAxes,
                    )
                    ax.set_xticks([])
                    ax.set_yticks([])

                # Axis labels on edges only
                if i == nv - 1:
                    ax.set_xlabel(vj, fontsize=9)
                else:
                    ax.set_xticklabels([])
                if j == 0 and i != j:
                    ax.set_ylabel(vi, fontsize=9)

        if title:
            fig.suptitle(title, fontsize=12, y=1.01)
        fig.tight_layout()
        return fig, axes

    # ── Console summary ───────────────────────────────────────────────────────

    def print_summary(self, variables: Optional[List[str]] = None) -> None:
        """Print a per-variable stats table followed by the correlation matrix."""
        vars_ = variables or self._variables

        # Build reverse map: var → iid prefix (for annotation)
        var_to_iid: Dict[str, str] = {}
        for prefix, elems in self._iid_groups.items():
            for e in elems:
                var_to_iid[e] = prefix

        COL = 12
        print(f"\n  {'Var':<{COL}}  {'Mean':>9}  {'Std':>9}  {'p5':>9}  {'p50':>9}  {'p95':>9}  {'Note'}")
        print("  " + "─" * (COL + 2 + 9 * 5 + 4 * 2 + 10))
        for v in vars_:
            note = ""
            if v in var_to_iid:
                prefix = var_to_iid[v]
                n_elem = len(self._iid_groups[prefix])
                note = f"iid '{prefix}' (pooled {n_elem}×)"
            print(
                f"  {v:<{COL}}  {self.mean(v):9.4f}  {self.std(v):9.4f}"
                f"  {self.percentile(v, 0.05):9.4f}"
                f"  {self.percentile(v, 0.50):9.4f}"
                f"  {self.percentile(v, 0.95):9.4f}"
                + (f"  {note}" if note else "")
            )

        print(f"\n  Correlation matrix  (n = {self._n} samples)")
        header = f"  {'':>{COL}}  " + "  ".join(f"{v:>7}" for v in vars_)
        print(header)
        print("  " + "─" * (len(header) - 2))
        for vi in vars_:
            row = f"  {vi:<{COL}}  " + "  ".join(
                f"{self.correlation(vi, vj):7.3f}" for vj in vars_
            )
            print(row)

    def __repr__(self) -> str:
        return f"JointDistribution(n={self._n}, vars={self._variables})"


# ─── Factory ──────────────────────────────────────────────────────────────────

def joint_from_stat(
    stat_result,
    variables: Optional[List[str]] = None,
) -> JointDistribution:
    """Create a :class:`JointDistribution` from the particles of a *StatResult*.

    Parameters
    ----------
    stat_result:
        The return value of ``dag.predict()``.  Must have
        ``stat_result.particles`` populated (not ``None``).
    variables:
        Subset of variable names to include.  Defaults to all non-internal
        variables (those not starting with ``__``).

    Raises
    ------
    ValueError
        If the result has no particles — use ``dag.predict()`` instead
        of ``dag.predict()``.
    """
    parts = stat_result.particles
    if parts is None:
        raise ValueError(
            "StatResult has no particles. "
            "Call dag.predict() to enable joint distribution analysis."
        )
    return JointDistribution(parts, variables)
