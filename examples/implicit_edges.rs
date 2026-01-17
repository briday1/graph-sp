//! Example demonstrating implicit edge mapping (auto-connect).
//!
//! This shows how to build graphs WITHOUT explicit add_edge() calls.
//! Edges are automatically created by matching port names between nodes.

use graph_sp::core::{Graph, Node, NodeConfig, Port, PortData};
use graph_sp::executor::Executor;
use graph_sp::inspector::Inspector;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Graph-SP Example: Implicit Edge Mapping ===\n");

    // Example 1: Simple pipeline with matching port names
    println!("Example 1: Simple Pipeline (auto-connected)");
    println!("{}", "-".repeat(50));

    let mut graph = Graph::new();

    // Source node with output port "data"
    let source = NodeConfig::new(
        "source",
        "Data Source",
        vec![],
        vec![Port::new("data", "Data Output")],
        Arc::new(|_: &HashMap<String, PortData>| {
            println!("[source] Generating data...");
            Ok(HashMap::from([(
                "data".to_string(),
                PortData::List(vec![
                    PortData::Int(1),
                    PortData::Int(2),
                    PortData::Int(3),
                    PortData::Int(4),
                    PortData::Int(5),
                ]),
            )]))
        }),
    );

    // Processor node with input port "data" (matches source!) and output port "result"
    let processor = NodeConfig::new(
        "processor",
        "Data Processor",
        vec![Port::new("data", "Data Input")],
        vec![Port::new("result", "Result Output")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            println!("[processor] Processing data...");
            if let Some(PortData::List(data)) = inputs.get("data") {
                let doubled: Vec<PortData> = data
                    .iter()
                    .map(|item| {
                        if let PortData::Int(val) = item {
                            PortData::Int(val * 2)
                        } else {
                            item.clone()
                        }
                    })
                    .collect();
                Ok(HashMap::from([(
                    "result".to_string(),
                    PortData::List(doubled),
                )]))
            } else {
                Ok(HashMap::new())
            }
        }),
    );

    // Sink node with input port "result" (matches processor!)
    let sink = NodeConfig::new(
        "sink",
        "Result Sink",
        vec![Port::new("result", "Result Input")],
        vec![],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            println!("[sink] Receiving result...");
            if let Some(result) = inputs.get("result") {
                println!("[sink] Final result: {}", result);
            }
            Ok(HashMap::new())
        }),
    );

    graph.add_node(Node::new(source))?;
    graph.add_node(Node::new(processor))?;
    graph.add_node(Node::new(sink))?;

    // NO add_edge() calls! Auto-connect instead
    let edges_created = graph.auto_connect()?;
    println!(
        "✓ Auto-connected {} edges based on port name matching!\n",
        edges_created
    );

    // Validate
    println!("Validating graph...");
    graph.validate()?;
    println!("✓ Graph is valid (no cycles detected)\n");

    // Analyze
    println!("=== Graph Analysis ===");
    let analysis = Inspector::analyze(&graph);
    println!("Nodes: {}", analysis.node_count);
    println!("Edges: {}", analysis.edge_count);
    println!("Depth: {}", analysis.depth);
    println!("Summary: {}\n", analysis.summary());

    // Visualize
    println!("=== Mermaid Diagram ===");
    let mermaid = Inspector::to_mermaid(&graph)?;
    println!("{}", mermaid);

    // Execute
    println!("=== Executing Graph ===");
    let executor = Executor::new();
    let result = executor.execute(&mut graph).await?;
    println!("✓ Execution completed!\n");

    // Example 2: Parallel branches with implicit connections
    println!("\nExample 2: Parallel Branches (auto-connected)");
    println!("{}", "-".repeat(50));

    let mut graph2 = Graph::new();

    // Source with output "value"
    let source2 = NodeConfig::new(
        "source",
        "Value Source",
        vec![],
        vec![Port::new("value", "Value")],
        Arc::new(|_: &HashMap<String, PortData>| {
            println!("[source] Generating value...");
            Ok(HashMap::from([("value".to_string(), PortData::Int(100))]))
        }),
    );

    // Branch A: input "value", output "branch_a_out"
    let branch_a = NodeConfig::new(
        "branch_a",
        "Branch A\\n(×2)", // Multi-line label!
        vec![Port::new("value", "Input")],
        vec![Port::new("branch_a_out", "Output")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            println!("[branch_a] Processing...");
            if let Some(PortData::Int(val)) = inputs.get("value") {
                Ok(HashMap::from([(
                    "branch_a_out".to_string(),
                    PortData::Int(val * 2),
                )]))
            } else {
                Ok(HashMap::new())
            }
        }),
    );

    // Branch B: input "value", output "branch_b_out"
    let branch_b = NodeConfig::new(
        "branch_b",
        "Branch B\\n(+50)", // Multi-line label!
        vec![Port::new("value", "Input")],
        vec![Port::new("branch_b_out", "Output")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            println!("[branch_b] Processing...");
            if let Some(PortData::Int(val)) = inputs.get("value") {
                Ok(HashMap::from([(
                    "branch_b_out".to_string(),
                    PortData::Int(val + 50),
                )]))
            } else {
                Ok(HashMap::new())
            }
        }),
    );

    // Merger: inputs "branch_a_out" and "branch_b_out", output "final"
    let merger = NodeConfig::new(
        "merger",
        "Result Merger",
        vec![
            Port::new("branch_a_out", "Branch A"),
            Port::new("branch_b_out", "Branch B"),
        ],
        vec![Port::new("final", "Final Result")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            println!("[merger] Merging...");
            let a = if let Some(PortData::Int(val)) = inputs.get("branch_a_out") {
                *val
            } else {
                0
            };
            let b = if let Some(PortData::Int(val)) = inputs.get("branch_b_out") {
                *val
            } else {
                0
            };
            Ok(HashMap::from([(
                "final".to_string(),
                PortData::Int(a + b),
            )]))
        }),
    );

    // Collector: input "final"
    let collector = NodeConfig::new(
        "collector",
        "Result Collector",
        vec![Port::new("final", "Final")],
        vec![],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            if let Some(PortData::Int(val)) = inputs.get("final") {
                println!("[collector] Final result: {}", val);
            }
            Ok(HashMap::new())
        }),
    );

    graph2.add_node(Node::new(source2))?;
    graph2.add_node(Node::new(branch_a))?;
    graph2.add_node(Node::new(branch_b))?;
    graph2.add_node(Node::new(merger))?;
    graph2.add_node(Node::new(collector))?;

    // Auto-connect based on port names
    let edges_created2 = graph2.auto_connect()?;
    println!("✓ Auto-connected {} edges!\n", edges_created2);

    // Validate
    graph2.validate()?;
    println!("✓ Graph is valid!\n");

    // Show Mermaid with parallel groups and multi-line labels
    println!("=== Mermaid Diagram (with parallel groups & multi-line labels) ===");
    let mermaid2 = Inspector::to_mermaid(&graph2)?;
    println!("{}", mermaid2);

    // Execute
    println!("=== Executing Graph ===");
    let result2 = executor.execute(&mut graph2).await?;
    println!("✓ Execution completed!\n");

    println!("=== Example Complete ===");
    println!("\nKey Features Demonstrated:");
    println!("  ✓ Implicit edge mapping (no add_edge() needed)");
    println!("  ✓ Port name matching for automatic connections");
    println!("  ✓ Multi-line labels in Mermaid (\\n → <br/>)");
    println!("  ✓ Parallel group detection and visualization");
    println!("  ✓ Fan-out/fan-in patterns auto-detected");

    Ok(())
}
