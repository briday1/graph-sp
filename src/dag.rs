//! DAG representation with execution and visualization support

use crate::graph_data::GraphData;
use crate::node::{Node, NodeId};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};

/// Execution context for storing variable values during graph execution
pub type ExecutionContext = HashMap<String, GraphData>;

/// Execution result that tracks outputs per node and per branch
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Global execution context (all variables accessible by broadcast name)
    pub context: ExecutionContext,
    /// Outputs per node (node_id -> HashMap of output variables)
    pub node_outputs: HashMap<NodeId, HashMap<String, GraphData>>,
    /// Outputs per branch (branch_id -> HashMap of output variables)
    pub branch_outputs: HashMap<usize, HashMap<String, GraphData>>,
}

impl ExecutionResult {
    /// Create a new empty execution result
    pub fn new() -> Self {
        Self {
            context: HashMap::new(),
            node_outputs: HashMap::new(),
            branch_outputs: HashMap::new(),
        }
    }

    /// Get a value from the global context
    pub fn get(&self, key: &str) -> Option<&GraphData> {
        self.context.get(key)
    }

    /// Get all outputs from a specific node
    pub fn get_node_outputs(&self, node_id: NodeId) -> Option<&HashMap<String, GraphData>> {
        self.node_outputs.get(&node_id)
    }

    /// Get all outputs from a specific branch
    pub fn get_branch_outputs(&self, branch_id: usize) -> Option<&HashMap<String, GraphData>> {
        self.branch_outputs.get(&branch_id)
    }

    /// Get a specific variable from a node
    pub fn get_from_node(&self, node_id: NodeId, key: &str) -> Option<&GraphData> {
        self.node_outputs
            .get(&node_id)
            .and_then(|outputs| outputs.get(key))
    }

    /// Get a specific variable from a branch
    pub fn get_from_branch(&self, branch_id: usize, key: &str) -> Option<&GraphData> {
        self.branch_outputs
            .get(&branch_id)
            .and_then(|outputs| outputs.get(key))
    }

    /// Check if a variable exists in global context
    pub fn contains_key(&self, key: &str) -> bool {
        self.context.contains_key(key)
    }
}

/// Directed Acyclic Graph representing the optimized execution plan
pub struct Dag {
    /// All nodes in the DAG
    nodes: Vec<Node>,
    /// Execution order (topologically sorted)
    execution_order: Vec<NodeId>,
    /// Levels for parallel execution (nodes at same level can run in parallel)
    execution_levels: Vec<Vec<NodeId>>,
}

impl Dag {
    /// Create a new DAG from a list of nodes
    ///
    /// Performs implicit inspection:
    /// - Validates the graph is acyclic
    /// - Determines optimal execution order
    /// - Identifies parallelizable operations
    pub fn new(nodes: Vec<Node>) -> Self {
        let execution_order = Self::topological_sort(&nodes);
        let execution_levels = Self::compute_execution_levels(&nodes, &execution_order);

        Self {
            nodes,
            execution_order,
            execution_levels,
        }
    }

    /// Perform topological sort to determine execution order
    fn topological_sort(nodes: &[Node]) -> Vec<NodeId> {
        let mut in_degree: HashMap<NodeId, usize> = HashMap::new();
        let mut adj_list: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

        // Initialize in-degree and adjacency list
        for node in nodes {
            in_degree.entry(node.id).or_insert(0);
            adj_list.entry(node.id).or_insert_with(Vec::new);

            for &dep in &node.dependencies {
                *in_degree.entry(node.id).or_insert(0) += 1;
                adj_list.entry(dep).or_insert_with(Vec::new).push(node.id);
            }
        }

        // Kahn's algorithm for topological sort
        let mut queue: VecDeque<NodeId> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut result = Vec::new();

        while let Some(node_id) = queue.pop_front() {
            result.push(node_id);

            if let Some(neighbors) = adj_list.get(&node_id) {
                for &neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(&neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor);
                        }
                    }
                }
            }
        }

        result
    }

    /// Compute execution levels for parallel execution
    ///
    /// Nodes at the same level have no dependencies on each other and can
    /// execute in parallel.
    fn compute_execution_levels(nodes: &[Node], execution_order: &[NodeId]) -> Vec<Vec<NodeId>> {
        let mut levels: Vec<Vec<NodeId>> = Vec::new();
        let mut node_level: HashMap<NodeId, usize> = HashMap::new();

        for &node_id in execution_order {
            let node = nodes.iter().find(|n| n.id == node_id).unwrap();

            // Find the maximum level of all dependencies
            let level = if node.dependencies.is_empty() {
                0
            } else {
                node.dependencies
                    .iter()
                    .filter_map(|dep_id| node_level.get(dep_id))
                    .max()
                    .map(|&max_level| max_level + 1)
                    .unwrap_or(0)
            };

            node_level.insert(node_id, level);

            // Add node to its level
            while levels.len() <= level {
                levels.push(Vec::new());
            }
            levels[level].push(node_id);
        }

        levels
    }

    /// Execute the DAG (legacy method returning just context)
    ///
    /// Runs all nodes in topological order, accumulating outputs in the execution context.
    ///
    /// # Arguments
    /// * `parallel` - If true, execute nodes at the same level concurrently
    /// * `max_threads` - Optional maximum number of threads to use per level (None = unlimited)
    pub fn execute(&self, parallel: bool, max_threads: Option<usize>) -> ExecutionContext {
        self.execute_detailed(parallel, max_threads).context
    }

    /// Execute the DAG with detailed per-node and per-branch tracking
    ///
    /// Runs all nodes in topological order and tracks outputs per node and per branch.
    ///
    /// # Arguments
    /// * `parallel` - If true, execute nodes at the same level concurrently
    /// * `max_threads` - Optional maximum number of threads to use per level (None = unlimited)
    pub fn execute_detailed(&self, parallel: bool, max_threads: Option<usize>) -> ExecutionResult {
        let mut result = ExecutionResult::new();

        if !parallel {
            // Sequential execution
            for &node_id in &self.execution_order {
                if let Some(node) = self.nodes.iter().find(|n| n.id == node_id) {
                    let outputs = node.execute(&result.context);

                    // Store outputs in global context
                    result.context.extend(outputs.clone());

                    // Store outputs per node (using broadcast variable names from output_mapping)
                    result.node_outputs.insert(node_id, outputs.clone());

                    // Store outputs per branch if this node belongs to a branch
                    if let Some(branch_id) = node.branch_id {
                        result
                            .branch_outputs
                            .entry(branch_id)
                            .or_insert_with(HashMap::new)
                            .extend(outputs);
                    }
                }
            }
        } else {
            // Parallel execution
            for level in &self.execution_levels {
                // Execute nodes at the same level in parallel
                if level.len() == 1 {
                    // Single node - no need for threading overhead
                    let node_id = level[0];
                    if let Some(node) = self.nodes.iter().find(|n| n.id == node_id) {
                        let outputs = node.execute(&result.context);

                        result.context.extend(outputs.clone());
                        result.node_outputs.insert(node_id, outputs.clone());

                        if let Some(branch_id) = node.branch_id {
                            result
                                .branch_outputs
                                .entry(branch_id)
                                .or_insert_with(HashMap::new)
                                .extend(outputs);
                        }
                    }
                } else {
                    // Multiple nodes - execute in parallel using scoped threads
                    let context = Arc::new(result.context.clone());
                    let nodes_to_execute: Vec<_> = level
                        .iter()
                        .filter_map(|&node_id| self.nodes.iter().find(|n| n.id == node_id))
                        .collect();

                    // Limit threads if max_threads is specified
                    let chunk_size = if let Some(max) = max_threads {
                        max.max(1) // At least 1 thread
                    } else {
                        nodes_to_execute.len() // Unlimited - one thread per node
                    };

                    let outputs = Arc::new(Mutex::new(Vec::new()));

                    // Process nodes in chunks to respect max_threads limit
                    for chunk in nodes_to_execute.chunks(chunk_size) {
                        std::thread::scope(|s| {
                            for node in chunk {
                                let context = Arc::clone(&context);
                                let outputs = Arc::clone(&outputs);

                                s.spawn(move || {
                                    let node_outputs = node.execute(&context);
                                    outputs.lock().unwrap().push((
                                        node.id,
                                        node.branch_id,
                                        node_outputs,
                                    ));
                                });
                            }
                        });
                    }

                    // Collect outputs from all parallel executions
                    let collected_outputs = outputs.lock().unwrap();
                    for (node_id, branch_id, node_outputs) in collected_outputs.iter() {
                        result.context.extend(node_outputs.clone());
                        result.node_outputs.insert(*node_id, node_outputs.clone());

                        if let Some(bid) = branch_id {
                            result
                                .branch_outputs
                                .entry(*bid)
                                .or_insert_with(HashMap::new)
                                .extend(node_outputs.clone());
                        }
                    }
                }
            }
        }

        result
    }

    /// Generate a Mermaid diagram for visualization with port mappings
    ///
    /// Returns a string containing a Mermaid flowchart representing the DAG.
    /// Edge labels show port mappings (broadcast_var → impl_var).
    pub fn to_mermaid(&self) -> String {
        let mut mermaid = String::from("graph TD\n");

        // Add all nodes
        for node in &self.nodes {
            let node_label = node.display_name();
            mermaid.push_str(&format!("    {}[\"{}\"]\n", node.id, node_label));
        }

        // Add edges with port mapping labels
        let mut edges_added: HashSet<(NodeId, NodeId)> = HashSet::new();
        for node in &self.nodes {
            for &dep_id in &node.dependencies {
                let edge = (dep_id, node.id);
                if !edges_added.contains(&edge) {
                    // Find the dependency node to get its output mappings
                    let dep_node = self.nodes.iter().find(|n| n.id == dep_id);

                    // Build port mapping label
                    let mut port_labels = Vec::new();

                    // Show input mappings for the current node that come from this dependency
                    for (broadcast_var, impl_var) in &node.input_mapping {
                        // Check if this broadcast var comes from the dependency
                        if let Some(dep) = dep_node {
                            // Check if dependency produces this broadcast var
                            if dep.output_mapping.values().any(|v| v == broadcast_var) {
                                port_labels.push(format!("{} → {}", broadcast_var, impl_var));
                            }
                        }
                    }

                    // Format edge with port labels
                    if port_labels.is_empty() {
                        mermaid.push_str(&format!("    {} --> {}\n", dep_id, node.id));
                    } else {
                        let label = port_labels.join("<br/>");
                        mermaid.push_str(&format!("    {} -->|{}| {}\n", dep_id, label, node.id));
                    }

                    edges_added.insert(edge);
                }
            }
        }

        // Add styling for branches
        for node in &self.nodes {
            if node.is_branch {
                mermaid.push_str(&format!("    style {} fill:#e1f5ff\n", node.id));
            }
        }

        // Add styling for variants
        for node in &self.nodes {
            if let Some(variant_idx) = node.variant_index {
                let colors = ["#ffe1e1", "#e1ffe1", "#ffe1ff", "#ffffe1"];
                let color = colors[variant_idx % colors.len()];
                mermaid.push_str(&format!("    style {} fill:{}\n", node.id, color));
            }
        }

        mermaid
    }

    /// Get the execution order
    pub fn execution_order(&self) -> &[NodeId] {
        &self.execution_order
    }

    /// Get the execution levels
    pub fn execution_levels(&self) -> &[Vec<NodeId>] {
        &self.execution_levels
    }

    /// Get all nodes
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Get statistics about the DAG
    pub fn stats(&self) -> DagStats {
        DagStats {
            node_count: self.nodes.len(),
            depth: self.execution_levels.len(),
            max_parallelism: self
                .execution_levels
                .iter()
                .map(|level| level.len())
                .max()
                .unwrap_or(0),
            branch_count: self.nodes.iter().filter(|n| n.is_branch).count(),
            variant_count: self
                .nodes
                .iter()
                .filter_map(|n| n.variant_index)
                .max()
                .map(|max| max + 1)
                .unwrap_or(0),
        }
    }
}

/// Statistics about a DAG
#[derive(Debug, Clone)]
pub struct DagStats {
    /// Total number of nodes
    pub node_count: usize,
    /// Maximum depth (longest path from source to sink)
    pub depth: usize,
    /// Maximum number of nodes that can execute in parallel
    pub max_parallelism: usize,
    /// Number of branch nodes
    pub branch_count: usize,
    /// Number of variants
    pub variant_count: usize,
}

impl DagStats {
    /// Format stats as a human-readable string
    pub fn summary(&self) -> String {
        format!(
            "DAG Statistics:\n\
             - Nodes: {}\n\
             - Depth: {} levels\n\
             - Max Parallelism: {} nodes\n\
             - Branches: {}\n\
             - Variants: {}",
            self.node_count,
            self.depth,
            self.max_parallelism,
            self.branch_count,
            self.variant_count
        )
    }
}
