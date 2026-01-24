// Example 01: Minimal Pipeline
// Demonstrates the simplest dataflow: generator → transformer → aggregator

mod benchmark_utils;

use dagex::{Graph, GraphData};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use benchmark_utils::{Benchmark, print_header, print_section};

fn generate(_inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut outputs = HashMap::new();
    outputs.insert("number".to_string(), GraphData::int(10));
    outputs
}

fn double(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let value = inputs.get("x").and_then(|d| d.as_int()).unwrap_or(0);
    
    // Simulate I/O-bound work (file read, network call, database query, etc.)
    // that benefits from parallelization
    thread::sleep(Duration::from_millis(10));
    
    let mut outputs = HashMap::new();
    outputs.insert("result".to_string(), GraphData::int(value * 2));
    outputs
}

fn add_five(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let value = inputs.get("y").and_then(|d| d.as_int()).unwrap_or(0);
    
    // Simulate I/O-bound work (file read, network call, database query, etc.)
    // that benefits from parallelization
    thread::sleep(Duration::from_millis(10));
    
    let mut outputs = HashMap::new();
    outputs.insert("final".to_string(), GraphData::int(value + 5));
    outputs
}

fn main() {
    print_header("Example 01: Minimal Pipeline");
    
    println!("📖 Story:");
    println!("   This example shows the simplest possible DAG pipeline:");
    println!("   A generator creates a number, a transformer doubles it,");
    println!("   and a final node adds five to produce the result.\n");
    
    print_section("Building the Graph");
    
    let mut graph = Graph::new();
    graph.add(
        generate,
        Some("Generator"),
        None,
        Some(vec![("number", "x")])
    );
    graph.add(
        double,
        Some("Doubler"),
        Some(vec![("x", "x")]),
        Some(vec![("result", "y")])
    );
    graph.add(
        add_five,
        Some("AddFive"),
        Some(vec![("y", "y")]),
        Some(vec![("final", "output")])
    );
    
    let dag = graph.build();
    
    print_section("Mermaid Diagram");
    println!("{}", dag.to_mermaid());
    
    print_section("ASCII Visualization");
    println!("  Generator → Doubler → AddFive");
    println!("     (10)       (20)       (25)");
    
    print_section("Sequential Execution (parallel=false)");
    
    let bench = Benchmark::start("Sequential execution");
    let context_seq = dag.execute(false, None);
    let result_seq = bench.finish_and_print();
    
    print_section("Parallel Execution (parallel=true)");
    
    let bench = Benchmark::start("Parallel execution");
    let context_par = dag.execute(true, Some(4));
    let result_par = bench.finish_and_print();
    
    print_section("Results");
    
    println!("Sequential execution:");
    if let Some(output) = context_seq.get("output") {
        if let Some(value) = output.as_int() {
            println!("  Final output: {}", value);
            println!("  Time: {:.3}ms", result_seq.duration_ms);
        }
    }
    
    println!("\nParallel execution:");
    if let Some(output) = context_par.get("output") {
        if let Some(value) = output.as_int() {
            println!("  Final output: {}", value);
            println!("  Time: {:.3}ms", result_par.duration_ms);
        }
    }
    
    println!("\n✅ Pipeline completed successfully!");
    println!("   (Started with 10, doubled to 20, added 5 = 25)");
    
    println!();
}
