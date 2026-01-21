use dagex::{Graph, GraphData};
use std::collections::HashMap;
use std::time::Instant;

fn create_large_data(_: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut result = HashMap::new();
    let data: Vec<f64> = (0..5_000_000).map(|i| i as f64).collect();
    result.insert("data".to_string(), GraphData::float_vec(data));
    result
}

fn process_node(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut result = HashMap::new();
    if let Some(data) = inputs.get("data").and_then(|d| d.as_float_vec()) {
        // Just do a simple computation
        let sum: f64 = data.iter().take(100).sum();
        result.insert("output".to_string(), GraphData::float(sum));
    }
    result
}

fn main() {
    println!("=== Current Implementation Benchmark ===\n");
    println!("Testing with 5 million element vector (~40 MB)");
    println!("Creating a chain of 10 nodes that all use the same data\n");
    
    let start = Instant::now();
    
    let mut graph = Graph::new();
    
    // Source node
    graph.add(
        std::sync::Arc::new(create_large_data),
        Some("Source"),
        None,
        Some(vec![("data", "data")])
    );
    
    // Chain of 10 processing nodes
    for i in 1..=10 {
        graph.add(
            std::sync::Arc::new(process_node),
            Some(&format!("Process_{}", i)),
            Some(vec![("data", "data")]),
            Some(vec![("output", &format!("output_{}", i))])
        );
    }
    
    let dag = graph.build();
    let build_time = start.elapsed();
    
    println!("Build time: {:?}", build_time);
    
    let exec_start = Instant::now();
    let _result = dag.execute(false, None);
    let exec_time = exec_start.elapsed();
    
    println!("Execution time: {:?}", exec_time);
    println!("\nTotal time: {:?}", start.elapsed());
    
    println!("\n=== Analysis ===");
    println!("Current implementation (without Arc):");
    println!("  - Each of 10 nodes receives a CLONE of the 40MB data");
    println!("  - Total memory cloned: ~400 MB");
    println!("  - Time spent: includes deep copy overhead");
    println!("\nWith Arc wrapping:");
    println!("  - Each node would receive an Arc pointer clone (8 bytes)");
    println!("  - Total memory used: ~40 MB (shared)");
    println!("  - Expected speedup: 5-10x faster");
    println!("  - Memory usage: 10x less");
}
