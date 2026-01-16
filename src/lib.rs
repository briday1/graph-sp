//! # graph-sp: A Comprehensive Rust-based DAG Execution Engine
//!
//! graph-sp is a high-performance, parallel DAG (Directed Acyclic Graph) execution engine
//! with Python bindings. It provides true parallelization, flawless graph inspection,
//! and port routing optimization.
//!
//! ## Features
//!
//! - **Port-based Architecture**: Nodes communicate through strongly-typed ports
//! - **Parallel Execution**: Automatic parallelization of independent nodes using tokio
//! - **Graph Inspection**: Comprehensive analysis and visualization tools
//! - **Cycle Detection**: Built-in DAG validation
//! - **Python Bindings**: Easy-to-use Python API via PyO3
//!
//! ## Core Components
//!
//! - `core`: Fundamental data structures (Graph, Node, Port, PortData)
//! - `executor`: Parallel execution engine
//! - `inspector`: Graph analysis and optimization tools
//! - `python`: Python bindings (optional, enabled with "python" feature)
//!
//! ## Example
//!
//! ```rust
//! use graph_sp::core::{Graph, Node, NodeConfig, Port, PortData};
//! use graph_sp::executor::Executor;
//! use std::collections::HashMap;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a graph
//!     let mut graph = Graph::new();
//!
//!     // Define a simple node that doubles its input
//!     let config = NodeConfig::new(
//!         "doubler",
//!         "Doubler Node",
//!         vec![Port::new("input", "Input Value")],
//!         vec![Port::new("output", "Output Value")],
//!         Arc::new(|inputs: &HashMap<String, PortData>| {
//!             let mut outputs = HashMap::new();
//!             if let Some(PortData::Int(val)) = inputs.get("input") {
//!                 outputs.insert("output".to_string(), PortData::Int(val * 2));
//!             }
//!             Ok(outputs)
//!         }),
//!     );
//!
//!     // Add node to graph
//!     let mut node = Node::new(config);
//!     node.set_input("input", PortData::Int(21));
//!     graph.add_node(node)?;
//!
//!     // Execute the graph
//!     let executor = Executor::new();
//!     let result = executor.execute(&mut graph).await?;
//!
//!     // Get the result
//!     if let Some(PortData::Int(val)) = result.get_output("doubler", "output") {
//!         println!("Result: {}", val); // Output: 42
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod core;
pub mod executor;
pub mod inspector;

#[cfg(feature = "python")]
pub mod python;

// Re-export commonly used types
pub use core::{Graph, Node, NodeConfig, Port, PortData, GraphData, Edge, GraphError, Result};
pub use executor::{Executor, ExecutionResult};
pub use inspector::{Inspector, GraphAnalysis, Optimization, OptimizationType};
