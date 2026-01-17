//! Python bindings for the graph execution engine.

// Allow non-local definitions for PyO3 macros
#![allow(non_local_definitions)]

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::{PyDict, PyList, PyTuple};
#[cfg(feature = "python")]
use std::collections::HashMap;
#[cfg(feature = "python")]
use uuid::Uuid;

#[cfg(feature = "python")]
use crate::core::{Edge, Graph, Node, NodeConfig, Port, PortData};
#[cfg(feature = "python")]
use crate::executor::{ExecutionResult, Executor};
#[cfg(feature = "python")]
use crate::inspector::{GraphAnalysis, Inspector};

#[cfg(feature = "python")]
/// Helper function to parse port specifications from Python
/// Accepts: Port objects, strings (for simple ports), or tuples of (broadcast_name, impl_name)
fn parse_ports(ports_any: &PyAny) -> PyResult<Vec<Port>> {
    let mut ports = Vec::new();
    
    if let Ok(ports_list) = ports_any.downcast::<PyList>() {
        for item in ports_list.iter() {
            // Try to extract as PyPort
            if let Ok(py_port) = item.extract::<PyRef<PyPort>>() {
                ports.push(py_port.inner.clone());
            }
            // Try to extract as string (simple port)
            else if let Ok(name) = item.extract::<String>() {
                ports.push(Port::simple(name));
            }
            // Try to extract as tuple (broadcast_name, impl_name)
            else if let Ok(tuple) = item.downcast::<PyTuple>() {
                if tuple.len() == 2 {
                    let broadcast_name: String = tuple.get_item(0)?.extract()?;
                    let impl_name: String = tuple.get_item(1)?.extract()?;
                    ports.push(Port::new(broadcast_name, impl_name));
                } else {
                    return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                        "Port tuples must have exactly 2 elements: (broadcast_name, impl_name)"
                    ));
                }
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "Port must be a Port object, string, or tuple of (broadcast_name, impl_name)"
                ));
            }
        }
    } else {
        return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Ports must be a list"
        ));
    }
    
    Ok(ports)
}

#[cfg(feature = "python")]
/// Python wrapper for PortData
#[pyclass(name = "PortData")]
pub struct PyPortData {
    inner: PortData,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyPortData {
    #[new]
    fn new(value: &PyAny) -> PyResult<Self> {
        let inner = python_to_port_data(value)?;
        Ok(Self { inner })
    }

    fn __repr__(&self) -> String {
        format!("{}", self.inner)
    }

    fn to_python(&self, py: Python) -> PyResult<PyObject> {
        port_data_to_python(py, &self.inner)
    }
}

#[cfg(feature = "python")]
/// Python wrapper for Port
#[pyclass(name = "Port")]
pub struct PyPort {
    inner: Port,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyPort {
    #[new]
    #[pyo3(signature = (broadcast_name, impl_name=None, display_name=None, required=None))]
    fn new(
        broadcast_name: String,
        impl_name: Option<String>,
        display_name: Option<String>,
        required: Option<bool>,
    ) -> Self {
        let impl_name = impl_name.unwrap_or_else(|| broadcast_name.clone());
        let mut port = Port::new(broadcast_name, impl_name);
        if let Some(display) = display_name {
            port.display_name = display;
        }
        if let Some(req) = required {
            port.required = req;
        }
        Self { inner: port }
    }

    #[getter]
    fn broadcast_name(&self) -> String {
        self.inner.broadcast_name.clone()
    }

    #[getter]
    fn impl_name(&self) -> String {
        self.inner.impl_name.clone()
    }

    #[getter]
    fn display_name(&self) -> String {
        self.inner.display_name.clone()
    }

    #[getter]
    fn required(&self) -> bool {
        self.inner.required
    }

    fn with_description(&mut self, description: String) {
        self.inner.description = Some(description);
    }
}

#[cfg(feature = "python")]
/// Python wrapper for Graph
#[pyclass(name = "Graph")]
pub struct PyGraph {
    inner: Graph,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyGraph {
    #[new]
    fn new() -> Self {
        Self {
            inner: Graph::new(),
        }
    }

    #[pyo3(signature = (function, label=None, inputs=None, outputs=None))]
    fn add(
        &mut self,
        function: PyObject,
        label: Option<String>,
        inputs: Option<&PyAny>,
        outputs: Option<&PyAny>,
    ) -> PyResult<()> {
        Python::with_gil(|py| {
            // Generate ID from function name if available
            let id = if let Ok(name) = function.getattr(py, "__name__") {
                name.extract::<String>(py).unwrap_or_else(|_| format!("node_{}", Uuid::new_v4()))
            } else {
                format!("node_{}", Uuid::new_v4())
            };

            let display_label = label.unwrap_or_else(|| id.clone());

            // Parse inputs - can be: None, [], [Port, ...], ["name", ...], [("broadcast", "impl"), ...]
            let input_ports = if let Some(inputs_any) = inputs {
                parse_ports(inputs_any)?
            } else {
                vec![]
            };

            // Parse outputs - same format as inputs
            let output_ports = if let Some(outputs_any) = outputs {
                parse_ports(outputs_any)?
            } else {
                vec![]
            };

            // Create a wrapper for the Python function
            let py_func = function.clone();
            let node_func = std::sync::Arc::new(
                move |port_inputs: &HashMap<String, PortData>| -> crate::core::Result<HashMap<String, PortData>> {
                    Python::with_gil(|py| {
                        // Convert inputs to Python dict using impl_name as keys
                        let py_dict = PyDict::new(py);
                        for (key, value) in port_inputs {
                            let py_value = port_data_to_python(py, value)
                                .map_err(|e| crate::core::GraphError::ExecutionError(e.to_string()))?;
                            py_dict.set_item(key, py_value)
                                .map_err(|e| crate::core::GraphError::ExecutionError(e.to_string()))?;
                        }

                        // Call the Python function
                        let result = py_func.call1(py, (py_dict,))
                            .map_err(|e| crate::core::GraphError::ExecutionError(e.to_string()))?;

                        // Convert result back to HashMap<String, PortData> using impl_name as keys
                        let result_dict = result.downcast::<PyDict>(py)
                            .map_err(|e| crate::core::GraphError::ExecutionError(format!("Function must return dict: {}", e)))?;

                        let mut outputs = HashMap::new();
                        for (key, value) in result_dict.iter() {
                            let key_str: String = key.extract()
                                .map_err(|e| crate::core::GraphError::ExecutionError(e.to_string()))?;
                            let port_data = python_to_port_data(value)
                                .map_err(|e| crate::core::GraphError::ExecutionError(e.to_string()))?;
                            outputs.insert(key_str, port_data);
                        }

                        Ok(outputs)
                    })
                }
            );

            let config = NodeConfig::new(id, display_label, input_ports, output_ports, node_func);
            let node = Node::new(config);

            self.inner
                .add(node)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
        })
    }

    fn add_edge(
        &mut self,
        from_node: String,
        from_port: String,
        to_node: String,
        to_port: String,
    ) -> PyResult<()> {
        let edge = Edge::new(from_node, from_port, to_node, to_port);
        self.inner
            .add_edge(edge)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn auto_connect(&mut self) -> PyResult<usize> {
        self.inner
            .auto_connect()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn validate(&self) -> PyResult<()> {
        self.inner
            .validate()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn node_count(&self) -> usize {
        self.inner.node_count()
    }

    fn edge_count(&self) -> usize {
        self.inner.edge_count()
    }

    fn visualize(&self) -> PyResult<String> {
        Inspector::visualize(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn to_mermaid(&self) -> PyResult<String> {
        Inspector::to_mermaid(&self.inner)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn analyze(&self) -> PyResult<PyGraphAnalysis> {
        let analysis = Inspector::analyze(&self.inner);
        Ok(PyGraphAnalysis { inner: analysis })
    }

    fn create_branch(&mut self, name: String) -> PyResult<()> {
        self.inner
            .create_branch(name)
            .map(|_| ())
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn has_branch(&self, name: String) -> bool {
        self.inner.has_branch(&name)
    }

    fn branch_names(&self) -> Vec<String> {
        self.inner.branch_names()
    }
}

#[cfg(feature = "python")]
/// Python wrapper for Executor
#[pyclass(name = "Executor")]
pub struct PyExecutor {
    inner: Executor,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyExecutor {
    #[new]
    fn new(max_concurrency: Option<usize>) -> Self {
        let inner = if let Some(max) = max_concurrency {
            Executor::with_concurrency(max)
        } else {
            Executor::new()
        };
        Self { inner }
    }

    fn execute(&self, graph: &mut PyGraph, py: Python) -> PyResult<PyExecutionResult> {
        let mut graph_clone = graph.inner.clone();
        let inner_executor = self.inner.clone();

        // Release the GIL before creating the runtime
        py.allow_threads(|| {
            // Use tokio runtime with multi-threaded scheduler
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(4)
                .enable_all()
                .build()
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            // Block on the execution (GIL is released, so with_gil calls inside will work)
            let result = rt
                .block_on(async move { inner_executor.execute(&mut graph_clone).await })
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            Ok(PyExecutionResult { inner: result })
        })
    }
}

#[cfg(feature = "python")]
/// Python wrapper for ExecutionResult
#[pyclass(name = "ExecutionResult")]
pub struct PyExecutionResult {
    inner: ExecutionResult,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyExecutionResult {
    fn is_success(&self) -> bool {
        self.inner.is_success()
    }

    fn get_output(&self, py: Python, node_id: String, port_id: String) -> PyResult<PyObject> {
        match self.inner.get_output(&node_id, &port_id) {
            Some(data) => port_data_to_python(py, data),
            None => Ok(py.None()),
        }
    }
}

#[cfg(feature = "python")]
/// Python wrapper for GraphAnalysis
#[pyclass(name = "GraphAnalysis")]
pub struct PyGraphAnalysis {
    inner: GraphAnalysis,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyGraphAnalysis {
    #[getter]
    fn node_count(&self) -> usize {
        self.inner.node_count
    }

    #[getter]
    fn edge_count(&self) -> usize {
        self.inner.edge_count
    }

    #[getter]
    fn depth(&self) -> usize {
        self.inner.depth
    }

    #[getter]
    fn width(&self) -> usize {
        self.inner.width
    }

    fn summary(&self) -> String {
        self.inner.summary()
    }
}

#[cfg(feature = "python")]
// Helper functions for Python<->Rust conversion
fn python_to_port_data(value: &PyAny) -> PyResult<PortData> {
    use pyo3::types::{PyBool, PyFloat, PyLong, PyString};

    if value.is_none() {
        Ok(PortData::None)
    } else if let Ok(b) = value.downcast::<PyBool>() {
        // Check bool first as it's more specific
        Ok(PortData::Bool(b.extract()?))
    } else if let Ok(i) = value.downcast::<PyLong>() {
        // Check for integer
        Ok(PortData::Int(i.extract()?))
    } else if let Ok(f) = value.downcast::<PyFloat>() {
        // Check for float
        Ok(PortData::Float(f.extract()?))
    } else if let Ok(s) = value.downcast::<PyString>() {
        // Check for string
        Ok(PortData::String(s.extract()?))
    } else if let Ok(list) = value.downcast::<PyList>() {
        // Check for list
        let mut items = Vec::new();
        for item in list.iter() {
            items.push(python_to_port_data(item)?);
        }
        Ok(PortData::List(items))
    } else if let Ok(dict) = value.downcast::<PyDict>() {
        // Check for dict
        let mut map = HashMap::new();
        for (k, v) in dict.iter() {
            let key: String = k.extract()?;
            map.insert(key, python_to_port_data(v)?);
        }
        Ok(PortData::Map(map))
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
            "Unsupported data type",
        ))
    }
}

#[cfg(feature = "python")]
fn port_data_to_python(py: Python, data: &PortData) -> PyResult<PyObject> {
    match data {
        PortData::None => Ok(py.None()),
        PortData::Bool(b) => Ok(b.into_py(py)),
        PortData::Int(i) => Ok(i.into_py(py)),
        PortData::Float(f) => Ok(f.into_py(py)),
        PortData::String(s) => Ok(s.into_py(py)),
        PortData::Bytes(b) => Ok(b.clone().into_py(py)),
        PortData::Json(j) => Ok(j.to_string().into_py(py)),
        PortData::List(items) => {
            let list = PyList::empty(py);
            for item in items {
                list.append(port_data_to_python(py, item)?)?;
            }
            Ok(list.into())
        }
        PortData::Map(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map {
                dict.set_item(k, port_data_to_python(py, v)?)?;
            }
            Ok(dict.into())
        }
    }
}

#[cfg(feature = "python")]
/// Python module initialization
#[pymodule]
fn graph_sp(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyPortData>()?;
    m.add_class::<PyPort>()?;
    m.add_class::<PyGraph>()?;
    m.add_class::<PyExecutor>()?;
    m.add_class::<PyExecutionResult>()?;
    m.add_class::<PyGraphAnalysis>()?;
    Ok(())
}
