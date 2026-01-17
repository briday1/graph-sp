//! Example: Branching, Merging, and Variants
//!
//! This example demonstrates the new features:
//! - Creating named branches (subgraphs)
//! - Creating variants for config sweeps
//! - Merging outputs from multiple branches
//! - Nested variants (cartesian product)

use graph_sp::core::{
    Graph, MergeConfig, Node, NodeConfig, Port, PortData, VariantConfig, VariantFunction,
};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Graph-SP Example: Branching, Merging, and Variants ===\n");

    // Example 1: Simple Branching
    println!("Example 1: Simple Branching");
    println!("---------------------------");
    let mut graph1 = Graph::new();

    // Create two branches
    let branch_a = graph1.create_branch("experiment_a")?;
    let source_a = NodeConfig::new(
        "source_a",
        "Source A",
        vec![],
        vec![Port::simple("value")],
        Arc::new(|_: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            outputs.insert("value".to_string(), PortData::Int(10));
            Ok(outputs)
        }),
    );
    branch_a.add(Node::new(source_a))?;

    let branch_b = graph1.create_branch("experiment_b")?;
    let source_b = NodeConfig::new(
        "source_b",
        "Source B",
        vec![],
        vec![Port::simple("value")],
        Arc::new(|_: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            outputs.insert("value".to_string(), PortData::Int(20));
            Ok(outputs)
        }),
    );
    branch_b.add(Node::new(source_b))?;

    println!("✓ Created branches: {:?}", graph1.branch_names());
    println!();

    // Example 2: Variants (Config Sweeps)
    println!("Example 2: Variants - Hyperparameter Sweep");
    println!("------------------------------------------");
    let mut graph2 = Graph::new();

    // Create variants with different learning rates
    let lr_variant_fn: VariantFunction = Arc::new(|i: usize| {
        let lr = (i as f64 + 1.0) * 0.01;
        PortData::Float(lr)
    });

    let variant_config = VariantConfig::new("learning_rate", 3, "lr", lr_variant_fn);
    let branch_names = graph2.create_variants(variant_config)?;

    println!("✓ Created {} variant branches:", branch_names.len());
    for (i, name) in branch_names.iter().enumerate() {
        println!("  - {} (lr = {})", name, (i + 1) as f64 * 0.01);
    }
    println!();

    // Example 3: Merge Functionality
    println!("Example 3: Merging Branch Outputs");
    println!("----------------------------------");
    let mut graph3 = Graph::new();

    // Create three branches with different values
    for i in 0..3 {
        let branch_name = format!("branch_{}", i);
        let branch = graph3.create_branch(&branch_name)?;

        let value = (i + 1) * 10;
        let source = NodeConfig::new(
            format!("source_{}", i),
            format!("Source {}", i),
            vec![],
            vec![Port::simple("result")],
            Arc::new(move |_: &HashMap<String, PortData>| {
                let mut outputs = HashMap::new();
                outputs.insert("result".to_string(), PortData::Int(value as i64));
                Ok(outputs)
            }),
        );
        branch.add(Node::new(source))?;
    }

    // Merge the outputs with default behavior (collect to list)
    let merge_config = MergeConfig::new(
        vec![
            "branch_0".to_string(),
            "branch_1".to_string(),
            "branch_2".to_string(),
        ],
        "result".to_string(),
    );
    graph3.merge("merge_node", merge_config)?;

    println!("✓ Created merge node to collect outputs from 3 branches");
    println!();

    // Example 4: Custom Merge Function
    println!("Example 4: Custom Merge Function (Max)");
    println!("---------------------------------------");
    let mut graph4 = Graph::new();

    // Create branches
    for i in 0..3 {
        let branch_name = format!("test_{}", i);
        let branch = graph4.create_branch(&branch_name)?;

        let value = (i + 1) * 5;
        let source = NodeConfig::new(
            format!("src_{}", i),
            format!("Source {}", i),
            vec![],
            vec![Port::simple("score")],
            Arc::new(move |_: &HashMap<String, PortData>| {
                let mut outputs = HashMap::new();
                outputs.insert("score".to_string(), PortData::Int(value as i64));
                Ok(outputs)
            }),
        );
        branch.add(Node::new(source))?;
    }

    // Create custom merge function that finds the maximum value
    let max_merge = Arc::new(
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
            "test_0".to_string(),
            "test_1".to_string(),
            "test_2".to_string(),
        ],
        "score".to_string(),
    )
    .with_merge_fn(max_merge);

    graph4.merge("max_score", merge_config)?;

    println!("✓ Created merge node with custom max function");
    println!();

    // Example 5: Nested Variants (Cartesian Product)
    println!("Example 5: Nested Variants - Cartesian Product");
    println!("-----------------------------------------------");
    let mut graph5 = Graph::new();

    // First level: learning rates
    let lr_fn: VariantFunction = Arc::new(|i: usize| PortData::Float((i as f64 + 1.0) * 0.01));
    let lr_config = VariantConfig::new("lr", 2, "learning_rate", lr_fn);
    let lr_branches = graph5.create_variants(lr_config)?;

    println!("✓ Created {} learning rate variants", lr_branches.len());

    // Second level: batch sizes (nested in each learning rate)
    let mut total_combinations = 0;
    for lr_branch in &lr_branches {
        let branch = graph5.get_branch_mut(lr_branch)?;

        let batch_fn: VariantFunction = Arc::new(|i: usize| PortData::Int((i as i64 + 1) * 16));
        let batch_config = VariantConfig::new("batch", 3, "batch_size", batch_fn);
        let batch_branches = branch.create_variants(batch_config)?;

        total_combinations += batch_branches.len();
        println!(
            "  - {} has {} batch size variants",
            lr_branch,
            batch_branches.len()
        );
    }

    println!(
        "✓ Total combinations (Cartesian product): 2 × 3 = {}",
        total_combinations
    );
    println!();

    // Example 6: Parallelization Toggle
    println!("Example 6: Variants with Parallelization Control");
    println!("------------------------------------------------");
    let mut graph6 = Graph::new();

    // Create variants with parallelization disabled
    let param_fn: VariantFunction = Arc::new(|i: usize| PortData::Int(i as i64));
    let config_sequential =
        VariantConfig::new("sequential", 4, "param", param_fn).with_parallel(false);

    let branches = graph6.create_variants(config_sequential)?;
    println!(
        "✅ Created {} variants with parallelization disabled",
        branches.len()
    );
    println!("  (Useful for debugging or when resources are limited)");
    println!();

    println!("=== Summary ===");
    println!("✓ All examples completed successfully!");
    println!("✓ Demonstrated: branches, variants, merging, and nested variants");
    println!("\nThese features enable:");
    println!("  - A/B testing with multiple experiment branches");
    println!("  - Hyperparameter sweeps with variants");
    println!("  - Ensemble methods with merging");
    println!("  - Grid search with nested variants (cartesian product)");

    Ok(())
}
