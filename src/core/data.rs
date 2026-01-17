//! Core data structures for the graph execution engine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A unique identifier for a port
pub type PortId = String;

/// A unique identifier for a node
pub type NodeId = String;

/// Represents data that can be passed between nodes through ports
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PortData {
    /// No data (unit type)
    None,
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// Floating point value
    Float(f64),
    /// String value
    String(String),
    /// Binary data
    Bytes(Vec<u8>),
    /// JSON value
    Json(serde_json::Value),
    /// List of port data
    List(Vec<PortData>),
    /// Map of port data
    Map(HashMap<String, PortData>),
}

impl fmt::Display for PortData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PortData::None => write!(f, "None"),
            PortData::Bool(b) => write!(f, "{}", b),
            PortData::Int(i) => write!(f, "{}", i),
            PortData::Float(fl) => write!(f, "{}", fl),
            PortData::String(s) => write!(f, "\"{}\"", s),
            PortData::Bytes(b) => write!(f, "Bytes({})", b.len()),
            PortData::Json(j) => write!(f, "{}", j),
            PortData::List(l) => write!(f, "List({})", l.len()),
            PortData::Map(m) => write!(f, "Map({})", m.len()),
        }
    }
}

/// Port configuration for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    /// Broadcast name (external name for connections between nodes)
    pub broadcast_name: PortId,
    /// Implementation name (internal name used within the node function)
    pub impl_name: String,
    /// Human-readable display name
    pub display_name: String,
    /// Port description
    pub description: Option<String>,
    /// Whether this port is required
    pub required: bool,
}

impl Port {
    /// Create a new required port with separate broadcast and implementation names
    pub fn new(broadcast_name: impl Into<String>, impl_name: impl Into<String>) -> Self {
        let broadcast = broadcast_name.into();
        let impl_name = impl_name.into();
        let display_name = broadcast.clone();
        Self {
            broadcast_name: broadcast,
            impl_name,
            display_name,
            description: None,
            required: true,
        }
    }

    /// Create a port where broadcast and implementation names are the same
    pub fn simple(name: impl Into<String>) -> Self {
        let name = name.into();
        Self::new(name.clone(), name)
    }

    /// Create a new optional port
    pub fn optional(broadcast_name: impl Into<String>, impl_name: impl Into<String>) -> Self {
        let mut port = Self::new(broadcast_name, impl_name);
        port.required = false;
        port
    }

    /// Set the display name for this port
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = display_name.into();
        self
    }

    /// Set the description for this port
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Container for port data within a graph
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphData {
    /// Map from port ID to port data
    ports: HashMap<PortId, PortData>,
}

impl GraphData {
    /// Create a new empty GraphData
    pub fn new() -> Self {
        Self {
            ports: HashMap::new(),
        }
    }

    /// Set data for a port
    pub fn set(&mut self, port_id: impl Into<PortId>, data: PortData) {
        self.ports.insert(port_id.into(), data);
    }

    /// Get data from a port
    pub fn get(&self, port_id: &str) -> Option<&PortData> {
        self.ports.get(port_id)
    }

    /// Remove data from a port
    pub fn remove(&mut self, port_id: &str) -> Option<PortData> {
        self.ports.remove(port_id)
    }

    /// Check if a port has data
    pub fn has(&self, port_id: &str) -> bool {
        self.ports.contains_key(port_id)
    }

    /// Get all port IDs
    pub fn port_ids(&self) -> impl Iterator<Item = &PortId> {
        self.ports.keys()
    }

    /// Clear all port data
    pub fn clear(&mut self) {
        self.ports.clear();
    }

    /// Get the number of ports with data
    pub fn len(&self) -> usize {
        self.ports.len()
    }

    /// Check if the GraphData is empty
    pub fn is_empty(&self) -> bool {
        self.ports.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_creation() {
        let port = Port::new("input1", "input1");
        assert_eq!(port.broadcast_name, "input1");
        assert_eq!(port.display_name, "input1");
        assert!(port.required);
        assert!(port.description.is_none());
    }

    #[test]
    fn test_optional_port() {
        let port = Port::optional("opt1", "Optional 1").with_description("An optional port");
        assert!(!port.required);
        assert_eq!(port.description.unwrap(), "An optional port");
    }

    #[test]
    fn test_graph_data_operations() {
        let mut data = GraphData::new();

        // Test set and get
        data.set("port1", PortData::Int(42));
        assert!(data.has("port1"));
        assert_eq!(data.len(), 1);

        if let Some(PortData::Int(val)) = data.get("port1") {
            assert_eq!(*val, 42);
        } else {
            panic!("Expected Int(42)");
        }

        // Test remove
        let removed = data.remove("port1");
        assert!(removed.is_some());
        assert!(!data.has("port1"));
        assert!(data.is_empty());
    }

    #[test]
    fn test_port_data_types() {
        let mut data = GraphData::new();

        data.set("bool", PortData::Bool(true));
        data.set("int", PortData::Int(123));
        data.set("float", PortData::Float(3.14));
        data.set("string", PortData::String("hello".to_string()));

        assert_eq!(data.len(), 4);
        assert!(data.has("bool"));
        assert!(data.has("int"));
        assert!(data.has("float"));
        assert!(data.has("string"));
    }
}
