//! Node representation and execution

use std::collections::HashMap;
use std::sync::Arc;

/// Unique identifier for a node
pub type NodeId = usize;

/// Type alias for node execution functions
/// Takes broadcast variables and variant parameters as input, returns output variables
pub type NodeFunction = Arc<dyn Fn(&HashMap<String, String>, &HashMap<String, String>) -> HashMap<String, String> + Send + Sync>;

/// Represents a node in the graph
#[derive(Clone)]
pub struct Node {
    /// Unique identifier
    pub id: NodeId,
    /// Optional label for visualization
    pub label: Option<String>,
    /// Function to execute
    pub function: NodeFunction,
    /// Broadcast variable names this node consumes from the graph
    pub broadcast_vars: Vec<String>,
    /// Output variable names this node produces
    pub output_vars: Vec<String>,
    /// Nodes that this node depends on (connected from)
    pub dependencies: Vec<NodeId>,
    /// Whether this node is part of a branch
    pub is_branch: bool,
    /// Variant index if this is part of a variant sweep
    pub variant_index: Option<usize>,
    /// Variant parameters for this node (param_name -> value)
    pub variant_params: HashMap<String, String>,
}

impl Node {
    /// Create a new node
    pub fn new(
        id: NodeId,
        function: NodeFunction,
        label: Option<String>,
        broadcast_vars: Vec<String>,
        output_vars: Vec<String>,
    ) -> Self {
        Self {
            id,
            label,
            function,
            broadcast_vars,
            output_vars,
            dependencies: Vec::new(),
            is_branch: false,
            variant_index: None,
            variant_params: HashMap::new(),
        }
    }

    /// Execute this node with the given context
    pub fn execute(&self, context: &HashMap<String, String>) -> HashMap<String, String> {
        // Filter context to only include broadcast vars this node needs
        let inputs: HashMap<String, String> = self
            .broadcast_vars
            .iter()
            .filter_map(|var| context.get(var).map(|val| (var.clone(), val.clone())))
            .collect();

        // Execute function with both inputs and variant parameters
        (self.function)(&inputs, &self.variant_params)
    }

    /// Get display name for this node
    pub fn display_name(&self) -> String {
        self.label
            .as_ref()
            .map(|l| l.clone())
            .unwrap_or_else(|| format!("Node {}", self.id))
    }
}
