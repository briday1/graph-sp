use dagex::{Graph, GraphData};
use std::collections::HashMap;

fn create_multiple_outputs(_: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut result = HashMap::new();
    
    // Create several large datasets
    let data_a: Vec<f64> = (0..1_000_000).map(|i| i as f64).collect();
    let data_b: Vec<f64> = (0..1_000_000).map(|i| (i * 2) as f64).collect();
    let data_c: Vec<f64> = (0..1_000_000).map(|i| (i * 3) as f64).collect();
    let data_d: Vec<f64> = (0..1_000_000).map(|i| (i * 4) as f64).collect();
    
    let size_mb = std::mem::size_of_val(&*data_a) / 1_000_000;
    println!("Source: Created 4 datasets, each {} MB (total ~{} MB)", size_mb, size_mb * 4);
    
    result.insert("data_a".to_string(), GraphData::float_vec(data_a));
    result.insert("data_b".to_string(), GraphData::float_vec(data_b));
    result.insert("data_c".to_string(), GraphData::float_vec(data_c));
    result.insert("data_d".to_string(), GraphData::float_vec(data_d));
    
    result
}

fn node_wants_only_a(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    println!("\nNode_A received {} inputs:", inputs.len());
    for key in inputs.keys() {
        if let Some(vec) = inputs.get(key).and_then(|d| d.as_float_vec()) {
            let size_mb = std::mem::size_of_val(&**vec) / 1_000_000;
            println!("  - {}: {} MB", key, size_mb);
        }
    }
    
    let mut result = HashMap::new();
    if let Some(data) = inputs.get("data_a").and_then(|d| d.as_float_vec()) {
        result.insert("result_a".to_string(), GraphData::float(data.iter().sum()));
    }
    result
}

fn node_wants_only_b(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    println!("\nNode_B received {} inputs:", inputs.len());
    for key in inputs.keys() {
        if let Some(vec) = inputs.get(key).and_then(|d| d.as_float_vec()) {
            let size_mb = std::mem::size_of_val(&**vec) / 1_000_000;
            println!("  - {}: {} MB", key, size_mb);
        }
    }
    
    let mut result = HashMap::new();
    if let Some(data) = inputs.get("data_b").and_then(|d| d.as_float_vec()) {
        result.insert("result_b".to_string(), GraphData::float(data.iter().sum()));
    }
    result
}

fn node_wants_c_and_d(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    println!("\nNode_CD received {} inputs:", inputs.len());
    for key in inputs.keys() {
        if let Some(vec) = inputs.get(key).and_then(|d| d.as_float_vec()) {
            let size_mb = std::mem::size_of_val(&**vec) / 1_000_000;
            println!("  - {}: {} MB", key, size_mb);
        }
    }
    
    let mut result = HashMap::new();
    if let Some(data_c) = inputs.get("data_c").and_then(|d| d.as_float_vec()) {
        if let Some(data_d) = inputs.get("data_d").and_then(|d| d.as_float_vec()) {
            result.insert("result_cd".to_string(), 
                GraphData::float(data_c.iter().sum::<f64>() + data_d.iter().sum::<f64>()));
        }
    }
    result
}

fn node_wants_nothing(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    println!("\nNode_Nothing received {} inputs:", inputs.len());
    for key in inputs.keys() {
        println!("  - {}", key);
    }
    
    let mut result = HashMap::new();
    result.insert("constant".to_string(), GraphData::float(42.0));
    result
}

fn main() {
    println!("=== Selective Data Access Test ===\n");
    println!("Testing whether nodes only receive data they request via input_mapping\n");
    println!("Context has 4 large datasets (data_a, data_b, data_c, data_d)");
    println!("Each node requests different subsets via input_mapping\n");
    
    let mut graph = Graph::new();
    
    // Source creates 4 datasets
    graph.add(
        std::sync::Arc::new(create_multiple_outputs),
        Some("Source"),
        None,
        Some(vec![
            ("data_a", "data_a"),
            ("data_b", "data_b"),
            ("data_c", "data_c"),
            ("data_d", "data_d"),
        ])
    );
    
    // Node only wants data_a
    graph.add(
        std::sync::Arc::new(node_wants_only_a),
        Some("Node_A"),
        Some(vec![("data_a", "data_a")]),  // ← Only requests data_a!
        Some(vec![("result_a", "result_a")])
    );
    
    // Node only wants data_b
    graph.add(
        std::sync::Arc::new(node_wants_only_b),
        Some("Node_B"),
        Some(vec![("data_b", "data_b")]),  // ← Only requests data_b!
        Some(vec![("result_b", "result_b")])
    );
    
    // Node wants data_c and data_d
    graph.add(
        std::sync::Arc::new(node_wants_c_and_d),
        Some("Node_CD"),
        Some(vec![("data_c", "data_c"), ("data_d", "data_d")]),  // ← Requests both
        Some(vec![("result_cd", "result_cd")])
    );
    
    // Node wants nothing from context
    graph.add(
        std::sync::Arc::new(node_wants_nothing),
        Some("Node_Nothing"),
        None,  // ← Requests nothing!
        Some(vec![("constant", "constant")])
    );
    
    let dag = graph.build();
    
    println!("\n--- Executing DAG ---");
    let _result = dag.execute(false, None);
    
    println!("\n=== Analysis ===");
    println!("EXPECTED:");
    println!("  - Node_A should receive ONLY data_a (1 input, ~8 MB)");
    println!("  - Node_B should receive ONLY data_b (1 input, ~8 MB)");
    println!("  - Node_CD should receive ONLY data_c and data_d (2 inputs, ~16 MB)");
    println!("  - Node_Nothing should receive NOTHING (0 inputs)");
    println!("\nIf nodes receive ALL context data instead:");
    println!("  - Each would receive 4+ inputs");
    println!("  - Memory usage would be (# nodes) × (total context size)");
}
