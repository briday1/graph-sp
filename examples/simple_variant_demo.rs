//! Simple examples demonstrating the variant syntax

use graph_sp::Graph;
use std::collections::HashMap;

fn main() {
    println!("=== Simple Variant Syntax Examples ===\n");

    // Example 1: Basic variant with a factory function
    println!("--- Example 1: Basic Variant (sigexec-style) ---\n");
    {
        let mut graph = Graph::new();

        // Add a source node
        graph.add(
            |_, _| {
                let mut out = HashMap::new();
                out.insert("x".to_string(), "10".to_string());
                out
            },
            Some("Source"),
            None,
            Some(vec!["x"]),
        );

        // Define a factory: takes a parameter, returns a node function
        fn make_multiplier(factor: f64) -> impl Fn(&HashMap<String, String>, &HashMap<String, String>) -> HashMap<String, String> {
            move |inputs: &HashMap<String, String>, _variant_params| {
                let mut outputs = HashMap::new();
                if let Some(x_str) = inputs.get("x") {
                    if let Ok(x) = x_str.parse::<f64>() {
                        outputs.insert("y".to_string(), (x * factor).to_string());
                    }
                }
                outputs
            }
        }

        // Create variants: one node per factor
        graph.variant_factory(
            make_multiplier,
            vec![2.0, 3.0, 5.0],
            Some("Multiply"),
            Some(vec!["x"]),
            Some(vec!["y"]),
        );

        println!("Syntax:");
        println!("  fn make_multiplier(factor) -> impl Fn(...) {{ ... }}");
        println!("  graph.variant_factory(make_multiplier, vec![2.0, 3.0, 5.0], ...)");
        println!();
        println!("Result: 3 nodes created (one for each factor: 2.0, 3.0, 5.0)");
        println!("  Input: x = 10");
        println!("  Outputs: y = 20, y = 30, y = 50");
        println!();
    }

    // Example 2: Same thing but more compact with inline factory
    println!("--- Example 2: Inline Factory ---\n");
    {
        let mut graph = Graph::new();

        graph.add(
            |_, _| {
                let mut out = HashMap::new();
                out.insert("data".to_string(), "100".to_string());
                out
            },
            Some("Init"),
            None,
            Some(vec!["data"]),
        );

        // Inline factory - no separate function definition needed
        graph.variant_factory(
            |scale: f64| {
                move |inputs: &HashMap<String, String>, _| {
                    let mut outputs = HashMap::new();
                    if let Some(val) = inputs.get("data").and_then(|s| s.parse::<f64>().ok()) {
                        outputs.insert("result".to_string(), (val * scale).to_string());
                    }
                    outputs
                }
            },
            vec![0.5, 1.0, 2.0],
            Some("Scale"),
            Some(vec!["data"]),
            Some(vec!["result"]),
        );

        println!("Syntax (inline):");
        println!("  graph.variant_factory(");
        println!("      |scale| move |inputs, _| {{ ... }},  // Factory inline");
        println!("      vec![0.5, 1.0, 2.0],                 // Parameters");
        println!("      ...                                  // Node config");
        println!("  )");
        println!();
        println!("Result: 3 scaling variants (0.5x, 1x, 2x)");
        println!();
    }

    // Example 3: Named function factory (cleaner for reuse)
    println!("--- Example 3: Reusable Factory Function ---\n");
    {
        // Define factory as a regular function (can be reused)
        fn make_adder(amount: i32) -> impl Fn(&HashMap<String, String>, &HashMap<String, String>) -> HashMap<String, String> {
            move |inputs, _| {
                let mut outputs = HashMap::new();
                if let Some(val) = inputs.get("value").and_then(|s| s.parse::<i32>().ok()) {
                    outputs.insert("sum".to_string(), (val + amount).to_string());
                }
                outputs
            }
        }

        let mut graph = Graph::new();

        graph.add(
            |_, _| {
                let mut out = HashMap::new();
                out.insert("value".to_string(), "5".to_string());
                out
            },
            Some("Start"),
            None,
            Some(vec!["value"]),
        );

        graph.variant_factory(
            make_adder,
            vec![10, 20, 30],
            Some("Add"),
            Some(vec!["value"]),
            Some(vec!["sum"]),
        );

        println!("Syntax (named factory):");
        println!("  fn make_adder(amount: i32) -> impl Fn(...) {{ ... }}");
        println!("  graph.variant_factory(make_adder, vec![10, 20, 30], ...)");
        println!();
        println!("Result: 3 variants adding 10, 20, 30 respectively");
        println!();
    }

    println!("=== Key Points ===");
    println!("1. Factory takes a parameter → returns a node function");
    println!("2. One variant node created per parameter value");
    println!("3. All variants branch from same point (parallel execution)");
    println!("4. Keeps port broadcasting (inputs + variant_params)");
}
