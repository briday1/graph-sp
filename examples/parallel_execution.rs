//! Example: Parallel execution demonstrating true parallelization
//!
//! This example shows how independent branches of the DAG can execute in parallel

use graph_sp::core::{Graph, Node, NodeConfig, Port, PortData};
use graph_sp::executor::Executor;
use graph_sp::inspector::Inspector;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Graph-SP Example: Parallel Execution ===\n");

    // Use strict edge mapping to avoid implicit previous-node wiring
    let mut graph = Graph::with_strict_edges();

    // Source node that outputs multiple values
    let source_config = NodeConfig::new(
        "source",
        "Data Source",
        vec![],
        vec![Port::simple("input")], // Output "input" to match branch inputs
        Arc::new(|_: &HashMap<String, PortData>| {
            println!("[source] Generating data...");
            let mut outputs = HashMap::new();
            outputs.insert("input".to_string(), PortData::Int(100));
            Ok(outputs)
        }),
    );

    // Branch A - Slow operation (simulating heavy computation)
    let branch_a = NodeConfig::new(
        "branch_a",
        "Branch A (Slow)",
        vec![Port::simple("input")],
        vec![Port::simple("a")], // Output port "a" will connect to merger input "a"
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let start = Instant::now();
            println!("[branch_a] Starting slow operation...");

            // Simulate heavy computation
            std::thread::sleep(Duration::from_millis(500));

            let mut outputs = HashMap::new();
            if let Some(PortData::Int(val)) = inputs.get("input") {
                outputs.insert("a".to_string(), PortData::Int(val * 2));
            }

            println!("[branch_a] Completed in {:?}", start.elapsed());
            Ok(outputs)
        }),
    );

    // Branch B - Fast operation
    let branch_b = NodeConfig::new(
        "branch_b",
        "Branch B (Fast)",
        vec![Port::simple("input")],
        vec![Port::simple("b")], // Output port "b" will connect to merger input "b"
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let start = Instant::now();
            println!("[branch_b] Starting fast operation...");

            // Simulate quick computation
            std::thread::sleep(Duration::from_millis(100));

            let mut outputs = HashMap::new();
            if let Some(PortData::Int(val)) = inputs.get("input") {
                outputs.insert("b".to_string(), PortData::Int(val + 50));
            }

            println!("[branch_b] Completed in {:?}", start.elapsed());
            Ok(outputs)
        }),
    );

    // Branch C - Medium operation
    let branch_c = NodeConfig::new(
        "branch_c",
        "Branch C (Medium)",
        vec![Port::simple("input")],
        vec![Port::simple("c")], // Output port "c" will connect to merger input "c"
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let start = Instant::now();
            println!("[branch_c] Starting medium operation...");

            // Simulate medium computation
            std::thread::sleep(Duration::from_millis(300));

            let mut outputs = HashMap::new();
            if let Some(PortData::Int(val)) = inputs.get("input") {
                outputs.insert("c".to_string(), PortData::Int(val / 2));
            }

            println!("[branch_c] Completed in {:?}", start.elapsed());
            Ok(outputs)
        }),
    );

    // Merge node - Combines results from all branches
    let merge_config = NodeConfig::new(
        "merger",
        "Result Merger",
        vec![Port::simple("a"), Port::simple("b"), Port::simple("c")],
        vec![Port::simple("result")],
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

    // Auto-connect everything! Port names match:
    // source "input" -> branch_a/b/c "input"
    // branch_a "a" -> merger "a"
    // branch_b "b" -> merger "b"
    // branch_c "c" -> merger "c"
    let auto_edges = graph.auto_connect().unwrap();
    println!("✓ Graph built! {} edges auto-connected\n", auto_edges);

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
