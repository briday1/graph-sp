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

/// Merge function type for combining outputs from multiple branches
pub type MergeFunction = Arc<dyn Fn(Vec<&PortData>) -> Result<PortData> + Send + Sync>;

/// Configuration for merging branch outputs
pub struct MergeConfig {
    /// Branches to merge
    pub branches: Vec<String>,
    /// Output port name on each branch to merge
    pub port: String,
    /// Custom merge function (default: collect into list)
    pub merge_fn: Option<MergeFunction>,
}

impl MergeConfig {
    /// Create a new merge configuration
    pub fn new(branches: Vec<String>, port: String) -> Self {
        Self {
            branches,
            port,
            merge_fn: None,
        }
    }

    /// Set a custom merge function
    pub fn with_merge_fn(mut self, merge_fn: MergeFunction) -> Self {
        self.merge_fn = Some(merge_fn);
        self
    }
}

/// Variant function type for generating parameter variations
pub type VariantFunction = Arc<dyn Fn(usize) -> PortData + Send + Sync>;

/// Configuration for creating variants (config sweeps)
pub struct VariantConfig {
    /// Name prefix for variant branches
    pub name_prefix: String,
    /// Number of variants to create
    pub count: usize,
    /// Function to generate variant parameter values
    pub variant_fn: VariantFunction,
    /// Parameter name to vary
    pub param_name: String,
    /// Whether to enable parallel execution (default: true)
    pub parallel: bool,
}

impl VariantConfig {
    /// Create a new variant configuration
    pub fn new(
        name_prefix: impl Into<String>,
        count: usize,
        param_name: impl Into<String>,
        variant_fn: VariantFunction,
    ) -> Self {
        Self {
            name_prefix: name_prefix.into(),
            count,
            variant_fn,
            param_name: param_name.into(),
            parallel: true,
        }
    }

    /// Set parallelization flag
    pub fn with_parallel(mut self, parallel: bool) -> Self {
        self.parallel = parallel;
        self
    }
}

/// The main graph structure representing a DAG
#[derive(Clone)]
pub struct Graph {
    /// Internal graph structure
    graph: DiGraph<Node, Edge>,
    /// Map from node ID to graph index
    node_indices: HashMap<NodeId, NodeIndex>,
    /// Named branches (subgraphs)
    branches: HashMap<String, Graph>,
    /// Track node addition order for implicit mapping
    node_order: Vec<NodeId>,
    /// Whether to use strict edge mapping (explicit add_edge required)
    strict_edge_mapping: bool,
}

impl Graph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
            branches: HashMap::new(),
            node_order: Vec::new(),
            strict_edge_mapping: false,
        }
    }

    /// Create a new graph with strict edge mapping enabled
    /// When enabled, edges must be explicitly added with add_edge()
    /// When disabled (default), edges are automatically created based on node order
    pub fn with_strict_edges() -> Self {
        Self {
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
            branches: HashMap::new(),
            node_order: Vec::new(),
            strict_edge_mapping: true,
        }
    }

    /// Set strict edge mapping mode
    pub fn set_strict_edge_mapping(&mut self, strict: bool) {
        self.strict_edge_mapping = strict;
    }

    /// Add a node to the graph
    pub fn add(&mut self, node: Node) -> Result<()> {
        let node_id = node.config.id.clone();

        if self.node_indices.contains_key(&node_id) {
            return Err(GraphError::InvalidGraph(format!(
                "Node with ID '{}' already exists",
                node_id
            )));
        }

        let index = self.graph.add_node(node);
        self.node_indices.insert(node_id.clone(), index);

        // Implicit edge mapping: connect to previous node if not in strict mode
        if !self.strict_edge_mapping && !self.node_order.is_empty() {
            self.auto_connect_to_previous(&node_id)?;
        }

        self.node_order.push(node_id);
        Ok(())
    }

    /// Automatically connect the new node to the previous node based on port names
    fn auto_connect_to_previous(&mut self, new_node_id: &str) -> Result<()> {
        let edges_to_add = if let Some(prev_node_id) = self.node_order.last().cloned() {
            let prev_node = self.get_node(&prev_node_id)?;
            let new_node = self.get_node(new_node_id)?;

            let mut edges = Vec::new();
            // Match output ports from previous node to input ports of new node
            for out_port in &prev_node.config.output_ports {
                for in_port in &new_node.config.input_ports {
                    // Connect if port names match or if they're the only ports
                    let should_connect = out_port.id == in_port.id ||
                        (prev_node.config.output_ports.len() == 1 &&
                         new_node.config.input_ports.len() == 1);

                    if should_connect {
                        edges.push(Edge::new(
                            &prev_node_id,
                            &out_port.id,
                            new_node_id,
                            &in_port.id,
                        ));
                        break; // Only connect first matching port
                    }
                }
            }
            edges
        } else {
            Vec::new()
        };

        // Add all collected edges
        for edge in edges_to_add {
            self.add_edge(edge)?;
        }

        Ok(())
    }

    /// Alias for add() for backward compatibility
    #[deprecated(since = "0.2.0", note = "Use `add` instead")]
    pub fn add_node(&mut self, node: Node) -> Result<()> {
        self.add(node)
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

    /// Create a new branch (subgraph) with the given name
    pub fn create_branch(&mut self, name: impl Into<String>) -> Result<&mut Graph> {
        let name = name.into();
        if self.branches.contains_key(&name) {
            return Err(GraphError::InvalidGraph(format!(
                "Branch '{}' already exists",
                name
            )));
        }
        self.branches.insert(name.clone(), Graph::new());
        Ok(self.branches.get_mut(&name).unwrap())
    }

    /// Get a reference to a branch by name
    pub fn get_branch(&self, name: &str) -> Result<&Graph> {
        self.branches
            .get(name)
            .ok_or_else(|| GraphError::InvalidGraph(format!("Branch '{}' not found", name)))
    }

    /// Get a mutable reference to a branch by name
    pub fn get_branch_mut(&mut self, name: &str) -> Result<&mut Graph> {
        self.branches
            .get_mut(name)
            .ok_or_else(|| GraphError::InvalidGraph(format!("Branch '{}' not found", name)))
    }

    /// Get all branch names
    pub fn branch_names(&self) -> Vec<String> {
        self.branches.keys().cloned().collect()
    }

    /// Check if a branch exists
    pub fn has_branch(&self, name: &str) -> bool {
        self.branches.contains_key(name)
    }

    /// Create a merge node that combines outputs from multiple branches
    ///
    /// The merge node will collect outputs from the specified branches and combine them
    /// using the provided merge function (or collect into a list by default).
    pub fn merge(
        &mut self,
        node_id: impl Into<NodeId>,
        config: MergeConfig,
    ) -> Result<()> {
        // Validate that all branches exist
        for branch_name in &config.branches {
            if !self.has_branch(branch_name) {
                return Err(GraphError::InvalidGraph(format!(
                    "Branch '{}' not found for merge operation",
                    branch_name
                )));
            }
        }

        let branch_names = config.branches.clone();

        // Create the merge function
        let merge_fn = config.merge_fn.unwrap_or_else(|| {
            // Default merge function: collect into a list
            Arc::new(|inputs: Vec<&PortData>| -> Result<PortData> {
                Ok(PortData::List(inputs.iter().map(|&d| d.clone()).collect()))
            })
        });

        // Create input ports - one for each branch
        let input_ports: Vec<Port> = branch_names
            .iter()
            .map(|name| Port::new(name.clone(), format!("Input from {}", name)))
            .collect();

        // Create a merge node
        let node_config = NodeConfig::new(
            node_id,
            "Merge Node",
            input_ports,
            vec![Port::new("merged", "Merged Output")],
            Arc::new(move |inputs: &HashMap<PortId, PortData>| {
                // Collect inputs in branch order
                let mut collected_inputs = Vec::new();
                for branch_name in &branch_names {
                    if let Some(data) = inputs.get(branch_name.as_str()) {
                        collected_inputs.push(data);
                    }
                }

                // Apply merge function
                let merged = merge_fn(collected_inputs)?;

                let mut outputs = HashMap::new();
                outputs.insert("merged".to_string(), merged);
                Ok(outputs)
            }),
        );

        self.add(Node::new(node_config))
    }

    /// Create variant branches for config sweeps
    ///
    /// This creates multiple isolated branches, each with a different parameter value.
    /// Variants can be used for hyperparameter sweeps, A/B testing, or any scenario
    /// where you want to run the same computation with different inputs.
    ///
    /// Returns the names of the created variant branches.
    pub fn create_variants(&mut self, config: VariantConfig) -> Result<Vec<String>> {
        let mut branch_names = Vec::new();

        for i in 0..config.count {
            let branch_name = format!("{}_{}", config.name_prefix, i);

            // Check if branch already exists
            if self.has_branch(&branch_name) {
                return Err(GraphError::InvalidGraph(format!(
                    "Variant branch '{}' already exists",
                    branch_name
                )));
            }

            // Create the branch
            let branch = self.create_branch(&branch_name)?;

            // Add a source node to the branch with the variant parameter
            let param_value = (config.variant_fn)(i);
            let param_name = config.param_name.clone();

            let source_config = NodeConfig::new(
                format!("{}_source", branch_name),
                format!("Variant Source {}", i),
                vec![],
                vec![Port::new(&param_name, "Variant Parameter")],
                // Note: param_name and param_value must be cloned into the closure
                // because the closure is moved into an Arc and needs to own these values
                // to ensure they remain valid for the lifetime of the node function
                Arc::new(move |_: &HashMap<PortId, PortData>| {
                    let mut outputs = HashMap::new();
                    outputs.insert(param_name.clone(), param_value.clone());
                    Ok(outputs)
                }),
            );

            branch.add(Node::new(source_config))?;
            branch_names.push(branch_name);
        }

        Ok(branch_names)
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
        assert!(graph.add(node).is_ok());
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

        assert!(graph.add(Node::new(config1)).is_ok());
        assert!(graph.add(Node::new(config2)).is_err());
    }

    #[test]
    fn test_add_edge() {
        let mut graph = Graph::with_strict_edges();

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

        graph.add(Node::new(config1)).unwrap();
        graph.add(Node::new(config2)).unwrap();

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
            graph.add(Node::new(config)).unwrap();
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

        graph.add(Node::new(config1)).unwrap();
        graph.add(Node::new(config2)).unwrap();

        graph
            .add_edge(Edge::new("node1", "output", "node2", "input"))
            .unwrap();
        graph
            .add_edge(Edge::new("node2", "output", "node1", "input"))
            .unwrap();

        assert!(graph.validate().is_err());
    }

    #[test]
    fn test_create_branch() {
        let mut graph = Graph::new();

        // Create a branch
        let branch = graph.create_branch("branch_a");
        assert!(branch.is_ok());

        // Verify branch exists
        assert!(graph.has_branch("branch_a"));
        assert_eq!(graph.branch_names().len(), 1);
        assert_eq!(graph.branch_names()[0], "branch_a");
    }

    #[test]
    fn test_duplicate_branch_name() {
        let mut graph = Graph::new();

        graph.create_branch("branch_a").unwrap();
        let result = graph.create_branch("branch_a");
        assert!(result.is_err());
    }

    #[test]
    fn test_branch_isolation() {
        let mut graph = Graph::new();

        // Create two branches
        let branch_a = graph.create_branch("branch_a").unwrap();
        let config_a = NodeConfig::new(
            "node_a",
            "Node A",
            vec![],
            vec![Port::new("output", "Output")],
            Arc::new(dummy_function),
        );
        branch_a.add(Node::new(config_a)).unwrap();

        let branch_b = graph.create_branch("branch_b").unwrap();
        let config_b = NodeConfig::new(
            "node_b",
            "Node B",
            vec![],
            vec![Port::new("output", "Output")],
            Arc::new(dummy_function),
        );
        branch_b.add(Node::new(config_b)).unwrap();

        // Verify each branch has only one node
        assert_eq!(graph.get_branch("branch_a").unwrap().node_count(), 1);
        assert_eq!(graph.get_branch("branch_b").unwrap().node_count(), 1);

        // Verify branches don't share nodes
        assert!(graph
            .get_branch("branch_a")
            .unwrap()
            .get_node("node_b")
            .is_err());
        assert!(graph
            .get_branch("branch_b")
            .unwrap()
            .get_node("node_a")
            .is_err());
    }

    #[test]
    fn test_get_nonexistent_branch() {
        let graph = Graph::new();
        assert!(graph.get_branch("nonexistent").is_err());
    }

    #[test]
    fn test_merge_basic() {
        let mut graph = Graph::new();

        // Create two branches
        graph.create_branch("branch_a").unwrap();
        graph.create_branch("branch_b").unwrap();

        // Create merge configuration
        let merge_config = MergeConfig::new(
            vec!["branch_a".to_string(), "branch_b".to_string()],
            "output".to_string(),
        );

        // Create merge node
        let result = graph.merge("merge_node", merge_config);
        assert!(result.is_ok());

        // Verify merge node was created
        assert_eq!(graph.node_count(), 1);
        assert!(graph.get_node("merge_node").is_ok());
    }

    #[test]
    fn test_merge_with_nonexistent_branch() {
        let mut graph = Graph::new();

        graph.create_branch("branch_a").unwrap();

        let merge_config = MergeConfig::new(
            vec!["branch_a".to_string(), "nonexistent".to_string()],
            "output".to_string(),
        );

        let result = graph.merge("merge_node", merge_config);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_with_custom_function() {
        let mut graph = Graph::new();

        graph.create_branch("branch_a").unwrap();
        graph.create_branch("branch_b").unwrap();

        // Custom merge function that finds max
        let max_merge = Arc::new(|inputs: Vec<&PortData>| -> Result<PortData> {
            let mut max_val = i64::MIN;
            for data in inputs {
                if let PortData::Int(val) = data {
                    max_val = max_val.max(*val);
                }
            }
            Ok(PortData::Int(max_val))
        });

        let merge_config = MergeConfig::new(
            vec!["branch_a".to_string(), "branch_b".to_string()],
            "output".to_string(),
        )
        .with_merge_fn(max_merge);

        let result = graph.merge("merge_node", merge_config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_variants() {
        let mut graph = Graph::new();

        // Create variants with integer values
        let variant_fn = Arc::new(|i: usize| PortData::Int(i as i64 * 10));
        let config = VariantConfig::new("test_variant", 3, "param", variant_fn);

        let result = graph.create_variants(config);
        assert!(result.is_ok());

        let branch_names = result.unwrap();
        assert_eq!(branch_names.len(), 3);
        assert_eq!(branch_names[0], "test_variant_0");
        assert_eq!(branch_names[1], "test_variant_1");
        assert_eq!(branch_names[2], "test_variant_2");

        // Verify each branch was created with a source node
        for branch_name in &branch_names {
            assert!(graph.has_branch(branch_name));
            let branch = graph.get_branch(branch_name).unwrap();
            assert_eq!(branch.node_count(), 1);
        }
    }

    #[test]
    fn test_variants_with_parallelization_flag() {
        let mut graph = Graph::new();

        let variant_fn = Arc::new(|i: usize| PortData::Float(i as f64 * 0.5));
        let config =
            VariantConfig::new("param_sweep", 5, "learning_rate", variant_fn).with_parallel(false);

        let result = graph.create_variants(config);
        assert!(result.is_ok());

        let branch_names = result.unwrap();
        assert_eq!(branch_names.len(), 5);
    }

    #[test]
    fn test_duplicate_variant_branch() {
        let mut graph = Graph::new();

        // Create initial variant
        let variant_fn = Arc::new(|i: usize| PortData::Int(i as i64));
        let config = VariantConfig::new("test", 2, "param", variant_fn.clone());

        graph.create_variants(config).unwrap();

        // Try to create the same variants again
        let config2 = VariantConfig::new("test", 2, "param", variant_fn);
        let result = graph.create_variants(config2);
        assert!(result.is_err());
    }

    #[test]
    fn test_implicit_edge_mapping() {
        // Default mode: implicit edge mapping
        let mut graph = Graph::new();

        let config1 = NodeConfig::new(
            "source",
            "Source",
            vec![],
            vec![Port::new("output", "Output")],
            Arc::new(dummy_function),
        );

        let config2 = NodeConfig::new(
            "processor",
            "Processor",
            vec![Port::new("output", "Input")], // Port name matches prev output
            vec![Port::new("result", "Result")],
            Arc::new(dummy_function),
        );

        let config3 = NodeConfig::new(
            "sink",
            "Sink",
            vec![Port::new("result", "Input")], // Port name matches prev output
            vec![],
            Arc::new(dummy_function),
        );

        // Add nodes - edges should be created automatically
        graph.add(Node::new(config1)).unwrap();
        graph.add(Node::new(config2)).unwrap();
        graph.add(Node::new(config3)).unwrap();

        // Should have 2 edges (source->processor, processor->sink)
        assert_eq!(graph.edge_count(), 2);
        assert_eq!(graph.node_count(), 3);
    }

    #[test]
    fn test_strict_edge_mapping() {
        // Strict mode: explicit edges required
        let mut graph = Graph::with_strict_edges();

        let config1 = NodeConfig::new(
            "source",
            "Source",
            vec![],
            vec![Port::new("output", "Output")],
            Arc::new(dummy_function),
        );

        let config2 = NodeConfig::new(
            "sink",
            "Sink",
            vec![Port::new("output", "Input")],
            vec![],
            Arc::new(dummy_function),
        );

        // Add nodes - NO edges should be created automatically
        graph.add(Node::new(config1)).unwrap();
        graph.add(Node::new(config2)).unwrap();

        // Should have 0 edges in strict mode
        assert_eq!(graph.edge_count(), 0);
        assert_eq!(graph.node_count(), 2);
    }

    #[test]
    fn test_auto_connect() {
        let mut graph = Graph::with_strict_edges();

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

        graph.add(Node::new(config1)).unwrap();
        graph.add(Node::new(config2)).unwrap();
        graph.add(Node::new(config3)).unwrap();

        // Initially no edges in strict mode
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
        let mut graph = Graph::with_strict_edges();

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

        graph.add(Node::new(source)).unwrap();
        graph.add(Node::new(branch1)).unwrap();
        graph.add(Node::new(branch2)).unwrap();
        graph.add(Node::new(merger)).unwrap();

        // Auto-connect should create 4 edges (fan-out + fan-in)
        let edges_created = graph.auto_connect().unwrap();
        assert_eq!(edges_created, 4);
        assert_eq!(graph.edge_count(), 4);

        // Graph should be valid
        assert!(graph.validate().is_ok());
    }
}
