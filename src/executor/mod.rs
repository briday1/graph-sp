//! Parallel execution engine for DAG graphs.

use crate::core::{Graph, PortData, Result};
use dashmap::DashMap;
use futures::stream::{FuturesUnordered, StreamExt};
use std::collections::HashMap;
use std::sync::Arc;

/// Executor for running graphs with parallel execution
#[derive(Clone)]
pub struct Executor {
    /// Maximum number of concurrent tasks (reserved for future use)
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

        // Get topological order to determine dependencies
        let topo_order = graph.topological_order()?;

        // Track execution state - map from node_id to outputs
        let execution_state: Arc<DashMap<String, HashMap<String, PortData>>> =
            Arc::new(DashMap::new());

        // Build dependency levels for parallel execution
        let levels = self.build_dependency_levels(graph, &topo_order)?;

        // Execute each level in parallel
        for level in levels {
            let mut tasks = FuturesUnordered::new();

            for node_id in level {
                let node = graph.get_node(&node_id)?.clone();
                let edges = graph
                    .incoming_edges(&node_id)?
                    .iter()
                    .map(|e| (*e).clone())
                    .collect::<Vec<_>>();
                let state = Arc::clone(&execution_state);

                // Spawn a blocking task for each node (nodes execute synchronously)
                let task = tokio::task::spawn_blocking(move || {
                    let mut node = node;

                    // Collect inputs from incoming edges
                    for edge in edges {
                        if let Some(source_outputs) = state.get(&edge.from_node) {
                            if let Some(data) = source_outputs.get(&edge.from_port) {
                                node.set_input(edge.to_port.clone(), data.clone());
                            }
                        }
                    }

                    // Execute the node
                    let result = node.execute();

                    (node.config.id.clone(), node.outputs.clone(), result)
                });

                tasks.push(task);
            }

            // Wait for all nodes in this level to complete
            while let Some(result) = tasks.next().await {
                let (node_id, outputs, exec_result) = result.map_err(|e| {
                    crate::core::GraphError::ExecutionError(format!("Task join error: {}", e))
                })?;
                exec_result?;
                execution_state.insert(node_id, outputs);
            }
        }

        // Collect results
        let mut node_outputs = HashMap::new();
        for entry in execution_state.iter() {
            node_outputs.insert(entry.key().clone(), entry.value().clone());
        }

        Ok(ExecutionResult {
            success: true,
            node_outputs,
            errors: Vec::new(),
        })
    }

    /// Build dependency levels for parallel execution
    /// All nodes in the same level can execute in parallel
    fn build_dependency_levels(
        &self,
        graph: &Graph,
        topo_order: &[String],
    ) -> Result<Vec<Vec<String>>> {
        let mut levels: Vec<Vec<String>> = Vec::new();
        let mut node_level: HashMap<String, usize> = HashMap::new();

        // Assign each node to a level based on its dependencies
        for node_id in topo_order {
            let incoming = graph.incoming_edges(node_id)?;

            // Find the maximum level of all dependencies
            let max_dep_level = incoming
                .iter()
                .filter_map(|edge| node_level.get(&edge.from_node))
                .max()
                .copied();

            // This node goes one level after its dependencies
            let level = max_dep_level.map(|l| l + 1).unwrap_or(0);
            node_level.insert(node_id.clone(), level);

            // Ensure we have enough levels
            while levels.len() <= level {
                levels.push(Vec::new());
            }

            levels[level].push(node_id.clone());
        }

        Ok(levels)
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
    pub node_outputs:
        std::collections::HashMap<String, std::collections::HashMap<String, PortData>>,
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
    use crate::core::{Edge, Node, NodeConfig, Port};
    use std::collections::HashMap;
    use std::sync::Arc;

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

        graph
            .add_edge(Edge::new("source", "output", "double", "input"))
            .unwrap();
        graph
            .add_edge(Edge::new("double", "output", "add5", "input"))
            .unwrap();

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
