//! Core module containing fundamental data structures and graph definitions.

pub mod data;
pub mod graph;
pub mod error;

pub use data::{GraphData, Port, PortData, PortId, NodeId};
pub use graph::{Graph, Node, Edge, NodeConfig};
pub use error::{GraphError, Result};
