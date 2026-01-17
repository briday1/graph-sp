//! Comprehensive Example: All Features with Mermaid Visualization
//!
//! This example demonstrates:
//! - Basic graph creation with add()
//! - Branch creation and isolation
//! - Variant creation (config sweeps)
//! - Nested variants (cartesian product)
//! - Merge operations with custom functions
//! - Complete mermaid diagram output

use graph_sp::core::{
    Edge, Graph, MergeConfig, Node, NodeConfig, Port, PortData, VariantConfig, VariantFunction,
};
use graph_sp::executor::Executor;
use graph_sp::inspector::Inspector;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║  Graph-SP: Comprehensive Feature Demonstration                  ║");
    println!("║  Showing: Branching, Variants, Merging, and Mermaid Diagrams   ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // ========================================================================
    // EXAMPLE 1: Simple Pipeline with Basic Nodes
    // ========================================================================
    println!("═══════════════════════════════════════════════════════════════════");
    println!("Example 1: Basic Pipeline (add, edges, mermaid)");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let mut graph1 = Graph::new();

    // Create a simple 3-node pipeline
    let source = NodeConfig::new(
        "source",
        "Data Source",
        vec![],
        vec![Port::new("data", "Output Data")],
        Arc::new(|_: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            outputs.insert(
                "data".to_string(),
                PortData::List(vec![PortData::Int(10), PortData::Int(20), PortData::Int(30)]),
            );
            Ok(outputs)
        }),
    );

    let processor = NodeConfig::new(
        "processor",
        "Data Processor",
        vec![Port::new("input", "Input Data")],
        vec![Port::new("output", "Processed Data")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            if let Some(PortData::List(items)) = inputs.get("input") {
                let doubled: Vec<PortData> = items
                    .iter()
                    .map(|item| {
                        if let PortData::Int(val) = item {
                            PortData::Int(val * 2)
                        } else {
                            item.clone()
                        }
                    })
                    .collect();
                outputs.insert("output".to_string(), PortData::List(doubled));
            }
            Ok(outputs)
        }),
    );

    let sink = NodeConfig::new(
        "sink",
        "Result Sink",
        vec![Port::new("input", "Final Data")],
        vec![],
        Arc::new(|_inputs: &HashMap<String, PortData>| Ok(HashMap::new())),
    );

    graph1.add(Node::new(source))?;
    graph1.add(Node::new(processor))?;
    graph1.add(Node::new(sink))?;

    graph1.add_edge(Edge::new("source", "data", "processor", "input"))?;
    graph1.add_edge(Edge::new("processor", "output", "sink", "input"))?;

    println!("📊 Graph Structure:");
    println!("   Nodes: {}", graph1.node_count());
    println!("   Edges: {}", graph1.edge_count());
    println!();

    println!("🎨 Mermaid Diagram:");
    println!("{}", Inspector::to_mermaid(&graph1)?);

    // ========================================================================
    // EXAMPLE 2: Branching - Multiple Isolated Experiments
    // ========================================================================
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("Example 2: Branching (isolated subgraphs)");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let mut graph2 = Graph::new();

    // Create two experimental branches
    let branch_a = graph2.create_branch("experiment_a")?;
    let exp_a_source = NodeConfig::new(
        "exp_a_src",
        "Experiment A Source",
        vec![],
        vec![Port::new("value", "Value")],
        Arc::new(|_: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            outputs.insert("value".to_string(), PortData::Int(100));
            Ok(outputs)
        }),
    );
    branch_a.add(Node::new(exp_a_source))?;

    let branch_b = graph2.create_branch("experiment_b")?;
    let exp_b_source = NodeConfig::new(
        "exp_b_src",
        "Experiment B Source",
        vec![],
        vec![Port::new("value", "Value")],
        Arc::new(|_: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            outputs.insert("value".to_string(), PortData::Int(200));
            Ok(outputs)
        }),
    );
    branch_b.add(Node::new(exp_b_source))?;

    println!("📊 Branch Information:");
    println!("   Branches created: {:?}", graph2.branch_names());
    println!("   Branch A nodes: {}", graph2.get_branch("experiment_a")?.node_count());
    println!("   Branch B nodes: {}", graph2.get_branch("experiment_b")?.node_count());
    println!();

    println!("🎨 Main Graph Mermaid (showing branches):");
    println!("{}", Inspector::to_mermaid(&graph2)?);

    // ========================================================================
    // EXAMPLE 3: Variants - Hyperparameter Sweep
    // ========================================================================
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("Example 3: Variants (config sweeps with 5 learning rates)");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let mut graph3 = Graph::new();

    // Create 5 learning rate variants
    let lr_fn: VariantFunction = Arc::new(|i: usize| PortData::Float((i as f64 + 1.0) * 0.01));

    let variant_config = VariantConfig::new("learning_rate", 5, "lr", lr_fn);
    let lr_branches = graph3.create_variants(variant_config)?;

    println!("📊 Variant Information:");
    println!("   Variants created: {}", lr_branches.len());
    for (i, branch_name) in lr_branches.iter().enumerate() {
        println!("   - {} (lr = {})", branch_name, (i + 1) as f64 * 0.01);
    }
    println!();

    println!("🎨 Mermaid Diagram (shows '5 variants' hexagon):");
    println!("{}", Inspector::to_mermaid(&graph3)?);

    // ========================================================================
    // EXAMPLE 4: Merge - Combining Branch Outputs
    // ========================================================================
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("Example 4: Merge (combining outputs with custom function)");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let mut graph4 = Graph::new();

    // Create 3 branches with different scores
    for i in 0..3 {
        let branch_name = format!("model_{}", i);
        let branch = graph4.create_branch(&branch_name)?;

        let score = (i + 1) * 25;
        let source = NodeConfig::new(
            format!("src_{}", i),
            format!("Model {} Source", i),
            vec![],
            vec![Port::new("score", "Model Score")],
            Arc::new(move |_: &HashMap<String, PortData>| {
                let mut outputs = HashMap::new();
                outputs.insert("score".to_string(), PortData::Int(score as i64));
                Ok(outputs)
            }),
        );
        branch.add(Node::new(source))?;
    }

    // Merge with MAX function
    let max_fn = Arc::new(
        |inputs: Vec<&PortData>| -> graph_sp::core::Result<PortData> {
            let mut max_val = i64::MIN;
            for data in inputs {
                if let PortData::Int(val) = data {
                    max_val = max_val.max(*val);
                }
            }
            Ok(PortData::Int(max_val))
        },
    );

    let merge_config = MergeConfig::new(
        vec![
            "model_0".to_string(),
            "model_1".to_string(),
            "model_2".to_string(),
        ],
        "score".to_string(),
    )
    .with_merge_fn(max_fn);

    graph4.merge("best_model", merge_config)?;

    println!("📊 Merge Information:");
    println!("   Branches to merge: model_0, model_1, model_2");
    println!("   Merge function: MAX");
    println!("   Merge node: best_model");
    println!();

    println!("🎨 Mermaid Diagram (showing merge node):");
    println!("{}", Inspector::to_mermaid(&graph4)?);

    // ========================================================================
    // EXAMPLE 5: Nested Variants - Cartesian Product (Grid Search)
    // ========================================================================
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("Example 5: Nested Variants (2 LR × 3 Batch = 6 combinations)");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let mut graph5 = Graph::new();

    // First level: 2 learning rates
    let lr_fn2: VariantFunction = Arc::new(|i: usize| PortData::Float((i as f64 + 1.0) * 0.01));
    let lr_config = VariantConfig::new("lr", 2, "learning_rate", lr_fn2);
    let lr_branches = graph5.create_variants(lr_config)?;

    println!("📊 Nested Variant Structure:");
    println!("   Level 1 (Learning Rates): {} variants", lr_branches.len());

    // Second level: 3 batch sizes in each LR
    for (idx, lr_branch) in lr_branches.iter().enumerate() {
        let branch = graph5.get_branch_mut(lr_branch)?;

        let batch_fn: VariantFunction = Arc::new(|i: usize| PortData::Int((i as i64 + 1) * 16));
        let batch_config = VariantConfig::new("batch", 3, "batch_size", batch_fn);
        let batch_branches = branch.create_variants(batch_config)?;

        println!(
            "   Level 2 in {}: {} batch size variants",
            lr_branch,
            batch_branches.len()
        );
    }

    println!("   Total combinations: 2 × 3 = 6");
    println!();

    println!("🎨 Main Graph Mermaid (showing LR variants):");
    println!("{}", Inspector::to_mermaid(&graph5)?);

    println!("\n🎨 Branch 'lr_0' Mermaid (showing nested batch variants):");
    let lr0_branch = graph5.get_branch("lr_0")?;
    println!("{}", Inspector::to_mermaid(lr0_branch)?);

    // ========================================================================
    // EXAMPLE 6: Complete Workflow - Variants → Merge → Execute
    // ========================================================================
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("Example 6: Complete Workflow (variants + processing + merge)");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let mut graph6 = Graph::new();

    // Create 3 parameter variants
    let param_fn: VariantFunction = Arc::new(|i: usize| PortData::Int((i + 1) as i64 * 10));
    let param_config = VariantConfig::new("param", 3, "parameter", param_fn);
    let param_branches = graph6.create_variants(param_config)?;

    // Add processing to each variant branch
    for (i, branch_name) in param_branches.iter().enumerate() {
        let branch = graph6.get_branch_mut(branch_name)?;

        let processor = NodeConfig::new(
            format!("processor_{}", i),
            format!("Processor {}", i),
            vec![Port::new("parameter", "Parameter")],
            vec![Port::new("result", "Result")],
            Arc::new(|inputs: &HashMap<String, PortData>| {
                let mut outputs = HashMap::new();
                if let Some(PortData::Int(param)) = inputs.get("parameter") {
                    // Simulate some computation
                    outputs.insert("result".to_string(), PortData::Int(param * param));
                }
                Ok(outputs)
            }),
        );

        branch.add(Node::new(processor))?;

        // Connect source to processor
        let source_id = format!("{}_source", branch_name);
        branch.add_edge(Edge::new(
            source_id,
            "parameter",
            format!("processor_{}", i),
            "parameter",
        ))?;
    }

    // Create merge node with list collection (default)
    let merge_config = MergeConfig::new(param_branches.clone(), "result".to_string());
    graph6.merge("aggregator", merge_config)?;

    println!("📊 Complete Workflow:");
    println!("   Variants: {}", param_branches.len());
    println!("   Processing: Each variant squares its parameter");
    println!("   Merge: Collects all results into a list");
    println!();

    println!("🎨 Final Mermaid Diagram:");
    println!("{}", Inspector::to_mermaid(&graph6)?);

    // ========================================================================
    // Summary
    // ========================================================================
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║  Summary of Features Demonstrated                               ║");
    println!("╚══════════════════════════════════════════════════════════════════╝");
    println!();
    println!("✅ Basic API: add() for nodes, add_edge() for connections");
    println!("✅ Branching: create_branch() for isolated subgraphs");
    println!("✅ Variants: create_variants() for parameter sweeps");
    println!("✅ Nested Variants: Cartesian product (2 × 3 = 6 configs)");
    println!("✅ Merge: Custom and default merge functions");
    println!("✅ Mermaid: Complete visualization with special variant styling");
    println!();
    println!("📖 Legend:");
    println!("   🔵 Blue nodes      = Source nodes (no inputs)");
    println!("   🟣 Purple nodes    = Sink nodes (no outputs)");
    println!("   🟠 Orange nodes    = Processing nodes");
    println!("   🟢 Green hexagons  = Variant groups (with count)");
    println!();

    Ok(())
}
