"""Quick smoke test for vector flattening, assume_iid, conditional, covariance_matrix."""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.dirname(__file__)))

import random
import numpy as np
import dagex
from dagex.analysis import JointDistribution

# ── 1. FloatVec flattening in predict ────────────────────────────────────────
g = dagex.Graph()
g.add(
    lambda _: {"weights": [1.0, 2.0, 3.0]},  # plain Python list → FloatVec
    label="vec_source",
    inputs=[],
    outputs=[("weights", "weights")],
)
dag = g.build()
stat = dag.predict({}, n_samples=200)
part = stat.particles[0]
assert "weights[0]" in part, f"missing weights[0], got: {sorted(part)}"
assert "weights[1]" in part
assert "weights[2]" in part
assert "__veclen__weights" in part
assert part["__veclen__weights"] == 3.0
print(f"  1. FloatVec flattening:  weights[0]={part['weights[0]']:.1f}  "
      f"weights[1]={part['weights[1]']:.1f}  weights[2]={part['weights[2]']:.1f}  ✓")

# ── 2. JointDistribution sees flattened variables ─────────────────────────────
jd = dagex.joint(stat)
assert "weights[0]" in jd.variables, f"not in variables: {jd.variables}"
assert "__veclen__weights" not in jd.variables, "veclen marker leaked into variables"
print(f"  2. jd.variables = {jd.variables}  ✓")

# ── 3. assume_iid — auto-detect elements and pool ────────────────────────────
# Use a pipeline where each element comes from a random Normal
g2 = dagex.Graph()
g2.add(
    lambda ctx: {"v": [ctx["x"] + random.gauss(0, 0.2) for _ in range(5)]},  # list → FloatVec
    label="noisy_vec",
    inputs=[("x", "x")],
    outputs=[("v", "v")],
)
dag2 = g2.build()
stat2 = dag2.predict({"x": dagex.normal(3.0, 1.0)}, n_samples=500)
jd2 = dagex.joint(stat2)
print(f"  3. vector pipeline vars: {[v for v in jd2.variables if v.startswith('v[')]}")

jd2_iid = jd2.assume_iid("v")
assert jd2_iid.iid_groups == {"v": ["v[0]", "v[1]", "v[2]", "v[3]", "v[4]"]}
# With iid pooling, marginal_pdf uses 5×500 = 2500 points instead of 500
x_grid, pdf_vals = jd2_iid.marginal_pdf("v[0]")
assert len(x_grid) == 200
# mean should be close to 3.0 for all elements
for k in range(5):
    m = jd2_iid.mean(f"v[{k}]")
    assert 2.5 < m < 3.5, f"pooled mean out of range: {m}"
print(f"  3. assume_iid pooled mean(v[0]) = {jd2_iid.mean('v[0]'):.4f}  ✓")
jd2_iid.print_summary(variables=[f"v[{k}]" for k in range(5)])

# ── 4. conditional ────────────────────────────────────────────────────────────
random.seed(42)
particles3 = [{"a": random.gauss(5.0, 2.0), "b": random.gauss(0.0, 1.0)} for _ in range(4000)]
# Add correlation: b = 0.5*a + noise
particles3 = [{"a": p["a"], "b": 0.5 * p["a"] + p["b"]} for p in particles3]
jd3 = JointDistribution(particles3)
print(f"\n  4. Full correlation r(a,b) = {jd3.correlation('a','b'):.3f}")

# Condition on a ≈ 5
jd_cond = jd3.conditional({"a": 5.0})
print(f"     conditional on a≈5: mean(b) = {jd_cond.mean('b'):.4f}  (expect ~2.5)")
assert abs(jd_cond.mean("b") - 2.5) < 0.5, f"conditional mean(b) off: {jd_cond.mean('b')}"
assert "a" not in jd_cond.variables, "conditioned var should be removed"
print(f"     remaining vars: {jd_cond.variables}  ✓")

# ── 5. covariance_matrix returns numpy array ──────────────────────────────────
labels, cov = jd3.covariance_matrix()
assert isinstance(cov, np.ndarray)
assert cov.shape == (2, 2)
assert cov[0, 0] > 0  # Var(a)
print(f"\n  5. covariance_matrix shape={cov.shape}  Var(a)={cov[0,0]:.3f}  Cov(a,b)={cov[0,1]:.3f}  ✓")

print("\n  ══════════════════════════════════════════════")
print("  All smoke tests passed  ✓")
