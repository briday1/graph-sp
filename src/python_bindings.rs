//! Python bindings for graph-sp
//!
//! This module provides PyO3 bindings to expose the Rust graph executor to Python.
//! It is gated behind the "python" feature flag.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::exceptions::PyValueError;
use std::collections::HashMap;
use std::sync::Arc;

use crate::builder::Graph;
use crate::dag::Dag;

/// Python wrapper for Graph builder
#[pyclass]
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
        let graph = self.graph.as_mut().ok_or_else(|| {
            PyValueError::new_err("Graph has already been built or consumed")
        })?;
        
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
            // Wrap Python callable in a Rust closure
            let rust_function = create_python_node_function(py_func);
            
            graph.add(
                rust_function,
                label.as_deref(),
                if input_refs.is_empty() { None } else { Some(input_refs) },
                if output_refs.is_empty() { None } else { Some(output_refs) },
            );
        } else {
            // No-op function if None provided
            let noop = |_: &HashMap<String, String>, _: &HashMap<String, String>| HashMap::new();
            graph.add(
                noop,
                label.as_deref(),
                if input_refs.is_empty() { None } else { Some(input_refs) },
                if output_refs.is_empty() { None } else { Some(output_refs) },
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
        let graph = self.graph.as_mut().ok_or_else(|| {
            PyValueError::new_err("Graph has already been built or consumed")
        })?;
        
        let subgraph_inner = subgraph.graph.take().ok_or_else(|| {
            PyValueError::new_err("Subgraph has already been built or consumed")
        })?;
        
        Ok(graph.branch(subgraph_inner))
    }

    /// Build the DAG from the graph
    ///
    /// Returns:
    ///     PyDag instance ready for execution
    fn build(&mut self) -> PyResult<PyDag> {
        let graph = self.graph.take().ok_or_else(|| {
            PyValueError::new_err("Graph has already been built")
        })?;
        
        Ok(PyDag {
            dag: graph.build(),
        })
    }
}

/// Python wrapper for DAG executor
#[pyclass]
struct PyDag {
    dag: Dag,
}

#[pymethods]
impl PyDag {
    /// Execute the DAG sequentially
    ///
    /// Returns:
    ///     Dictionary containing the execution context
    fn execute(&self, py: Python) -> PyResult<PyObject> {
        // Release GIL during Rust execution
        let context = py.allow_threads(|| self.dag.execute());
        
        // Convert HashMap to Python dict
        let py_dict = PyDict::new(py);
        for (key, value) in context.iter() {
            py_dict.set_item(key, value)?;
        }
        Ok(py_dict.to_object(py))
    }

    /// Execute the DAG with parallel execution where possible
    ///
    /// Returns:
    ///     Dictionary containing the execution context
    fn execute_parallel(&self, py: Python) -> PyResult<PyObject> {
        // Release GIL during Rust execution
        let context = py.allow_threads(|| self.dag.execute_parallel());
        
        // Convert HashMap to Python dict
        let py_dict = PyDict::new(py);
        for (key, value) in context.iter() {
            py_dict.set_item(key, value)?;
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
) -> impl Fn(&HashMap<String, String>, &HashMap<String, String>) -> HashMap<String, String> + Send + Sync + 'static {
    // Wrap in Arc to make it cloneable and shareable
    let py_func = Arc::new(py_func);
    
    move |inputs: &HashMap<String, String>, variant_params: &HashMap<String, String>| {
        // Acquire GIL only for the duration of this call
        Python::with_gil(|py| {
            // Convert inputs to Python dict
            let py_inputs = PyDict::new(py);
            for (key, value) in inputs.iter() {
                if let Err(e) = py_inputs.set_item(key, value) {
                    // Log to Python's stderr for better integration
                    let _ = py.import("sys")
                        .and_then(|sys| sys.getattr("stderr"))
                        .and_then(|stderr| stderr.call_method1("write", (format!("Error setting input '{}': {}\n", key, e),)));
                    return HashMap::new();
                }
            }

            // Convert variant_params to Python dict
            let py_variant_params = PyDict::new(py);
            for (key, value) in variant_params.iter() {
                if let Err(e) = py_variant_params.set_item(key, value) {
                    let _ = py.import("sys")
                        .and_then(|sys| sys.getattr("stderr"))
                        .and_then(|stderr| stderr.call_method1("write", (format!("Error setting variant param '{}': {}\n", key, e),)));
                    return HashMap::new();
                }
            }

            // Call the Python function
            let result = py_func.call1(py, (py_inputs, py_variant_params));
            
            match result {
                Ok(py_result) => {
                    // Convert result back to HashMap
                    if let Ok(result_dict) = py_result.downcast::<PyDict>(py) {
                        let mut output = HashMap::new();
                        for (key, value) in result_dict.iter() {
                            if let (Ok(k), Ok(v)) = (key.extract::<String>(), value.extract::<String>()) {
                                output.insert(k, v);
                            }
                        }
                        output
                    } else {
                        let _ = py.import("sys")
                            .and_then(|sys| sys.getattr("stderr"))
                            .and_then(|stderr| stderr.call_method1("write", ("Error: Python function did not return a dict\n",)));
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

/// Initialize the Python module
#[pymodule]
fn graph_sp(_py: Python, m: &PyModule) -> PyResult<()> {
    // PyO3 0.18.3 with auto-initialize feature handles multi-threading initialization automatically
    m.add_class::<PyGraph>()?;
    m.add_class::<PyDag>()?;
    Ok(())
}
