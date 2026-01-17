//! Example showing the new generic variant syntax

use graph_sp::{Generator, Geomspace, Graph, Linspace, Logspace};
use std::collections::HashMap;

fn process_data(inputs: &HashMap<String, String>) -> HashMap<String, String> {
    let mut outputs = HashMap::new();
    if let Some(data) = inputs.get("input") {
        outputs.insert("output".to_string(), format!("Processed: {}", data));
    }
    outputs
}

fn main() {
    println!("=== Generic Variant Syntax Demo ===\n");

    // Example 1: Variant with a simple list of values
    println!("--- Example 1: Variant with Vec ---");
    {
        let mut graph = Graph::new();

        graph.add(
            |_| {
                let mut out = HashMap::new();
                out.insert("input".to_string(), "data".to_string());
                out
            },
            Some("Source"),
            None,
            Some(vec!["input"]),
        );

        graph.add(process_data, Some("Process"), Some(vec!["input"]), Some(vec!["output"]));

        // Pass a list directly to variant!
        graph.variant("learning_rate", vec!["0.001", "0.01", "0.1"]);

        let dag = graph.build();
        let stats = dag.stats();

        println!("Created {} variants from list", stats.variant_count);
        println!("Values: 0.001, 0.01, 0.1\n");
    }

    // Example 2: Variant with numeric vec
    println!("--- Example 2: Variant with Vec<f64> ---");
    {
        let mut graph = Graph::new();

        graph.add(
            |_| {
                let mut out = HashMap::new();
                out.insert("x".to_string(), "start".to_string());
                out
            },
            Some("Init"),
            None,
            Some(vec!["x"]),
        );

        graph.add(process_data, Some("Process"), Some(vec!["x"]), Some(vec!["y"]));

        // Pass numeric values directly
        graph.variant("rate", vec![0.001, 0.01, 0.1, 1.0]);

        let dag = graph.build();
        let stats = dag.stats();

        println!("Created {} variants from Vec<f64>", stats.variant_count);
        println!("Values: 0.001, 0.01, 0.1, 1.0\n");
    }

    // Example 3: Variant with Linspace helper
    println!("--- Example 3: Variant with Linspace ---");
    {
        let mut graph = Graph::new();

        graph.add(
            |_| {
                let mut out = HashMap::new();
                out.insert("data".to_string(), "input".to_string());
                out
            },
            Some("Source"),
            None,
            Some(vec!["data"]),
        );

        graph.add(process_data, Some("Model"), Some(vec!["data"]), Some(vec!["result"]));

        // Use Linspace helper for evenly spaced values
        graph.variant("learning_rate", Linspace::new(0.001, 0.1, 10));

        let dag = graph.build();
        let stats = dag.stats();

        println!("Created {} variants with Linspace", stats.variant_count);
        println!("Range: 0.001 to 0.1, evenly spaced\n");
    }

    // Example 4: Variant with Logspace helper
    println!("--- Example 4: Variant with Logspace ---");
    {
        let mut graph = Graph::new();

        graph.add(
            |_| {
                let mut out = HashMap::new();
                out.insert("input".to_string(), "data".to_string());
                out
            },
            Some("Init"),
            None,
            Some(vec!["input"]),
        );

        graph.add(process_data, Some("Process"), Some(vec!["input"]), Some(vec!["output"]));

        // Use Logspace for logarithmically spaced values
        graph.variant("lr", Logspace::new(0.0001, 0.1, 5));

        let dag = graph.build();
        let stats = dag.stats();

        println!("Created {} variants with Logspace", stats.variant_count);
        println!("Range: 0.0001 to 0.1, logarithmically spaced\n");
    }

    // Example 5: Variant with Geomspace helper
    println!("--- Example 5: Variant with Geomspace ---");
    {
        let mut graph = Graph::new();

        graph.add(
            |_| {
                let mut out = HashMap::new();
                out.insert("value".to_string(), "test".to_string());
                out
            },
            Some("Start"),
            None,
            Some(vec!["value"]),
        );

        graph.add(process_data, Some("Process"), Some(vec!["value"]), Some(vec!["result"]));

        // Geometric progression: each value multiplied by ratio
        graph.variant("lr", Geomspace::new(0.001, 10.0, 5));

        let dag = graph.build();
        let stats = dag.stats();

        println!("Created {} variants with Geomspace", stats.variant_count);
        println!("Values: 0.001, 0.01, 0.1, 1.0, 10.0 (ratio=10)\n");
    }

    // Example 6: Variant with custom Generator
    println!("--- Example 6: Variant with Generator (Lambda) ---");
    {
        let mut graph = Graph::new();

        graph.add(
            |_| {
                let mut out = HashMap::new();
                out.insert("x".to_string(), "data".to_string());
                out
            },
            Some("Source"),
            None,
            Some(vec!["x"]),
        );

        graph.add(process_data, Some("Work"), Some(vec!["x"]), Some(vec!["y"]));

        // Custom generator function - powers of 2
        graph.variant("batch_size", Generator::new(5, |i| {
            format!("{}", 2_u32.pow(i as u32 + 3))  // 8, 16, 32, 64, 128
        }));

        let dag = graph.build();
        let stats = dag.stats();

        println!("Created {} variants with custom Generator", stats.variant_count);
        println!("Batch sizes: 8, 16, 32, 64, 128 (powers of 2)\n");
    }

    // Example 7: Mix and match - different variant types
    println!("--- Example 7: Integer Vec ---");
    {
        let mut graph = Graph::new();

        graph.add(
            |_| {
                let mut out = HashMap::new();
                out.insert("n".to_string(), "5".to_string());
                out
            },
            Some("Start"),
            None,
            Some(vec!["n"]),
        );

        graph.add(process_data, Some("Compute"), Some(vec!["n"]), Some(vec!["result"]));

        // Use Vec<i32> for integer values
        graph.variant("epochs", vec![10, 20, 50, 100]);

        let dag = graph.build();
        let stats = dag.stats();

        println!("Created {} variants from Vec<i32>", stats.variant_count);
        println!("Epochs: 10, 20, 50, 100\n");
    }

    println!("=== All Variant Syntaxes Demonstrated ===");
}
