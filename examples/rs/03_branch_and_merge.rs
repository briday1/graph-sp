// Example 03: Branch and Merge
// Demonstrates fan-out (branching) and fan-in (merging) patterns

mod benchmark_utils;

use dagex::{Graph, GraphData};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use benchmark_utils::{Benchmark, print_header, print_section};

fn source(_inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut outputs = HashMap::new();
    outputs.insert("data".to_string(), GraphData::int(50));
    outputs
}

fn path_a(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let value = inputs.get("x").and_then(|d| d.as_int()).unwrap_or(0);
    
    // Simulate I/O-bound work (file read, network call, database query, etc.)
    // that benefits from parallelization
    thread::sleep(Duration::from_millis(150));
    
    let mut outputs = HashMap::new();
    outputs.insert("result".to_string(), GraphData::int(value + 10));
    outputs
}

fn path_b(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let value = inputs.get("x").and_then(|d| d.as_int()).unwrap_or(0);
    
    // Simulate I/O-bound work (file read, network call, database query, etc.)
    // that benefits from parallelization
    thread::sleep(Duration::from_millis(150));
    
    let mut outputs = HashMap::new();
    outputs.insert("result".to_string(), GraphData::int(value + 20));
    outputs
}

fn merge(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let a = inputs.get("from_a").and_then(|d| d.as_int()).unwrap_or(0);
    let b = inputs.get("from_b").and_then(|d| d.as_int()).unwrap_or(0);
    let mut outputs = HashMap::new();
    outputs.insert("combined".to_string(), GraphData::int(a + b));
    outputs
}

fn main() {
    print_header("Example 03: Branch and Merge");
    
    println!("📖 Story:");
    println!("   Fan-out (branch): Create independent subgraphs that run in parallel.");
    println!("   Fan-in (merge): Combine branch-specific outputs safely.");
    println!("   This pattern is useful for processing data through multiple");
    println!("   independent pipelines and then combining the results.\n");
    
    print_section("Building the Graph");
    
    let mut graph = Graph::new();
    
    // Add source
    graph.add(
        source,
        Some("Source"),
        None,
        Some(vec![("data", "x")])
    );
    
    // Create branch A
    let mut branch_a = Graph::new();
    branch_a.add(
        path_a,
        Some("PathA (+10)"),
        Some(vec![("x", "x")]),
        Some(vec![("result", "result")])
    );
    let branch_a_id = graph.branch(branch_a);
    
    // Create branch B
    let mut branch_b = Graph::new();
    branch_b.add(
        path_b,
        Some("PathB (+20)"),
        Some(vec![("x", "x")]),
        Some(vec![("result", "result")])
    );
    let branch_b_id = graph.branch(branch_b);
    
    // Merge branches
    graph.merge(
        merge,
        Some("Merge"),
        vec![
            (branch_a_id, "result", "from_a"),
            (branch_b_id, "result", "from_b"),
        ],
        Some(vec![("combined", "final")])
    );
    
    let dag = graph.build();
    
    print_section("Mermaid Diagram");
    println!("{}", dag.to_mermaid());
    
    print_section("ASCII Visualization");
    println!("          PathA (+10) ──┐");
    println!("         /                \\");
    println!("  Source                   Merge");
    println!("         \\                /");
    println!("          PathB (+20) ──┘");
    
    print_section("Sequential Execution (parallel=false)");
    
    let bench = Benchmark::start("Sequential execution");
    let context_seq = dag.execute(false, None);
    let result_seq = bench.finish_and_print();
    
    print_section("Parallel Execution (parallel=true)");
    
    let bench = Benchmark::start("Parallel execution");
    let context_par = dag.execute(true, Some(4));
    let result_par = bench.finish_and_print();
    
    print_section("Results");
    
    println!("📊 Execution flow:");
    println!("   Source: 50");
    println!("   PathA: 50 + 10 = 60");
    println!("   PathB: 50 + 20 = 70");
    println!("   Merge: 60 + 70 = 130");
    
    println!("\nSequential execution:");
    if let Some(output) = context_seq.get("final") {
        if let Some(value) = output.as_int() {
            println!("  Final output: {}", value);
            println!("  Time: {:.3}ms", result_seq.duration_ms);
        }
    }
    
    println!("\nParallel execution:");
    if let Some(output) = context_par.get("final") {
        if let Some(value) = output.as_int() {
            println!("  Final output: {}", value);
            println!("  Time: {:.3}ms", result_par.duration_ms);
        }
    }
    
    println!("\n✅ Branch and merge completed successfully!");
    
    println!();
}
