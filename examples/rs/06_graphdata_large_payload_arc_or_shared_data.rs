// Example 06: GraphData with Arc (Large Payload Sharing)
// Demonstrates efficient memory sharing for large data using Arc

mod benchmark_utils;

use dagex::{Graph, GraphData};
use std::collections::HashMap;
use benchmark_utils::{Benchmark, print_header, print_section};

fn create_large_data(_inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    // Create a large vector (simulating a large dataset)
    let large_vec: Vec<i64> = (0..1_000_000).collect();
    
    let mut outputs = HashMap::new();
    // Store using Arc (int_vec automatically wraps in Arc)
    outputs.insert("large_data".to_string(), GraphData::int_vec(large_vec));
    outputs
}

fn consumer_a(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut outputs = HashMap::new();
    
    if let Some(data) = inputs.get("data") {
        if let Some(vec) = data.as_int_vec() {
            // Access the data through Arc - no copying!
            let sum: i64 = vec.iter().take(1000).sum();
            outputs.insert("sum_a".to_string(), GraphData::int(sum));
        }
    }
    
    outputs
}

fn consumer_b(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut outputs = HashMap::new();
    
    if let Some(data) = inputs.get("data") {
        if let Some(vec) = data.as_int_vec() {
            // Access the data through Arc - no copying!
            let sum: i64 = vec.iter().skip(1000).take(1000).sum();
            outputs.insert("sum_b".to_string(), GraphData::int(sum));
        }
    }
    
    outputs
}

fn consumer_c(inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut outputs = HashMap::new();
    
    if let Some(data) = inputs.get("data") {
        if let Some(vec) = data.as_int_vec() {
            // Access the data through Arc - no copying!
            let sum: i64 = vec.iter().skip(2000).take(1000).sum();
            outputs.insert("sum_c".to_string(), GraphData::int(sum));
        }
    }
    
    outputs
}

fn main() {
    print_header("Example 06: GraphData with Arc (Large Payload Sharing)");
    
    println!("📖 Story:");
    println!("   When working with large data, copying it between nodes is expensive.");
    println!("   GraphData automatically wraps large vectors (int_vec, float_vec) in Arc");
    println!("   for zero-copy sharing. Multiple nodes can read the same data without");
    println!("   duplication, saving both time and memory.\n");
    
    print_section("Building the Graph");
    
    let mut graph = Graph::new();
    
    // Create large data source
    graph.add(
        create_large_data,
        Some("CreateLargeData"),
        None,
        Some(vec![("large_data", "data")])
    );
    
    // Add multiple consumers that share the large data
    graph.add(
        consumer_a,
        Some("ConsumerA"),
        Some(vec![("data", "data")]),
        Some(vec![("sum_a", "sum_a")])
    );
    
    graph.add(
        consumer_b,
        Some("ConsumerB"),
        Some(vec![("data", "data")]),
        Some(vec![("sum_b", "sum_b")])
    );
    
    graph.add(
        consumer_c,
        Some("ConsumerC"),
        Some(vec![("data", "data")]),
        Some(vec![("sum_c", "sum_c")])
    );
    
    let dag = graph.build();
    
    print_section("Mermaid Diagram");
    println!("{}", dag.to_mermaid());
    
    print_section("ASCII Visualization");
    println!("                      ConsumerA");
    println!("                     /");
    println!("  CreateLargeData  -- ConsumerB");
    println!("   (1M integers)    \\");
    println!("                      ConsumerC");
    
    println!("💡 Key insight: The large data (1M integers) is created once");
    println!("   and shared via Arc. No copies are made when distributing");
    println!("   to multiple consumers.\n");
    
    print_section("Sequential Execution (parallel=false)");
    
    let bench = Benchmark::start("Sequential execution");
    let _context_seq = dag.execute(false, None);
    let result_seq = bench.finish_and_print();
    
    print_section("Parallel Execution (parallel=true)");
    
    let bench = Benchmark::start("Parallel execution");
    let context_par = dag.execute(true, Some(4));
    let result_par = bench.finish_and_print();
    
    print_section("Results");
    
    println!("📊 Consumer outputs (each processes different segments):");
    
    if let Some(sum_a) = context_par.get("sum_a").and_then(|d| d.as_int()) {
        println!("   ConsumerA (first 1000):  sum = {}", sum_a);
    }
    
    if let Some(sum_b) = context_par.get("sum_b").and_then(|d| d.as_int()) {
        println!("   ConsumerB (next 1000):   sum = {}", sum_b);
    }
    
    if let Some(sum_c) = context_par.get("sum_c").and_then(|d| d.as_int()) {
        println!("   ConsumerC (next 1000):   sum = {}", sum_c);
    }
    
    println!("\nSequential execution:");
    println!("  Time: {:.3}ms", result_seq.duration_ms);
    
    println!("\nParallel execution:");
    println!("  Time: {:.3}ms", result_par.duration_ms);
    
    println!("\n✅ Zero-copy data sharing successful!");
    println!("   Memory benefit: Only 1 copy of data exists, shared by all consumers");
    
    println!();
}
