use dagex::{Graph, GraphData};
use std::collections::HashMap;

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn create_large_data(_: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut result = HashMap::new();
    // Create 10MB of data
    let large_vec: Vec<f64> = (0..1_250_000).map(|i| i as f64).collect();
    println!("Created vector with {} elements ({} MB)", 
             large_vec.len(), 
             large_vec.len() * 8 / 1_000_000);
    result.insert("data".to_string(), GraphData::float_vec(large_vec));
    result
}

fn use_subset(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut result = HashMap::new();
    if let Some(data) = inputs.get("data").and_then(|d| d.as_float_vec()) {
        println!("Received vector with {} elements", data.len());
        // Only use first 10 elements
        let subset: Vec<f64> = data.iter().take(10).copied().collect();
        result.insert("subset".to_string(), GraphData::float_vec(subset));
    }
    result
}

fn main() {
    use tikv_jemalloc_ctl::{stats, epoch};
    
    // Get initial memory
    epoch::mib().unwrap().advance().unwrap();
    let allocated_start = stats::allocated::mib().unwrap().read().unwrap();
    println!("Initial memory: {} MB\n", allocated_start / 1_000_000);
    
    let mut graph = Graph::new();
    graph.add(create_large_data, Some("Source"), None, Some(vec![("data", "data")]));
    graph.add(use_subset, Some("User"), Some(vec![("data", "data")]), Some(vec![("subset", "subset")]));
    
    let dag = graph.build();
    
    epoch::mib().unwrap().advance().unwrap();
    let allocated_after_build = stats::allocated::mib().unwrap().read().unwrap();
    println!("After build: {} MB", allocated_after_build / 1_000_000);
    
    let result = dag.execute(false, None);
    
    epoch::mib().unwrap().advance().unwrap();
    let allocated_after_exec = stats::allocated::mib().unwrap().read().unwrap();
    println!("After execution: {} MB", allocated_after_exec / 1_000_000);
    
    println!("\nMemory increase during execution: {} MB", 
             (allocated_after_exec - allocated_after_build) / 1_000_000);
    
    if let Some(subset) = result.get("subset").and_then(|d| d.as_float_vec()) {
        println!("Output subset has {} elements", subset.len());
    }
    
    // Expected: If data is cloned, we'd see ~10-20MB increase (original + clone)
    // If data is shared via Arc, we'd see ~10MB increase only
}
