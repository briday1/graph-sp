//! Python bindings for the graph execution engine.

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::types::{PyDict, PyList};
#[cfg(feature = "python")]
use std::collections::HashMap;

#[cfg(feature = "python")]
use crate::core::{Graph, Node, NodeConfig, Port, Edge, PortData, GraphData};
#[cfg(feature = "python")]
use crate::executor::{Executor, ExecutionResult};
#[cfg(feature = "python")]
use crate::inspector::{Inspector, GraphAnalysis};

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
    fn new(id: String, name: String, required: Option<bool>) -> Self {
        let mut port = Port::new(id, name);
        if let Some(req) = required {
            port.required = req;
        }
        Self { inner: port }
    }

    #[getter]
    fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[getter]
    fn name(&self) -> String {
        self.inner.name.clone()
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

    fn add_node(
        &mut self,
        id: String,
        name: String,
        input_ports: Vec<PyRef<PyPort>>,
        output_ports: Vec<PyRef<PyPort>>,
        function: PyObject,
    ) -> PyResult<()> {
        let inputs: Vec<Port> = input_ports.iter().map(|p| p.inner.clone()).collect();
        let outputs: Vec<Port> = output_ports.iter().map(|p| p.inner.clone()).collect();

        // Create a wrapper for the Python function
        let py_func = function.clone();
        let node_func = std::sync::Arc::new(
            move |port_inputs: &HashMap<String, PortData>| -> crate::core::Result<HashMap<String, PortData>> {
                Python::with_gil(|py| {
                    // Convert inputs to Python dict
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

                    // Convert result back to HashMap<String, PortData>
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

        let config = NodeConfig::new(id, name, inputs, outputs, node_func);
        let node = Node::new(config);

        self.inner.add_node(node)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn add_edge(
        &mut self,
        from_node: String,
        from_port: String,
        to_node: String,
        to_port: String,
    ) -> PyResult<()> {
        let edge = Edge::new(from_node, from_port, to_node, to_port);
        self.inner.add_edge(edge)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))
    }

    fn validate(&self) -> PyResult<()> {
        self.inner.validate()
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

    fn analyze(&self) -> PyResult<PyGraphAnalysis> {
        let analysis = Inspector::analyze(&self.inner);
        Ok(PyGraphAnalysis { inner: analysis })
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

    fn execute(&self, graph: &mut PyGraph) -> PyResult<PyExecutionResult> {
        let mut graph_clone = graph.inner.clone();
        
        // Use tokio runtime to execute async code
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let result = rt.block_on(async {
            self.inner.execute(&mut graph_clone).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(PyExecutionResult { inner: result })
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
    if value.is_none() {
        Ok(PortData::None)
    } else if let Ok(b) = value.extract::<bool>() {
        Ok(PortData::Bool(b))
    } else if let Ok(i) = value.extract::<i64>() {
        Ok(PortData::Int(i))
    } else if let Ok(f) = value.extract::<f64>() {
        Ok(PortData::Float(f))
    } else if let Ok(s) = value.extract::<String>() {
        Ok(PortData::String(s))
    } else if let Ok(list) = value.downcast::<PyList>() {
        let mut items = Vec::new();
        for item in list.iter() {
            items.push(python_to_port_data(item)?);
        }
        Ok(PortData::List(items))
    } else if let Ok(dict) = value.downcast::<PyDict>() {
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
        PortData::Bytes(b) => Ok(b.into_py(py)),
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
