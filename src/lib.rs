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
//! use graph_sp::{Graph, GraphData};
//! use std::collections::HashMap;
//!
//! fn data_source(_: &HashMap<String, GraphData>, _: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
//!     let mut result = HashMap::new();
//!     result.insert("output".to_string(), GraphData::string("Hello, World!"));
//!     result
//! }
//!
//! fn processor(inputs: &HashMap<String, GraphData>, _: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
//!     let mut result = HashMap::new();
//!     if let Some(data) = inputs.get("input").and_then(|d| d.as_string()) {
//!         result.insert("output".to_string(), GraphData::string(data.to_uppercase()));
//!     }
//!     result
//! }
//!
//! let mut graph = Graph::new();
//! graph.add(data_source, Some("Source"), None, Some(vec![("output", "output")]));
//! graph.add(processor, Some("Processor"), Some(vec![("output", "input")]), Some(vec![("output", "output")]));
//!
//! let dag = graph.build();
//! ```

mod builder;
mod dag;
mod graph_data;
mod node;

#[cfg(feature = "python")]
mod python_bindings;

pub use builder::{Generator, Geomspace, Graph, IntoVariantValues, Linspace, Logspace};
pub use dag::{Dag, ExecutionContext, ExecutionResult};
pub use graph_data::GraphData;
pub use node::{NodeFunction, NodeId};
