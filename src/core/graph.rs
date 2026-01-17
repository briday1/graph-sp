//! Graph structure and node definitions for the DAG execution engine.

use crate::core::data::{NodeId, Port, PortData, PortId};
use crate::core::error::{GraphError, Result};
use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Function type for node execution
pub type NodeFunction =
    Arc<dyn Fn(&HashMap<PortId, PortData>) -> Result<HashMap<PortId, PortData>> + Send + Sync>;

/// Configuration for a node in the graph
#[derive(Clone)]
pub struct NodeConfig {
    /// Unique identifier for the node
    pub id: NodeId,
    /// Human-readable name
    pub name: String,
    /// Node description
    pub description: Option<String>,
    /// Input ports
    pub input_ports: Vec<Port>,
    /// Output ports
    pub output_ports: Vec<Port>,
    /// Execution function
    pub function: NodeFunction,
}

impl NodeConfig {
    /// Create a new node configuration
    pub fn new(
        id: impl Into<NodeId>,
        name: impl Into<String>,
        input_ports: Vec<Port>,
        output_ports: Vec<Port>,
        function: NodeFunction,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            input_ports,
            output_ports,
            function,
        }
    }

    /// Set the description for this node
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Represents a node in the execution graph
#[derive(Clone)]
pub struct Node {
    /// Node configuration
    pub config: NodeConfig,
    /// Current input data
    pub inputs: HashMap<PortId, PortData>,
    /// Current output data
    pub outputs: HashMap<PortId, PortData>,
}

impl Node {
    /// Create a new node from a configuration
    pub fn new(config: NodeConfig) -> Self {
        Self {
            config,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
        }
    }

    /// Set input data for a port
    pub fn set_input(&mut self, port_id: impl Into<PortId>, data: PortData) {
        self.inputs.insert(port_id.into(), data);
    }

    /// Get output data from a port
    pub fn get_output(&self, port_id: &str) -> Option<&PortData> {
        self.outputs.get(port_id)
    }

    /// Execute the node's function
    pub fn execute(&mut self) -> Result<()> {
        // Validate required inputs
        for port in &self.config.input_ports {
            if port.required && !self.inputs.contains_key(&port.id) {
                return Err(GraphError::MissingInput {
                    node: self.config.id.clone(),
                    port: port.id.clone(),
                });
            }
        }

        // Execute the function
        let outputs = (self.config.function)(&self.inputs)?;
        self.outputs = outputs;
        Ok(())
    }

    /// Clear input data
    pub fn clear_inputs(&mut self) {
        self.inputs.clear();
    }

    /// Clear output data
    pub fn clear_outputs(&mut self) {
        self.outputs.clear();
    }
}

/// Represents an edge connecting two nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// Source node ID
    pub from_node: NodeId,
    /// Source port ID
    pub from_port: PortId,
    /// Target node ID
    pub to_node: NodeId,
    /// Target port ID
    pub to_port: PortId,
}

impl Edge {
    /// Create a new edge
    pub fn new(
        from_node: impl Into<NodeId>,
        from_port: impl Into<PortId>,
        to_node: impl Into<NodeId>,
        to_port: impl Into<PortId>,
    ) -> Self {
        Self {
            from_node: from_node.into(),
            from_port: from_port.into(),
            to_node: to_node.into(),
            to_port: to_port.into(),
        }
    }
}

/// The main graph structure representing a DAG
#[derive(Clone)]
pub struct Graph {
    /// Internal graph structure
    graph: DiGraph<Node, Edge>,
    /// Map from node ID to graph index
    node_indices: HashMap<NodeId, NodeIndex>,
}

impl Graph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: Node) -> Result<()> {
        let node_id = node.config.id.clone();

        if self.node_indices.contains_key(&node_id) {
            return Err(GraphError::InvalidGraph(format!(
                "Node with ID '{}' already exists",
                node_id
            )));
        }

        let index = self.graph.add_node(node);
        self.node_indices.insert(node_id, index);
        Ok(())
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: Edge) -> Result<()> {
        let from_idx = self
            .node_indices
            .get(&edge.from_node)
            .ok_or_else(|| GraphError::NodeNotFound(edge.from_node.clone()))?;
        let to_idx = self
            .node_indices
            .get(&edge.to_node)
            .ok_or_else(|| GraphError::NodeNotFound(edge.to_node.clone()))?;

        // Check if the output port exists
        let from_node = &self.graph[*from_idx];
        if !from_node
            .config
            .output_ports
            .iter()
            .any(|p| p.id == edge.from_port)
        {
            return Err(GraphError::PortError(format!(
                "Output port '{}' not found on node '{}'",
                edge.from_port, edge.from_node
            )));
        }

        // Check if the input port exists
        let to_node = &self.graph[*to_idx];
        if !to_node
            .config
            .input_ports
            .iter()
            .any(|p| p.id == edge.to_port)
        {
            return Err(GraphError::PortError(format!(
                "Input port '{}' not found on node '{}'",
                edge.to_port, edge.to_node
            )));
        }

        self.graph.add_edge(*from_idx, *to_idx, edge);
        Ok(())
    }

    /// Get a node by ID
    pub fn get_node(&self, node_id: &str) -> Result<&Node> {
        let idx = self
            .node_indices
            .get(node_id)
            .ok_or_else(|| GraphError::NodeNotFound(node_id.to_string()))?;
        Ok(&self.graph[*idx])
    }

    /// Get a mutable reference to a node by ID
    pub fn get_node_mut(&mut self, node_id: &str) -> Result<&mut Node> {
        let idx = self
            .node_indices
            .get(node_id)
            .ok_or_else(|| GraphError::NodeNotFound(node_id.to_string()))?;
        Ok(&mut self.graph[*idx])
    }

    /// Validate the graph (check for cycles)
    pub fn validate(&self) -> Result<()> {
        match toposort(&self.graph, None) {
            Ok(_) => Ok(()),
            Err(cycle) => {
                let node = &self.graph[cycle.node_id()];
                Err(GraphError::CycleDetected(node.config.id.clone()))
            }
        }
    }

    /// Get a topological ordering of the nodes
    pub fn topological_order(&self) -> Result<Vec<NodeId>> {
        let sorted = toposort(&self.graph, None).map_err(|cycle| {
            let node = &self.graph[cycle.node_id()];
            GraphError::CycleDetected(node.config.id.clone())
        })?;

        Ok(sorted
            .into_iter()
            .map(|idx| self.graph[idx].config.id.clone())
            .collect())
    }

    /// Get all nodes in the graph
    pub fn nodes(&self) -> Vec<&Node> {
        self.graph
            .node_indices()
            .map(|idx| &self.graph[idx])
            .collect()
    }

    /// Get all edges in the graph
    pub fn edges(&self) -> Vec<&Edge> {
        self.graph
            .edge_indices()
            .map(|idx| &self.graph[idx])
            .collect()
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Get the number of edges
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Get incoming edges for a node
    pub fn incoming_edges(&self, node_id: &str) -> Result<Vec<&Edge>> {
        let idx = self
            .node_indices
            .get(node_id)
            .ok_or_else(|| GraphError::NodeNotFound(node_id.to_string()))?;

        Ok(self
            .graph
            .edges_directed(*idx, Direction::Incoming)
            .map(|e| e.weight())
            .collect())
    }

    /// Get outgoing edges for a node
    pub fn outgoing_edges(&self, node_id: &str) -> Result<Vec<&Edge>> {
        let idx = self
            .node_indices
            .get(node_id)
            .ok_or_else(|| GraphError::NodeNotFound(node_id.to_string()))?;

        Ok(self
            .graph
            .edges_directed(*idx, Direction::Outgoing)
            .map(|e| e.weight())
            .collect())
    }

    /// Automatically connect nodes based on matching port names
    /// This enables implicit edge mapping without explicit add_edge() calls
    /// 
    /// # Matching Strategy
    /// - Connects output ports to input ports with the same name
    /// - Only creates edges if the port names match exactly
    /// - Respects topological ordering to avoid cycles
    /// 
    /// # Returns
    /// The number of edges created
    pub fn auto_connect(&mut self) -> Result<usize> {
        let mut edges_created = 0;
        let node_ids: Vec<NodeId> = self.nodes().iter().map(|n| n.config.id.clone()).collect();

        for from_node_id in &node_ids {
            let from_node = self.get_node(from_node_id)?;
            let output_ports: Vec<PortId> = from_node
                .config
                .output_ports
                .iter()
                .map(|p| p.id.clone())
                .collect();

            for to_node_id in &node_ids {
                if from_node_id == to_node_id {
                    continue;
                }

                let to_node = self.get_node(to_node_id)?;
                let input_ports: Vec<PortId> = to_node
                    .config
                    .input_ports
                    .iter()
                    .map(|p| p.id.clone())
                    .collect();

                // Find matching port names
                for output_port in &output_ports {
                    for input_port in &input_ports {
                        if output_port == input_port {
                            // Check if edge already exists
                            let edge_exists = self.edges().iter().any(|e| {
                                e.from_node == *from_node_id
                                    && e.from_port == *output_port
                                    && e.to_node == *to_node_id
                                    && e.to_port == *input_port
                            });

                            if !edge_exists {
                                let edge = Edge::new(
                                    from_node_id.clone(),
                                    output_port.clone(),
                                    to_node_id.clone(),
                                    input_port.clone(),
                                );
                                self.add_edge(edge)?;
                                edges_created += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(edges_created)
    }

    /// Build a graph with strict mode disabled - uses implicit edge mapping
    /// This is a convenience method that calls auto_connect() after all nodes are added
    pub fn with_auto_connect(mut self) -> Result<Self> {
        self.auto_connect()?;
        Ok(self)
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::data::PortData;

    fn dummy_function(inputs: &HashMap<PortId, PortData>) -> Result<HashMap<PortId, PortData>> {
        let mut outputs = HashMap::new();
        if let Some(PortData::Int(val)) = inputs.get("input") {
            outputs.insert("output".to_string(), PortData::Int(val * 2));
        }
        Ok(outputs)
    }

    #[test]
    fn test_graph_creation() {
        let graph = Graph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_add_node() {
        let mut graph = Graph::new();

        let config = NodeConfig::new(
            "node1",
            "Node 1",
            vec![Port::new("input", "Input")],
            vec![Port::new("output", "Output")],
            Arc::new(dummy_function),
        );

        let node = Node::new(config);
        assert!(graph.add_node(node).is_ok());
        assert_eq!(graph.node_count(), 1);
    }

    #[test]
    fn test_duplicate_node_id() {
        let mut graph = Graph::new();

        let config1 = NodeConfig::new("node1", "Node 1", vec![], vec![], Arc::new(dummy_function));

        let config2 = NodeConfig::new(
            "node1",
            "Node 1 Duplicate",
            vec![],
            vec![],
            Arc::new(dummy_function),
        );

        assert!(graph.add_node(Node::new(config1)).is_ok());
        assert!(graph.add_node(Node::new(config2)).is_err());
    }

    #[test]
    fn test_add_edge() {
        let mut graph = Graph::new();

        let config1 = NodeConfig::new(
            "node1",
            "Node 1",
            vec![],
            vec![Port::new("output", "Output")],
            Arc::new(dummy_function),
        );

        let config2 = NodeConfig::new(
            "node2",
            "Node 2",
            vec![Port::new("input", "Input")],
            vec![],
            Arc::new(dummy_function),
        );

        graph.add_node(Node::new(config1)).unwrap();
        graph.add_node(Node::new(config2)).unwrap();

        let edge = Edge::new("node1", "output", "node2", "input");
        assert!(graph.add_edge(edge).is_ok());
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_topological_order() {
        let mut graph = Graph::new();

        // Create a simple linear graph: node1 -> node2 -> node3
        for i in 1..=3 {
            let outputs = if i < 3 {
                vec![Port::new("output", "Output")]
            } else {
                vec![]
            };
            let inputs = if i > 1 {
                vec![Port::new("input", "Input")]
            } else {
                vec![]
            };

            let config = NodeConfig::new(
                format!("node{}", i),
                format!("Node {}", i),
                inputs,
                outputs,
                Arc::new(dummy_function),
            );
            graph.add_node(Node::new(config)).unwrap();
        }

        graph
            .add_edge(Edge::new("node1", "output", "node2", "input"))
            .unwrap();
        graph
            .add_edge(Edge::new("node2", "output", "node3", "input"))
            .unwrap();

        let order = graph.topological_order().unwrap();
        assert_eq!(order.len(), 3);
        assert_eq!(order[0], "node1");
        assert_eq!(order[1], "node2");
        assert_eq!(order[2], "node3");
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = Graph::new();

        // Create a cycle: node1 -> node2 -> node1
        let config1 = NodeConfig::new(
            "node1",
            "Node 1",
            vec![Port::new("input", "Input")],
            vec![Port::new("output", "Output")],
            Arc::new(dummy_function),
        );

        let config2 = NodeConfig::new(
            "node2",
            "Node 2",
            vec![Port::new("input", "Input")],
            vec![Port::new("output", "Output")],
            Arc::new(dummy_function),
        );

        graph.add_node(Node::new(config1)).unwrap();
        graph.add_node(Node::new(config2)).unwrap();

        graph
            .add_edge(Edge::new("node1", "output", "node2", "input"))
            .unwrap();
        graph
            .add_edge(Edge::new("node2", "output", "node1", "input"))
            .unwrap();

        assert!(graph.validate().is_err());
    }

    #[test]
    fn test_auto_connect() {
        let mut graph = Graph::new();

        // Create nodes with matching port names
        let config1 = NodeConfig::new(
            "source",
            "Source",
            vec![],
            vec![Port::new("data", "Data")],
            Arc::new(dummy_function),
        );

        let config2 = NodeConfig::new(
            "processor",
            "Processor",
            vec![Port::new("data", "Data")], // Matches source output!
            vec![Port::new("result", "Result")],
            Arc::new(dummy_function),
        );

        let config3 = NodeConfig::new(
            "sink",
            "Sink",
            vec![Port::new("result", "Result")], // Matches processor output!
            vec![],
            Arc::new(dummy_function),
        );

        graph.add_node(Node::new(config1)).unwrap();
        graph.add_node(Node::new(config2)).unwrap();
        graph.add_node(Node::new(config3)).unwrap();

        // Initially no edges
        assert_eq!(graph.edge_count(), 0);

        // Auto-connect should create 2 edges
        let edges_created = graph.auto_connect().unwrap();
        assert_eq!(edges_created, 2);
        assert_eq!(graph.edge_count(), 2);

        // Graph should be valid
        assert!(graph.validate().is_ok());
    }

    #[test]
    fn test_auto_connect_parallel_branches() {
        let mut graph = Graph::new();

        // Source with output "value"
        let source = NodeConfig::new(
            "source",
            "Source",
            vec![],
            vec![Port::new("value", "Value")],
            Arc::new(dummy_function),
        );

        // Two branches with same input port name
        let branch1 = NodeConfig::new(
            "branch1",
            "Branch 1",
            vec![Port::new("value", "Value")],
            vec![Port::new("out1", "Output 1")],
            Arc::new(dummy_function),
        );

        let branch2 = NodeConfig::new(
            "branch2",
            "Branch 2",
            vec![Port::new("value", "Value")],
            vec![Port::new("out2", "Output 2")],
            Arc::new(dummy_function),
        );

        // Merger with inputs matching branch outputs
        let merger = NodeConfig::new(
            "merger",
            "Merger",
            vec![Port::new("out1", "Input 1"), Port::new("out2", "Input 2")],
            vec![],
            Arc::new(dummy_function),
        );

        graph.add_node(Node::new(source)).unwrap();
        graph.add_node(Node::new(branch1)).unwrap();
        graph.add_node(Node::new(branch2)).unwrap();
        graph.add_node(Node::new(merger)).unwrap();

        // Auto-connect should create 4 edges (fan-out + fan-in)
        let edges_created = graph.auto_connect().unwrap();
        assert_eq!(edges_created, 4);
        assert_eq!(graph.edge_count(), 4);

        // Graph should be valid
        assert!(graph.validate().is_ok());
    }
}
