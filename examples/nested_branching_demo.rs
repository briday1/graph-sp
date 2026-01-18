use graph_sp::Graph;
use std::collections::HashMap;

// Example: Nested Branching Pattern
// Branches can contain their own sub-branches

fn preprocess(_inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let mut outputs = HashMap::new();
    outputs.insert("clean_data".to_string(), "preprocessed_data".to_string());
    println!("✓ Preprocess: Cleaned data");
    outputs
}

fn feature_a(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let data = inputs.get("clean_data").unwrap();
    let mut outputs = HashMap::new();
    outputs.insert("features_a".to_string(), format!("feat_a[{}]", data));
    println!("  ✓ Path A: Extracted features A");
    outputs
}

fn model_a1(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let features = inputs.get("features_a").unwrap();
    let mut outputs = HashMap::new();
    outputs.insert("model_a1_out".to_string(), format!("a1_result[{}]", features));
    println!("    ✓ Path A1: Model variant 1");
    outputs
}

fn model_a2(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let features = inputs.get("features_a").unwrap();
    let mut outputs = HashMap::new();
    outputs.insert("model_a2_out".to_string(), format!("a2_result[{}]", features));
    println!("    ✓ Path A2: Model variant 2");
    outputs
}

fn feature_b(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let data = inputs.get("clean_data").unwrap();
    let mut outputs = HashMap::new();
    outputs.insert("features_b".to_string(), format!("feat_b[{}]", data));
    println!("  ✓ Path B: Extracted features B");
    outputs
}

fn model_b1(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let features = inputs.get("features_b").unwrap();
    let mut outputs = HashMap::new();
    outputs.insert("model_b1_out".to_string(), format!("b1_result[{}]", features));
    println!("    ✓ Path B1: Model variant 1");
    outputs
}

fn ensemble(_inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    println!("\n✓ Ensemble: Combined all model outputs");
    let mut outputs = HashMap::new();
    outputs.insert("final".to_string(), "ensemble_prediction".to_string());
    outputs
}

fn main() {
    println!("\n=== Example: Nested Branching ===\n");
    
    let mut graph = Graph::new();
    
    // Main preprocessing
    graph.add(preprocess, Some("Preprocess"), None, Some(vec!["clean_data"]));
    
    // Branch A with nested sub-branches
    let mut branch_a = Graph::new();
    branch_a.add(feature_a, Some("Feature A"), Some(vec!["clean_data"]), Some(vec!["features_a"]));
    
    // Sub-branches within branch A
    let mut sub_a1 = Graph::new();
    sub_a1.add(model_a1, Some("Model A1"), Some(vec!["features_a"]), Some(vec!["model_a1_out"]));
    
    let mut sub_a2 = Graph::new();
    sub_a2.add(model_a2, Some("Model A2"), Some(vec!["features_a"]), Some(vec!["model_a2_out"]));
    
    branch_a.branch(sub_a1);
    branch_a.branch(sub_a2);
    
    // Branch B with nested sub-branch
    let mut branch_b = Graph::new();
    branch_b.add(feature_b, Some("Feature B"), Some(vec!["clean_data"]), Some(vec!["features_b"]));
    
    let mut sub_b1 = Graph::new();
    sub_b1.add(model_b1, Some("Model B1"), Some(vec!["features_b"]), Some(vec!["model_b1_out"]));
    
    branch_b.branch(sub_b1);
    
    // Add both main branches
    graph.branch(branch_a);
    graph.branch(branch_b);
    
    // Merge everything with ensemble function
    graph.merge(
        ensemble,
        Some("Ensemble"),
        vec!["model_a1_out", "model_a2_out", "model_b1_out"],
        Some(vec!["final"])
    );
    
    println!("\nGraph structure:");
    println!("                      ┌─→ Model A1");
    println!("              ┌─→ Feature A ─┤");
    println!("              |       └─→ Model A2");
    println!("  Preprocess ─┤");
    println!("              |");
    println!("              └─→ Feature B ─→ Model B1");
    println!("                      ↓");
    println!("                  (merge) → Ensemble");
    
    println!("\n--- Building and executing ---\n");
    let dag = graph.build();
    dag.execute();
    
    println!("\n=== Mermaid Visualization ===\n");
    println!("{}", dag.to_mermaid());
}
