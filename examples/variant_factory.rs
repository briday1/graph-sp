//! Example showing sigexec-style variant factory pattern

use graph_sp::Graph;
use std::collections::HashMap;

// Factory function that creates a scaler with a given factor
fn make_scaler(factor: f64) -> impl Fn(&HashMap<String, String>, &HashMap<String, String>) -> HashMap<String, String> {
    move |inputs, _variant_params| {
        let mut outputs = HashMap::new();
        if let Some(val) = inputs.get("data") {
            if let Ok(num) = val.parse::<f64>() {
                outputs.insert("result".to_string(), (num * factor).to_string());
            }
        }
        outputs
    }
}

// Factory function that creates an offsetter with a given amount
fn make_offsetter(amount: f64) -> impl Fn(&HashMap<String, String>, &HashMap<String, String>) -> HashMap<String, String> {
    move |inputs, _variant_params| {
        let mut outputs = HashMap::new();
        if let Some(val) = inputs.get("result") {
            if let Ok(num) = val.parse::<f64>() {
                outputs.insert("final".to_string(), (num + amount).to_string());
            }
        }
        outputs
    }
}

fn main() {
    println!("=== Sigexec-Style Variant Factory Demo ===\n");

    // Example 1: Single variant sweep
    println!("--- Example 1: Scale Variants ---");
    {
        let mut graph = Graph::new();

        // Source node
        graph.add(
            |_, _| {
                let mut outputs = HashMap::new();
                outputs.insert("data".to_string(), "10.0".to_string());
                outputs
            },
            Some("Generate"),
            None,
            Some(vec!["data"]),
        );

        // Variant sweep: create multiple scalers with different factors
        // This is the sigexec pattern: factory function + array of parameters
        graph.variant_factory(
            make_scaler,
            vec![2.0, 3.0, 5.0],
            Some("Scale"),
            Some(vec!["data"]),
            Some(vec!["result"]),
        );

        let dag = graph.build();
        let stats = dag.stats();

        println!("Created {} scale variants", stats.variant_count);
        println!("Scale factors: 2.0, 3.0, 5.0");
        println!("Input: 10.0");
        println!("Expected outputs: 20.0, 30.0, 50.0\n");
    }

    // Example 2: Multiple variant sweeps (combinatorial)
    println!("--- Example 2: Combinatorial Variants ---");
    {
        let mut graph = Graph::new();

        // Source
        graph.add(
            |_, _| {
                let mut outputs = HashMap::new();
                outputs.insert("data".to_string(), "5.0".to_string());
                outputs
            },
            Some("Source"),
            None,
            Some(vec!["data"]),
        );

        // First variant: scaling
        graph.variant_factory(
            make_scaler,
            vec![2.0, 3.0],
            Some("Scale"),
            Some(vec!["data"]),
            Some(vec!["result"]),
        );

        // Merge the scale variants
        graph.merge();

        // Second variant: offsetting
        // Each scale variant will be followed by each offset variant
        graph.variant_factory(
            make_offsetter,
            vec![10.0, 20.0],
            Some("Offset"),
            Some(vec!["result"]),
            Some(vec!["final"]),
        );

        let dag = graph.build();
        let stats = dag.stats();

        println!("Created combinatorial variants");
        println!("2 scale factors × 2 offsets = {} total paths", stats.variant_count);
        println!("Mermaid diagram:\n{}\n", dag.to_mermaid());
    }

    // Example 3: Comparison with traditional approach
    println!("--- Example 3: Traditional vs Factory Pattern ---");
    println!("Traditional (node duplicates entire downstream graph):");
    println!("  graph.variant(\"param\", vec![1, 2, 3])");
    println!("  // Duplicates ALL downstream nodes 3 times");
    println!();
    println!("Factory pattern (creates specific variant nodes):");
    println!("  graph.variant_factory(make_node, vec![1, 2, 3], ...)");
    println!("  // Creates 3 variant instances of this specific node");
    println!("  // Useful for exploring single-node parameter spaces");
    println!();

    // Example 4: Inline factory with closure
    println!("--- Example 4: Inline Factory with Closure ---");
    {
        let mut graph = Graph::new();

        graph.add(
            |_, _| {
                let mut outputs = HashMap::new();
                outputs.insert("x".to_string(), "100".to_string());
                outputs
            },
            Some("Init"),
            None,
            Some(vec!["x"]),
        );

        // Inline factory using a closure
        graph.variant_factory(
            |multiplier: f64| {
                move |inputs: &HashMap<String, String>, _variant_params| {
                    let mut outputs = HashMap::new();
                    if let Some(x_str) = inputs.get("x") {
                        if let Ok(x) = x_str.parse::<f64>() {
                            outputs.insert("y".to_string(), (x * multiplier).to_string());
                        }
                    }
                    outputs
                }
            },
            vec![0.5, 1.0, 1.5, 2.0],
            Some("Multiply"),
            Some(vec!["x"]),
            Some(vec!["y"]),
        );

        let dag = graph.build();
        let stats = dag.stats();

        println!("Created {} variants using inline factory closure", stats.variant_count);
        println!("Multipliers: 0.5, 1.0, 1.5, 2.0");
    }

    println!("\n=== Demo Complete ===");
}
