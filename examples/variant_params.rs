//! Example showing how variant parameters are passed to node functions

use graph_sp::{Graph, Linspace};
use std::collections::HashMap;

// Node function that uses variant parameters
fn train_model(
    inputs: &HashMap<String, String>,
    variant_params: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut outputs = HashMap::new();
    
    // Get regular input
    let default_data = "no_data".to_string();
    let data = inputs.get("training_data").unwrap_or(&default_data);
    
    // Get variant parameter (learning rate)
    let default_lr = "0.01".to_string();
    let learning_rate = variant_params.get("learning_rate").unwrap_or(&default_lr);
    
    // Simulate training with the learning rate
    let result = format!("Model trained on {} with lr={}", data, learning_rate);
    outputs.insert("model".to_string(), result);
    
    outputs
}

fn evaluate_model(
    inputs: &HashMap<String, String>,
    variant_params: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut outputs = HashMap::new();
    
    let default_model = "no_model".to_string();
    let model = inputs.get("model").unwrap_or(&default_model);
    let default_lr = "unknown".to_string();
    let learning_rate = variant_params.get("learning_rate").unwrap_or(&default_lr);
    
    // Simulate evaluation - in reality, this would depend on the actual learning rate used
    let accuracy = format!("Accuracy for lr={}: {}", learning_rate, model);
    outputs.insert("accuracy".to_string(), accuracy);
    
    outputs
}

fn main() {
    println!("=== Variant Parameters Demo ===\n");

    // Example 1: Simple variant sweep with parameter injection
    println!("--- Example 1: Learning Rate Sweep ---");
    {
        let mut graph = Graph::new();

        // Data source node (no variant params needed)
        graph.add(
            |_inputs, _variant_params| {
                let mut outputs = HashMap::new();
                outputs.insert("training_data".to_string(), "dataset_v1".to_string());
                outputs
            },
            Some("Load Data"),
            None,
            Some(vec!["training_data"]),
        );

        // Training node - will receive variant parameter "learning_rate"
        graph.add(
            train_model,
            Some("Train Model"),
            Some(vec!["training_data"]),
            Some(vec!["model"]),
        );

        // Evaluation node - also receives the learning_rate variant parameter
        graph.add(
            evaluate_model,
            Some("Evaluate"),
            Some(vec!["model"]),
            Some(vec!["accuracy"]),
        );

        // Create 5 variants with different learning rates
        graph.variant("learning_rate", Linspace::new(0.001, 0.1, 5));

        let dag = graph.build();
        let result = dag.execute();

        println!("Created {} variants with different learning rates", dag.stats().variant_count);
        println!("\nSample output:");
        
        // Try to get some results (note: the actual keys depend on node IDs)
        for (key, value) in result.iter().take(5) {
            println!("  {}: {}", key, value);
        }
        println!();
    }

    // Example 2: Multiple parameters in variant sweep
    println!("--- Example 2: Using Variant Parameters in Computation ---");
    {
        let mut graph = Graph::new();

        graph.add(
            |_inputs, _variant_params| {
                let mut outputs = HashMap::new();
                outputs.insert("x".to_string(), "10".to_string());
                outputs
            },
            Some("Source"),
            None,
            Some(vec!["x"]),
        );

        // This node uses the variant parameter in its computation
        graph.add(
            |inputs, variant_params| {
                let mut outputs = HashMap::new();
                
                let x = inputs.get("x").and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
                let multiplier = variant_params.get("multiplier")
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(1.0);
                
                let result = x * multiplier;
                outputs.insert("result".to_string(), result.to_string());
                
                outputs
            },
            Some("Multiply"),
            Some(vec!["x"]),
            Some(vec!["result"]),
        );

        // Variant sweep with different multipliers
        graph.variant("multiplier", vec![1.0, 2.0, 5.0, 10.0]);

        let dag = graph.build();
        let result = dag.execute();

        println!("Created {} variants with multipliers: 1, 2, 5, 10", dag.stats().variant_count);
        println!("Each variant computes: x * multiplier\n");
    }

    // Example 3: Lambda functions with variant parameters
    println!("--- Example 3: Inline Lambdas with Variants ---");
    {
        let mut graph = Graph::new();

        graph.add(
            |_inputs, _variant_params| {
                let mut out = HashMap::new();
                out.insert("base".to_string(), "100".to_string());
                out
            },
            Some("Initialize"),
            None,
            Some(vec!["base"]),
        );

        // Inline lambda that uses variant parameter
        graph.add(
            |inputs, variant_params| {
                let mut outputs = HashMap::new();
                
                if let (Some(base), Some(rate)) = 
                    (inputs.get("base").and_then(|s| s.parse::<f64>().ok()),
                     variant_params.get("discount_rate").and_then(|s| s.parse::<f64>().ok()))
                {
                    let discounted = base * (1.0 - rate);
                    outputs.insert("price".to_string(), discounted.to_string());
                }
                
                outputs
            },
            Some("Apply Discount"),
            Some(vec!["base"]),
            Some(vec!["price"]),
        );

        graph.variant("discount_rate", vec![0.0, 0.1, 0.2, 0.3, 0.5]);

        let dag = graph.build();
        
        println!("Created {} variants with discount rates: 0%, 10%, 20%, 30%, 50%", 
                 dag.stats().variant_count);
        println!("Mermaid diagram:\n{}", dag.to_mermaid());
    }

    println!("=== Demo Complete ===");
}
