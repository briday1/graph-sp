//! Graph builder with implicit connections API

use crate::dag::Dag;
use crate::node::{Node, NodeId};
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
        Self { start, ratio, count }
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
    /// Subgraph builders for branches
    branches: Vec<Graph>,
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
            merge_targets: Vec::new(),
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
    ///
    /// # Function Signature
    ///
    /// Functions receive two parameters:
    /// - `inputs: &HashMap<String, String>` - Regular broadcast variables
    /// - `variant_params: &HashMap<String, String>` - Variant parameter values (e.g., {"learning_rate": "0.01"})
    pub fn add<F>(
        &mut self,
        function_handle: F,
        label: Option<&str>,
        broadcast_vars: Option<Vec<&str>>,
        output_vars: Option<Vec<&str>>,
    ) -> &mut Self
    where
        F: Fn(&std::collections::HashMap<String, String>, &std::collections::HashMap<String, String>) -> std::collections::HashMap<String, String>
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
    pub fn branch(&mut self, mut subgraph: Graph) -> &mut Self {
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

    /// Create configuration sweep variants using a factory function (sigexec-style)
    ///
    /// Takes a factory function and an array of parameter values. The factory is called
    /// with each parameter value to create a node function for that variant.
    ///
    /// # Arguments
    ///
    /// * `factory` - Function that takes a parameter value and returns a node function
    /// * `param_values` - Array of parameter values to sweep over
    /// * `label` - Optional label for visualization
    /// * `broadcast_vars` - Optional list of broadcast variables from graph context
    /// * `output_vars` - Optional list of output variables this node produces
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn make_scaler(factor: f64) -> impl Fn(&HashMap<String, String>, &HashMap<String, String>) -> HashMap<String, String> {
    ///     move |inputs, _variant_params| {
    ///         let mut outputs = HashMap::new();
    ///         if let Some(val) = inputs.get("data").and_then(|s| s.parse::<f64>().ok()) {
    ///             outputs.insert("result".to_string(), (val * factor).to_string());
    ///         }
    ///         outputs
    ///     }
    /// }
    ///
    /// graph.variant_factory(make_scaler, vec![2.0, 3.0, 5.0], Some("Scale"), Some(vec!["data"]), Some(vec!["result"]));
    /// ```
    ///
    /// # Behavior
    ///
    /// - Creates one node per parameter value
    /// - Each node is created by calling factory(param_value)
    /// - Nodes still receive both regular inputs and variant_params
    /// - All variants branch from the same point and can execute in parallel
    pub fn variant_factory<F, P, NF>(
        &mut self,
        factory: F,
        param_values: Vec<P>,
        label: Option<&str>,
        broadcast_vars: Option<Vec<&str>>,
        output_vars: Option<Vec<&str>>,
    ) -> &mut Self
    where
        F: Fn(P) -> NF,
        P: ToString + Clone,
        NF: Fn(&std::collections::HashMap<String, String>, &std::collections::HashMap<String, String>) -> std::collections::HashMap<String, String>
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

            let broadcast_vars_vec = broadcast_vars
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .map(|s| s.to_string())
                .collect();

            let output_vars_vec = output_vars
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .map(|s| s.to_string())
                .collect();

            let mut node = Node::new(
                id,
                Arc::new(node_fn),
                label.map(|s| format!("{} (v{})", s, idx)),
                broadcast_vars_vec,
                output_vars_vec,
            );

            // Set variant index and param value
            node.variant_index = Some(idx);
            node.variant_params.insert("param_value".to_string(), param_value.to_string());

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

    /// Create configuration sweep variants
    ///
    /// This method creates multiple copies of the remainder of the graph structure,
    /// each with different configuration values. It's generic and accepts multiple input types:
    ///
    /// # Accepting a List of Values
    ///
    /// ```ignore
    /// graph.variant("learning_rate", vec!["0.001", "0.01", "0.1"]);
    /// ```
    ///
    /// # Accepting a Generator Function
    ///
    /// ```ignore
    /// graph.variant("learning_rate", |i, count| {
    ///     format!("{}", 0.001 * 10_f64.powi(i as i32))
    /// }, 5);
    /// ```
    ///
    /// # Using Built-in Helpers
    ///
    /// ```ignore
    /// use graph_sp::Linspace;
    /// graph.variant("learning_rate", Linspace::new(0.001, 0.1, 10));
    /// ```
    ///
    /// # Behavior
    ///
    /// - Copies all downstream nodes for each variant
    /// - Each variant gets a unique configuration value
    /// - Variants can execute in parallel
    pub fn variant<V>(&mut self, param_name: &str, values: V) -> &mut Self
    where
        V: IntoVariantValues,
    {
        let variant_values = values.into_variant_values();
        self.apply_variants(param_name, variant_values)
    }

    /// Internal method to apply variants given a list of values
    fn apply_variants(&mut self, param_name: &str, values: Vec<String>) -> &mut Self {
        // Store the current graph state as a template
        let template_nodes = self.nodes.clone();
        let template_last_id = self.last_node_id;

        // For each variant, create a copy of the downstream graph
        for (idx, value) in values.iter().enumerate() {
            if idx == 0 {
                // First variant reuses the current graph
                // Mark nodes as part of variant 0 and set variant params
                for node in &mut self.nodes {
                    node.variant_index = Some(0);
                    node.variant_params.insert(param_name.to_string(), value.clone());
                }
            } else {
                // Subsequent variants need new copies
                let mut variant_builder = Graph::new();
                variant_builder.next_id = self.next_id;

                // Clone template nodes
                for template_node in &template_nodes {
                    let mut node = template_node.clone();
                    node.id = variant_builder.next_id;
                    variant_builder.next_id += 1;
                    node.variant_index = Some(idx);
                    // Set the variant parameter value for this variant
                    node.variant_params.insert(param_name.to_string(), value.clone());
                    
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

    /// Merge multiple branches back together
    ///
    /// After branching, use `.merge()` to bring parallel paths back to a single point.
    /// The merge node will depend on all branches that were added since the last branch point.
    ///
    /// # Example
    ///
    /// ```ignore
    /// graph.add(source_fn, Some("Source"), None, Some(vec!["data"]));
    /// 
    /// let mut branch_a = Graph::new();
    /// branch_a.add(process_a, Some("Process A"), Some(vec!["data"]), Some(vec!["result_a"]));
    /// 
    /// let mut branch_b = Graph::new();
    /// branch_b.add(process_b, Some("Process B"), Some(vec!["data"]), Some(vec!["result_b"]));
    /// 
    /// graph.branch(branch_a);
    /// graph.branch(branch_b);
    /// graph.merge(); // Merges both branches
    /// 
    /// graph.add(combine_fn, Some("Combine"), Some(vec!["result_a", "result_b"]), Some(vec!["final"]));
    /// ```
    pub fn merge(&mut self) -> &mut Self {
        // Collect all terminal nodes from branches that need to be merged
        let mut branch_terminals = Vec::new();
        
        for branch in &self.branches {
            if let Some(last_id) = branch.last_node_id {
                branch_terminals.push(last_id);
            }
        }
        
        // Store these as merge targets for the next added node
        self.merge_targets = branch_terminals;
        
        // Reset last_node_id so next add() will depend on merge targets
        self.last_node_id = None;
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
        for branch in branches {
            self.merge_branch(branch);
        }

        Dag::new(self.nodes)
    }

    /// Merge a branch builder's nodes into this builder
    fn merge_branch(&mut self, branch: Graph) {
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

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}
