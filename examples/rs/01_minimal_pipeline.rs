// Example 01: Minimal Pipeline
// Demonstrates the simplest dataflow: generator → transformer → aggregator

mod benchmark_utils;

use dagex::{Graph, GraphData};
use std::collections::HashMap;
use std::sync::Arc;
use benchmark_utils::{Benchmark, print_header, print_section};

fn generate(_inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut outputs = HashMap::new();
    outputs.insert("number".to_string(), GraphData::int(10));
    outputs
}

fn double(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let value = inputs.get("x").and_then(|d| d.as_int()).unwrap_or(0);
    let mut outputs = HashMap::new();
    outputs.insert("result".to_string(), GraphData::int(value * 2));
    outputs
}

fn add_five(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let value = inputs.get("y").and_then(|d| d.as_int()).unwrap_or(0);
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
        Arc::new(generate),
        Some("Generator"),
        None,
        Some(vec![("number", "x")])
    );
    graph.add(
        Arc::new(double),
        Some("Doubler"),
        Some(vec![("x", "x")]),
        Some(vec![("result", "y")])
    );
    graph.add(
        Arc::new(add_five),
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
    
    print_section("Execution");
    
    let bench = Benchmark::start("Pipeline execution");
    let context = dag.execute(false, None);
    let _result = bench.finish_and_print();
    
    print_section("Results");
    
    if let Some(output) = context.get("output") {
        if let Some(value) = output.as_int() {
            println!("✅ Final output: {}", value);
            println!("   (Started with 10, doubled to 20, added 5 = 25)");
        }
    }
    
    println!();
}
