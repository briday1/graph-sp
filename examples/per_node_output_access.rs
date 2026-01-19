//! Comprehensive demonstration of per-node and per-branch output access
//!
//! This example shows how to:
//! - Access outputs from specific nodes
//! - Access outputs from specific branches
//! - Track execution history through per-node outputs
//! - Debug data flow by inspecting individual node results

use graph_sp::{Graph, GraphData};
use std::collections::HashMap;

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  Per-Node and Per-Branch Output Access Demo");
    println!("  Track execution results at every level");
    println!("═══════════════════════════════════════════════════════════\n");

    demo_per_node_access();
    demo_per_branch_access();
    demo_variant_per_node_access();
    demo_execution_history_tracking();
}

fn demo_per_node_access() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 1: Per-Node Output Access");
    println!("─────────────────────────────────────────────────────────\n");

    let mut graph = Graph::new();

    // Node 0: Source
    graph.add(
        |_: &HashMap<String, GraphData>, _| {
            let mut result = HashMap::new();
            result.insert("value".to_string(), GraphData::int(10));
            result
        },
        Some("Source"),
        None,
        Some(vec![("value", "initial_data")]),
    );

    // Node 1: Double
    graph.add(
        |inputs: &HashMap<String, GraphData>, _| {
            let value = inputs.get("in").and_then(|d| d.as_int()).unwrap();
            let mut result = HashMap::new();
            result.insert("doubled".to_string(), GraphData::int(value * 2));
            result
        },
        Some("Double"),
        Some(vec![("initial_data", "in")]),
        Some(vec![("doubled", "doubled_data")]),
    );

    // Node 2: Add Ten
    graph.add(
        |inputs: &HashMap<String, GraphData>, _| {
            let value = inputs.get("in").and_then(|d| d.as_int()).unwrap();
            let mut result = HashMap::new();
            result.insert("added".to_string(), GraphData::int(value + 10));
            result
        },
        Some("AddTen"),
        Some(vec![("doubled_data", "in")]),
        Some(vec![("added", "final_result")]),
    );

    // Execute and get detailed results
    let dag = graph.build();
    let result = dag.execute_detailed(false, None);

    println!("🌍 Global Context (all variables):");
    for (key, value) in &result.context {
        println!("  {} = {}", key, value.to_string_repr());
    }

    println!("\n📦 Per-Node Outputs:");
    println!("\nNode 0 (Source) outputs:");
    if let Some(outputs) = result.get_node_outputs(0) {
        for (key, value) in outputs {
            println!("  {} = {}", key, value.to_string_repr());
        }
    }

    println!("\nNode 1 (Double) outputs:");
    if let Some(outputs) = result.get_node_outputs(1) {
        for (key, value) in outputs {
            println!("  {} = {}", key, value.to_string_repr());
        }
    }

    println!("\nNode 2 (AddTen) outputs:");
    if let Some(outputs) = result.get_node_outputs(2) {
        for (key, value) in outputs {
            println!("  {} = {}", key, value.to_string_repr());
        }
    }

    println!("\n🎯 Accessing specific node outputs:");
    if let Some(value) = result.get_from_node(0, "initial_data") {
        println!("  Node 0 'initial_data': {}", value.to_string_repr());
    }
    if let Some(value) = result.get_from_node(1, "doubled_data") {
        println!("  Node 1 'doubled_data': {}", value.to_string_repr());
    }
    if let Some(value) = result.get_from_node(2, "final_result") {
        println!("  Node 2 'final_result': {}", value.to_string_repr());
    }

    println!();
}

fn demo_per_branch_access() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 2: Per-Branch Output Access");
    println!("─────────────────────────────────────────────────────────\n");

    let mut graph = Graph::new();

    // Main graph: Source node
    graph.add(
        |_: &HashMap<String, GraphData>, _| {
            let mut result = HashMap::new();
            result.insert("dataset".to_string(), GraphData::int(100));
            result
        },
        Some("Source"),
        None,
        Some(vec![("dataset", "data")]),
    );

    // Branch A: Statistics
    let mut branch_a = Graph::new();
    branch_a.add(
        |inputs: &HashMap<String, GraphData>, _| {
            let value = inputs.get("input").and_then(|d| d.as_int()).unwrap();
            let mut result = HashMap::new();
            result.insert(
                "stat_result".to_string(),
                GraphData::string(&format!("Mean of {}", value)),
            );
            result
        },
        Some("Statistics"),
        Some(vec![("data", "input")]),
        Some(vec![("stat_result", "statistics")]),
    );

    // Branch B: Model Training
    let mut branch_b = Graph::new();
    branch_b.add(
        |inputs: &HashMap<String, GraphData>, _| {
            let value = inputs.get("input").and_then(|d| d.as_int()).unwrap();
            let mut result = HashMap::new();
            result.insert(
                "model_result".to_string(),
                GraphData::string(&format!("Model trained on {}", value)),
            );
            result
        },
        Some("ModelTraining"),
        Some(vec![("data", "input")]),
        Some(vec![("model_result", "trained_model")]),
    );

    // Branch C: Visualization
    let mut branch_c = Graph::new();
    branch_c.add(
        |inputs: &HashMap<String, GraphData>, _| {
            let value = inputs.get("input").and_then(|d| d.as_int()).unwrap();
            let mut result = HashMap::new();
            result.insert(
                "viz_result".to_string(),
                GraphData::string(&format!("Plot of {}", value)),
            );
            result
        },
        Some("Visualization"),
        Some(vec![("data", "input")]),
        Some(vec![("viz_result", "plot")]),
    );

    let branch_a_id = graph.branch(branch_a);
    let branch_b_id = graph.branch(branch_b);
    let branch_c_id = graph.branch(branch_c);

    // Execute and get detailed results
    let dag = graph.build();
    let result = dag.execute_detailed(false, None);

    println!("🌍 Global Context:");
    for (key, value) in &result.context {
        println!("  {} = {}", key, value.to_string_repr());
    }

    println!("\n🌿 Per-Branch Outputs:");

    println!("\nBranch {} (Statistics) outputs:", branch_a_id);
    if let Some(outputs) = result.get_branch_outputs(branch_a_id) {
        for (key, value) in outputs {
            println!("  {} = {}", key, value.to_string_repr());
        }
    }

    println!("\nBranch {} (Model Training) outputs:", branch_b_id);
    if let Some(outputs) = result.get_branch_outputs(branch_b_id) {
        for (key, value) in outputs {
            println!("  {} = {}", key, value.to_string_repr());
        }
    }

    println!("\nBranch {} (Visualization) outputs:", branch_c_id);
    if let Some(outputs) = result.get_branch_outputs(branch_c_id) {
        for (key, value) in outputs {
            println!("  {} = {}", key, value.to_string_repr());
        }
    }

    println!("\n🎯 Accessing specific branch outputs:");
    if let Some(value) = result.get_from_branch(branch_a_id, "statistics") {
        println!(
            "  Branch {} 'statistics': {}",
            branch_a_id,
            value.to_string_repr()
        );
    }
    if let Some(value) = result.get_from_branch(branch_b_id, "trained_model") {
        println!(
            "  Branch {} 'trained_model': {}",
            branch_b_id,
            value.to_string_repr()
        );
    }
    if let Some(value) = result.get_from_branch(branch_c_id, "plot") {
        println!(
            "  Branch {} 'plot': {}",
            branch_c_id,
            value.to_string_repr()
        );
    }

    println!();
}

fn demo_variant_per_node_access() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 3: Variant Outputs with Per-Node Tracking");
    println!("─────────────────────────────────────────────────────────\n");

    let mut graph = Graph::new();

    // Source node
    graph.add(
        |_: &HashMap<String, GraphData>, _| {
            let mut result = HashMap::new();
            result.insert("base_value".to_string(), GraphData::int(10));
            result
        },
        Some("Source"),
        None,
        Some(vec![("base_value", "data")]),
    );

    // Variant factory for scaling
    fn make_scaler(
        factor: f64,
    ) -> impl Fn(&HashMap<String, GraphData>, &HashMap<String, GraphData>) -> HashMap<String, GraphData>
    {
        move |inputs: &HashMap<String, GraphData>, _| {
            let value = inputs.get("input_data").and_then(|d| d.as_int()).unwrap() as f64;
            let mut result = HashMap::new();
            result.insert("scaled_value".to_string(), GraphData::float(value * factor));
            result
        }
    }

    // Create variants with unique output names to preserve all results
    graph.variant(
        make_scaler,
        vec![2.0, 3.0, 5.0],
        Some("Scale"),
        Some(vec![("data", "input_data")]),
        Some(vec![("scaled_value", "result")]), // Note: will overwrite in global context
    );

    let dag = graph.build();
    let result = dag.execute_detailed(false, None);

    println!("🌍 Global Context (note: 'result' contains last variant's output):");
    for (key, value) in &result.context {
        println!("  {} = {}", key, value.to_string_repr());
    }

    println!("\n📦 Per-Node Outputs (each variant tracked separately):");

    // Node 0 is the source
    // Nodes 1, 2, 3 are the variant nodes (2x, 3x, 5x scalers)
    for node_id in 1..=3 {
        println!("\nNode {} (Variant Scaler) outputs:", node_id);
        if let Some(outputs) = result.get_node_outputs(node_id) {
            for (key, value) in outputs {
                println!("  {} = {}", key, value.to_string_repr());
            }
        }
    }

    println!("\n💡 Key Insight:");
    println!(
        "  - Global context has 'result' = {} (last variant overwrites)",
        result.get("result").unwrap().to_string_repr()
    );
    println!("  - But per-node outputs preserve ALL variant results:");
    println!(
        "    Node 1 (2x): result = {}",
        result.get_from_node(1, "result").unwrap().to_string_repr()
    );
    println!(
        "    Node 2 (3x): result = {}",
        result.get_from_node(2, "result").unwrap().to_string_repr()
    );
    println!(
        "    Node 3 (5x): result = {}",
        result.get_from_node(3, "result").unwrap().to_string_repr()
    );

    println!();
}

fn demo_execution_history_tracking() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 4: Execution History Tracking");
    println!("─────────────────────────────────────────────────────────\n");

    let mut graph = Graph::new();

    // Multi-stage pipeline
    graph.add(
        |_: &HashMap<String, GraphData>, _| {
            let mut result = HashMap::new();
            result.insert("raw".to_string(), GraphData::int(5));
            result
        },
        Some("Load"),
        None,
        Some(vec![("raw", "input")]),
    );

    graph.add(
        |inputs: &HashMap<String, GraphData>, _| {
            let value = inputs.get("x").and_then(|d| d.as_int()).unwrap();
            let mut result = HashMap::new();
            result.insert("cleaned".to_string(), GraphData::int(value + 1));
            result
        },
        Some("Clean"),
        Some(vec![("input", "x")]),
        Some(vec![("cleaned", "clean_data")]),
    );

    graph.add(
        |inputs: &HashMap<String, GraphData>, _| {
            let value = inputs.get("x").and_then(|d| d.as_int()).unwrap();
            let mut result = HashMap::new();
            result.insert("normalized".to_string(), GraphData::int(value * 10));
            result
        },
        Some("Normalize"),
        Some(vec![("clean_data", "x")]),
        Some(vec![("normalized", "norm_data")]),
    );

    graph.add(
        |inputs: &HashMap<String, GraphData>, _| {
            let value = inputs.get("x").and_then(|d| d.as_int()).unwrap();
            let mut result = HashMap::new();
            result.insert(
                "transformed".to_string(),
                GraphData::string(&format!("FINAL_{}", value)),
            );
            result
        },
        Some("Transform"),
        Some(vec![("norm_data", "x")]),
        Some(vec![("transformed", "output")]),
    );

    let dag = graph.build();
    let result = dag.execute_detailed(false, None);

    println!("📊 Execution History (Data Flow Tracking):");
    println!();
    println!("Step-by-step transformation:");
    println!(
        "  1. Load:      input = {}",
        result.get_from_node(0, "input").unwrap().to_string_repr()
    );
    println!(
        "  2. Clean:     clean_data = {}",
        result
            .get_from_node(1, "clean_data")
            .unwrap()
            .to_string_repr()
    );
    println!(
        "  3. Normalize: norm_data = {}",
        result
            .get_from_node(2, "norm_data")
            .unwrap()
            .to_string_repr()
    );
    println!(
        "  4. Transform: output = {}",
        result.get_from_node(3, "output").unwrap().to_string_repr()
    );

    println!("\n🔍 Debugging: Inspect any intermediate result:");
    println!("  Need to debug the normalization step?");
    println!(
        "  Just check Node 2: {}",
        result
            .get_from_node(2, "norm_data")
            .unwrap()
            .to_string_repr()
    );

    println!("\n✅ Benefits of Per-Node Access:");
    println!("  ✓ Track data transformations step-by-step");
    println!("  ✓ Debug issues by inspecting intermediate values");
    println!("  ✓ Validate each processing stage independently");
    println!("  ✓ Preserve all variant outputs even with name collisions");

    println!();
}
