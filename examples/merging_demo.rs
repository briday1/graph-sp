use graph_sp::Graph;
use std::collections::HashMap;

// Example: Fan-Out then Fan-In Pattern
// Multiple branches process data in parallel, then merge results

fn data_source(_inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let mut outputs = HashMap::new();
    outputs.insert("dataset".to_string(), "experiment_data_v2".to_string());
    println!("✓ Source: Generated dataset");
    outputs
}

fn statistical_analysis(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let data = inputs.get("dataset").unwrap();
    let mut outputs = HashMap::new();
    outputs.insert("stats".to_string(), format!("stats_from_{}", data));
    println!("  ✓ Stats Branch: Computed statistics");
    outputs
}

fn ml_model(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let data = inputs.get("dataset").unwrap();
    let mut outputs = HashMap::new();
    outputs.insert("predictions".to_string(), format!("predictions_from_{}", data));
    println!("  ✓ ML Branch: Generated predictions");
    outputs
}

fn visualization(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let data = inputs.get("dataset").unwrap();
    let mut outputs = HashMap::new();
    outputs.insert("plots".to_string(), format!("plots_from_{}", data));
    println!("  ✓ Viz Branch: Created visualizations");
    outputs
}

fn combine_results(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let stats = inputs.get("stats").unwrap();
    let predictions = inputs.get("predictions").unwrap();
    let plots = inputs.get("plots").unwrap();
    
    let mut outputs = HashMap::new();
    outputs.insert("report".to_string(), format!("report[{}, {}, {}]", stats, predictions, plots));
    println!("\n✓ Combiner: Merged all results into report");
    outputs
}

fn publish(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let report = inputs.get("report").unwrap();
    println!("✓ Publisher: Published {}", report);
    HashMap::new()
}

fn main() {
    println!("\n=== Example: Fan-Out then Fan-In Pattern ===\n");
    
    let mut graph = Graph::new();
    
    // Single source
    graph.add(data_source, Some("Source"), None, Some(vec!["dataset"]));
    
    // Create three parallel processing branches
    let mut stats_branch = Graph::new();
    stats_branch.add(statistical_analysis, Some("Stats"), Some(vec!["dataset"]), Some(vec!["stats"]));
    
    let mut ml_branch = Graph::new();
    ml_branch.add(ml_model, Some("ML Model"), Some(vec!["dataset"]), Some(vec!["predictions"]));
    
    let mut viz_branch = Graph::new();
    viz_branch.add(visualization, Some("Visualize"), Some(vec!["dataset"]), Some(vec!["plots"]));
    
    // All three branch from source
    graph.branch(stats_branch);
    graph.branch(ml_branch);
    graph.branch(viz_branch);
    
    // MERGE: Bring all branches back together with merge function
    graph.merge(
        combine_results,
        Some("Combine"),
        vec!["stats", "predictions", "plots"],
        Some(vec!["report"])
    );
    
    // Continue after merge
    graph.add(publish, Some("Publish"), Some(vec!["report"]), None);
    
    println!("\nGraph structure:");
    println!("                 ┌─→ Stats ──────┐");
    println!("  Source ────────┼─→ ML Model ───┼─→ Combine → Publish");
    println!("                 └─→ Visualize ──┘");
    println!("                     (merge)");
    
    println!("\n--- Building and executing ---\n");
    let dag = graph.build();
    dag.execute();
    
    println!("\n=== Mermaid Visualization ===\n");
    println!("{}", dag.to_mermaid());
}
