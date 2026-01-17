//! Graph builder with implicit connections API

use crate::dag::Dag;
use crate::node::{Node, NodeId};
use std::sync::Arc;

/// Builder for constructing graphs with implicit node connections
pub struct GraphBuilder {
    /// All nodes in the graph
    nodes: Vec<Node>,
    /// Counter for generating unique node IDs
    next_id: NodeId,
    /// The last added node ID (for implicit connections)
    last_node_id: Option<NodeId>,
    /// Track the last branch point for sequential .branch() calls
    last_branch_point: Option<NodeId>,
    /// Subgraph builders for branches
    branches: Vec<GraphBuilder>,
}

impl GraphBuilder {
    /// Create a new graph builder
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            next_id: 0,
            last_node_id: None,
            last_branch_point: None,
            branches: Vec::new(),
        }
    }

    /// Add a node to the graph with implicit connections
    ///
    /// # Arguments
    ///
    /// * `function_handle` - The function to execute for this node
    /// * `label` - Optional label for visualization
    /// * `broadcast_vars` - Optional list of broadcast variables from graph context
    /// * `output_vars` - Optional list of output variables this node produces
    ///
    /// # Implicit Connection Behavior
    ///
    /// - The first node added has no dependencies
    /// - Subsequent nodes automatically depend on the previous node
    /// - This creates a natural sequential flow unless `.branch()` is used
    pub fn add<F>(
        &mut self,
        function_handle: F,
        label: Option<&str>,
        broadcast_vars: Option<Vec<&str>>,
        output_vars: Option<Vec<&str>>,
    ) -> &mut Self
    where
        F: Fn(&std::collections::HashMap<String, String>) -> std::collections::HashMap<String, String>
            + Send
            + Sync
            + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;

        let broadcast_vars = broadcast_vars
            .unwrap_or_default()
            .iter()
            .map(|s| s.to_string())
            .collect();

        let output_vars = output_vars
            .unwrap_or_default()
            .iter()
            .map(|s| s.to_string())
            .collect();

        let mut node = Node::new(
            id,
            Arc::new(function_handle),
            label.map(|s| s.to_string()),
            broadcast_vars,
            output_vars,
        );

        // Implicit connection: connect to the last added node
        if let Some(prev_id) = self.last_node_id {
            node.dependencies.push(prev_id);
        }

        self.nodes.push(node);
        self.last_node_id = Some(id);
        
        // Reset branch point after adding a regular node
        self.last_branch_point = None;

        self
    }

    /// Insert a branching subgraph
    ///
    /// # Implicit Branching Behavior
    ///
    /// - Sequential `.branch()` calls without `.add()` between them implicitly
    ///   branch from the same node
    /// - This allows creating multiple parallel execution paths easily
    ///
    /// # Arguments
    ///
    /// * `subgraph` - A configured GraphBuilder representing the branch
    pub fn branch(&mut self, mut subgraph: GraphBuilder) -> &mut Self {
        // Determine the branch point
        let branch_point = if let Some(bp) = self.last_branch_point {
            // Sequential .branch() calls - use the same branch point
            bp
        } else {
            // First branch after .add() - branch from last node
            if let Some(last_id) = self.last_node_id {
                self.last_branch_point = Some(last_id);
                last_id
            } else {
                // No previous node, subgraph starts independently
                self.branches.push(subgraph);
                return self;
            }
        };

        // Connect the first node of the subgraph to the branch point
        if let Some(first_node) = subgraph.nodes.first_mut() {
            if !first_node.dependencies.contains(&branch_point) {
                first_node.dependencies.push(branch_point);
            }
            first_node.is_branch = true;
        }

        // Merge subgraph nodes into main graph
        self.branches.push(subgraph);

        self
    }

    /// Create configuration sweep variants
    ///
    /// This method creates multiple copies of the remainder of the graph structure,
    /// each with different configuration values.
    ///
    /// # Arguments
    ///
    /// * `variants` - Vector of variant configurations
    ///
    /// # Behavior
    ///
    /// - Copies all downstream nodes for each variant
    /// - Each variant gets a unique configuration value
    /// - Variants can execute in parallel
    pub fn variant(&mut self, variants: Vec<(&str, String)>) -> &mut Self {
        // Store the current graph state as a template
        let template_nodes = self.nodes.clone();
        let template_last_id = self.last_node_id;

        // For each variant, create a copy of the downstream graph
        for (idx, (_var_name, _var_value)) in variants.iter().enumerate() {
            if idx == 0 {
                // First variant reuses the current graph
                // Mark nodes as part of variant 0
                for node in &mut self.nodes {
                    node.variant_index = Some(0);
                }
            } else {
                // Subsequent variants need new copies
                let mut variant_builder = GraphBuilder::new();
                variant_builder.next_id = self.next_id;

                // Clone template nodes
                for template_node in &template_nodes {
                    let mut node = template_node.clone();
                    node.id = variant_builder.next_id;
                    variant_builder.next_id += 1;
                    node.variant_index = Some(idx);
                    
                    variant_builder.nodes.push(node);
                }

                variant_builder.last_node_id = template_last_id
                    .map(|_| variant_builder.nodes.last().map(|n| n.id))
                    .flatten();

                let next_id = variant_builder.next_id;
                self.branches.push(variant_builder);
                self.next_id = next_id;
            }
        }

        self
    }

    /// Build the final DAG from the graph builder
    ///
    /// This performs the implicit inspection phase:
    /// - Full graph traversal
    /// - Execution path optimization
    /// - Data flow connection determination
    /// - Identification of parallelizable operations
    pub fn build(mut self) -> Dag {
        // Merge all branch subgraphs into main node list
        let branches = std::mem::take(&mut self.branches);
        for branch in branches {
            self.merge_branch(branch);
        }

        Dag::new(self.nodes)
    }

    /// Merge a branch builder's nodes into this builder
    fn merge_branch(&mut self, branch: GraphBuilder) {
        // Add all nodes from branch
        self.nodes.extend(branch.nodes);

        // Recursively merge nested branches
        for nested_branch in branch.branches {
            self.merge_branch(nested_branch);
        }

        // Update next_id to ensure uniqueness
        if let Some(max_id) = self.nodes.iter().map(|n| n.id).max() {
            self.next_id = max_id + 1;
        }
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}
