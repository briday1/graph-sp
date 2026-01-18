//! Comprehensive demonstration of all graph-sp features
//!
//! This example shows:
//! - Tuple-based variable mapping
//! - Sequential pipelines
//! - Parallel branching
//! - Branch merging
//! - Variant parameter sweeps
//! - DAG statistics and visualization

use graph_sp::{Graph, Linspace};
use std::collections::HashMap;

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  Graph-SP Comprehensive Demo");
    println!("  Pure Rust Graph Execution Engine");
    println!("═══════════════════════════════════════════════════════════\n");
    
    // Run all demonstrations
    demo_simple_pipeline();
    demo_branching();
    demo_merging();
    demo_variants();
    demo_complex_graph();
}

fn demo_simple_pipeline() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 1: Simple Sequential Pipeline");
    println!("─────────────────────────────────────────────────────────");
    
    let mut graph = Graph::new();
    
    // Data source node
    graph.add(
        |_: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            result.insert("value".to_string(), "42".to_string());
            result
        },
        Some("DataSource"),
        None,                                      // No inputs (source node)
        Some(vec![("value", "data")])              // (impl_var, broadcast_var)
    );
    
    // Processing node: multiply by 2
    graph.add(
        |inputs: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("x").and_then(|s| s.parse::<i32>().ok()) {
                result.insert("doubled".to_string(), (val * 2).to_string());
            }
            result
        },
        Some("Multiply"),
        Some(vec![("data", "x")]),                 // (broadcast_var, impl_var)
        Some(vec![("doubled", "result")])          // (impl_var, broadcast_var)
    );
    
    // Final processing: add 10
    graph.add(
        |inputs: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("num").and_then(|s| s.parse::<i32>().ok()) {
                result.insert("sum".to_string(), (val + 10).to_string());
            }
            result
        },
        Some("AddTen"),
        Some(vec![("result", "num")]),
        Some(vec![("sum", "final")])
    );
    
    let dag = graph.build();
    println!("\n📊 Execution:");
    let context = dag.execute();
    
    println!("  Input:  data = {}", context.get("data").unwrap());
    println!("  Step 1: result = {} (data * 2)", context.get("result").unwrap());
    println!("  Step 2: final = {} (result + 10)", context.get("final").unwrap());
    
    println!("\n📈 DAG Statistics:");
    let stats = dag.stats();
    println!("{}", stats.summary());
    
    println!("\n🔍 Mermaid Visualization:");
    println!("{}", dag.to_mermaid());
    println!();
}

fn demo_branching() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 2: Parallel Branching (Fan-Out)");
    println!("─────────────────────────────────────────────────────────");
    
    let mut graph = Graph::new();
    
    // Source node
    graph.add(
        |_: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            result.insert("dataset".to_string(), "100".to_string());
            result
        },
        Some("Source"),
        None,
        Some(vec![("dataset", "data")])
    );
    
    // Branch A: Compute statistics
    let mut branch_a = Graph::new();
    branch_a.add(
        |inputs: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("input") {
                result.insert("stats".to_string(), format!("Mean: {}", val));
            }
            result
        },
        Some("Statistics"),
        Some(vec![("data", "input")]),
        Some(vec![("stats", "stats_result")])
    );
    
    // Branch B: Train model
    let mut branch_b = Graph::new();
    branch_b.add(
        |inputs: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("input") {
                result.insert("model".to_string(), format!("Model trained on {}", val));
            }
            result
        },
        Some("MLModel"),
        Some(vec![("data", "input")]),
        Some(vec![("model", "model_result")])
    );
    
    // Branch C: Generate visualization
    let mut branch_c = Graph::new();
    branch_c.add(
        |inputs: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("input") {
                result.insert("plot".to_string(), format!("Plot of {}", val));
            }
            result
        },
        Some("Visualization"),
        Some(vec![("data", "input")]),
        Some(vec![("plot", "viz_result")])
    );
    
    graph.branch(branch_a);
    graph.branch(branch_b);
    graph.branch(branch_c);
    
    let dag = graph.build();
    println!("\n📊 Execution:");
    let context = dag.execute();
    
    println!("  Source: data = {}", context.get("data").unwrap());
    println!("  Branch A (Stats): {}", context.get("stats_result").unwrap());
    println!("  Branch B (Model): {}", context.get("model_result").unwrap());
    println!("  Branch C (Viz):   {}", context.get("viz_result").unwrap());
    
    println!("\n📈 DAG Statistics:");
    let stats = dag.stats();
    println!("{}", stats.summary());
    println!("  ⚡ All 3 branches can execute in parallel!");
    
    println!("\n🔍 Mermaid Visualization:");
    println!("{}", dag.to_mermaid());
    println!();
}

fn demo_merging() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 3: Branching + Merging (Fan-Out + Fan-In)");
    println!("─────────────────────────────────────────────────────────");
    
    let mut graph = Graph::new();
    
    // Source
    graph.add(
        |_: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            result.insert("value".to_string(), "50".to_string());
            result
        },
        Some("Source"),
        None,
        Some(vec![("value", "data")])
    );
    
    // Branch A: Add 10
    let mut branch_a = Graph::new();
    branch_a.add(
        |inputs: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("x").and_then(|s| s.parse::<i32>().ok()) {
                result.insert("output".to_string(), (val + 10).to_string());
            }
            result
        },
        Some("PathA (+10)"),
        Some(vec![("data", "x")]),
        Some(vec![("output", "result")])  // Both branches use same output name!
    );
    
    // Branch B: Add 20
    let mut branch_b = Graph::new();
    branch_b.add(
        |inputs: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("x").and_then(|s| s.parse::<i32>().ok()) {
                result.insert("output".to_string(), (val + 20).to_string());
            }
            result
        },
        Some("PathB (+20)"),
        Some(vec![("data", "x")]),
        Some(vec![("output", "result")])  // Both branches use same output name!
    );
    
    let branch_a_id = graph.branch(branch_a);
    let branch_b_id = graph.branch(branch_b);
    
    // Merge node: Combine results from both branches
    graph.merge(
        |inputs: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            let a = inputs.get("from_a").and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
            let b = inputs.get("from_b").and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
            result.insert("combined".to_string(), format!("{} + {} = {}", a, b, a + b));
            result
        },
        Some("Merge"),
        vec![
            (branch_a_id, "result", "from_a"),  // Map branch A's "result" to merge fn's "from_a"
            (branch_b_id, "result", "from_b")   // Map branch B's "result" to merge fn's "from_b"
        ],
        Some(vec![("combined", "final")])
    );
    
    let dag = graph.build();
    println!("\n📊 Execution:");
    let context = dag.execute();
    
    println!("  Source: data = {}", context.get("data").unwrap());
    println!("  Branch A: 50 + 10 = 60");
    println!("  Branch B: 50 + 20 = 70");
    println!("  Merged: {}", context.get("final").unwrap());
    
    println!("\n📈 DAG Statistics:");
    let stats = dag.stats();
    println!("{}", stats.summary());
    
    println!("\n🔍 Mermaid Visualization:");
    println!("{}", dag.to_mermaid());
    println!();
}

fn demo_variants() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 4: Parameter Sweep with Variants");
    println!("─────────────────────────────────────────────────────────");
    
    let mut graph = Graph::new();
    
    // Source node
    graph.add(
        |_: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            result.insert("base_value".to_string(), "10.0".to_string());
            result
        },
        Some("DataSource"),
        None,
        Some(vec![("base_value", "data")])
    );
    
    // Variant factory: Scale by different learning rates
    fn make_scaler(learning_rate: f64) -> impl Fn(&HashMap<String, String>, &HashMap<String, String>) -> HashMap<String, String> {
        move |inputs: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("input").and_then(|s| s.parse::<f64>().ok()) {
                let scaled = val * learning_rate;
                result.insert("scaled_value".to_string(), format!("{:.2}", scaled));
            }
            result
        }
    }
    
    // Create variants using Linspace for learning rate sweep
    graph.variant(
        make_scaler,
        vec![0.001, 0.01, 0.1, 1.0],
        Some("ScaleLR"),
        Some(vec![("data", "input")]),
        Some(vec![("scaled_value", "result")])
    );
    
    let dag = graph.build();
    println!("\n📊 Execution:");
    let context = dag.execute();
    
    println!("  Source: data = {}", context.get("data").unwrap());
    println!("  Variants created for learning rates: [0.001, 0.01, 0.1, 1.0]");
    println!("  (Each variant computes: data * learning_rate)");
    
    println!("\n📈 DAG Statistics:");
    let stats = dag.stats();
    println!("{}", stats.summary());
    println!("  ⚡ All {} variants can execute in parallel!", stats.variant_count);
    
    println!("\n🔍 Mermaid Visualization:");
    println!("{}", dag.to_mermaid());
    println!();
}

fn demo_complex_graph() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 5: Complex Graph (All Features Combined)");
    println!("─────────────────────────────────────────────────────────");
    
    let mut graph = Graph::new();
    
    // 1. Data ingestion
    graph.add(
        |_: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            result.insert("raw_data".to_string(), "1000".to_string());
            result
        },
        Some("Ingest"),
        None,
        Some(vec![("raw_data", "data")])
    );
    
    // 2. Preprocessing
    graph.add(
        |inputs: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("raw").and_then(|s| s.parse::<i32>().ok()) {
                result.insert("cleaned".to_string(), (val / 10).to_string());
            }
            result
        },
        Some("Preprocess"),
        Some(vec![("data", "raw")]),
        Some(vec![("cleaned", "clean_data")])
    );
    
    // 3. Branch for different analyses
    let mut stats_branch = Graph::new();
    stats_branch.add(
        |inputs: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("data") {
                result.insert("stats".to_string(), format!("Stats({})", val));
            }
            result
        },
        Some("Stats"),
        Some(vec![("clean_data", "data")]),
        Some(vec![("stats", "statistics")])
    );
    
    let mut ml_branch = Graph::new();
    ml_branch.add(
        |inputs: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("data") {
                result.insert("prediction".to_string(), format!("Pred({})", val));
            }
            result
        },
        Some("ML"),
        Some(vec![("clean_data", "data")]),
        Some(vec![("prediction", "ml_result")])
    );
    
    let stats_id = graph.branch(stats_branch);
    let ml_id = graph.branch(ml_branch);
    
    // 4. Merge branches
    graph.merge(
        |inputs: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            let stats = inputs.get("stats_in").cloned().unwrap_or_default();
            let ml = inputs.get("ml_in").cloned().unwrap_or_default();
            result.insert("report".to_string(), format!("{} & {}", stats, ml));
            result
        },
        Some("Combine"),
        vec![
            (stats_id, "statistics", "stats_in"),
            (ml_id, "ml_result", "ml_in")
        ],
        Some(vec![("report", "final_report")])
    );
    
    // 5. Final output formatting
    graph.add(
        |inputs: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            if let Some(report) = inputs.get("report") {
                result.insert("formatted".to_string(), format!("[FINAL] {}", report));
            }
            result
        },
        Some("Format"),
        Some(vec![("final_report", "report")]),
        Some(vec![("formatted", "output")])
    );
    
    let dag = graph.build();
    println!("\n📊 Execution:");
    let context = dag.execute();
    
    println!("  Step 1: Ingest      → data = {}", context.get("data").unwrap());
    println!("  Step 2: Preprocess  → clean_data = {}", context.get("clean_data").unwrap());
    println!("  Step 3: Branch A    → statistics = {}", context.get("statistics").unwrap());
    println!("          Branch B    → ml_result = {}", context.get("ml_result").unwrap());
    println!("  Step 4: Merge       → final_report = {}", context.get("final_report").unwrap());
    println!("  Step 5: Format      → output = {}", context.get("output").unwrap());
    
    println!("\n📈 DAG Statistics:");
    let stats = dag.stats();
    println!("{}", stats.summary());
    
    println!("\n📋 Execution Order:");
    for (level_idx, level) in dag.execution_levels().iter().enumerate() {
        println!("  Level {}: {} nodes", level_idx, level.len());
        for &node_id in level {
            let node = dag.nodes().iter().find(|n| n.id == node_id).unwrap();
            println!("    - {}", node.display_name());
        }
    }
    
    println!("\n🔍 Mermaid Visualization:");
    println!("{}", dag.to_mermaid());
    println!();
    
    println!("═══════════════════════════════════════════════════════════");
    println!("  Demo Complete!");
    println!("═══════════════════════════════════════════════════════════");
}
