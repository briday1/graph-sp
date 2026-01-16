//! Error types for the graph execution engine.

use thiserror::Error;

/// Result type for graph operations
pub type Result<T> = std::result::Result<T, GraphError>;

/// Errors that can occur during graph operations
#[derive(Error, Debug)]
pub enum GraphError {
    /// Node not found in the graph
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    /// Edge not found in the graph
    #[error("Edge not found: from {from} to {to}")]
    EdgeNotFound { from: String, to: String },

    /// Cycle detected in the graph
    #[error("Cycle detected in graph involving node: {0}")]
    CycleDetected(String),

    /// Invalid graph structure
    #[error("Invalid graph structure: {0}")]
    InvalidGraph(String),

    /// Port validation error
    #[error("Port validation error: {0}")]
    PortError(String),

    /// Execution error
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Data type mismatch
    #[error("Data type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    /// Missing required input
    #[error("Missing required input for node {node}: {port}")]
    MissingInput { node: String, port: String },

    /// Generic error
    #[error("{0}")]
    Other(String),
}
