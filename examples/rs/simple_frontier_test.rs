use dagex::{Graph, GraphData, NodeFunction};
use std::collections::HashMap;
use std::sync::Arc;

fn main() {
    println!("Testing frontier-based variant dispatch...\n");

    let mut graph = Graph::new();

    // Add a source node
    graph.add(
        Arc::new(|_| {
            let mut result = HashMap::new();
            result.insert("x".to_string(), GraphData::int(10));
            result
        }),
        Some("Source"),
        None,
        Some(vec![("x", "data")]),
    );

    // Test: variants should create multiple paths
    let factors: Vec<NodeFunction> = vec![
        Arc::new(|inputs: &HashMap<String, GraphData>| {
            let val = inputs.get("input").unwrap().as_int().unwrap();
            let mut result = HashMap::new();
            result.insert("scaled".to_string(), GraphData::int(val * 2));
            println!("  Variant 2x: {} -> {}", val, val * 2);
            result
        }),
        Arc::new(|inputs: &HashMap<String, GraphData>| {
            let val = inputs.get("input").unwrap().as_int().unwrap();
            let mut result = HashMap::new();
            result.insert("scaled".to_string(), GraphData::int(val * 3));
            println!("  Variant 3x: {} -> {}", val, val * 3);
            result
        }),
        Arc::new(|inputs: &HashMap<String, GraphData>| {
            let val = inputs.get("input").unwrap().as_int().unwrap();
            let mut result = HashMap::new();
            result.insert("scaled".to_string(), GraphData::int(val * 5));
            println!("  Variant 5x: {} -> {}", val, val * 5);
            result
        }),
    ];

    graph.variants(
        factors,
        Some("Scale"),
        Some(vec![("data", "input")]),
        Some(vec![("scaled", "result")]),
    );

    // Add a downstream node AFTER variants - this should connect to ALL variants
    graph.add(
        Arc::new(|inputs: &HashMap<String, GraphData>| {
            let val = inputs.get("r").unwrap().as_int().unwrap();
            let mut result = HashMap::new();
            result.insert("final".to_string(), GraphData::int(val + 100));
            println!("  Downstream: {} + 100 = {}", val, val + 100);
            result
        }),
        Some("Downstream"),
        Some(vec![("result", "r")]),
        Some(vec![("final", "output")]),
    );

    let dag = graph.build();
    println!("\nMermaid diagram:");
    println!("{}", dag.to_mermaid());
    
    println!("\nExecuting...");
    let results = dag.execute(true, None);
    
    println!("\nResults:");
    for (key, value) in &results {
        println!("  {}: {:?}", key, value);
    }

    // Count nodes to verify replication
    println!("\nNode analysis:");
    println!("Total nodes: {}", dag.nodes().len());
    
    let scale_nodes = dag.nodes().iter()
        .filter(|n| n.label.as_ref().map_or(false, |l| l.contains("Scale")))
        .count();
    println!("Scale nodes: {}", scale_nodes);
    
    let downstream_nodes = dag.nodes().iter()
        .filter(|n| n.label.as_ref().map_or(false, |l| l.contains("Downstream")))
        .count();
    println!("Downstream nodes: {}", downstream_nodes);

    println!("\nExpected behavior:");
    println!("- Should see 3 Scale variants (2x, 3x, 5x)");
    println!("- Should see 3 Downstream nodes (one for each variant)"); 
    println!("- Total: 1 Source + 3 Scale + 3 Downstream = 7 nodes");
    if dag.nodes().len() == 7 {
        println!("✅ Frontier-based replication working correctly!");
    } else {
        println!("❌ Expected 7 nodes, got {}", dag.nodes().len());
    }
}