//! Comprehensive example showing all API features

use graph_sp::GraphBuilder;
use std::collections::HashMap;

fn main() {
    println!("=== Graph-SP: Complete API Demo ===\n");

    // Example 1: Simple sequential pipeline with implicit connections
    println!("--- Example 1: Simple Sequential Pipeline ---");
    {
        let mut builder = GraphBuilder::new();

        // Node 1: Data source (no inputs, produces "data")
        builder.add(
            |_inputs| {
                let mut outputs = HashMap::new();
                outputs.insert("data".to_string(), "hello".to_string());
                outputs
            },
            Some("Source"),
            None, // No broadcast_vars needed
            Some(vec!["data"]), // Produces "data"
        );

        // Node 2: Processor (consumes "data", produces "result")
        // Implicitly connected to Node 1
        builder.add(
            |inputs| {
                let mut outputs = HashMap::new();
                if let Some(data) = inputs.get("data") {
                    outputs.insert("result".to_string(), data.to_uppercase());
                }
                outputs
            },
            Some("Uppercase"),
            Some(vec!["data"]), // Consumes "data" from context
            Some(vec!["result"]), // Produces "result"
        );

        // Node 3: Final processor (consumes "result", produces "final")
        // Implicitly connected to Node 2
        builder.add(
            |inputs| {
                let mut outputs = HashMap::new();
                if let Some(result) = inputs.get("result") {
                    outputs.insert("final".to_string(), format!("{}!", result));
                }
                outputs
            },
            Some("Append"),
            Some(vec!["result"]),
            Some(vec!["final"]),
        );

        let dag = builder.build();
        let result = dag.execute();

        println!("Input: hello");
        println!("Output: {}", result.get("final").unwrap());
        println!();
    }

    // Example 2: Branching - parallel execution paths
    println!("--- Example 2: Branching ---");
    {
        let mut main_builder = GraphBuilder::new();

        // Main path: source node
        main_builder.add(
            |_| {
                let mut outputs = HashMap::new();
                outputs.insert("value".to_string(), "10".to_string());
                outputs
            },
            Some("Source"),
            None,
            Some(vec!["value"]),
        );

        // Branch A: multiply by 2
        let mut branch_a = GraphBuilder::new();
        branch_a.add(
            |inputs| {
                let mut outputs = HashMap::new();
                if let Some(val) = inputs.get("value") {
                    if let Ok(num) = val.parse::<i32>() {
                        outputs.insert("result_a".to_string(), (num * 2).to_string());
                    }
                }
                outputs
            },
            Some("Multiply x2"),
            Some(vec!["value"]),
            Some(vec!["result_a"]),
        );

        // Branch B: multiply by 3
        let mut branch_b = GraphBuilder::new();
        branch_b.add(
            |inputs| {
                let mut outputs = HashMap::new();
                if let Some(val) = inputs.get("value") {
                    if let Ok(num) = val.parse::<i32>() {
                        outputs.insert("result_b".to_string(), (num * 3).to_string());
                    }
                }
                outputs
            },
            Some("Multiply x3"),
            Some(vec!["value"]),
            Some(vec!["result_b"]),
        );

        // Add branches - they both branch from "Source"
        main_builder.branch(branch_a);
        main_builder.branch(branch_b);

        let dag = main_builder.build();
        let result = dag.execute();

        println!("Source value: 10");
        println!("Branch A result: {}", result.get("result_a").unwrap_or(&"N/A".to_string()));
        println!("Branch B result: {}", result.get("result_b").unwrap_or(&"N/A".to_string()));
        println!();
    }

    // Example 3: Variants - config sweeps
    println!("--- Example 3: Variants (Config Sweeps) ---");
    {
        let mut builder = GraphBuilder::new();

        // Source node
        builder.add(
            |_| {
                let mut outputs = HashMap::new();
                outputs.insert("base_value".to_string(), "100".to_string());
                outputs
            },
            Some("Source"),
            None,
            Some(vec!["base_value"]),
        );

        // Processor node that will be replicated with variants
        builder.add(
            |inputs| {
                let mut outputs = HashMap::new();
                if let Some(val) = inputs.get("base_value") {
                    if let Ok(num) = val.parse::<i32>() {
                        // In a real variant, we'd use the variant config
                        outputs.insert("processed".to_string(), (num * 2).to_string());
                    }
                }
                outputs
            },
            Some("Processor"),
            Some(vec!["base_value"]),
            Some(vec!["processed"]),
        );

        // Create 3 variants with different configs
        builder.variant(vec![
            ("rate", "0.1".to_string()),
            ("rate", "0.5".to_string()),
            ("rate", "1.0".to_string()),
        ]);

        let dag = builder.build();
        let stats = dag.stats();

        println!("Created {} variants", stats.variant_count);
        println!("Total nodes: {}", stats.node_count);
        println!("Max parallelism: {}", stats.max_parallelism);
        println!();
    }

    // Example 4: Mermaid visualization
    println!("--- Example 4: Mermaid Visualization ---");
    {
        let mut builder = GraphBuilder::new();

        builder.add(
            |_| {
                let mut outputs = HashMap::new();
                outputs.insert("data".to_string(), "start".to_string());
                outputs
            },
            Some("Start"),
            None,
            Some(vec!["data"]),
        );

        builder.add(
            |inputs| {
                let mut outputs = HashMap::new();
                if let Some(data) = inputs.get("data") {
                    outputs.insert("step1".to_string(), format!("{}-step1", data));
                }
                outputs
            },
            Some("Process"),
            Some(vec!["data"]),
            Some(vec!["step1"]),
        );

        builder.add(
            |inputs| {
                let mut outputs = HashMap::new();
                if let Some(data) = inputs.get("step1") {
                    outputs.insert("final".to_string(), format!("{}-done", data));
                }
                outputs
            },
            Some("Finish"),
            Some(vec!["step1"]),
            Some(vec!["final"]),
        );

        let dag = builder.build();
        println!("{}", dag.to_mermaid());
    }

    println!("=== Demo Complete ===");
}
