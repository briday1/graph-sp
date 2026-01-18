//! Demonstration of accessing outputs after execution
//!
//! This example shows how to access execution results for:
//! - Regular sequential graphs
//! - Branched graphs (parallel execution)
//! - Variant parameter sweeps
//! - Complex graphs with multiple outputs

use graph_sp::Graph;
use std::collections::HashMap;

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  Output Access Demo");
    println!("  How to retrieve execution results");
    println!("═══════════════════════════════════════════════════════════\n");
    
    demo_simple_output_access();
    demo_branch_output_access();
    demo_variant_output_access();
    demo_multiple_outputs();
}

fn demo_simple_output_access() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 1: Simple Pipeline Output Access");
    println!("─────────────────────────────────────────────────────────\n");
    
    let mut graph = Graph::new();
    
    // Node 1: Generate initial data
    graph.add(
        |_: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            result.insert("initial_value".to_string(), "100".to_string());
            result
        },
        Some("Source"),
        None,
        Some(vec![("initial_value", "raw_data")])  // Maps initial_value → raw_data
    );
    
    // Node 2: Process the data
    graph.add(
        |inputs: &HashMap<String, String>, _| {
            let value = inputs.get("input").unwrap().parse::<i32>().unwrap();
            let mut result = HashMap::new();
            result.insert("processed".to_string(), (value * 2).to_string());
            result
        },
        Some("Process"),
        Some(vec![("raw_data", "input")]),      // Maps raw_data → input
        Some(vec![("processed", "final_result")])  // Maps processed → final_result
    );
    
    // Build and execute
    let dag = graph.build();
    let context = dag.execute();
    
    println!("📦 Execution Context (all variables):");
    for (key, value) in &context {
        println!("  {} = {}", key, value);
    }
    
    println!("\n🎯 Accessing specific outputs:");
    
    // Access by broadcast variable name (what's in the graph context)
    if let Some(raw_data) = context.get("raw_data") {
        println!("  raw_data: {}", raw_data);
    }
    
    if let Some(final_result) = context.get("final_result") {
        println!("  final_result: {}", final_result);
    }
    
    // Check if a variable exists
    if context.contains_key("final_result") {
        println!("\n✅ Final result is available in context");
    }
    
    println!();
}

fn demo_branch_output_access() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 2: Branch Output Access (Parallel Paths)");
    println!("─────────────────────────────────────────────────────────\n");
    
    let mut graph = Graph::new();
    
    // Source node
    graph.add(
        |_: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            result.insert("value".to_string(), "50".to_string());
            result
        },
        Some("Source"),
        None,
        Some(vec![("value", "shared_data")])
    );
    
    // Branch A: Statistics computation
    let mut branch_a = Graph::new();
    branch_a.add(
        |inputs: &HashMap<String, String>, _| {
            let val = inputs.get("data").unwrap();
            let mut result = HashMap::new();
            result.insert("stats_output".to_string(), format!("Stats of {}", val));
            result
        },
        Some("Stats"),
        Some(vec![("shared_data", "data")]),
        Some(vec![("stats_output", "statistics")])  // Branch A produces "statistics"
    );
    
    // Branch B: Model training
    let mut branch_b = Graph::new();
    branch_b.add(
        |inputs: &HashMap<String, String>, _| {
            let val = inputs.get("data").unwrap();
            let mut result = HashMap::new();
            result.insert("model_output".to_string(), format!("Model trained on {}", val));
            result
        },
        Some("Train"),
        Some(vec![("shared_data", "data")]),
        Some(vec![("model_output", "model")])  // Branch B produces "model"
    );
    
    // Branch C: Visualization
    let mut branch_c = Graph::new();
    branch_c.add(
        |inputs: &HashMap<String, String>, _| {
            let val = inputs.get("data").unwrap();
            let mut result = HashMap::new();
            result.insert("viz_output".to_string(), format!("Plot of {}", val));
            result
        },
        Some("Visualize"),
        Some(vec![("shared_data", "data")]),
        Some(vec![("viz_output", "visualization")])  // Branch C produces "visualization"
    );
    
    // Add branches to main graph
    graph.branch(branch_a);
    graph.branch(branch_b);
    graph.branch(branch_c);
    
    // Execute
    let dag = graph.build();
    let context = dag.execute();
    
    println!("📦 All outputs from parallel branches:");
    
    // Access each branch's output
    if let Some(stats) = context.get("statistics") {
        println!("  Branch A (Statistics): {}", stats);
    }
    
    if let Some(model) = context.get("model") {
        println!("  Branch B (Model): {}", model);
    }
    
    if let Some(viz) = context.get("visualization") {
        println!("  Branch C (Visualization): {}", viz);
    }
    
    println!("\n🔍 All variables in context:");
    for (key, value) in &context {
        println!("  {} = {}", key, value);
    }
    
    println!();
}

fn demo_variant_output_access() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 3: Variant Output Access (Parameter Sweep)");
    println!("─────────────────────────────────────────────────────────\n");
    
    let mut graph = Graph::new();
    
    // Source node
    graph.add(
        |_: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            result.insert("value".to_string(), "10.0".to_string());
            result
        },
        Some("DataSource"),
        None,
        Some(vec![("value", "input_data")])
    );
    
    // Variant nodes: scale by different factors
    // Factory function that creates a scaler for each factor
    fn make_scaler(factor: f64) -> impl Fn(&HashMap<String, String>, &HashMap<String, String>) -> HashMap<String, String> {
        move |inputs: &HashMap<String, String>, _| {
            let value = inputs.get("data").unwrap().parse::<f64>().unwrap();
            let mut result = HashMap::new();
            result.insert("scaled".to_string(), (value * factor).to_string());
            result
        }
    }
    
    graph.variant(
        make_scaler,
        vec![2.0, 3.0, 5.0],
        Some("Scale"),
        Some(vec![("input_data", "data")]),
        Some(vec![("scaled", "result")])  // Each variant produces "result"
    );
    
    // Execute
    let dag = graph.build();
    let context = dag.execute();
    
    println!("📦 Variant outputs:");
    println!("  Note: Variants with same output name overwrite each other");
    println!("  The last variant (factor=5.0) writes to 'result'\n");
    
    // Access the result (will be from the last variant)
    if let Some(result) = context.get("result") {
        println!("  result = {} (from last variant: 10.0 * 5.0)", result);
    }
    
    println!("\n💡 Tip: To preserve all variant outputs, use unique output names:");
    println!("   Option 1: Map each variant to a different broadcast variable");
    println!("   Option 2: Collect results in merge node");
    println!("   Option 3: Use variant_params or closure capture to distinguish");
    
    // Better approach: unique output names per variant
    let mut graph2 = Graph::new();
    
    graph2.add(
        |_: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            result.insert("value".to_string(), "10.0".to_string());
            result
        },
        Some("DataSource"),
        None,
        Some(vec![("value", "input_data")])
    );
    
    // Variant 1: 2x
    fn make_scaler_unique(label: &str, factor: f64) -> impl Fn(&HashMap<String, String>, &HashMap<String, String>) -> HashMap<String, String> + '_ {
        move |inputs: &HashMap<String, String>, _| {
            let value = inputs.get("data").unwrap().parse::<f64>().unwrap();
            let mut result = HashMap::new();
            result.insert(label.to_string(), (value * factor).to_string());
            result
        }
    }
    
    graph2.variant(
        |_x: &str| make_scaler_unique("scaled_2x", 2.0),
        vec!["2x"],
        Some("Scale2x"),
        Some(vec![("input_data", "data")]),
        Some(vec![("scaled_2x", "result_2x")])
    );
    
    graph2.variant(
        |_x: &str| make_scaler_unique("scaled_3x", 3.0),
        vec!["3x"],
        Some("Scale3x"),
        Some(vec![("input_data", "data")]),
        Some(vec![("scaled_3x", "result_3x")])
    );
    
    let dag2 = graph2.build();
    let context2 = dag2.execute();
    
    println!("\n✅ Better approach - unique output names:");
    if let Some(result_2x) = context2.get("result_2x") {
        println!("  result_2x = {}", result_2x);
    }
    if let Some(result_3x) = context2.get("result_3x") {
        println!("  result_3x = {}", result_3x);
    }
    
    println!();
}

fn demo_multiple_outputs() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 4: Multiple Outputs from Single Node");
    println!("─────────────────────────────────────────────────────────\n");
    
    let mut graph = Graph::new();
    
    // Node that produces multiple outputs
    graph.add(
        |_: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            result.insert("mean".to_string(), "50.5".to_string());
            result.insert("median".to_string(), "48.0".to_string());
            result.insert("stddev".to_string(), "12.3".to_string());
            result.insert("count".to_string(), "100".to_string());
            result
        },
        Some("Statistics"),
        None,
        Some(vec![
            ("mean", "stat_mean"),
            ("median", "stat_median"),
            ("stddev", "stat_stddev"),
            ("count", "sample_count")
        ])
    );
    
    // Execute
    let dag = graph.build();
    let context = dag.execute();
    
    println!("📊 Multiple outputs from single node:");
    
    // Access each output individually
    println!("  Mean:   {}", context.get("stat_mean").unwrap());
    println!("  Median: {}", context.get("stat_median").unwrap());
    println!("  StdDev: {}", context.get("stat_stddev").unwrap());
    println!("  Count:  {}", context.get("sample_count").unwrap());
    
    println!("\n📋 Complete execution context:");
    for (key, value) in &context {
        println!("  {} = {}", key, value);
    }
    
    println!("\n💡 Summary:");
    println!("  ✓ dag.execute() returns HashMap<String, String>");
    println!("  ✓ Keys are broadcast variable names (from output mappings)");
    println!("  ✓ Use context.get(\"variable_name\") to access specific outputs");
    println!("  ✓ All outputs accumulate in the context throughout execution");
    
    println!();
}
