//! Core module containing fundamental data structures and graph definitions.

pub mod data;
pub mod error;
pub mod graph;

pub use data::{GraphData, NodeId, Port, PortData, PortId};
pub use error::{GraphError, Result};
pub use graph::{Edge, Graph, Node, NodeConfig, MergeConfig, MergeFunction, VariantConfig, VariantFunction};
