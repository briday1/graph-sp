//! Graph inspection and analysis tools.

use crate::core::{Graph, Result};
use std::collections::{HashMap, HashSet};

/// Graph inspector for analyzing and optimizing graphs
pub struct Inspector;

impl Inspector {
    /// Analyze a graph and return statistics
    pub fn analyze(graph: &Graph) -> GraphAnalysis {
        let node_count = graph.node_count();
        let edge_count = graph.edge_count();
        
        // Calculate depth and width
        let (depth, width) = Self::calculate_dimensions(graph);
        
        // Find source and sink nodes
        let sources = Self::find_source_nodes(graph);
        let sinks = Self::find_sink_nodes(graph);
        
        // Calculate complexity metrics
        let avg_connections = if node_count > 0 {
            edge_count as f64 / node_count as f64
        } else {
            0.0
        };

        GraphAnalysis {
            node_count,
            edge_count,
            depth,
            width,
            source_nodes: sources,
            sink_nodes: sinks,
            avg_connections_per_node: avg_connections,
            has_cycles: graph.validate().is_err(),
        }
    }

    /// Find source nodes (nodes with no incoming edges)
    fn find_source_nodes(graph: &Graph) -> Vec<String> {
        graph
            .nodes()
            .iter()
            .filter(|node| {
                graph.incoming_edges(&node.config.id)
                    .map(|edges| edges.is_empty())
                    .unwrap_or(false)
            })
            .map(|node| node.config.id.clone())
            .collect()
    }

    /// Find sink nodes (nodes with no outgoing edges)
    fn find_sink_nodes(graph: &Graph) -> Vec<String> {
        graph
            .nodes()
            .iter()
            .filter(|node| {
                graph.outgoing_edges(&node.config.id)
                    .map(|edges| edges.is_empty())
                    .unwrap_or(false)
            })
            .map(|node| node.config.id.clone())
            .collect()
    }

    /// Calculate graph depth (longest path) and width (max nodes at same level)
    fn calculate_dimensions(graph: &Graph) -> (usize, usize) {
        // Get topological order to determine levels
        let order = match graph.topological_order() {
            Ok(o) => o,
            Err(_) => return (0, 0),
        };

        if order.is_empty() {
            return (0, 0);
        }

        // Calculate level for each node
        let mut levels: HashMap<String, usize> = HashMap::new();
        
        for node_id in &order {
            // Find max level of predecessors
            let incoming = graph.incoming_edges(node_id).unwrap_or_default();
            let max_pred_level = incoming
                .iter()
                .filter_map(|edge| levels.get(&edge.from_node))
                .max()
                .copied()
                .unwrap_or(0);
            
            let level = if incoming.is_empty() { 
                0 
            } else { 
                max_pred_level + 1 
            };
            levels.insert(node_id.clone(), level);
        }

        let depth = levels.values().max().copied().unwrap_or(0) + 1;

        // Calculate width (max nodes at same level)
        let mut level_counts: HashMap<usize, usize> = HashMap::new();
        for level in levels.values() {
            *level_counts.entry(*level).or_insert(0) += 1;
        }
        let width = level_counts.values().max().copied().unwrap_or(0);

        (depth, width)
    }

    /// Suggest optimizations for the graph
    pub fn suggest_optimizations(graph: &Graph) -> Vec<Optimization> {
        let mut suggestions = Vec::new();

        // Check for isolated nodes
        for node in graph.nodes() {
            let incoming = graph.incoming_edges(&node.config.id).unwrap_or_default();
            let outgoing = graph.outgoing_edges(&node.config.id).unwrap_or_default();
            
            if incoming.is_empty() && outgoing.is_empty() && graph.node_count() > 1 {
                suggestions.push(Optimization {
                    optimization_type: OptimizationType::RemoveIsolatedNode,
                    description: format!("Node '{}' is isolated (no connections)", node.config.id),
                    node_ids: vec![node.config.id.clone()],
                });
            }
        }

        // Check for redundant edges (multiple edges between same nodes)
        let mut connections: HashSet<(String, String)> = HashSet::new();
        for edge in graph.edges() {
            let pair = (edge.from_node.clone(), edge.to_node.clone());
            if connections.contains(&pair) {
                suggestions.push(Optimization {
                    optimization_type: OptimizationType::RemoveRedundantEdge,
                    description: format!(
                        "Multiple edges between '{}' and '{}'",
                        edge.from_node, edge.to_node
                    ),
                    node_ids: vec![edge.from_node.clone(), edge.to_node.clone()],
                });
            }
            connections.insert(pair);
        }

        suggestions
    }

    /// Visualize graph structure as a simple text representation
    pub fn visualize(graph: &Graph) -> Result<String> {
        let order = graph.topological_order()?;
        let mut output = String::new();
        
        output.push_str("Graph Structure:\n");
        output.push_str("================\n\n");
        
        for node_id in order {
            let node = graph.get_node(&node_id)?;
            output.push_str(&format!("Node: {} ({})\n", node.config.name, node.config.id));
            
            // Show inputs
            if !node.config.input_ports.is_empty() {
                output.push_str("  Inputs:\n");
                for port in &node.config.input_ports {
                    let required = if port.required { "*" } else { "" };
                    output.push_str(&format!("    - {}{} ({})\n", port.name, required, port.id));
                }
            }
            
            // Show outputs
            if !node.config.output_ports.is_empty() {
                output.push_str("  Outputs:\n");
                for port in &node.config.output_ports {
                    output.push_str(&format!("    - {} ({})\n", port.name, port.id));
                }
            }
            
            // Show connections
            let outgoing = graph.outgoing_edges(&node_id)?;
            if !outgoing.is_empty() {
                output.push_str("  Connections:\n");
                for edge in outgoing {
                    output.push_str(&format!(
                        "    - {} -> {}:{}\n",
                        edge.from_port, edge.to_node, edge.to_port
                    ));
                }
            }
            
            output.push('\n');
        }
        
        Ok(output)
    }

    /// Generate a Mermaid diagram representation of the graph
    pub fn to_mermaid(graph: &Graph) -> Result<String> {
        let mut output = String::new();
        
        output.push_str("```mermaid\n");
        output.push_str("graph TD\n");
        
        // Add nodes with styling
        for node in graph.nodes() {
            let node_id = &node.config.id;
            let node_name = &node.config.name;
            
            // Sanitize node ID for Mermaid (replace special chars)
            let safe_id = node_id.replace('-', "_").replace(' ', "_");
            
            // Style nodes based on whether they're source/sink
            let incoming = graph.incoming_edges(node_id).unwrap_or_default();
            let outgoing = graph.outgoing_edges(node_id).unwrap_or_default();
            
            if incoming.is_empty() && !outgoing.is_empty() {
                // Source node
                output.push_str(&format!("    {}[\"{}\"]\n", safe_id, node_name));
                output.push_str(&format!("    style {} fill:#e1f5ff,stroke:#01579b,stroke-width:2px\n", safe_id));
            } else if outgoing.is_empty() && !incoming.is_empty() {
                // Sink node
                output.push_str(&format!("    {}[\"{}\"]\n", safe_id, node_name));
                output.push_str(&format!("    style {} fill:#f3e5f5,stroke:#4a148c,stroke-width:2px\n", safe_id));
            } else {
                // Processing node
                output.push_str(&format!("    {}[\"{}\"]\n", safe_id, node_name));
                output.push_str(&format!("    style {} fill:#fff3e0,stroke:#e65100,stroke-width:2px\n", safe_id));
            }
        }
        
        output.push('\n');
        
        // Add edges with labels
        for edge in graph.edges() {
            let from_safe = edge.from_node.replace('-', "_").replace(' ', "_");
            let to_safe = edge.to_node.replace('-', "_").replace(' ', "_");
            let label = format!("{}→{}", edge.from_port, edge.to_port);
            
            output.push_str(&format!(
                "    {} -->|\"{}\"| {}\n",
                from_safe, label, to_safe
            ));
        }
        
        output.push_str("```\n");
        
        Ok(output)
    }
}

/// Analysis results for a graph
#[derive(Debug, Clone)]
pub struct GraphAnalysis {
    /// Total number of nodes
    pub node_count: usize,
    /// Total number of edges
    pub edge_count: usize,
    /// Maximum depth (longest path from source to sink)
    pub depth: usize,
    /// Maximum width (max nodes at same level)
    pub width: usize,
    /// Source nodes (no incoming edges)
    pub source_nodes: Vec<String>,
    /// Sink nodes (no outgoing edges)
    pub sink_nodes: Vec<String>,
    /// Average connections per node
    pub avg_connections_per_node: f64,
    /// Whether the graph has cycles
    pub has_cycles: bool,
}

impl GraphAnalysis {
    /// Get a summary string
    pub fn summary(&self) -> String {
        format!(
            "Nodes: {}, Edges: {}, Depth: {}, Width: {}, Sources: {}, Sinks: {}, Avg Connections: {:.2}, Cycles: {}",
            self.node_count,
            self.edge_count,
            self.depth,
            self.width,
            self.source_nodes.len(),
            self.sink_nodes.len(),
            self.avg_connections_per_node,
            if self.has_cycles { "Yes" } else { "No" }
        )
    }
}

/// Suggested optimization for a graph
#[derive(Debug, Clone)]
pub struct Optimization {
    /// Type of optimization
    pub optimization_type: OptimizationType,
    /// Human-readable description
    pub description: String,
    /// Node IDs involved in the optimization
    pub node_ids: Vec<String>,
}

/// Types of optimizations
#[derive(Debug, Clone, PartialEq)]
pub enum OptimizationType {
    /// Remove an isolated node
    RemoveIsolatedNode,
    /// Remove a redundant edge
    RemoveRedundantEdge,
    /// Merge nodes
    MergeNodes,
    /// Parallelize independent branches
    ParallelizeBranches,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Node, NodeConfig, Port, Edge};
    use std::sync::Arc;
    use std::collections::HashMap;

    fn dummy_function(
        _inputs: &HashMap<String, crate::core::PortData>,
    ) -> Result<HashMap<String, crate::core::PortData>> {
        Ok(HashMap::new())
    }

    #[test]
    fn test_analyze_empty_graph() {
        let graph = Graph::new();
        let analysis = Inspector::analyze(&graph);
        
        assert_eq!(analysis.node_count, 0);
        assert_eq!(analysis.edge_count, 0);
        assert_eq!(analysis.depth, 0);
        assert_eq!(analysis.width, 0);
    }

    #[test]
    fn test_find_source_and_sink_nodes() {
        let mut graph = Graph::new();
        
        // Create linear graph: source -> middle -> sink
        let config1 = NodeConfig::new(
            "source",
            "Source",
            vec![],
            vec![Port::new("out", "Output")],
            Arc::new(dummy_function),
        );
        
        let config2 = NodeConfig::new(
            "middle",
            "Middle",
            vec![Port::new("in", "Input")],
            vec![Port::new("out", "Output")],
            Arc::new(dummy_function),
        );
        
        let config3 = NodeConfig::new(
            "sink",
            "Sink",
            vec![Port::new("in", "Input")],
            vec![],
            Arc::new(dummy_function),
        );
        
        graph.add_node(Node::new(config1)).unwrap();
        graph.add_node(Node::new(config2)).unwrap();
        graph.add_node(Node::new(config3)).unwrap();
        
        graph.add_edge(Edge::new("source", "out", "middle", "in")).unwrap();
        graph.add_edge(Edge::new("middle", "out", "sink", "in")).unwrap();
        
        let analysis = Inspector::analyze(&graph);
        
        assert_eq!(analysis.source_nodes.len(), 1);
        assert_eq!(analysis.source_nodes[0], "source");
        assert_eq!(analysis.sink_nodes.len(), 1);
        assert_eq!(analysis.sink_nodes[0], "sink");
        assert_eq!(analysis.depth, 3);
        assert_eq!(analysis.width, 1);
    }

    #[test]
    fn test_suggest_optimizations_isolated_node() {
        let mut graph = Graph::new();
        
        let config1 = NodeConfig::new(
            "node1",
            "Node 1",
            vec![],
            vec![],
            Arc::new(dummy_function),
        );
        
        let config2 = NodeConfig::new(
            "node2",
            "Node 2",
            vec![],
            vec![Port::new("out", "Output")],
            Arc::new(dummy_function),
        );
        
        graph.add_node(Node::new(config1)).unwrap();
        graph.add_node(Node::new(config2)).unwrap();
        
        let optimizations = Inspector::suggest_optimizations(&graph);
        
        assert!(!optimizations.is_empty());
        assert!(optimizations
            .iter()
            .any(|o| o.optimization_type == OptimizationType::RemoveIsolatedNode));
    }
}
