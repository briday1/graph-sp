//! # graph-sp
//!
//! A pure Rust graph executor supporting implicit node connections, branching, and config sweeps.
//!
//! ## Features
//!
//! - **Implicit Node Connections**: Nodes are automatically connected based on execution order
//! - **Branching**: Create parallel execution paths with `.branch()`
//! - **Config Sweeps**: Use `.variant()` to create configuration variations
//! - **DAG Optimization**: Automatic inspection and optimization of execution paths
//! - **Mermaid Visualization**: Generate diagrams with `to_mermaid()`
//!
//! ## Example
//!
//! ```rust
//! use graph_sp::Graph;
//! use std::collections::HashMap;
//!
//! fn data_source(_: &HashMap<String, String>) -> HashMap<String, String> {
//!     let mut result = HashMap::new();
//!     result.insert("output".to_string(), "Hello, World!".to_string());
//!     result
//! }
//!
//! fn processor(inputs: &HashMap<String, String>) -> HashMap<String, String> {
//!     let mut result = HashMap::new();
//!     if let Some(data) = inputs.get("input") {
//!         result.insert("output".to_string(), data.to_uppercase());
//!     }
//!     result
//! }
//!
//! let mut graph = Graph::new();
//! graph.add(data_source, Some("Source"), None, Some(vec!["output"]));
//! graph.add(processor, Some("Processor"), Some(vec!["input"]), Some(vec!["output"]));
//!
//! let dag = graph.build();
//! ```

mod builder;
mod dag;
mod node;
mod graph_data;

#[cfg(feature = "python")]
mod python_bindings;

pub use builder::{Generator, Geomspace, Graph, IntoVariantValues, Linspace, Logspace};
pub use dag::{Dag, ExecutionContext, ExecutionResult};
pub use node::{NodeFunction, NodeId};
pub use graph_data::GraphData;
