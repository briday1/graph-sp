use graph_sp::Graph;
use std::collections::HashMap;

// Example 1: Simple Fan-Out Pattern
// One source feeds multiple parallel processing paths

fn data_source(_inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let mut outputs = HashMap::new();
    outputs.insert("raw_data".to_string(), "sensor_reading_42".to_string());
    println!("✓ Source: Generated raw_data");
    outputs
}

fn process_alpha(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let data = inputs.get("raw_data").unwrap();
    let mut outputs = HashMap::new();
    outputs.insert("alpha_result".to_string(), format!("{}_alpha", data));
    println!("  ✓ Branch Alpha: Processed {}", data);
    outputs
}

fn process_beta(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let data = inputs.get("raw_data").unwrap();
    let mut outputs = HashMap::new();
    outputs.insert("beta_result".to_string(), format!("{}_beta", data));
    println!("  ✓ Branch Beta: Processed {}", data);
    outputs
}

fn process_gamma(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let data = inputs.get("raw_data").unwrap();
    let mut outputs = HashMap::new();
    outputs.insert("gamma_result".to_string(), format!("{}_gamma", data));
    println!("  ✓ Branch Gamma: Processed {}", data);
    outputs
}

fn main() {
    println!("\n=== Example 1: Simple Fan-Out (3 branches) ===\n");
    
    let mut graph = Graph::new();
    
    // Single source node
    graph.add(data_source, Some("Source"), None, Some(vec!["raw_data"]));
    
    // Create three parallel branches
    let mut branch_a = Graph::new();
    branch_a.add(process_alpha, Some("Alpha"), Some(vec!["raw_data"]), Some(vec!["alpha_result"]));
    
    let mut branch_b = Graph::new();
    branch_b.add(process_beta, Some("Beta"), Some(vec!["raw_data"]), Some(vec!["beta_result"]));
    
    let mut branch_c = Graph::new();
    branch_c.add(process_gamma, Some("Gamma"), Some(vec!["raw_data"]), Some(vec!["gamma_result"]));
    
    // All three branch from the source node (sequential .branch() calls)
    graph.branch(branch_a);
    graph.branch(branch_b);
    graph.branch(branch_c);
    
    println!("\nGraph structure (3 parallel paths from source):");
    println!("           ┌─→ Alpha");
    println!("  Source ──┼─→ Beta");
    println!("           └─→ Gamma");
    
    println!("\n--- Building and executing ---\n");
    let dag = graph.build();
    dag.execute();
    
    println!("\n=== Mermaid Visualization ===\n");
    println!("{}", dag.to_mermaid());
}
