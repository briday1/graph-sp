//! Example: Parallel execution demonstrating true parallelization
//!
//! This example shows how independent branches of the DAG can execute in parallel

use graph_sp::core::{Edge, Graph, Node, NodeConfig, Port, PortData};
use graph_sp::executor::Executor;
use graph_sp::inspector::Inspector;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Graph-SP Example: Parallel Execution ===\n");

    let mut graph = Graph::new();

    // Source node that outputs multiple values
    let source_config = NodeConfig::new(
        "source",
        "Data Source",
        vec![],
        vec![Port::new("value", "Value")],
        Arc::new(|_: &HashMap<String, PortData>| {
            println!("[source] Generating data...");
            let mut outputs = HashMap::new();
            outputs.insert("value".to_string(), PortData::Int(100));
            Ok(outputs)
        }),
    );

    // Branch A - Slow operation (simulating heavy computation)
    let branch_a = NodeConfig::new(
        "branch_a",
        "Branch A (Slow)",
        vec![Port::new("input", "Input")],
        vec![Port::new("output", "Output")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let start = Instant::now();
            println!("[branch_a] Starting slow operation...");

            // Simulate heavy computation
            std::thread::sleep(Duration::from_millis(500));

            let mut outputs = HashMap::new();
            if let Some(PortData::Int(val)) = inputs.get("input") {
                outputs.insert("output".to_string(), PortData::Int(val * 2));
            }

            println!("[branch_a] Completed in {:?}", start.elapsed());
            Ok(outputs)
        }),
    );

    // Branch B - Fast operation
    let branch_b = NodeConfig::new(
        "branch_b",
        "Branch B (Fast)",
        vec![Port::new("input", "Input")],
        vec![Port::new("output", "Output")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let start = Instant::now();
            println!("[branch_b] Starting fast operation...");

            // Simulate quick computation
            std::thread::sleep(Duration::from_millis(100));

            let mut outputs = HashMap::new();
            if let Some(PortData::Int(val)) = inputs.get("input") {
                outputs.insert("output".to_string(), PortData::Int(val + 50));
            }

            println!("[branch_b] Completed in {:?}", start.elapsed());
            Ok(outputs)
        }),
    );

    // Branch C - Medium operation
    let branch_c = NodeConfig::new(
        "branch_c",
        "Branch C (Medium)",
        vec![Port::new("input", "Input")],
        vec![Port::new("output", "Output")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let start = Instant::now();
            println!("[branch_c] Starting medium operation...");

            // Simulate medium computation
            std::thread::sleep(Duration::from_millis(300));

            let mut outputs = HashMap::new();
            if let Some(PortData::Int(val)) = inputs.get("input") {
                outputs.insert("output".to_string(), PortData::Int(val / 2));
            }

            println!("[branch_c] Completed in {:?}", start.elapsed());
            Ok(outputs)
        }),
    );

    // Merge node - Combines results from all branches
    let merge_config = NodeConfig::new(
        "merger",
        "Result Merger",
        vec![
            Port::new("a", "Branch A Result"),
            Port::new("b", "Branch B Result"),
            Port::new("c", "Branch C Result"),
        ],
        vec![Port::new("result", "Final Result")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let start = Instant::now();
            println!("[merger] Merging results...");

            let mut outputs = HashMap::new();

            let a = inputs
                .get("a")
                .and_then(|d| match d {
                    PortData::Int(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or(0);

            let b = inputs
                .get("b")
                .and_then(|d| match d {
                    PortData::Int(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or(0);

            let c = inputs
                .get("c")
                .and_then(|d| match d {
                    PortData::Int(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or(0);

            let combined = a + b + c;
            outputs.insert("result".to_string(), PortData::Int(combined));

            println!("[merger] Completed in {:?}", start.elapsed());
            Ok(outputs)
        }),
    );

    // Build the graph
    println!("Building parallel graph...");
    graph.add(Node::new(source_config)).unwrap();
    graph.add(Node::new(branch_a)).unwrap();
    graph.add(Node::new(branch_b)).unwrap();
    graph.add(Node::new(branch_c)).unwrap();
    graph.add(Node::new(merge_config)).unwrap();

    // Connect source to all branches (fan-out)
    graph
        .add_edge(Edge::new("source", "value", "branch_a", "input"))
        .unwrap();
    graph
        .add_edge(Edge::new("source", "value", "branch_b", "input"))
        .unwrap();
    graph
        .add_edge(Edge::new("source", "value", "branch_c", "input"))
        .unwrap();

    // Connect all branches to merger (fan-in)
    graph
        .add_edge(Edge::new("branch_a", "output", "merger", "a"))
        .unwrap();
    graph
        .add_edge(Edge::new("branch_b", "output", "merger", "b"))
        .unwrap();
    graph
        .add_edge(Edge::new("branch_c", "output", "merger", "c"))
        .unwrap();

    println!("✓ Graph built successfully!\n");

    // Validate
    println!("Validating graph...");
    graph.validate()?;
    println!("✓ Graph is valid (no cycles detected)\n");

    // Analyze
    println!("=== Graph Analysis ===");
    let analysis = Inspector::analyze(&graph);
    println!("{}\n", analysis.summary());
    println!("This graph has 3 independent branches that can execute in parallel!\n");

    // Visualize structure
    println!("=== Graph Structure ===");
    let visualization = Inspector::visualize(&graph)?;
    println!("{}", visualization);

    // Generate Mermaid diagram
    println!("=== Mermaid Diagram ===");
    let mermaid = Inspector::to_mermaid(&graph)?;
    println!("{}", mermaid);

    // Execute with timing
    println!("=== Executing Graph ===");
    println!("Note: Branches A, B, and C will execute in parallel after the source completes.\n");

    let overall_start = Instant::now();
    let executor = Executor::new();
    let result = executor.execute(&mut graph).await?;
    let total_time = overall_start.elapsed();

    println!("\n✓ Execution completed!\n");

    // Display results
    println!("=== Results ===");

    if let Some(PortData::Int(val)) = result.get_output("source", "value") {
        println!("Source value: {}", val);
    }

    if let Some(PortData::Int(val)) = result.get_output("branch_a", "output") {
        println!("Branch A result (×2): {}", val);
    }

    if let Some(PortData::Int(val)) = result.get_output("branch_b", "output") {
        println!("Branch B result (+50): {}", val);
    }

    if let Some(PortData::Int(val)) = result.get_output("branch_c", "output") {
        println!("Branch C result (÷2): {}", val);
    }

    if let Some(PortData::Int(val)) = result.get_output("merger", "result") {
        println!("Final merged result: {}", val);
    }

    println!("\n=== Performance Analysis ===");
    println!("Total execution time: {:?}", total_time);
    println!("\nExpected times:");
    println!("  - Sequential execution: ~900ms (500 + 100 + 300)");
    println!("  - Parallel execution: ~500ms (max of branch times)");

    if total_time.as_millis() < 700 {
        println!("\n✓ Parallel execution confirmed! Branches executed concurrently.");
        println!("The executor identified 3 independent branches and ran them in parallel.");
    } else {
        println!("\n⚠ Sequential execution detected. Execution time matches sequential.");
    }

    println!("\n=== Example Complete ===");

    Ok(())
}
