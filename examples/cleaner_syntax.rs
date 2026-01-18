//! Example showing cleaner syntax with named functions

use graph_sp::Graph;
use std::collections::HashMap;

// Define function handles as regular functions
fn data_source(_inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let mut outputs = HashMap::new();
    outputs.insert("data".to_string(), "hello world".to_string());
    outputs
}

fn uppercase(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let mut outputs = HashMap::new();
    if let Some(data) = inputs.get("data") {
        outputs.insert("result".to_string(), data.to_uppercase());
    }
    outputs
}

fn add_exclamation(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let mut outputs = HashMap::new();
    if let Some(result) = inputs.get("result") {
        outputs.insert("final".to_string(), format!("{}!", result));
    }
    outputs
}

fn multiply_by_2(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let mut outputs = HashMap::new();
    if let Some(val) = inputs.get("value") {
        if let Ok(num) = val.parse::<i32>() {
            outputs.insert("result_a".to_string(), (num * 2).to_string());
        }
    }
    outputs
}

fn multiply_by_3(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let mut outputs = HashMap::new();
    if let Some(val) = inputs.get("value") {
        if let Ok(num) = val.parse::<i32>() {
            outputs.insert("result_b".to_string(), (num * 3).to_string());
        }
    }
    outputs
}

fn combine_results(inputs: &HashMap<String, String>, _variant_params: &HashMap<String, String>) -> HashMap<String, String> {
    let mut outputs = HashMap::new();
    let a = inputs.get("result_a").and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
    let b = inputs.get("result_b").and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
    outputs.insert("combined".to_string(), format!("Sum: {}", a + b));
    outputs
}

fn main() {
    println!("=== Cleaner Syntax Demo ===\n");

    // Example 1: Simple pipeline with named functions
    println!("--- Example 1: Named Functions ---");
    {
        let mut graph = Graph::new();

        // Much cleaner - just pass the function name!
        graph.add(data_source, Some("Source"), None, Some(vec!["data"]));
        graph.add(uppercase, Some("Uppercase"), Some(vec!["data"]), Some(vec!["result"]));
        graph.add(add_exclamation, Some("Add !"), Some(vec!["result"]), Some(vec!["final"]));

        let dag = graph.build();
        let result = dag.execute();

        println!("Output: {}\n", result.get("final").unwrap());
    }

    // Example 2: Branching with named functions
    println!("--- Example 2: Branching (Named Functions) ---");
    {
        let mut graph = Graph::new();

        // Source with inline closure (when simple)
        graph.add(
            |_, _| {
                let mut outputs = HashMap::new();
                outputs.insert("value".to_string(), "10".to_string());
                outputs
            },
            Some("Source"),
            None,
            Some(vec!["value"]),
        );

        // Create branches with named functions
        let mut branch_a = Graph::new();
        branch_a.add(multiply_by_2, Some("Multiply x2"), Some(vec!["value"]), Some(vec!["result_a"]));

        let mut branch_b = Graph::new();
        branch_b.add(multiply_by_3, Some("Multiply x3"), Some(vec!["value"]), Some(vec!["result_b"]));

        graph.branch(branch_a);
        graph.branch(branch_b);
        graph.merge();

        graph.add(combine_results, Some("Combine"), Some(vec!["result_a", "result_b"]), Some(vec!["combined"]));

        let dag = graph.build();
        let result = dag.execute();

        println!("Combined result: {}\n", result.get("combined").unwrap());
    }

    // Example 3: Mix of styles - closures for simple, functions for complex
    println!("--- Example 3: Mixed Style ---");
    {
        let mut graph = Graph::new();

        // Simple inline closure for trivial sources
        graph.add(
            |_, _| {
                let mut out = HashMap::new();
                out.insert("x".to_string(), "100".to_string());
                out
            },
            Some("Init"),
            None,
            Some(vec!["x"]),
        );

        // Named function for more complex logic
        graph.add(uppercase, Some("Process"), Some(vec!["x"]), Some(vec!["y"]));

        let dag = graph.build();
        println!("Mermaid:\n{}", dag.to_mermaid());
    }
}
