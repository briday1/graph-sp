// Example 05: Output Access
// Demonstrates accessing individual node and branch outputs

mod benchmark_utils;

use dagex::{Graph, GraphData};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use benchmark_utils::{Benchmark, print_header, print_section};

fn source(_inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut outputs = HashMap::new();
    outputs.insert("data".to_string(), GraphData::int(100));
    outputs
}

fn processor_a(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let value = inputs.get("input").and_then(|d| d.as_int()).unwrap_or(0);
    
    // Simulate I/O-bound work (file read, network call, database query, etc.)
    // that benefits from parallelization
    thread::sleep(Duration::from_millis(150));
    
    let mut outputs = HashMap::new();
    outputs.insert("processed".to_string(), GraphData::int(value * 2));
    outputs
}

fn processor_b(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let value = inputs.get("input").and_then(|d| d.as_int()).unwrap_or(0);
    
    // Simulate I/O-bound work (file read, network call, database query, etc.)
    // that benefits from parallelization
    thread::sleep(Duration::from_millis(150));
    
    let mut outputs = HashMap::new();
    outputs.insert("processed".to_string(), GraphData::int(value + 50));
    outputs
}

fn main() {
    print_header("Example 05: Output Access");
    
    println!("📖 Story:");
    println!("   Sometimes you need access to intermediate results, not just the");
    println!("   final outputs. The execute_detailed() method returns an ExecutionResult");
    println!("   with context (final outputs), node_outputs (per-node results), and");
    println!("   branch_outputs (per-branch results).\n");
    
    print_section("Building the Graph");
    
    let mut graph = Graph::new();
    
    // Add source
    graph.add(
        source,
        Some("Source"),
        None,
        Some(vec![("data", "input")])
    );
    
    // Create branch A
    let mut branch_a = Graph::new();
    branch_a.add(
        processor_a,
        Some("ProcessorA"),
        Some(vec![("input", "input")]),
        Some(vec![("processed", "result_a")])
    );
    let branch_a_id = graph.branch(branch_a);
    
    // Create branch B
    let mut branch_b = Graph::new();
    branch_b.add(
        processor_b,
        Some("ProcessorB"),
        Some(vec![("input", "input")]),
        Some(vec![("processed", "result_b")])
    );
    let branch_b_id = graph.branch(branch_b);
    
    // Merge branches
    graph.merge(
        |inputs: &HashMap<String, GraphData>| -> HashMap<String, GraphData> {
            let a = inputs.get("from_a").and_then(|d| d.as_int()).unwrap_or(0);
            let b = inputs.get("from_b").and_then(|d| d.as_int()).unwrap_or(0);
            let mut outputs = HashMap::new();
            outputs.insert("final".to_string(), GraphData::int(a + b + 1));
            outputs
        },
        Some("MergeNode"),
        vec![
            (branch_a_id, "result_a", "from_a"),
            (branch_b_id, "result_b", "from_b"),
        ],
        Some(vec![("final", "output")])
    );
    
    let dag = graph.build();
    
    print_section("Mermaid Diagram");
    println!("{}", dag.to_mermaid());
    
    print_section("Sequential Execution (parallel=false)");
    
    let bench = Benchmark::start("Sequential execution");
    let _result_seq = dag.execute_detailed(false, None);
    let bench_result_seq = bench.finish_and_print();
    
    print_section("Parallel Execution (parallel=true)");
    
    let bench = Benchmark::start("Parallel execution");
    let result_par = dag.execute_detailed(true, Some(4));
    let bench_result_par = bench.finish_and_print();
    
    print_section("Results");
    
    println!("📊 Accessing different output levels:\n");
    
    println!("Sequential execution:");
    println!("  Time: {:.3}ms", bench_result_seq.duration_ms);
    
    println!("\nParallel execution:");
    println!("  Time: {:.3}ms", bench_result_par.duration_ms);
    
    // Final context outputs
    println!("\n1. Final context outputs:");
    if let Some(output) = result_par.context.get("output") {
        if let Some(value) = output.as_int() {
            println!("   output: {}", value);
        }
    }
    
    // Node outputs
    println!("\n2. Individual node outputs:");
    println!("   Total nodes executed: {}", result_par.node_outputs.len());
    for (node_id, outputs) in result_par.node_outputs.iter() {
        println!("   Node {}: {} outputs", node_id, outputs.len());
    }
    
    // Branch outputs
    println!("\n3. Branch-specific outputs:");
    println!("   Total branches: {}", result_par.branch_outputs.len());
    for (branch_id, outputs) in result_par.branch_outputs.iter() {
        println!("   Branch {}:", branch_id);
        for (key, value) in outputs.iter() {
            if let Some(v) = value.as_int() {
                println!("     {}: {}", key, v);
            }
        }
    }
    
    println!("\n✅ Successfully accessed all output levels!");
    
    println!();
}
