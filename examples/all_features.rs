//! Comprehensive example showing all API features

use graph_sp::Graph;
use std::collections::HashMap;

fn main() {
    println!("=== Graph-SP: Complete API Demo ===\n");

    // Example 1: Simple sequential pipeline with implicit connections
    println!("--- Example 1: Simple Sequential Pipeline ---");
    {
        let mut graph = Graph::new();

        // Node 1: Data source (no inputs, produces "data")
        graph.add(
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
        graph.add(
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
        graph.add(
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

        let dag = graph.build();
        let result = dag.execute();

        println!("Input: hello");
        println!("Output: {}", result.get("final").unwrap());
        println!();
    }

    // Example 2: Branching with merge - parallel execution paths
    println!("--- Example 2: Branching with Merge ---");
    {
        let mut graph = Graph::new();

        // Main path: source node
        graph.add(
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
        let mut branch_a = Graph::new();
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
        let mut branch_b = Graph::new();
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
        graph.branch(branch_a);
        graph.branch(branch_b);
        graph.merge(); // Merge branches back together

        // Add a node that combines results from both branches
        graph.add(
            |inputs| {
                let mut outputs = HashMap::new();
                let a = inputs.get("result_a").and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
                let b = inputs.get("result_b").and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
                outputs.insert("combined".to_string(), format!("Sum: {}", a + b));
                outputs
            },
            Some("Combine"),
            Some(vec!["result_a", "result_b"]),
            Some(vec!["combined"]),
        );

        let dag = graph.build();
        let result = dag.execute();

        println!("Source value: 10");
        println!("Branch A result (x2): {}", result.get("result_a").unwrap_or(&"N/A".to_string()));
        println!("Branch B result (x3): {}", result.get("result_b").unwrap_or(&"N/A".to_string()));
        println!("Combined: {}", result.get("combined").unwrap_or(&"N/A".to_string()));
        println!();
    }

    // Example 3: Variants with linspace - parameter sweeps
    println!("--- Example 3: Variants with Linspace ---");
    {
        let mut graph = Graph::new();

        // Source node
        graph.add(
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
        graph.add(
            |inputs| {
                let mut outputs = HashMap::new();
                if let Some(val) = inputs.get("base_value") {
                    if let Ok(num) = val.parse::<i32>() {
                        outputs.insert("processed".to_string(), (num * 2).to_string());
                    }
                }
                outputs
            },
            Some("Processor"),
            Some(vec!["base_value"]),
            Some(vec!["processed"]),
        );

        // Create 5 variants with learning rates from 0.001 to 0.1 (linearly spaced)
        graph.variant("learning_rate", graph_sp::Linspace::new(0.001, 0.1, 5));

        let dag = graph.build();
        let stats = dag.stats();

        println!("Created {} variants (linspace: 0.001 to 0.1)", stats.variant_count);
        println!("Total nodes: {}", stats.node_count);
        println!("Max parallelism: {}", stats.max_parallelism);
        println!();
    }

    // Example 4: Variants with logspace - logarithmic parameter sweeps
    println!("--- Example 4: Variants with Logspace ---");
    {
        let mut graph = Graph::new();

        graph.add(
            |_| {
                let mut outputs = HashMap::new();
                outputs.insert("data".to_string(), "1000".to_string());
                outputs
            },
            Some("Source"),
            None,
            Some(vec!["data"]),
        );

        graph.add(
            |inputs| {
                let mut outputs = HashMap::new();
                if let Some(data) = inputs.get("data") {
                    outputs.insert("result".to_string(), format!("Processed: {}", data));
                }
                outputs
            },
            Some("Model"),
            Some(vec!["data"]),
            Some(vec!["result"]),
        );

        // Create 4 variants with log-spaced learning rates (0.0001 to 0.1)
        graph.variant("lr", graph_sp::Logspace::new(0.0001, 0.1, 4));

        let dag = graph.build();
        let stats = dag.stats();

        println!("Created {} variants (logspace: 0.0001 to 0.1)", stats.variant_count);
        println!("Total nodes: {}", stats.node_count);
        println!();
    }

    // Example 5: Custom variant sweep with generator function
    println!("--- Example 5: Custom Variant Sweep ---");
    {
        let mut graph = Graph::new();

        graph.add(
            |_| {
                let mut outputs = HashMap::new();
                outputs.insert("input".to_string(), "test".to_string());
                outputs
            },
            Some("Input"),
            None,
            Some(vec!["input"]),
        );

        graph.add(
            |inputs| {
                let mut outputs = HashMap::new();
                if let Some(input) = inputs.get("input") {
                    outputs.insert("output".to_string(), format!("Output: {}", input));
                }
                outputs
            },
            Some("Process"),
            Some(vec!["input"]),
            Some(vec!["output"]),
        );

        // Custom generator: powers of 2
        graph.variant("batch_size", graph_sp::Generator::new(4, |i| {
            format!("{}", 2_u32.pow(i as u32 + 3)) // 8, 16, 32, 64
        }));

        let dag = graph.build();
        let stats = dag.stats();

        println!("Created {} variants with custom generator (powers of 2)", stats.variant_count);
        println!("Batch sizes: 8, 16, 32, 64");
        println!();
    }

    // Example 6: Mermaid visualization
    println!("--- Example 6: Mermaid Visualization ---");
    {
        let mut graph = Graph::new();

        graph.add(
            |_| {
                let mut outputs = HashMap::new();
                outputs.insert("data".to_string(), "start".to_string());
                outputs
            },
            Some("Start"),
            None,
            Some(vec!["data"]),
        );

        graph.add(
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

        graph.add(
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

        let dag = graph.build();
        println!("{}", dag.to_mermaid());
    }

    println!("\n=== Demo Complete ===");
}
