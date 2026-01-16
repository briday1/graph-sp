//! Example: Simple pipeline demonstrating graph-sp capabilities

use graph_sp::core::{Graph, Node, NodeConfig, Port, PortData, Edge};
use graph_sp::executor::Executor;
use graph_sp::inspector::Inspector;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Graph-SP Example: Data Processing Pipeline ===\n");

    // Create a new graph
    let mut graph = Graph::new();

    // Node 1: Data Source - generates initial data
    let source_config = NodeConfig::new(
        "data_source",
        "Data Source",
        vec![],
        vec![Port::new("numbers", "Numbers")],
        Arc::new(|_inputs: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            // Output a list of numbers
            outputs.insert(
                "numbers".to_string(),
                PortData::List(vec![
                    PortData::Int(1),
                    PortData::Int(2),
                    PortData::Int(3),
                    PortData::Int(4),
                    PortData::Int(5),
                ]),
            );
            Ok(outputs)
        }),
    );

    // Node 2: Multiplier - multiplies each number by 2
    let multiplier_config = NodeConfig::new(
        "multiplier",
        "Multiplier (x2)",
        vec![Port::new("input", "Input")],
        vec![Port::new("output", "Output")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            if let Some(PortData::List(numbers)) = inputs.get("input") {
                let multiplied: Vec<PortData> = numbers
                    .iter()
                    .map(|n| {
                        if let PortData::Int(val) = n {
                            PortData::Int(val * 2)
                        } else {
                            n.clone()
                        }
                    })
                    .collect();
                outputs.insert("output".to_string(), PortData::List(multiplied));
            }
            Ok(outputs)
        }),
    );

    // Node 3: Sum Calculator - sums all numbers
    let sum_config = NodeConfig::new(
        "sum_calculator",
        "Sum Calculator",
        vec![Port::new("input", "Input")],
        vec![Port::new("sum", "Sum"), Port::new("count", "Count")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            if let Some(PortData::List(numbers)) = inputs.get("input") {
                let sum: i64 = numbers
                    .iter()
                    .filter_map(|n| {
                        if let PortData::Int(val) = n {
                            Some(*val)
                        } else {
                            None
                        }
                    })
                    .sum();
                let count = numbers.len() as i64;
                outputs.insert("sum".to_string(), PortData::Int(sum));
                outputs.insert("count".to_string(), PortData::Int(count));
            }
            Ok(outputs)
        }),
    );

    // Node 4: Average Calculator - calculates average
    let avg_config = NodeConfig::new(
        "avg_calculator",
        "Average Calculator",
        vec![Port::new("sum", "Sum"), Port::new("count", "Count")],
        vec![Port::new("average", "Average")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            if let (Some(PortData::Int(sum)), Some(PortData::Int(count))) =
                (inputs.get("sum"), inputs.get("count"))
            {
                if *count > 0 {
                    let avg = *sum as f64 / *count as f64;
                    outputs.insert("average".to_string(), PortData::Float(avg));
                }
            }
            Ok(outputs)
        }),
    );

    // Add all nodes to the graph
    println!("Building graph...");
    graph.add_node(Node::new(source_config))?;
    graph.add_node(Node::new(multiplier_config))?;
    graph.add_node(Node::new(sum_config))?;
    graph.add_node(Node::new(avg_config))?;

    // Connect the nodes
    graph.add_edge(Edge::new("data_source", "numbers", "multiplier", "input"))?;
    graph.add_edge(Edge::new("multiplier", "output", "sum_calculator", "input"))?;
    graph.add_edge(Edge::new("sum_calculator", "sum", "avg_calculator", "sum"))?;
    graph.add_edge(Edge::new("sum_calculator", "count", "avg_calculator", "count"))?;

    println!("Graph built successfully!\n");

    // Validate the graph
    println!("Validating graph...");
    graph.validate()?;
    println!("✓ Graph is valid (no cycles detected)\n");

    // Analyze the graph
    println!("=== Graph Analysis ===");
    let analysis = Inspector::analyze(&graph);
    println!("{}\n", analysis.summary());

    // Visualize the graph structure
    println!("=== Graph Structure ===");
    let visualization = Inspector::visualize(&graph)?;
    println!("{}", visualization);

    // Generate Mermaid diagram
    println!("=== Mermaid Diagram ===");
    let mermaid = Inspector::to_mermaid(&graph)?;
    println!("{}", mermaid);

    // Get optimization suggestions
    println!("=== Optimization Suggestions ===");
    let optimizations = Inspector::suggest_optimizations(&graph);
    if optimizations.is_empty() {
        println!("No optimizations suggested - graph is well-structured!\n");
    } else {
        for opt in optimizations {
            println!("- {}", opt.description);
        }
        println!();
    }

    // Execute the graph
    println!("=== Executing Graph ===");
    let executor = Executor::new();
    println!("Running parallel execution...");
    
    let result = executor.execute(&mut graph).await?;
    
    println!("✓ Execution completed successfully!\n");

    // Display results
    println!("=== Results ===");
    
    // Get original numbers from data source
    if let Some(PortData::List(numbers)) = result.get_output("data_source", "numbers") {
        print!("Original numbers: [");
        for (i, num) in numbers.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{}", num);
        }
        println!("]");
    }

    // Get multiplied numbers
    if let Some(PortData::List(numbers)) = result.get_output("multiplier", "output") {
        print!("After multiplying by 2: [");
        for (i, num) in numbers.iter().enumerate() {
            if i > 0 {
                print!(", ");
            }
            print!("{}", num);
        }
        println!("]");
    }

    // Get sum
    if let Some(PortData::Int(sum)) = result.get_output("sum_calculator", "sum") {
        println!("Sum: {}", sum);
    }

    // Get average
    if let Some(PortData::Float(avg)) = result.get_output("avg_calculator", "average") {
        println!("Average: {:.2}", avg);
    }

    println!("\n=== Example Complete ===");

    Ok(())
}
