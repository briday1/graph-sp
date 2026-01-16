//! Parallel execution engine for DAG graphs.

use crate::core::{Graph, Result, PortData};
use dashmap::DashMap;
use std::sync::Arc;

/// Executor for running graphs with parallel execution
#[derive(Clone)]
pub struct Executor {
    /// Maximum number of concurrent tasks (reserved for future parallel execution)
    #[allow(dead_code)]
    max_concurrency: usize,
}

impl Executor {
    /// Create a new executor with default concurrency
    pub fn new() -> Self {
        Self {
            max_concurrency: num_cpus::get(),
        }
    }

    /// Create a new executor with specified concurrency limit
    pub fn with_concurrency(max_concurrency: usize) -> Self {
        Self { max_concurrency }
    }

    /// Execute a graph and return the results
    pub async fn execute(&self, graph: &mut Graph) -> Result<ExecutionResult> {
        // Validate the graph first
        graph.validate()?;

        // Get topological order
        let order = graph.topological_order()?;

        // Track execution state - map from node_id to outputs
        let execution_state: Arc<DashMap<String, std::collections::HashMap<String, PortData>>> = 
            Arc::new(DashMap::new());
        
        // Execute nodes in topological order
        for node_id in order {
            // Get the node and prepare inputs from dependencies
            let mut node = graph.get_node(&node_id)?.clone();
            
            // Collect inputs from incoming edges
            for edge in graph.incoming_edges(&node_id)? {
                if let Some(source_outputs) = execution_state.get(&edge.from_node) {
                    if let Some(data) = source_outputs.get(&edge.from_port) {
                        node.set_input(edge.to_port.clone(), data.clone());
                    }
                }
            }

            // Execute the node
            node.execute()?;
            
            // Store outputs
            execution_state.insert(node_id.clone(), node.outputs.clone());
        }

        // Collect results
        let mut node_outputs = std::collections::HashMap::new();
        for entry in execution_state.iter() {
            node_outputs.insert(entry.key().clone(), entry.value().clone());
        }

        Ok(ExecutionResult {
            success: true,
            node_outputs,
            errors: Vec::new(),
        })
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of graph execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Whether execution was successful
    pub success: bool,
    /// Outputs from each node
    pub node_outputs: std::collections::HashMap<String, std::collections::HashMap<String, PortData>>,
    /// Any errors that occurred
    pub errors: Vec<String>,
}

impl ExecutionResult {
    /// Get output from a specific node and port
    pub fn get_output(&self, node_id: &str, port_id: &str) -> Option<&PortData> {
        self.node_outputs.get(node_id)?.get(port_id)
    }

    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        self.success
    }
}

// Helper function to get number of CPUs
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Node, NodeConfig, Port, Edge};
    use std::sync::Arc;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_executor_simple_graph() {
        let mut graph = Graph::new();

        // Create a simple node that doubles input
        let config = NodeConfig::new(
            "double",
            "Double Node",
            vec![Port::new("input", "Input")],
            vec![Port::new("output", "Output")],
            Arc::new(|inputs: &HashMap<String, PortData>| {
                let mut outputs = HashMap::new();
                if let Some(PortData::Int(val)) = inputs.get("input") {
                    outputs.insert("output".to_string(), PortData::Int(val * 2));
                }
                Ok(outputs)
            }),
        );

        let mut node = Node::new(config);
        node.set_input("input", PortData::Int(21));
        
        graph.add_node(node).unwrap();

        let executor = Executor::new();
        let result = executor.execute(&mut graph).await.unwrap();

        assert!(result.is_success());
        if let Some(PortData::Int(val)) = result.get_output("double", "output") {
            assert_eq!(*val, 42);
        } else {
            panic!("Expected output");
        }
    }

    #[tokio::test]
    async fn test_executor_linear_pipeline() {
        let mut graph = Graph::new();

        // Node 1: Output 10
        let config1 = NodeConfig::new(
            "source",
            "Source Node",
            vec![],
            vec![Port::new("output", "Output")],
            Arc::new(|_: &HashMap<String, PortData>| {
                let mut outputs = HashMap::new();
                outputs.insert("output".to_string(), PortData::Int(10));
                Ok(outputs)
            }),
        );

        // Node 2: Double the input
        let config2 = NodeConfig::new(
            "double",
            "Double Node",
            vec![Port::new("input", "Input")],
            vec![Port::new("output", "Output")],
            Arc::new(|inputs: &HashMap<String, PortData>| {
                let mut outputs = HashMap::new();
                if let Some(PortData::Int(val)) = inputs.get("input") {
                    outputs.insert("output".to_string(), PortData::Int(val * 2));
                }
                Ok(outputs)
            }),
        );

        // Node 3: Add 5
        let config3 = NodeConfig::new(
            "add5",
            "Add 5 Node",
            vec![Port::new("input", "Input")],
            vec![Port::new("output", "Output")],
            Arc::new(|inputs: &HashMap<String, PortData>| {
                let mut outputs = HashMap::new();
                if let Some(PortData::Int(val)) = inputs.get("input") {
                    outputs.insert("output".to_string(), PortData::Int(val + 5));
                }
                Ok(outputs)
            }),
        );

        graph.add_node(Node::new(config1)).unwrap();
        graph.add_node(Node::new(config2)).unwrap();
        graph.add_node(Node::new(config3)).unwrap();

        graph.add_edge(Edge::new("source", "output", "double", "input")).unwrap();
        graph.add_edge(Edge::new("double", "output", "add5", "input")).unwrap();

        let executor = Executor::new();
        let result = executor.execute(&mut graph).await.unwrap();

        assert!(result.is_success());
        
        // Source outputs 10
        if let Some(PortData::Int(val)) = result.get_output("source", "output") {
            assert_eq!(*val, 10);
        }
        
        // Double outputs 20
        if let Some(PortData::Int(val)) = result.get_output("double", "output") {
            assert_eq!(*val, 20);
        }
        
        // Add5 outputs 25
        if let Some(PortData::Int(val)) = result.get_output("add5", "output") {
            assert_eq!(*val, 25);
        }
    }
}
