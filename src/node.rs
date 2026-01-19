//! Node representation and execution

use crate::graph_data::GraphData;
use std::collections::HashMap;
use std::sync::Arc;

/// Unique identifier for a node
pub type NodeId = usize;

/// Type alias for node execution functions using GraphData
/// Takes GraphData ports and variant parameters as input, returns output ports
pub type NodeFunction = Arc<
    dyn Fn(&HashMap<String, GraphData>, &HashMap<String, GraphData>) -> HashMap<String, GraphData>
        + Send
        + Sync,
>;

/// Represents a node in the graph
#[derive(Clone)]
pub struct Node {
    /// Unique identifier
    pub id: NodeId,
    /// Optional label for visualization
    pub label: Option<String>,
    /// Function to execute
    pub function: NodeFunction,
    /// Input mapping: broadcast_var -> impl_var (what the function sees)
    pub input_mapping: HashMap<String, String>,
    /// Output mapping: impl_var -> broadcast_var (where function output goes in context)
    pub output_mapping: HashMap<String, String>,
    /// Branch ID for branch-specific variable resolution (None for main graph nodes)
    pub branch_id: Option<usize>,
    /// Nodes that this node depends on (connected from)
    pub dependencies: Vec<NodeId>,
    /// Whether this node is part of a branch
    pub is_branch: bool,
    /// Variant index if this is part of a variant sweep
    pub variant_index: Option<usize>,
    /// Variant parameters for this node (param_name -> value)
    pub variant_params: HashMap<String, GraphData>,
}

impl Node {
    /// Create a new node
    pub fn new(
        id: NodeId,
        function: NodeFunction,
        label: Option<String>,
        input_mapping: HashMap<String, String>,
        output_mapping: HashMap<String, String>,
    ) -> Self {
        Self {
            id,
            label,
            function,
            input_mapping,
            output_mapping,
            branch_id: None,
            dependencies: Vec::new(),
            is_branch: false,
            variant_index: None,
            variant_params: HashMap::new(),
        }
    }

    /// Execute this node with the given context
    pub fn execute(&self, context: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
        // Map broadcast context vars to impl vars using input_mapping
        // input_mapping: broadcast_var -> impl_var
        let inputs: HashMap<String, GraphData> = self
            .input_mapping
            .iter()
            .filter_map(|(broadcast_var, impl_var)| {
                context
                    .get(broadcast_var)
                    .map(|val| (impl_var.clone(), val.clone()))
            })
            .collect();

        // Execute function with both inputs and variant parameters
        let func_outputs = (self.function)(&inputs, &self.variant_params);

        // Map function outputs to broadcast vars using output_mapping
        // output_mapping: impl_var -> broadcast_var
        let mut context_outputs = HashMap::new();
        for (impl_var, broadcast_var) in &self.output_mapping {
            if let Some(value) = func_outputs.get(impl_var) {
                context_outputs.insert(broadcast_var.clone(), value.clone());
            }
        }

        context_outputs
    }

    /// Get display name for this node
    pub fn display_name(&self) -> String {
        self.label
            .as_ref()
            .map(|l| l.clone())
            .unwrap_or_else(|| format!("Node {}", self.id))
    }
}
