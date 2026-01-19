//! Graph builder with implicit connections API

use crate::dag::Dag;
use crate::graph_data::GraphData;
use crate::node::{Node, NodeId};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Trait for types that can be converted into variant values
pub trait IntoVariantValues {
    fn into_variant_values(self) -> Vec<String>;
}

/// Implement for Vec<String> - direct list of values
impl IntoVariantValues for Vec<String> {
    fn into_variant_values(self) -> Vec<String> {
        self
    }
}

/// Implement for Vec<&str> - direct list of string slices
impl IntoVariantValues for Vec<&str> {
    fn into_variant_values(self) -> Vec<String> {
        self.into_iter().map(|s| s.to_string()).collect()
    }
}

/// Implement for Vec<f64> - list of numeric values
impl IntoVariantValues for Vec<f64> {
    fn into_variant_values(self) -> Vec<String> {
        self.into_iter().map(|v| v.to_string()).collect()
    }
}

/// Implement for Vec<i32> - list of integer values
impl IntoVariantValues for Vec<i32> {
    fn into_variant_values(self) -> Vec<String> {
        self.into_iter().map(|v| v.to_string()).collect()
    }
}

/// Helper struct for linearly spaced values
pub struct Linspace {
    start: f64,
    end: f64,
    count: usize,
}

impl Linspace {
    pub fn new(start: f64, end: f64, count: usize) -> Self {
        Self { start, end, count }
    }
}

impl IntoVariantValues for Linspace {
    fn into_variant_values(self) -> Vec<String> {
        if self.count == 0 {
            return Vec::new();
        }

        let step = if self.count > 1 {
            (self.end - self.start) / (self.count - 1) as f64
        } else {
            0.0
        };

        (0..self.count)
            .map(|i| {
                let value = self.start + step * i as f64;
                value.to_string()
            })
            .collect()
    }
}

/// Helper struct for logarithmically spaced values
pub struct Logspace {
    start: f64,
    end: f64,
    count: usize,
}

impl Logspace {
    pub fn new(start: f64, end: f64, count: usize) -> Self {
        Self { start, end, count }
    }
}

impl IntoVariantValues for Logspace {
    fn into_variant_values(self) -> Vec<String> {
        if self.count == 0 || self.start <= 0.0 || self.end <= 0.0 {
            return Vec::new();
        }

        let log_start = self.start.ln();
        let log_end = self.end.ln();
        let step = if self.count > 1 {
            (log_end - log_start) / (self.count - 1) as f64
        } else {
            0.0
        };

        (0..self.count)
            .map(|i| {
                let value = (log_start + step * i as f64).exp();
                value.to_string()
            })
            .collect()
    }
}

/// Helper struct for geometric progression
pub struct Geomspace {
    start: f64,
    ratio: f64,
    count: usize,
}

impl Geomspace {
    pub fn new(start: f64, ratio: f64, count: usize) -> Self {
        Self {
            start,
            ratio,
            count,
        }
    }
}

impl IntoVariantValues for Geomspace {
    fn into_variant_values(self) -> Vec<String> {
        (0..self.count)
            .map(|i| {
                let value = self.start * self.ratio.powi(i as i32);
                value.to_string()
            })
            .collect()
    }
}

/// Helper struct for custom generator functions
pub struct Generator<F>
where
    F: Fn(usize) -> String,
{
    count: usize,
    generator: F,
}

impl<F> Generator<F>
where
    F: Fn(usize) -> String,
{
    pub fn new(count: usize, generator: F) -> Self {
        Self { count, generator }
    }
}

impl<F> IntoVariantValues for Generator<F>
where
    F: Fn(usize) -> String,
{
    fn into_variant_values(self) -> Vec<String> {
        (0..self.count).map(|i| (self.generator)(i)).collect()
    }
}

/// Graph builder for constructing graphs with implicit node connections
pub struct Graph {
    /// All nodes in the graph
    nodes: Vec<Node>,
    /// Counter for generating unique node IDs
    next_id: NodeId,
    /// The last added node ID (for implicit connections)
    last_node_id: Option<NodeId>,
    /// Track the last branch point for sequential .branch() calls
    last_branch_point: Option<NodeId>,
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
            last_node_id: None,
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
    /// * `function_handle` - The function to execute for this node
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
    /// Functions receive two parameters:
    /// - `inputs: &HashMap<String, GraphData>` - Mapped input variables (impl_var names)
    /// - `variant_params: &HashMap<String, GraphData>` - Variant parameter values
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
        function_handle: F,
        label: Option<&str>,
        inputs: Option<Vec<(&str, &str)>>,
        outputs: Option<Vec<(&str, &str)>>,
    ) -> &mut Self
    where
        F: Fn(
                &HashMap<String, GraphData>,
                &HashMap<String, GraphData>,
            ) -> HashMap<String, GraphData>
            + Send
            + Sync
            + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;

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

        let mut node = Node::new(
            id,
            Arc::new(function_handle),
            label.map(|s| s.to_string()),
            input_mapping,
            output_mapping,
        );

        // Implicit connection: connect to the last added node or merge targets
        if !self.merge_targets.is_empty() {
            // Connect to all merge targets
            node.dependencies.extend(self.merge_targets.iter().copied());
            self.merge_targets.clear();
        } else if let Some(prev_id) = self.last_node_id {
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
    /// * `subgraph` - A configured Graph representing the branch
    ///
    /// # Returns
    ///
    /// Returns the branch ID for use in merge operations
    pub fn branch(&mut self, mut subgraph: Graph) -> usize {
        // Assign a branch ID to this subgraph
        let branch_id = self.get_branch_id();

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
                self.branches.push((branch_id, subgraph));
                return branch_id;
            }
        };

        // Connect the first node of the subgraph to the branch point
        if let Some(first_node) = subgraph.nodes.first_mut() {
            if !first_node.dependencies.contains(&branch_point) {
                first_node.dependencies.push(branch_point);
            }
            first_node.is_branch = true;
            first_node.branch_id = Some(branch_id);
        }

        // Mark all nodes in this branch with the branch ID
        for node in &mut subgraph.nodes {
            node.branch_id = Some(branch_id);
        }

        // Store subgraph with its branch ID
        self.branches.push((branch_id, subgraph));

        branch_id
    }

    /// Create configuration sweep variants using a factory function (sigexec-style)
    ///
    /// Takes a factory function and an array of parameter values. The factory is called
    /// with each parameter value to create a node function for that variant.
    ///
    /// # Arguments
    ///
    /// * `factory` - Function that takes a parameter value and returns a node function
    /// * `param_values` - Array of parameter values to sweep over
    /// * `label` - Optional label for visualization (default: None)
    /// * `inputs` - Optional list of (broadcast_var, impl_var) tuples for inputs
    /// * `outputs` - Optional list of (impl_var, broadcast_var) tuples for outputs
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn make_scaler(factor: f64) -> impl Fn(&HashMap<String, GraphData>, &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    ///     move |inputs, _variant_params| {
    ///         let mut outputs = HashMap::new();
    ///         if let Some(val) = inputs.get("x").and_then(|d| d.as_float()) {
    ///             outputs.insert("scaled_x".to_string(), GraphData::float(val * factor));
    ///         }
    ///         outputs
    ///     }
    /// }
    ///
    /// graph.variant(
    ///     make_scaler,
    ///     vec![2.0, 3.0, 5.0],
    ///     Some("Scale"),
    ///     Some(vec![("data", "x")]),          // (broadcast, impl)
    ///     Some(vec![("scaled_x", "result")])  // (impl, broadcast)
    /// );
    /// ```
    ///
    /// # Behavior
    ///
    /// - Creates one node per parameter value
    /// - Each node is created by calling factory(param_value)
    /// - Nodes still receive both regular inputs and variant_params
    /// - All variants branch from the same point and can execute in parallel
    pub fn variant<F, P, NF>(
        &mut self,
        factory: F,
        param_values: Vec<P>,
        label: Option<&str>,
        inputs: Option<Vec<(&str, &str)>>,
        outputs: Option<Vec<(&str, &str)>>,
    ) -> &mut Self
    where
        F: Fn(P) -> NF,
        P: ToString + Clone,
        NF: Fn(
                &HashMap<String, GraphData>,
                &HashMap<String, GraphData>,
            ) -> HashMap<String, GraphData>
            + Send
            + Sync
            + 'static,
    {
        // Remember the branch point before adding variants
        let branch_point = self.last_node_id;

        // Create a variant node for each parameter value
        for (idx, param_value) in param_values.iter().enumerate() {
            // Create the node function using the factory
            let node_fn = factory(param_value.clone());

            let id = self.next_id;
            self.next_id += 1;

            // Build input_mapping: broadcast_var -> impl_var
            let input_mapping: HashMap<String, String> = inputs
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .map(|(broadcast, impl_var)| (broadcast.to_string(), impl_var.to_string()))
                .collect();

            // Build output_mapping: impl_var -> broadcast_var
            let output_mapping: HashMap<String, String> = outputs
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .map(|(impl_var, broadcast)| (impl_var.to_string(), broadcast.to_string()))
                .collect();

            let mut node = Node::new(
                id,
                Arc::new(node_fn),
                label.map(|s| format!("{} (v{})", s, idx)),
                input_mapping,
                output_mapping,
            );

            // Set variant index and param value
            node.variant_index = Some(idx);
            node.variant_params.insert(
                "param_value".to_string(),
                GraphData::from_string(&param_value.to_string()),
            );

            // Connect to branch point (all variants branch from same node)
            if let Some(bp_id) = branch_point {
                node.dependencies.push(bp_id);
                node.is_branch = true;
            }

            self.nodes.push(node);
        }

        // Don't update last_node_id - variants don't create sequential flow
        // Set last_branch_point for potential merge
        self.last_branch_point = branch_point;

        self
    }

    /// Merge multiple branches back together with a merge function
    ///
    /// After branching, use `.merge()` to bring parallel paths back to a single point.
    /// The merge function receives outputs from all specified branches and combines them.
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
        F: Fn(
                &HashMap<String, GraphData>,
                &HashMap<String, GraphData>,
            ) -> HashMap<String, GraphData>
            + Send
            + Sync
            + 'static,
    {
        // First, integrate all pending branches into the main graph
        let branches = std::mem::take(&mut self.branches);
        let mut branch_terminals = Vec::new();

        for (_branch_id, branch) in branches {
            if let Some(last_id) = branch.last_node_id {
                branch_terminals.push(last_id);
            }
            self.merge_branch(branch);
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
        self.last_node_id = Some(id);

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

        Dag::new(self.nodes)
    }

    /// Merge a branch builder's nodes into this builder
    fn merge_branch(&mut self, branch: Graph) {
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

        // Recursively merge nested branches
        for (_branch_id, nested_branch) in branch.branches {
            self.merge_branch(nested_branch);
        }
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}
