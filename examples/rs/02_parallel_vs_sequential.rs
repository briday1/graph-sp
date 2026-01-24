// Example 02: Parallel vs Sequential Execution
// Demonstrates how independent nodes execute in parallel

mod benchmark_utils;

use dagex::{Graph, GraphData};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use benchmark_utils::{Benchmark, print_header, print_section};

fn source(_inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut outputs = HashMap::new();
    outputs.insert("value".to_string(), GraphData::int(100));
    outputs
}

fn task_a(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let value = inputs.get("input").and_then(|d| d.as_int()).unwrap_or(0);
    // Simulate some work
    thread::sleep(Duration::from_millis(150));
    let mut outputs = HashMap::new();
    outputs.insert("result_a".to_string(), GraphData::int(value + 10));
    outputs
}

fn task_b(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let value = inputs.get("input").and_then(|d| d.as_int()).unwrap_or(0);
    // Simulate some work
    thread::sleep(Duration::from_millis(150));
    let mut outputs = HashMap::new();
    outputs.insert("result_b".to_string(), GraphData::int(value + 20));
    outputs
}

fn task_c(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let value = inputs.get("input").and_then(|d| d.as_int()).unwrap_or(0);
    // Simulate some work
    thread::sleep(Duration::from_millis(150));
    let mut outputs = HashMap::new();
    outputs.insert("result_c".to_string(), GraphData::int(value + 30));
    outputs
}

fn main() {
    print_header("Example 02: Parallel vs Sequential Execution");
    
    println!("📖 Story:");
    println!("   When nodes at the same level have no dependencies between them,");
    println!("   they can execute in parallel. This example shows three independent");
    println!("   tasks (A, B, C) that each take ~50ms. Sequential execution takes");
    println!("   ~150ms total, while parallel execution takes only ~50ms.\n");
    
    print_section("Building the Graph");
    
    let mut graph = Graph::new();
    graph.add(
        source,
        Some("Source"),
        None,
        Some(vec![("value", "input")])
    );
    graph.add(
        task_a,
        Some("TaskA"),
        Some(vec![("input", "input")]),
        Some(vec![("result_a", "a")])
    );
    graph.add(
        task_b,
        Some("TaskB"),
        Some(vec![("input", "input")]),
        Some(vec![("result_b", "b")])
    );
    graph.add(
        task_c,
        Some("TaskC"),
        Some(vec![("input", "input")]),
        Some(vec![("result_c", "c")])
    );
    
    let dag = graph.build();
    
    print_section("Mermaid Diagram");
    println!("{}", dag.to_mermaid());
    
    print_section("ASCII Visualization");
    println!("        TaskA (+10)");
    println!("       /");
    println!("  Source -- TaskB (+20)");
    println!("       \\");
    println!("        TaskC (+30)");
    
    print_section("Sequential Execution");
    
    let bench = Benchmark::start("Sequential execution");
    let context_seq = dag.execute(false, None);
    let result_seq = bench.finish_and_print();
    
    print_section("Parallel Execution");
    
    let bench = Benchmark::start("Parallel execution");
    let context_par = dag.execute(true, Some(4));
    let result_par = bench.finish_and_print();
    
    print_section("Results");
    
    println!("Sequential results:");
    println!("  TaskA: {}", context_seq.get("a").and_then(|d| d.as_int()).unwrap_or(0));
    println!("  TaskB: {}", context_seq.get("b").and_then(|d| d.as_int()).unwrap_or(0));
    println!("  TaskC: {}", context_seq.get("c").and_then(|d| d.as_int()).unwrap_or(0));
    println!("  Time: {:.3}ms", result_seq.duration_ms);
    
    println!("\nParallel results:");
    println!("  TaskA: {}", context_par.get("a").and_then(|d| d.as_int()).unwrap_or(0));
    println!("  TaskB: {}", context_par.get("b").and_then(|d| d.as_int()).unwrap_or(0));
    println!("  TaskC: {}", context_par.get("c").and_then(|d| d.as_int()).unwrap_or(0));
    println!("  Time: {:.3}ms", result_par.duration_ms);
    
    let speedup = result_seq.duration_ms / result_par.duration_ms;
    println!("\n⚡ Speedup: {:.2}x faster with parallel execution!", speedup);
    
    println!();
}
