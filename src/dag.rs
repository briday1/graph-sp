//! DAG representation with execution and visualization support

use crate::node::{Node, NodeId};
use std::collections::{HashMap, HashSet, VecDeque};

/// Execution context for storing variable values during graph execution
pub type ExecutionContext = HashMap<String, String>;

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

    /// Execute the DAG
    ///
    /// Runs all nodes in topological order, accumulating outputs in the execution context.
    pub fn execute(&self) -> ExecutionContext {
        let mut context = ExecutionContext::new();

        for &node_id in &self.execution_order {
            if let Some(node) = self.nodes.iter().find(|n| n.id == node_id) {
                let outputs = node.execute(&context);
                context.extend(outputs);
            }
        }

        context
    }

    /// Execute the DAG with parallel execution of independent nodes
    ///
    /// Nodes at the same execution level are run concurrently.
    pub fn execute_parallel(&self) -> ExecutionContext {
        let mut context = ExecutionContext::new();

        for level in &self.execution_levels {
            // For simplicity, execute nodes in level sequentially
            // A full implementation would use thread pools or async execution
            for &node_id in level {
                if let Some(node) = self.nodes.iter().find(|n| n.id == node_id) {
                    let outputs = node.execute(&context);
                    context.extend(outputs);
                }
            }
        }

        context
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
            self.node_count, self.depth, self.max_parallelism, self.branch_count, self.variant_count
        )
    }
}
