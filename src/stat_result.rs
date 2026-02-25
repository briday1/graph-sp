//! `StatResult` — the statistical analogue of `ExecutionResult`.
//!
//! Returned by `Dag::predict()`.  Holds a `Distribution` for every broadcast variable
//! that was reached during the forward pass, plus per-node, per-branch, and per-variant
//! views mirroring the structure of `ExecutionResult`.

use crate::distribution::{DistContext, Distribution, PortSummary};
use crate::node::NodeId;
use std::collections::HashMap;

/// Output of `Dag::predict()`.
///
/// Every key matches a broadcast variable name from the execution layer.
/// Branch variables use the same `__branch_{id}__{var}` prefix convention.
#[derive(Debug, Clone)]
pub struct StatResult {
    /// Full distribution context — all broadcast variables reachable in the graph.
    pub dist_context: DistContext,

    /// Per-node distribution outputs keyed by broadcast variable name.
    pub node_dists: HashMap<NodeId, DistContext>,

    /// Per-branch distribution outputs (branch_id → broadcast_var → Distribution).
    pub branch_dists: HashMap<usize, DistContext>,

    /// Per-variant distribution outputs (variant_index → broadcast_var → Distribution).
    pub variant_dists: HashMap<usize, DistContext>,
}

impl StatResult {
    /// Create a new empty `StatResult`.
    pub fn new() -> Self {
        Self {
            dist_context: HashMap::new(),
            node_dists: HashMap::new(),
            branch_dists: HashMap::new(),
            variant_dists: HashMap::new(),
        }
    }

    // ── Global context accessors ──────────────────────────────────────────────

    /// Get the distribution for a broadcast variable from the global context.
    pub fn get(&self, key: &str) -> Option<&Distribution> {
        self.dist_context.get(key)
    }

    /// Returns `true` if the broadcast variable has a distribution in the result.
    pub fn contains(&self, key: &str) -> bool {
        self.dist_context.contains_key(key)
    }

    /// Compute the `PortSummary` for a broadcast variable.
    pub fn summary(&self, key: &str) -> Option<PortSummary> {
        self.dist_context.get(key).map(|d| d.summary())
    }

    /// Iterate over all (name, Distribution) entries in the global context.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Distribution)> {
        self.dist_context.iter()
    }

    // ── Per-node accessors ────────────────────────────────────────────────────

    /// Get all output distributions produced by a specific node.
    pub fn get_node_dists(&self, node_id: NodeId) -> Option<&DistContext> {
        self.node_dists.get(&node_id)
    }

    /// Get a specific output distribution from a node.
    pub fn get_from_node(&self, node_id: NodeId, key: &str) -> Option<&Distribution> {
        self.node_dists.get(&node_id)?.get(key)
    }

    // ── Per-branch accessors ──────────────────────────────────────────────────

    /// Get all output distributions from a specific branch.
    ///
    /// Keys are bare broadcast variable names (without the `__branch_N__` prefix).
    pub fn for_branch(&self, branch_id: usize) -> Option<&DistContext> {
        self.branch_dists.get(&branch_id)
    }

    /// Get a specific output distribution from a branch.
    pub fn get_from_branch(&self, branch_id: usize, key: &str) -> Option<&Distribution> {
        self.branch_dists.get(&branch_id)?.get(key)
    }

    // ── Per-variant accessors ─────────────────────────────────────────────────

    /// Get all output distributions from a specific variant (by zero-based index).
    pub fn for_variant(&self, variant_idx: usize) -> Option<&DistContext> {
        self.variant_dists.get(&variant_idx)
    }

    /// Get a specific output distribution from a variant.
    pub fn get_from_variant(&self, variant_idx: usize, key: &str) -> Option<&Distribution> {
        self.variant_dists.get(&variant_idx)?.get(key)
    }

    // ── Convenience summary prints ────────────────────────────────────────────

    /// Print a human-readable summary of all non-branch, non-internal variables.
    pub fn print_summary(&self) {
        let mut keys: Vec<&String> = self
            .dist_context
            .keys()
            .filter(|k| !k.starts_with("__branch_"))
            .collect();
        keys.sort();
        for key in keys {
            let dist = &self.dist_context[key];
            println!("  {key}: {}", dist.summary());
        }
    }
}

impl Default for StatResult {
    fn default() -> Self {
        Self::new()
    }
}
