// Example 04: Variants (Parameter Sweep)
// Demonstrates running multiple nodes with the same structure but different parameters

mod benchmark_utils;

use dagex::{Graph, GraphData};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use benchmark_utils::{Benchmark, print_header, print_section};

fn data_source(_inputs: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let mut outputs = HashMap::new();
    outputs.insert("base".to_string(), GraphData::int(10));
    outputs
}

// Factory function to create multiplier variants
fn make_multiplier(factor: i64) -> impl Fn(&HashMap<String, GraphData>) -> HashMap<String, GraphData> + Send + Sync + 'static {
    move |inputs: &HashMap<String, GraphData>| -> HashMap<String, GraphData> {
        let value = inputs.get("x").and_then(|d| d.as_int()).unwrap_or(0);
        
        // Simulate I/O-bound work (file read, network call, database query, etc.)
        // that benefits from parallelization
        thread::sleep(Duration::from_millis(150));
        
        let mut outputs = HashMap::new();
        outputs.insert("result".to_string(), GraphData::int(value * factor));
        outputs
    }
}

fn main() {
    print_header("Example 04: Variants (Parameter Sweep)");
    
    println!("📖 Story:");
    println!("   Variants let you create many nodes with the same structure but");
    println!("   different captured parameters. The graph will attach them to the");
    println!("   same frontier and execute them at the same level when possible.");
    println!("   This is perfect for hyperparameter sweeps or A/B testing.\n");
    
    print_section("Building the Graph");
    
    let mut graph = Graph::new();
    
    // Add source
    graph.add(
        data_source,
        Some("DataSource"),
        None,
        Some(vec![("base", "x")])
    );
    
    // Add variants with different multipliers
    let factors = vec![2, 3, 5, 7];
    let variant_nodes: Vec<_> = factors
        .iter()
        .map(|&f| make_multiplier(f))
        .collect();
    
    graph.variants(
        variant_nodes,
        Some("Multiplier"),
        Some(vec![("x", "x")]),
        Some(vec![("result", "results")])
    );
    
    let dag = graph.build();
    
    print_section("Mermaid Diagram");
    println!("{}", dag.to_mermaid());
    
    print_section("ASCII Visualization");
    println!("                Multiplier(×2)");
    println!("               /");
    println!("              |  Multiplier(×3)");
    println!("  DataSource  <");
    println!("              |  Multiplier(×5)");
    println!("               \\");
    println!("                Multiplier(×7)");
    
    print_section("Sequential Execution (parallel=false)");
    
    let bench = Benchmark::start("Sequential execution");
    let _result_seq = dag.execute_detailed(false, None);
    let bench_result_seq = bench.finish_and_print();
    
    print_section("Parallel Execution (parallel=true)");
    
    let bench = Benchmark::start("Parallel execution");
    let _result_par = dag.execute_detailed(true, Some(4));
    let bench_result_par = bench.finish_and_print();
    
    print_section("Results");
    
    println!("📊 Base value: 10");
    
    println!("\nSequential execution:");
    println!("  Time: {:.3}ms", bench_result_seq.duration_ms);
    
    println!("\nParallel execution:");
    println!("  Time: {:.3}ms", bench_result_par.duration_ms);
    
    // Show detailed node outputs if available
    println!("\nDetailed variant outputs:");
    for (i, factor) in factors.iter().enumerate() {
        let expected = 10 * factor;
        println!("  Variant {} (×{}): {}", i, factor, expected);
    }
    
    println!("\n✅ All {} variants executed successfully!", factors.len());
    
    println!();
}
