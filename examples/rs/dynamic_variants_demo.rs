use dagex::{Graph, GraphData, NodeFunction};
use std::collections::HashMap;
use std::sync::Arc;

fn main() {
    println!("🚀 Dynamic Variant Generation Demo");
    println!("{}", "=".repeat(50));
    
    let mut graph = Graph::new();
    
    // Add source
    graph.add(
        Arc::new(|_| {
            let mut result = HashMap::new();
            result.insert("data".to_string(), GraphData::float(100.0));
            result
        }),
        Some("Source"),
        None,
        Some(vec![("data", "input")]),
    );
    
    // Example 1: Simple range sweep
    println!("1. Generating 20 learning rate variants...");
    let lr_variants = generate_learning_rate_sweep(0.001, 1.0, 20);
    graph.variants(lr_variants, Some("LearningRate"), Some(vec![("input", "value")]), None);
    
    // Example 2: Multi-dimensional hyperparameter sweep
    println!("2. Generating hyperparameter combinations...");
    let mut graph2 = Graph::new();
    graph2.add(
        Arc::new(|_| {
            let mut result = HashMap::new();
            result.insert("dataset".to_string(), GraphData::string("mnist".to_string()));
            result
        }),
        Some("DataSource"),
        None,
        Some(vec![("dataset", "data")]),
    );
    
    let hyper_variants = generate_hyperparameter_combinations();
    println!("   Generated {} hyperparameter combinations", hyper_variants.len());
    graph2.variants(hyper_variants, Some("HyperParams"), Some(vec![("data", "dataset")]), None);
    
    // Example 3: From external configuration
    println!("3. Generating from configuration data...");
    let config_variants = generate_from_config();
    println!("   Generated {} configuration variants", config_variants.len());
    
    // Execute and show results
    println!("\n📊 Execution Results:");
    let dag = graph.build();
    let results = dag.execute(true, None);
    println!("Graph 1 generated {} global outputs", results.len());
    
    let dag2 = graph2.build();
    let results2 = dag2.execute(true, None);
    println!("Graph 2 generated {} global outputs", results2.len());
}

/// Generate learning rate sweep variants
fn generate_learning_rate_sweep(start: f64, end: f64, steps: usize) -> Vec<NodeFunction> {
    (0..steps)
        .map(|i| {
            let lr = start * (end / start).powf(i as f64 / (steps - 1) as f64); // Log scale
            Arc::new(move |inputs: &HashMap<String, GraphData>| {
                let input_val = inputs.get("value").unwrap().as_float().unwrap();
                let adjusted = input_val * lr;
                let mut result = HashMap::new();
                result.insert("trained".to_string(), GraphData::float(adjusted));
                result.insert("lr_used".to_string(), GraphData::float(lr));
                println!("   Training with LR={:.6}: {:.2} -> {:.6}", lr, input_val, adjusted);
                result
            }) as NodeFunction
        })
        .collect()
}

/// Generate all combinations of hyperparameters
fn generate_hyperparameter_combinations() -> Vec<NodeFunction> {
    let learning_rates = vec![0.01, 0.1, 1.0];
    let batch_sizes = vec![32, 64, 128];
    let architectures = vec!["shallow", "deep", "wide"];
    
    let mut variants = Vec::new();
    
    for &lr in &learning_rates {
        for &bs in &batch_sizes {
            for &arch in &architectures {
                variants.push(create_hyperparameter_variant(lr, bs, arch));
            }
        }
    }
    
    variants
}

fn create_hyperparameter_variant(lr: f64, batch_size: i32, architecture: &str) -> NodeFunction {
    let arch_string = architecture.to_string();
    Arc::new(move |inputs: &HashMap<String, GraphData>| {
        let dataset = inputs.get("dataset").unwrap().as_string().unwrap();
        let mut result = HashMap::new();
        
        // Simulate training with these hyperparameters
        let accuracy = simulate_training(lr, batch_size, &arch_string, dataset);
        
        result.insert("accuracy".to_string(), GraphData::float(accuracy));
        result.insert("config".to_string(), 
            GraphData::string(format!("lr={}, bs={}, arch={}", lr, batch_size, arch_string)));
        
        println!("   Trained {}/{}/{}: accuracy={:.3}", lr, batch_size, arch_string, accuracy);
        result
    })
}

fn simulate_training(lr: f64, batch_size: i32, architecture: &str, _dataset: &str) -> f64 {
    // Fake training simulation - in real code this would be your ML training
    let base_accuracy = match architecture {
        "shallow" => 0.85,
        "deep" => 0.92,
        "wide" => 0.88,
        _ => 0.80,
    };
    
    // LR and batch size effects (simplified)
    let lr_factor = if lr > 0.5 { 0.95 } else if lr < 0.05 { 0.98 } else { 1.0 };
    let bs_factor = if batch_size > 100 { 1.02 } else { 1.0 };
    
    base_accuracy * lr_factor * bs_factor
}

/// Generate variants from external configuration
fn generate_from_config() -> Vec<NodeFunction> {
    // This could come from JSON, YAML, database, command line args, etc.
    let config_data = vec![
        ("experiment_1", vec![1.0, 2.0, 3.0]),
        ("experiment_2", vec![0.5, 1.5, 2.5, 3.5]),
        ("experiment_3", vec![0.1, 0.2, 0.3, 0.4, 0.5]),
    ];
    
    let mut variants = Vec::new();
    
    for (experiment_name, values) in config_data {
        for value in values {
            let exp_name = experiment_name.to_string();
            variants.push(Arc::new(move |_inputs: &HashMap<String, GraphData>| {
                let mut result = HashMap::new();
                result.insert("experiment".to_string(), GraphData::string(exp_name.clone()));
                result.insert("result".to_string(), GraphData::float(value * 10.0));
                result
            }) as NodeFunction);
        }
    }
    
    variants
}