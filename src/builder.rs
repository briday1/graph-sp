//! Graph builder with implicit connections API

use crate::dag::Dag;
use crate::graph_data::GraphData;
use crate::node::{Node, NodeId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Graph builder for constructing graphs with implicit node connections
pub struct Graph {
    /// All nodes in the graph
    nodes: Vec<Node>,
    /// Counter for generating unique node IDs
    next_id: NodeId,
    /// Current frontier node IDs (active attach points)
    frontier: Vec<NodeId>,
    /// Track the last branch points for sequential `.branch()` calls (copies of `frontier`)
    last_branch_point: Option<Vec<NodeId>>,
    /// Subgraph builders for branches with their IDs
    branches: Vec<(usize, Graph)>,
    /// Next branch ID counter
    next_branch_id: usize,
    /// Track nodes that should be merged together
    merge_targets: Vec<NodeId>,
}

impl Graph {
    /// Create a new graph
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            next_id: 0,
            frontier: Vec::new(),
            last_branch_point: None,
            branches: Vec::new(),
            next_branch_id: 1,
            merge_targets: Vec::new(),
        }
    }

    /// Get a unique branch ID for tracking branches
    fn get_branch_id(&mut self) -> usize {
        let id = self.next_branch_id;
        self.next_branch_id += 1;
        id
    }

    /// Add a node to the graph with implicit connections
    ///
    /// # Arguments
    ///
    /// * `function` - The function to execute for this node (automatically wrapped in Arc)
    /// * `label` - Optional label for visualization
    /// * `inputs` - Optional list of (broadcast_var, impl_var) tuples for inputs
    /// * `outputs` - Optional list of (impl_var, broadcast_var) tuples for outputs
    ///
    /// # Implicit Connection Behavior
    ///
    /// - The first node added has no dependencies
    /// - Subsequent nodes automatically depend on the previous node
    /// - This creates a natural sequential flow unless `.branch()` is used
    ///
    /// # Function Signature
    ///
    /// Functions receive a single parameter:
    /// - `inputs: &HashMap<String, GraphData>` - Mapped input variables (impl_var names)
    ///
    /// Functions return outputs using impl_var names, which get mapped to broadcast_var names.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Function sees "input_data", context has "data"
    /// // Function returns "output_value", gets stored as "result" in context
    /// graph.add(
    ///     process_fn,
    ///     Some("Process"),
    ///     Some(vec![("data", "input_data")]),     // (broadcast, impl)
    ///     Some(vec![("output_value", "result")])  // (impl, broadcast)
    /// );
    /// ```
    pub fn add<F>(
        &mut self,
        function: F,
        label: Option<&str>,
        inputs: Option<Vec<(&str, &str)>>,
        outputs: Option<Vec<(&str, &str)>>,
    ) -> &mut Self
    where
        F: Fn(&HashMap<String, GraphData>) -> HashMap<String, GraphData>
            + Send
            + Sync
            + 'static,
    {   
        // Build input_mapping: broadcast_var -> impl_var
        let input_mapping: HashMap<String, String> = inputs
            .unwrap_or_default()
            .iter()
            .map(|(broadcast, impl_var)| (broadcast.to_string(), impl_var.to_string()))
            .collect();

        // Build output_mapping: impl_var -> broadcast_var
        let output_mapping: HashMap<String, String> = outputs
            .unwrap_or_default()
            .iter()
            .map(|(impl_var, broadcast)| (impl_var.to_string(), broadcast.to_string()))
            .collect();

        // Determine parents for replication: if frontier is empty, we create a single node
        let parents: Vec<Option<NodeId>> = if self.frontier.is_empty() {
            vec![None]
        } else {
            self.frontier.iter().map(|&id| Some(id)).collect()
        };

        let mut created_ids: Vec<NodeId> = Vec::new();

        // Automatically wrap the function in Arc for thread-safe sharing
        let func_arc: crate::node::NodeFunction = Arc::new(function);
        for _parent in parents {
            let id = self.next_id;
            self.next_id += 1;

            let mut node = Node::new(
                id,
                Arc::clone(&func_arc),
                label.map(|s| s.to_string()),
                input_mapping.clone(),
                output_mapping.clone(),
            );

            // Connect to merge targets if present
            // For branch operations, we still use explicit parent connections
            if !self.merge_targets.is_empty() {
                node.dependencies.extend(self.merge_targets.iter().copied());
                self.merge_targets.clear();
            }
            // Note: We no longer automatically add frontier dependencies here
            // Dependencies will be resolved based on data flow in build()

            self.nodes.push(node);
            created_ids.push(id);
        }

        // Update frontier to the newly created node(s)
        self.frontier = created_ids;

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
    /// * `subgraph` - A configured Graph representing the branch
    ///
    /// # Returns
    ///
    /// Returns the branch ID for use in merge operations
    pub fn branch(&mut self, subgraph: Graph) -> usize {
        // Assign a branch ID to this subgraph (shared for all replicates)
        let branch_id = self.get_branch_id();

        // Determine the branch points (could be multiple - frontier / last_branch_point)
        let branch_points: Vec<NodeId> = if let Some(bp_vec) = self.last_branch_point.clone() {
            // Sequential .branch() calls - use the same branch point(s)
            bp_vec
        } else if !self.frontier.is_empty() {
            // Branch from current frontier nodes
            let v = self.frontier.clone();
            self.last_branch_point = Some(v.clone());
            v
        } else {
            // No previous node, subgraph starts independently
            self.branches.push((branch_id, subgraph));
            return branch_id;
        };

        // For each branch point, append a cloned copy of the subgraph and attach to the branch point
        for bp in branch_points.iter() {
            // Map old node ids to new ids
            let mut id_map: HashMap<NodeId, NodeId> = HashMap::new();
            for node in &subgraph.nodes {
                let new_id = self.next_id;
                self.next_id += 1;
                id_map.insert(node.id, new_id);
            }

            // Clone nodes with remapped ids and dependencies
            for node in &subgraph.nodes {
                let new_id = *id_map.get(&node.id).unwrap();
                let mut new_node = Node::new(
                    new_id,
                    node.function.clone(),
                    node.label.clone(),
                    node.input_mapping.clone(),
                    node.output_mapping.clone(),
                );

                // Remap dependencies: if dependency is internal to subgraph, map it; otherwise, attach to branch point
                for &dep in &node.dependencies {
                    if let Some(&mapped) = id_map.get(&dep) {
                        new_node.dependencies.push(mapped);
                    }
                }

                // Ensure first node attaches to the branch point
                if node.dependencies.is_empty() {
                    new_node.dependencies.push(*bp);
                }

                new_node.is_branch = true;
                new_node.branch_id = Some(branch_id);

                self.nodes.push(new_node);
            }
        }

        // Store original subgraph as metadata under the branch ID for reference
        self.branches.push((branch_id, subgraph));

        branch_id
    }

    /// Create variant nodes from a vector of closures
    ///
    /// Takes a vector of closures, each representing a variant of the computation.
    /// Functions are automatically wrapped in Arc for thread-safe sharing.
    ///
    /// # Arguments
    ///
    /// * `functions` - Vector of node functions (closures) - automatically wrapped in Arc
    /// * `label` - Optional label for visualization (default: None)
    /// * `inputs` - Optional list of (broadcast_var, impl_var) tuples for inputs
    /// * `outputs` - Optional list of (impl_var, broadcast_var) tuples for outputs
    ///
    /// # Example
    ///
    /// ```ignore
    /// let factors = vec![2.0, 3.0, 5.0];
    /// graph.variants(
    ///     factors.iter().map(|&factor| {
    ///         move |inputs: &HashMap<String, GraphData>| {
    ///             let mut outputs = HashMap::new();
    ///             if let Some(val) = inputs.get("x").and_then(|d| d.as_float()) {
    ///                 outputs.insert("scaled".to_string(), GraphData::float(val * factor));
    ///             }
    ///             outputs
    ///         }
    ///     }).collect(),
    ///     Some("Scale"),
    ///     Some(vec![("data", "x")]),
    ///     Some(vec![("scaled", "result")])
    /// );
    /// ```
    pub fn variants<F>(
        &mut self,
        functions: Vec<F>,
        label: Option<&str>,
        inputs: Option<Vec<(&str, &str)>>,
        outputs: Option<Vec<(&str, &str)>>,
    ) -> &mut Self
    where
        F: Fn(&HashMap<String, GraphData>) -> HashMap<String, GraphData>
            + Send
            + Sync
            + 'static,
    {
        // Determine parent attach points (frontier). If frontier is empty, treat as a single None parent
        let parents: Vec<Option<NodeId>> = if self.frontier.is_empty() {
            vec![None]
        } else {
            self.frontier.iter().map(|&id| Some(id)).collect()
        };

        // Remember previous frontier as branch point for sequential .branch() calls
        let previous_frontier = if self.frontier.is_empty() {
            None
        } else {
            Some(self.frontier.clone())
        };

        // Prepare mappings
        let input_mapping: HashMap<String, String> = inputs
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .map(|(broadcast, impl_var)| (broadcast.to_string(), impl_var.to_string()))
            .collect();

        let output_mapping: HashMap<String, String> = outputs
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .map(|(impl_var, broadcast)| (impl_var.to_string(), broadcast.to_string()))
            .collect();

        let mut created_ids: Vec<NodeId> = Vec::new();

        for (idx, node_fn) in functions.into_iter().enumerate() {
            // Automatically wrap each function in Arc and cast to trait object
            let node_fn_arc: crate::node::NodeFunction = Arc::new(node_fn);
            for parent in &parents {
                let id = self.next_id;
                self.next_id += 1;

                let mut node = Node::new(
                    id,
                    Arc::clone(&node_fn_arc),
                    label.map(|s| format!("{} (v{})", s, idx)),
                    input_mapping.clone(),
                    output_mapping.clone(),
                );

                node.variant_index = Some(idx);

                if !self.merge_targets.is_empty() {
                    node.dependencies.extend(self.merge_targets.iter().copied());
                    self.merge_targets.clear();
                } else if let Some(pid) = parent.map(|v| v) {
                    node.dependencies.push(pid);
                    node.is_branch = true;
                }

                self.nodes.push(node);
                created_ids.push(id);
            }
        }

        // New frontier is the set of created nodes
        self.frontier = created_ids;

        // Set last_branch_point to previous frontier (if any) for sequential .branch() calls
        self.last_branch_point = previous_frontier;

        self
    }

    /// Merge multiple branches back together with a merge function
    ///
    /// After branching, use `.merge()` to bring parallel paths back to a single point.
    /// The merge function receives outputs from all specified branches and combines them.
    ///
    /// Branch nodes write their outputs with branch-prefixed keys in the global context
    /// to avoid conflicts, allowing merge nodes to correctly retrieve branch-specific values.
    ///
    /// # Arguments
    ///
    /// * `merge_fn` - Function that combines outputs from all branches
    /// * `label` - Optional label for visualization
    /// * `inputs` - List of (branch_id, broadcast_var, impl_var) tuples specifying which branch outputs to merge
    /// * `outputs` - Optional list of (impl_var, broadcast_var) tuples for outputs
    ///
    /// # Example
    ///
    /// ```ignore
    /// graph.add(source_fn, Some("Source"), None, Some(vec![("src_out", "data")]));
    ///
    /// let mut branch_a = Graph::new();
    /// branch_a.add(process_a, Some("Process A"), Some(vec![("data", "input")]), Some(vec![("output", "result")]));
    ///
    /// let mut branch_b = Graph::new();
    /// branch_b.add(process_b, Some("Process B"), Some(vec![("data", "input")]), Some(vec![("output", "result")]));
    ///
    /// let branch_a_id = graph.branch(branch_a);
    /// let branch_b_id = graph.branch(branch_b);
    ///
    /// // Merge function combines results from both branches
    /// // Branches can use same output name "result", merge maps them distinctly
    /// graph.merge(
    ///     combine_fn,
    ///     Some("Combine"),
    ///     vec![
    ///         (branch_a_id, "result", "a_result"),    // (branch, broadcast, impl)
    ///         (branch_b_id, "result", "b_result")
    ///     ],
    ///     Some(vec![("combined", "final")])            // (impl, broadcast)
    /// );
    /// ```
    pub fn merge<F>(
        &mut self,
        merge_fn: F,
        label: Option<&str>,
        inputs: Vec<(usize, &str, &str)>,
        outputs: Option<Vec<(&str, &str)>>,
    ) -> &mut Self
    where
        F: Fn(&HashMap<String, GraphData>) -> HashMap<String, GraphData>
            + Send
            + Sync
            + 'static,
    {
        // First, integrate all pending branches into the main graph
        let branches = std::mem::take(&mut self.branches);
        let mut branch_terminals = Vec::new();

        for (_branch_id, branch) in branches {
            let terminals = self.merge_branch(branch);
            branch_terminals.extend(terminals);
        }

        // Create the merge node
        let id = self.next_id;
        self.next_id += 1;

        // Build input_mapping with branch-specific resolution
        // For merge, we need special handling: (branch_id, broadcast_var) -> impl_var
        // This will be handled in execution by looking at branch_id field of dependency nodes
        let input_mapping: HashMap<String, String> = inputs
            .iter()
            .map(|(branch_id, broadcast_var, impl_var)| {
                // Store as "branch_id:broadcast_var" -> impl_var for unique identification
                (
                    format!("{}:{}", branch_id, broadcast_var),
                    impl_var.to_string(),
                )
            })
            .collect();

        // Build output_mapping: impl_var -> broadcast_var
        let output_mapping: HashMap<String, String> = outputs
            .unwrap_or_default()
            .iter()
            .map(|(impl_var, broadcast)| (impl_var.to_string(), broadcast.to_string()))
            .collect();

        let mut node = Node::new(
            id,
            Arc::new(merge_fn),
            label.map(|s| s.to_string()),
            input_mapping,
            output_mapping,
        );

        // Connect to all branch terminals
        node.dependencies.extend(branch_terminals);

        self.nodes.push(node);
        // Update frontier to the merge node
        self.frontier = vec![id];

        // Reset branch point
        self.last_branch_point = None;

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
        for (_branch_id, branch) in branches {
            self.merge_branch(branch);
        }

        // Resolve data dependencies based on input/output mappings
        self.resolve_data_dependencies();

        Dag::new(self.nodes)
    }

    /// Resolve dependencies based on data flow (input/output mappings)
    /// 
    /// For each node, determine which other nodes it depends on by finding
    /// nodes that produce the broadcast variables it consumes.
    fn resolve_data_dependencies(&mut self) {
        // Build a map of which nodes produce which broadcast variables
        let mut producers: HashMap<String, Vec<NodeId>> = HashMap::new();
        
        for node in &self.nodes {
            for broadcast_var in node.output_mapping.values() {
                producers.entry(broadcast_var.clone())
                    .or_insert_with(Vec::new)
                    .push(node.id);
            }
        }

        // For each node, find its dependencies based on required inputs
        for i in 0..self.nodes.len() {
            let node = &self.nodes[i];
            let required_inputs: Vec<String> = node.input_mapping.keys().cloned().collect();
            let node_id = node.id;
            
            let mut dependencies: HashSet<NodeId> = HashSet::new();
            
            // Keep any existing dependencies (from merge_targets or branches)
            dependencies.extend(node.dependencies.iter().copied());
            
            // Add dependencies based on data flow
            for input_key in &required_inputs {
                // Handle merge node special format: "branch_id:broadcast_var"
                let broadcast_var = if input_key.contains(':') {
                    // For merge nodes, extract the broadcast_var part after ':'
                    input_key.split(':').nth(1).unwrap_or(input_key.as_str())
                } else {
                    input_key.as_str()
                };
                
                if let Some(producer_ids) = producers.get(broadcast_var) {
                    for &producer_id in producer_ids {
                        // Don't depend on ourselves
                        if producer_id != node_id {
                            dependencies.insert(producer_id);
                        }
                    }
                }
            }
            
            // Update the node's dependencies
            self.nodes[i].dependencies = dependencies.into_iter().collect();
        }
    }

    /// Merge a branch builder's nodes into this builder
    fn merge_branch(&mut self, branch: Graph) -> Vec<NodeId> {
        // Determine terminal nodes in the branch (nodes that are not dependencies of any other node within the branch)
        let _branch_node_ids: HashSet<NodeId> = branch.nodes.iter().map(|n| n.id).collect();
        let branch_deps: HashSet<NodeId> = branch
            .nodes
            .iter()
            .flat_map(|n| n.dependencies.iter().copied())
            .collect();
        let terminal_old_ids: Vec<NodeId> = branch
            .nodes
            .iter()
            .filter(|n| !branch_deps.contains(&n.id))
            .map(|n| n.id)
            .collect();

        // Create a mapping from old branch IDs to new IDs
        let mut id_mapping: HashMap<NodeId, NodeId> = HashMap::new();

        // Get the set of existing node IDs in the main graph (before merging)
        let existing_ids: HashSet<NodeId> = self.nodes.iter().map(|n| n.id).collect();

        // Renumber all nodes from the branch
        for mut node in branch.nodes {
            let old_id = node.id;
            let new_id = self.next_id;
            self.next_id += 1;

            id_mapping.insert(old_id, new_id);
            node.id = new_id;

            // Update dependencies with new IDs
            // Only remap dependencies that were part of the branch (not from main graph)
            node.dependencies = node
                .dependencies
                .iter()
                .map(|&dep_id| {
                    if existing_ids.contains(&dep_id) {
                        // This dependency is from the main graph, keep it as-is
                        dep_id
                    } else {
                        // This dependency is from the branch, remap it
                        *id_mapping.get(&dep_id).unwrap_or(&dep_id)
                    }
                })
                .collect();

            self.nodes.push(node);
        }

        // Recursively merge nested branches and collect their terminals as well
        let mut terminals: Vec<NodeId> = terminal_old_ids
            .iter()
            .filter_map(|old| id_mapping.get(old).copied())
            .collect();

        for (_branch_id, nested_branch) in branch.branches {
            let nested_terminals = self.merge_branch(nested_branch);
            terminals.extend(nested_terminals);
        }

        terminals
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}
