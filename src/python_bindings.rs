//! Python bindings for graph-sp
//!
//! This module provides PyO3 bindings to expose the Rust graph executor to Python.
//! It is gated behind the "python" feature flag.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
#[cfg(feature = "radar_examples")]
use pyo3::types::PyComplex;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;
use std::sync::Arc;

use crate::builder::Graph;
use crate::dag::{Dag, PredictTarget};
use crate::distribution::{DistContext, Distribution};
use crate::graph_data::GraphData;
use crate::stat_result::StatResult;

// ─── PyDistribution ─────────────────────────────────────────────────────

/// Python-accessible wrapper around the Rust `Distribution` enum.
#[pyclass(name = "Distribution")]
#[derive(Clone)]
struct PyDistribution {
    inner: Distribution,
}

#[pymethods]
impl PyDistribution {
    /// E[X]
    #[getter]
    fn mean(&self) -> f64 {
        self.inner.mean()
    }

    /// Standard deviation
    #[getter]
    fn std(&self) -> f64 {
        self.inner.std()
    }

    /// Variance
    #[getter]
    fn variance(&self) -> f64 {
        self.inner.variance()
    }

    /// 5th percentile
    #[getter]
    fn p5(&self) -> f64 {
        self.inner.percentile(0.05)
    }

    /// Median
    #[getter]
    fn p50(&self) -> f64 {
        self.inner.percentile(0.50)
    }

    /// 95th percentile
    #[getter]
    fn p95(&self) -> f64 {
        self.inner.percentile(0.95)
    }

    /// Compute an arbitrary percentile (0.0 – 1.0).
    fn percentile(&self, p: f64) -> f64 {
        self.inner.percentile(p)
    }

    /// Raw samples (only non-None for Empirical distributions).
    #[getter]
    fn samples(&self, py: Python) -> PyObject {
        match &self.inner {
            Distribution::Empirical { samples } => {
                let v: Vec<f64> = samples.as_ref().clone();
                v.to_object(py)
            }
            _ => py.None(),
        }
    }

    /// Draw `n` samples from this distribution.
    fn sample_n(&self, n: usize) -> Vec<f64> {
        self.inner.sample_n(n)
    }

    /// One-line summary string.
    fn summary(&self) -> String {
        format!("{}", self.inner.summary())
    }

    fn __repr__(&self) -> String {
        format!("{}", self.inner)
    }
}

// ─── PyStatResult ──────────────────────────────────────────────────────

/// Result of `Dag.predict()`.  Dict-like access returns `Distribution` objects.
#[pyclass(name = "StatResult")]
struct PyStatResult {
    inner: StatResult,
}

#[pymethods]
impl PyStatResult {
    /// `result["var"]` → `Distribution`.
    fn __getitem__(&self, py: Python, key: &str) -> PyResult<PyObject> {
        match self.inner.get(key) {
            Some(dist) => Ok(PyDistribution { inner: dist.clone() }.into_py(py)),
            None => Err(PyValueError::new_err(format!(
                "Variable '{}' not found in StatResult",
                key
            ))),
        }
    }

    /// `result.get("var")` → `Distribution | None`.
    fn get(&self, py: Python, key: &str) -> PyObject {
        match self.inner.get(key) {
            Some(dist) => PyDistribution { inner: dist.clone() }.into_py(py),
            None => py.None(),
        }
    }

    /// Return a `{var: Distribution}` dict for a specific branch.
    fn for_branch(&self, py: Python, branch_id: usize) -> PyObject {
        dist_context_to_py_dict(py, self.inner.for_branch(branch_id))
    }

    /// Return a `{var: Distribution}` dict for a specific variant (0-based index).
    fn for_variant(&self, py: Python, variant_idx: usize) -> PyObject {
        dist_context_to_py_dict(py, self.inner.for_variant(variant_idx))
    }

    /// List broadcast variable names (excluding internal `__branch__` prefix keys).
    fn keys(&self, py: Python) -> PyObject {
        let mut ks: Vec<&str> = self
            .inner
            .dist_context
            .keys()
            .filter(|k| !k.starts_with("__branch_"))
            .map(|k| k.as_str())
            .collect();
        ks.sort();
        ks.to_object(py)
    }

    /// Print a human-readable summary to stdout.
    fn print_summary(&self) {
        self.inner.print_summary();
    }

    fn __repr__(&self) -> String {
        let keys: Vec<&str> = self
            .inner
            .dist_context
            .keys()
            .filter(|k| !k.starts_with("__branch_"))
            .map(|k| k.as_str())
            .collect();
        format!("StatResult(vars={:?})", keys)
    }

    /// Aligned per-sample trajectories, or `None` when the result came from `predict()`
    /// rather than `predict_particles()`.
    ///
    /// Returns a list of dicts: `particles[i]` maps every broadcast variable name to its
    /// concrete float value on sample `i`.  All variables in one dict share the same
    /// random seed, preserving the joint cross-variable distribution structure.
    #[getter]
    fn particles(&self, py: Python) -> PyObject {
        match &self.inner.particles {
            None => py.None(),
            Some(parts) => {
                let py_list = pyo3::types::PyList::empty(py);
                for particle in parts {
                    let d = PyDict::new(py);
                    for (k, v) in particle {
                        let _ = d.set_item(k, v);
                    }
                    let _ = py_list.append(d);
                }
                py_list.to_object(py)
            }
        }
    }
}

/// Convert an optional `&DistContext` into a Python dict of `{str: PyDistribution}`.
fn dist_context_to_py_dict(py: Python, ctx: Option<&DistContext>) -> PyObject {
    let dict = PyDict::new(py);
    if let Some(c) = ctx {
        for (k, v) in c {
            let _ = dict.set_item(k, PyDistribution { inner: v.clone() }.into_py(py));
        }
    }
    dict.to_object(py)
}

// ─── Python wrapper for Graph builder ─────────────────────────────────────

/// Python wrapper for Graph builder
#[pyclass(name = "Graph")]
struct PyGraph {
    graph: Option<Graph>,
}

#[pymethods]
impl PyGraph {
    /// Create a new graph builder
    #[new]
    fn new() -> Self {
        PyGraph {
            graph: Some(Graph::new()),
        }
    }

    /// Add a node to the graph
    ///
    /// Args:
    ///     function: Optional Python callable. If None, creates a no-op node.
    ///     label: Optional string label for the node
    ///     inputs: Optional list of (broadcast_var, impl_var) tuples or dict
    ///     outputs: Optional list of (impl_var, broadcast_var) tuples or dict
    ///
    /// Returns:
    ///     Self for method chaining
    #[pyo3(signature = (function=None, label=None, inputs=None, outputs=None))]
    fn add(
        &mut self,
        function: Option<PyObject>,
        label: Option<String>,
        inputs: Option<&PyAny>,
        outputs: Option<&PyAny>,
    ) -> PyResult<()> {
        let graph = self
            .graph
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("Graph has already been built or consumed"))?;

        // Parse inputs
        let input_vec = if let Some(inp) = inputs {
            parse_mapping(inp)?
        } else {
            Vec::new()
        };

        // Parse outputs
        let output_vec = if let Some(out) = outputs {
            parse_mapping(out)?
        } else {
            Vec::new()
        };

        // Convert to references for the add method
        let input_refs: Vec<(&str, &str)> = input_vec
            .iter()
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect();
        let output_refs: Vec<(&str, &str)> = output_vec
            .iter()
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect();

        // Create the node function
        if let Some(py_func) = function {
            // Wrap Python callable in a Rust closure - graph.add will handle Arc wrapping
            let rust_function = create_python_node_function(py_func);

            graph.add(
                rust_function,
                label.as_deref(),
                if input_refs.is_empty() {
                    None
                } else {
                    Some(input_refs)
                },
                if output_refs.is_empty() {
                    None
                } else {
                    Some(output_refs)
                },
            );
        } else {
            // No-op function if None provided - graph.add will handle Arc wrapping
            let noop = |_: &HashMap<String, GraphData>| HashMap::new();
            graph.add(
                noop,
                label.as_deref(),
                if input_refs.is_empty() {
                    None
                } else {
                    Some(input_refs)
                },
                if output_refs.is_empty() {
                    None
                } else {
                    Some(output_refs)
                },
            );
        }

        Ok(())
    }

    /// Create a branch in the graph
    ///
    /// Args:
    ///     subgraph: PyGraph instance representing the branch
    ///
    /// Returns:
    ///     Branch ID (usize)
    fn branch(&mut self, mut subgraph: PyRefMut<PyGraph>) -> PyResult<usize> {
        let graph = self
            .graph
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("Graph has already been built or consumed"))?;

        let subgraph_inner = subgraph
            .graph
            .take()
            .ok_or_else(|| PyValueError::new_err("Subgraph has already been built or consumed"))?;

        Ok(graph.branch(subgraph_inner))
    }

    /// Create variant nodes (parameter sweep)
    ///
    /// Args:
    ///     functions: List of Python callables, each with signature (inputs, variant_params) -> dict
    ///     label: Optional string label for the variant nodes
    ///     inputs: Optional list of (broadcast_var, impl_var) tuples or dict
    ///     outputs: Optional list of (impl_var, broadcast_var) tuples or dict
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     factors = np.linspace(0.5, 2.0, 5)
    ///     graph.variants(
    ///         [lambda inputs, params, f=f: {"scaled": inputs["x"] * f} for f in factors],
    ///         "Scale",
    ///         [("data", "x")],
    ///         [("scaled", "result")]
    ///     )
    #[pyo3(signature = (functions, label=None, inputs=None, outputs=None))]
    fn variants(
        &mut self,
        functions: Vec<PyObject>,
        label: Option<String>,
        inputs: Option<&PyAny>,
        outputs: Option<&PyAny>,
    ) -> PyResult<()> {
        let graph = self
            .graph
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("Graph has already been built or consumed"))?;

        // Parse inputs
        let input_vec = if let Some(inp) = inputs {
            parse_mapping(inp)?
        } else {
            Vec::new()
        };

        // Parse outputs
        let output_vec = if let Some(out) = outputs {
            parse_mapping(out)?
        } else {
            Vec::new()
        };

        // Convert to references for the variant method
        let input_refs: Vec<(&str, &str)> = input_vec
            .iter()
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect();
        let output_refs: Vec<(&str, &str)> = output_vec
            .iter()
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect();

        // Convert Python functions to Rust closures (Arc wrapping is now automatic in variants())
        let rust_functions: Vec<_> = functions
            .iter()
            .map(|func| create_python_node_function(func.clone()))
            .collect();

        // Call variants with the vector of closures
        graph.variants(
            rust_functions,
            label.as_deref(),
            if input_refs.is_empty() {
                None
            } else {
                Some(input_refs)
            },
            if output_refs.is_empty() {
                None
            } else {
                Some(output_refs)
            },
        );

        Ok(())
    }

    /// Build the DAG from the graph
    ///
    /// Returns:
    ///     PyDag instance ready for execution
    fn build(&mut self) -> PyResult<PyDag> {
        let graph = self
            .graph
            .take()
            .ok_or_else(|| PyValueError::new_err("Graph has already been built"))?;

        Ok(PyDag { dag: graph.build() })
    }

    /// Attach an analytical distribution transfer to all nodes with the given label.
    ///
    /// The `transfer_fn` must be a Python callable with signature:
    ///   `transfer_fn(input_dists: dict[str, Distribution]) -> dict[str, Distribution] | None`
    ///
    /// Keys are **impl_var** names (the same names the node function receives).
    /// Return `None` to signal that Monte Carlo fallback should be used for this node.
    ///
    /// Example::
    ///
    ///     def scale_dist(dists):
    ///         x = dists["x"]  # Distribution object
    ///         return {"y": dagex.normal(x.mean * 2.0, x.std * 2.0)}
    ///
    ///     graph.set_dist_transfer("Scale", scale_dist)
    fn set_dist_transfer(&mut self, label: String, transfer_fn: PyObject) -> PyResult<()> {
        let graph = self
            .graph
            .as_mut()
            .ok_or_else(|| PyValueError::new_err("Graph has already been built or consumed"))?;

        let rust_transfer = create_python_dist_transfer(transfer_fn);
        graph.set_dist_transfer_for(&label, Arc::new(rust_transfer));
        Ok(())
    }
}

/// Python wrapper for DAG executor
#[pyclass(name = "Dag")]
struct PyDag {
    dag: Dag,
}

#[pymethods]
impl PyDag {
    /// Execute the DAG
    ///
    /// Args:
    ///     parallel (bool): If True, execute nodes at the same level concurrently. Default: False
    ///     max_threads (Optional[int]): Maximum number of threads to use per level. None = unlimited. Default: None
    ///
    /// Returns:
    ///     Dictionary containing the execution context
    #[pyo3(signature = (parallel=false, max_threads=None))]
    fn execute(
        &self,
        py: Python,
        parallel: bool,
        max_threads: Option<usize>,
    ) -> PyResult<PyObject> {
        // Release GIL during Rust execution
        let context = py.allow_threads(|| self.dag.execute(parallel, max_threads));

        // Convert HashMap<String, GraphData> to Python dict
        let py_dict = PyDict::new(py);
        for (key, value) in context.iter() {
            py_dict.set_item(key, graph_data_to_python(py, value))?;
        }
        Ok(py_dict.to_object(py))
    }

    /// Get Mermaid diagram representation
    ///
    /// Returns:
    ///     String containing the Mermaid diagram
    fn to_mermaid(&self) -> String {
        self.dag.to_mermaid()
    }

    /// Get the number of nodes in the DAG
    ///
    /// Returns:
    ///     Number of nodes
    fn node_count(&self) -> usize {
        self.dag.nodes().len()
    }

    /// Return a sorted list of unique node labels present in the DAG.
    ///
    /// Useful for discovering valid values to pass as `at_node` to `predict()`.
    ///
    /// Example::
    ///
    ///     dag.node_labels()   # ["AddNoise", "Scale", "Source"]
    fn node_labels(&self, py: Python) -> PyObject {
        let mut labels: Vec<&str> = self
            .dag
            .nodes()
            .iter()
            .filter_map(|n| n.label.as_deref())
            .collect();
        labels.sort();
        labels.dedup();
        labels.to_object(py)
    }

    /// Return a sorted list of unique branch IDs present in the DAG.
    fn branch_ids(&self, py: Python) -> PyObject {
        let mut ids: Vec<usize> = self
            .dag
            .nodes()
            .iter()
            .filter_map(|n| n.branch_id)
            .collect();
        ids.sort();
        ids.dedup();
        ids.to_object(py)
    }

    /// Return a sorted list of unique variant indices present in the DAG.
    fn variant_indices(&self, py: Python) -> PyObject {
        let mut idxs: Vec<usize> = self
            .dag
            .nodes()
            .iter()
            .filter_map(|n| n.variant_index)
            .collect();
        idxs.sort();
        idxs.dedup();
        idxs.to_object(py)
    }

    /// Forward-propagate distributions through the graph.
    ///
    /// Args:
    ///     inputs (dict[str, Distribution]): Initial distributions keyed by broadcast variable name.
    ///     n_samples (int): Monte Carlo sample count for nodes without an analytical dist_transfer.
    ///         Default: 1000.
    ///     at_node (str | None): Stop after all nodes with this label have been computed.  Pass
    ///         ``None`` (default) to run the full graph.  Use :meth:`node_labels` to list valid
    ///         values.
    ///     at_branch (int | None): Stop after all nodes belonging to this branch ID have been
    ///         computed.  Use :meth:`branch_ids` to list valid values.
    ///     at_variant (int | None): Stop after all nodes with this variant index have been
    ///         computed.  Use :meth:`variant_indices` to list valid values.
    ///
    /// Only one of ``at_node``, ``at_branch``, ``at_variant`` should be given at a time.
    /// If more than one is supplied, ``at_node`` takes precedence, then ``at_branch``,
    /// then ``at_variant``.
    ///
    /// Returns:
    ///     StatResult: distribution over every broadcast variable computed up to the target.
    ///
    /// Example::
    ///
    ///     # Full graph
    ///     stat = dag.predict({"x": dagex.normal(0.0, 1.0)})
    ///
    ///     # Stop at node labelled "Scale"
    ///     stat = dag.predict({"x": dagex.normal(0.0, 1.0)}, at_node="Scale")
    ///
    ///     # Stop at branch 1
    ///     stat = dag.predict({"x": dagex.normal(0.0, 1.0)}, at_branch=1)
    ///
    ///     # Stop at variant 0
    ///     stat = dag.predict({"x": dagex.normal(0.0, 1.0)}, at_variant=0)
    #[pyo3(signature = (inputs, n_samples=1000, at_node=None, at_branch=None, at_variant=None))]
    fn predict(
        &self,
        py: Python,
        inputs: &PyDict,
        n_samples: usize,
        at_node: Option<String>,
        at_branch: Option<usize>,
        at_variant: Option<usize>,
    ) -> PyResult<PyStatResult> {
        // Convert Python dict -> DistContext
        let mut dist_ctx: DistContext = HashMap::new();
        for (key, val) in inputs.iter() {
            let k: String = key.extract()?;
            let cell = val
                .downcast::<PyCell<PyDistribution>>()
                .map_err(|_| PyValueError::new_err(format!(
                    "Value for key '{}' must be a Distribution (use dagex.normal(), dagex.gamma(), etc.)",
                    k
                )))?;
            dist_ctx.insert(k, cell.borrow().inner.clone());
        }

        // Build target (at_node > at_branch > at_variant)
        let target: Option<PredictTarget> = if let Some(label) = at_node {
            Some(PredictTarget::NodeLabel(label))
        } else if let Some(bid) = at_branch {
            Some(PredictTarget::BranchId(bid))
        } else if let Some(vi) = at_variant {
            Some(PredictTarget::VariantIndex(vi))
        } else {
            None
        };

        let stat = py.allow_threads(|| {
            self.dag.predict_at(dist_ctx, n_samples, target.as_ref())
        });
        Ok(PyStatResult { inner: stat })
    }

    /// Particle-based forward pass — preserves exact joint distribution structure.
    ///
    /// Runs `n_samples` full end-to-end trajectories through the graph so that
    /// ``stat.particles[i]`` contains the value of every variable from the **same**
    /// random draw.  Unlike ``predict()``, which runs per-node Monte Carlo (losing
    /// sample alignment), this method evaluates the node's concrete function for every
    /// particle — even when a ``dist_transfer`` is attached.  This gives you the exact
    /// joint (cross-variable) correlation structure.
    ///
    /// **Trade-off vs predict()**
    ///
    /// * ``predict()`` — fast; uses ``dist_transfer`` shortcuts; marginals are exact;
    ///   no joint structure.
    /// * ``predict_particles()`` — slightly slower; always evaluates node functions;
    ///   marginals AND correlations are exact; use this for joint/marginal PDFs and plots.
    ///
    /// Args:
    ///     inputs (dict[str, Distribution]): Prior distributions keyed by variable name.
    ///     n_samples (int): Number of full-graph trajectories.  Default: 1000.
    ///
    /// Returns:
    ///     StatResult: Marginal distributions in ``stat_result[var]`` (same as predict()),
    ///     plus ``stat_result.particles`` (list of dicts) for joint analysis.
    ///
    /// Example::
    ///
    ///     stat  = dag.predict_particles({"x": dagex.normal(0.0, 1.0)}, n_samples=2000)
    ///     joint = dagex.joint(stat)          # JointDistribution
    ///     joint.print_summary()              # table + correlation matrix
    ///     joint.plot_pairs()                 # pair plot of all variables
    ///     joint.plot_joint("x", "out")       # 2-D joint PDF
    #[pyo3(signature = (inputs, n_samples=1000))]
    fn predict_particles(
        &self,
        py: Python,
        inputs: &PyDict,
        n_samples: usize,
    ) -> PyResult<PyStatResult> {
        let mut dist_ctx: DistContext = HashMap::new();
        for (key, val) in inputs.iter() {
            let k: String = key.extract()?;
            let cell = val
                .downcast::<PyCell<PyDistribution>>()
                .map_err(|_| PyValueError::new_err(format!(
                    "Value for key '{}' must be a Distribution",
                    k
                )))?;
            dist_ctx.insert(k, cell.borrow().inner.clone());
        }
        let stat = py.allow_threads(|| {
            self.dag.predict_particles(dist_ctx, n_samples)
        });
        Ok(PyStatResult { inner: stat })
    }
}

/// Parse mapping from Python types (list of tuples or dict) to Vec<(String, String)>
fn parse_mapping(obj: &PyAny) -> PyResult<Vec<(String, String)>> {
    if let Ok(dict) = obj.downcast::<PyDict>() {
        // Dict: {"key": "value"}
        let mut result = Vec::new();
        for (key, value) in dict.iter() {
            let k: String = key.extract()?;
            let v: String = value.extract()?;
            result.push((k, v));
        }
        Ok(result)
    } else if let Ok(list) = obj.downcast::<PyList>() {
        // List of tuples: [("key", "value")]
        let mut result = Vec::new();
        for item in list.iter() {
            let tuple: (String, String) = item.extract()?;
            result.push(tuple);
        }
        Ok(result)
    } else {
        Err(PyValueError::new_err(
            "inputs/outputs must be a dict or list of tuples",
        ))
    }
}

/// Create a node function that wraps a Python callable
///
/// The returned closure is Send + Sync and properly handles GIL acquisition
/// when calling the Python function.
fn create_python_node_function(
    py_func: PyObject,
) -> impl Fn(&HashMap<String, GraphData>) -> HashMap<String, GraphData>
       + Send
       + Sync
       + 'static {
    // Wrap in Arc to make it cloneable and shareable
    let py_func = Arc::new(py_func);

    move |inputs: &HashMap<String, GraphData>| {
        // Acquire GIL only for the duration of this call
        Python::with_gil(|py| {
            // Convert inputs to Python dict
            let py_inputs = PyDict::new(py);
            for (key, value) in inputs.iter() {
                if let Err(e) = py_inputs.set_item(key, graph_data_to_python(py, value)) {
                    // Log to Python's stderr for better integration
                    let _ = py
                        .import("sys")
                        .and_then(|sys| sys.getattr("stderr"))
                        .and_then(|stderr| {
                            stderr.call_method1(
                                "write",
                                (format!("Error setting input '{}': {}\n", key, e),),
                            )
                        });
                    return HashMap::new();
                }
            }

            // Call the Python function with just inputs
            let result = py_func.call1(py, (py_inputs,));

            match result {
                Ok(py_result) => {
                    // Convert result back to HashMap
                    if let Ok(result_dict) = py_result.downcast::<PyDict>(py) {
                        let mut output = HashMap::new();
                        for (key, value) in result_dict.iter() {
                            if let Ok(k) = key.extract::<String>() {
                                output.insert(k, python_to_graph_data(value));
                            }
                        }
                        output
                    } else {
                        let _ = py
                            .import("sys")
                            .and_then(|sys| sys.getattr("stderr"))
                            .and_then(|stderr| {
                                stderr.call_method1(
                                    "write",
                                    ("Error: Python function did not return a dict\n",),
                                )
                            });
                        HashMap::new()
                    }
                }
                Err(e) => {
                    // Use Python's traceback printing for better error visibility
                    e.print(py);
                    HashMap::new()
                }
            }
        })
    }
}

/// Convert GraphData to Python object
fn graph_data_to_python(py: Python, data: &GraphData) -> PyObject {
    match data {
        GraphData::Int(v) => v.to_object(py),
        GraphData::Float(v) => v.to_object(py),
        GraphData::String(s) => s.to_object(py),
        GraphData::FloatVec(v) => v.to_object(py),
        GraphData::IntVec(v) => v.to_object(py),
        GraphData::Map(m) => {
            // Check if this is a complex array structure (keys are indices, values have "re" and "im")
            let mut is_complex_array = true;
            let mut max_idx = 0;
            for (k, v) in m.iter() {
                if let Ok(idx) = k.parse::<usize>() {
                    if idx > max_idx {
                        max_idx = idx;
                    }
                    // Check if value is a map with "re" and "im"
                    if let Some(inner_map) = v.as_map() {
                        if !inner_map.contains_key("re") || !inner_map.contains_key("im") {
                            is_complex_array = false;
                            break;
                        }
                    } else {
                        is_complex_array = false;
                        break;
                    }
                } else {
                    is_complex_array = false;
                    break;
                }
            }

            // Convert complex array structure back to list of tuples
            if is_complex_array && !m.is_empty() && m.len() == max_idx + 1 {
                let list = PyList::empty(py);
                for i in 0..m.len() {
                    if let Some(v) = m.get(&i.to_string()) {
                        if let Some(inner_map) = v.as_map() {
                            let re = inner_map
                                .get("re")
                                .and_then(|d| d.as_float())
                                .unwrap_or(0.0);
                            let im = inner_map
                                .get("im")
                                .and_then(|d| d.as_float())
                                .unwrap_or(0.0);
                            let _ = list.append((re, im).to_object(py));
                        }
                    }
                }
                return list.to_object(py);
            }

            // Check if all keys are numeric indices (0, 1, 2, ...)
            let mut is_list = true;
            let mut max_idx = 0;
            for k in m.keys() {
                if let Ok(idx) = k.parse::<usize>() {
                    if idx > max_idx {
                        max_idx = idx;
                    }
                } else {
                    is_list = false;
                    break;
                }
            }

            // If it looks like a list (sequential numeric keys), convert to list
            if is_list && !m.is_empty() && m.len() == max_idx + 1 {
                let list = PyList::empty(py);
                for i in 0..m.len() {
                    if let Some(v) = m.get(&i.to_string()) {
                        let _ = list.append(graph_data_to_python(py, v));
                    }
                }
                list.to_object(py)
            } else {
                // Otherwise, keep as dict
                let dict = PyDict::new(py);
                for (k, v) in m.iter() {
                    let _ = dict.set_item(k, graph_data_to_python(py, v));
                }
                dict.to_object(py)
            }
        }
        GraphData::None => py.None(),
        #[cfg(feature = "python")]
        GraphData::PyObject(obj) => {
            // Return the stored Python object directly without conversion
            obj.clone_ref(py)
        }
        #[cfg(feature = "radar_examples")]
        GraphData::Complex(c) => {
            // Convert to Python complex number (not tuple)
            PyComplex::from_doubles(py, c.re, c.im).to_object(py)
        }
        #[cfg(feature = "radar_examples")]
        GraphData::FloatArray(a) => {
            // Convert ndarray to Python list
            a.to_vec().to_object(py)
        }
        #[cfg(feature = "radar_examples")]
        GraphData::ComplexArray(a) => {
            // Convert complex array to list of Python complex numbers
            let list = PyList::empty(py);
            for c in a.iter() {
                let py_complex = PyComplex::from_doubles(py, c.re, c.im);
                let _ = list.append(py_complex);
            }
            list.to_object(py)
        }
    }
}

/// Convert Python object to GraphData
fn python_to_graph_data(obj: &PyAny) -> GraphData {
    // Try numeric scalars first
    if let Ok(f) = obj.extract::<f64>() {
        return GraphData::Float(f);
    }
    if let Ok(i) = obj.extract::<i64>() {
        return GraphData::Int(i);
    }
    if let Ok(s) = obj.extract::<String>() {
        return GraphData::String(s);
    }
    // Try list-of-floats → FloatVec, list-of-ints → IntVec
    if let Ok(list) = obj.extract::<Vec<f64>>() {
        return GraphData::FloatVec(std::sync::Arc::new(list));
    }
    if let Ok(list) = obj.extract::<Vec<i64>>() {
        return GraphData::IntVec(std::sync::Arc::new(list));
    }
    // Fall back to opaque PyObject for anything else (dicts, custom types, etc.)
    GraphData::PyObject(obj.to_object(obj.py()))
}

/// Create a dist_transfer closure that wraps a Python callable.
///
/// The Python function receives a dict of `{impl_var: Distribution}` and should
/// return a dict of `{impl_var: Distribution}`, or `None` to signal MC fallback.
fn create_python_dist_transfer(
    py_func: PyObject,
) -> impl Fn(&DistContext) -> Option<DistContext> + Send + Sync + 'static {
    let py_func = Arc::new(py_func);

    move |input_dists: &DistContext| -> Option<DistContext> {
        Python::with_gil(|py| {
            // Build input dict of {str: PyDistribution}
            let py_dict = PyDict::new(py);
            for (key, dist) in input_dists {
                let d = PyDistribution { inner: dist.clone() };
                py_dict.set_item(key, d.into_py(py)).ok()?;
            }

            // Call the Python callable
            let result = py_func.call1(py, (py_dict,)).ok()?;

            // None return → MC fallback
            if result.is_none(py) {
                return None;
            }

            // Convert returned dict back to DistContext
            let result_dict = result.downcast::<PyDict>(py).ok()?;
            let mut output: DistContext = HashMap::new();
            for (key, val) in result_dict.iter() {
                let k: String = key.extract().ok()?;
                if let Ok(cell) = val.downcast::<PyCell<PyDistribution>>() {
                    output.insert(k, cell.borrow().inner.clone());
                }
            }

            if output.is_empty() {
                None
            } else {
                Some(output)
            }
        })
    }
}

// ─── Module-level Distribution constructor functions ────────────────────────

/// Create a Normal (Gaussian) distribution: N(mean, std).
#[pyfunction]
#[pyo3(signature = (mean, std))]
fn normal(mean: f64, std: f64) -> PyDistribution {
    PyDistribution {
        inner: Distribution::normal(mean, std),
    }
}

/// Create a Uniform distribution: U(low, high).
#[pyfunction]
#[pyo3(signature = (low, high))]
fn uniform(low: f64, high: f64) -> PyDistribution {
    PyDistribution {
        inner: Distribution::uniform(low, high),
    }
}

/// Create a Beta distribution: Beta(α, β).  Support [0, 1].
#[pyfunction]
#[pyo3(signature = (alpha, beta))]
fn beta(alpha: f64, beta: f64) -> PyDistribution {
    PyDistribution {
        inner: Distribution::beta(alpha, beta),
    }
}

/// Create a Gamma distribution: Γ(shape, rate).  Mean = shape / rate.
#[pyfunction]
#[pyo3(signature = (shape, rate))]
fn gamma(shape: f64, rate: f64) -> PyDistribution {
    PyDistribution {
        inner: Distribution::gamma(shape, rate),
    }
}

/// Create a Log-Normal distribution.  `mu` and `sigma` are the underlying Normal parameters.
#[pyfunction]
#[pyo3(signature = (mu, sigma))]
fn lognormal(mu: f64, sigma: f64) -> PyDistribution {
    PyDistribution {
        inner: Distribution::lognormal(mu, sigma),
    }
}

/// Create a Deterministic point mass at `value`.
#[pyfunction]
#[pyo3(signature = (value))]
fn deterministic(value: f64) -> PyDistribution {
    PyDistribution {
        inner: Distribution::deterministic(value),
    }
}

/// Create an Empirical distribution from a list of float samples.
#[pyfunction]
#[pyo3(signature = (samples))]
fn empirical(samples: Vec<f64>) -> PyDistribution {
    PyDistribution {
        inner: Distribution::empirical(samples),
    }
}

/// Initialize the Python module
#[pymodule]
fn dagex(_py: Python, m: &PyModule) -> PyResult<()> {
    // PyO3 0.18.3 with auto-initialize feature handles multi-threading initialization automatically
    m.add_class::<PyGraph>()?;
    m.add_class::<PyDag>()?;
    m.add_class::<PyDistribution>()?;
    m.add_class::<PyStatResult>()?;
    // Distribution constructor functions
    m.add_function(wrap_pyfunction!(normal, m)?)?;
    m.add_function(wrap_pyfunction!(uniform, m)?)?;
    m.add_function(wrap_pyfunction!(beta, m)?)?;
    m.add_function(wrap_pyfunction!(gamma, m)?)?;
    m.add_function(wrap_pyfunction!(lognormal, m)?)?;
    m.add_function(wrap_pyfunction!(deterministic, m)?)?;
    m.add_function(wrap_pyfunction!(empirical, m)?)?;
    Ok(())
}
