//! Example: Mermaid Visualization with Variants.
//!
//! This example demonstrates how variants are visualized in Mermaid diagrams.

use graph_sp::core::{Graph, MergeConfig, PortData, VariantConfig, VariantFunction};
use graph_sp::inspector::Inspector;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Mermaid Visualization: Variants Example ===\n");

    // Example 1: Simple graph with variants
    println!("Example 1: Hyperparameter Sweep with 5 Variants");
    println!("================================================\n");

    let mut graph1 = Graph::new();

    // Create 5 variants with different learning rates
    let lr_fn: VariantFunction = Arc::new(|i: usize| PortData::Float((i as f64 + 1.0) * 0.01));

    let variant_config = VariantConfig::new("learning_rate", 5, "lr", lr_fn);
    graph1.create_variants(variant_config)?;

    // Create merge node
    let branch_names: Vec<String> = (0..5).map(|i| format!("learning_rate_{}", i)).collect();

    let merge_config = MergeConfig::new(branch_names, "result".to_string());
    graph1.merge("best_model", merge_config)?;

    println!("Mermaid Diagram:");
    println!("{}\n", Inspector::to_mermaid(&graph1)?);

    // Example 2: Nested variants (Cartesian Product)
    println!("Example 2: Nested Variants - 2 LR × 3 Batch Sizes = 6 Configs");
    println!("==============================================================\n");

    let mut graph2 = Graph::new();

    // First level: learning rates
    let lr_fn2: VariantFunction = Arc::new(|i: usize| PortData::Float((i as f64 + 1.0) * 0.01));
    let lr_config = VariantConfig::new("lr", 2, "learning_rate", lr_fn2);
    let lr_branches = graph2.create_variants(lr_config)?;

    // Second level: batch sizes (nested in each learning rate)
    for lr_branch in &lr_branches {
        let branch = graph2.get_branch_mut(lr_branch)?;

        let batch_fn: VariantFunction = Arc::new(|i: usize| PortData::Int((i as i64 + 1) * 16));
        let batch_config = VariantConfig::new("batch", 3, "batch_size", batch_fn);
        branch.create_variants(batch_config)?;
    }
    
    println!("Main Graph Mermaid Diagram (showing LR variants):");
    println!("{}\n", Inspector::to_mermaid(&graph2)?);

    println!("Branch 'lr_0' Mermaid Diagram (showing batch size variants):");
    let lr0_branch = graph2.get_branch("lr_0")?;
    println!("{}\n", Inspector::to_mermaid(lr0_branch)?);

    // Example 3: Mix of regular nodes and variants
    println!("Example 3: Mixed Graph with Regular Nodes and Variants");
    println!("=======================================================\n");

    let mut graph3 = Graph::new();

    // Create some variant branches
    let param_fn: VariantFunction = Arc::new(|i: usize| PortData::Int(i as i64));
    let config = VariantConfig::new("experiment", 3, "param", param_fn);
    let exp_branches = graph3.create_variants(config)?;

    // Add a merge node in the main graph
    let merge_config3 = MergeConfig::new(exp_branches, "output".to_string());
    graph3.merge("aggregator", merge_config3)?;

    println!("Mermaid Diagram:");
    println!("{}", Inspector::to_mermaid(&graph3)?);

    println!("\n=== Visualization Legend ===");
    println!("• Hexagon with dashed border {{{{...}}}} = Multiple variants (N variants)");
    println!("• Rectangle [...]              = Regular node or single branch");
    println!("• Blue nodes                   = Source nodes (no inputs)");
    println!("• Purple nodes                 = Sink nodes (no outputs)");
    println!("• Orange nodes                 = Processing nodes");
    println!("• Green hexagons               = Variant groups");

    Ok(())
}
