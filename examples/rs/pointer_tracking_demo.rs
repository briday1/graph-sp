use dagex::{Graph, GraphData};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static CLONE_COUNT: AtomicUsize = AtomicUsize::new(0);

fn data_source(_: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut result = HashMap::new();
    let data: Vec<f64> = (0..1_000_000).map(|i| i as f64).collect();
    println!("Source: Created {} MB of data", 
             std::mem::size_of_val(&*data) / 1_000_000);
    result.insert("large_data".to_string(), GraphData::float_vec(data));
    result
}

fn node1(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut result = HashMap::new();
    if let Some(data) = inputs.get("large_data").and_then(|d| d.as_float_vec()) {
        println!("Node1: Received data pointer: {:p}, len: {}", 
                 data.as_ptr(), data.len());
        result.insert("data".to_string(), GraphData::float_vec(data.clone()));
        CLONE_COUNT.fetch_add(1, Ordering::SeqCst);
    }
    result
}

fn node2(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut result = HashMap::new();
    if let Some(data) = inputs.get("large_data").and_then(|d| d.as_float_vec()) {
        println!("Node2: Received data pointer: {:p}, len: {}", 
                 data.as_ptr(), data.len());
        result.insert("processed".to_string(), GraphData::float(data.iter().sum()));
    }
    result
}

fn node3(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut result = HashMap::new();
    if let Some(data) = inputs.get("large_data").and_then(|d| d.as_float_vec()) {
        println!("Node3: Received data pointer: {:p}, len: {}", 
                 data.as_ptr(), data.len());
        result.insert("count".to_string(), GraphData::int(data.len() as i64));
    }
    result
}

fn main() {
    println!("=== Memory Efficiency Test ===\n");
    println!("This test checks if data is being cloned or shared.\n");
    println!("If pointers are DIFFERENT: data is being cloned (inefficient)");
    println!("If pointers are SAME: data is being shared (efficient)\n");
    
    let mut graph = Graph::new();
    
    graph.add(
        std::sync::Arc::new(data_source),
        Some("Source"),
        None,
        Some(vec![("large_data", "large_data")])
    );
    
    graph.add(
        std::sync::Arc::new(node1),
        Some("Node1"),
        Some(vec![("large_data", "large_data")]),
        Some(vec![("data", "data")])
    );
    
    graph.add(
        std::sync::Arc::new(node2),
        Some("Node2"),
        Some(vec![("large_data", "large_data")]),
        Some(vec![("processed", "processed")])
    );
    
    graph.add(
        std::sync::Arc::new(node3),
        Some("Node3"),
        Some(vec![("large_data", "large_data")]),
        Some(vec![("count", "count")])
    );
    
    let dag = graph.build();
    
    println!("\n--- Executing DAG ---\n");
    let _result = dag.execute(false, None);
    
    println!("\n=== Analysis ===");
    println!("Total explicit clones in nodes: {}", CLONE_COUNT.load(Ordering::SeqCst));
    println!("\nNOTE: If the pointers are different, each node received a CLONED copy,");
    println!("meaning memory usage = (number of nodes) × (data size)");
}
