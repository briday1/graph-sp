//! Demonstration of parallel execution capabilities
//!
//! This example shows:
//! - Simulated parallel execution with runtime measurements
//! - Data dependency analysis
//! - Maximum parallelism detection
//! - Port mapping visualization in Mermaid diagrams

use graph_sp::Graph;
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  Parallel Execution Demonstration");
    println!("  Showing runtime benefits of parallelization");
    println!("═══════════════════════════════════════════════════════════\n");
    
    demo_sequential_vs_parallel();
    demo_complex_dependencies();
    demo_variant_parallelism();
    demo_diamond_pattern();
}

/// Simulates a CPU-intensive operation
fn simulate_work(ms: u64, input: &str) -> String {
    thread::sleep(Duration::from_millis(ms));
    format!("{}_processed", input)
}

fn demo_sequential_vs_parallel() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 1: Sequential vs Parallel Execution");
    println!("─────────────────────────────────────────────────────────");
    
    let mut graph = Graph::new();
    
    // Source node
    graph.add(
        |_: &HashMap<String, String>, _| {
            let start = Instant::now();
            let mut result = HashMap::new();
            result.insert("data".to_string(), "source_data".to_string());
            println!("    [{}ms] Source completed", start.elapsed().as_millis());
            result
        },
        Some("Source"),
        None,
        Some(vec![("data", "data")])
    );
    
    // Branch A: 100ms work
    let mut branch_a = Graph::new();
    branch_a.add(
        |inputs: &HashMap<String, String>, _| {
            let start = Instant::now();
            let mut result = HashMap::new();
            if let Some(data) = inputs.get("input") {
                let processed = simulate_work(100, data);
                result.insert("result".to_string(), processed);
            }
            println!("    [{}ms] Branch A completed (100ms work)", start.elapsed().as_millis());
            result
        },
        Some("BranchA[100ms]"),
        Some(vec![("data", "input")]),
        Some(vec![("result", "result_a")])
    );
    
    // Branch B: 100ms work
    let mut branch_b = Graph::new();
    branch_b.add(
        |inputs: &HashMap<String, String>, _| {
            let start = Instant::now();
            let mut result = HashMap::new();
            if let Some(data) = inputs.get("input") {
                let processed = simulate_work(100, data);
                result.insert("result".to_string(), processed);
            }
            println!("    [{}ms] Branch B completed (100ms work)", start.elapsed().as_millis());
            result
        },
        Some("BranchB[100ms]"),
        Some(vec![("data", "input")]),
        Some(vec![("result", "result_b")])
    );
    
    // Branch C: 100ms work
    let mut branch_c = Graph::new();
    branch_c.add(
        |inputs: &HashMap<String, String>, _| {
            let start = Instant::now();
            let mut result = HashMap::new();
            if let Some(data) = inputs.get("input") {
                let processed = simulate_work(100, data);
                result.insert("result".to_string(), processed);
            }
            println!("    [{}ms] Branch C completed (100ms work)", start.elapsed().as_millis());
            result
        },
        Some("BranchC[100ms]"),
        Some(vec![("data", "input")]),
        Some(vec![("result", "result_c")])
    );
    
    graph.branch(branch_a);
    graph.branch(branch_b);
    graph.branch(branch_c);
    
    let dag = graph.build();
    
    println!("\n📊 Sequential Execution (simulated):");
    let start = Instant::now();
    let _ = dag.execute();
    let sequential_time = start.elapsed();
    println!("  Total time: {}ms", sequential_time.as_millis());
    
    println!("\n⚡ With Parallel Execution:");
    println!("  Expected time: ~100ms (all branches run simultaneously)");
    println!("  Speedup: ~3x faster than sequential");
    
    println!("\n📈 DAG Statistics:");
    let stats = dag.stats();
    println!("{}", stats.summary());
    println!("\n  Analysis:");
    println!("  - Level 0: 1 node  (Source)");
    println!("  - Level 1: 3 nodes (BranchA, BranchB, BranchC) ← Can run in parallel!");
    println!("  - Max parallelism: {} nodes can execute simultaneously", stats.max_parallelism);
    
    println!("\n🔍 Mermaid Visualization with Port Mappings:");
    println!("{}", dag.to_mermaid());
    println!();
}

fn demo_complex_dependencies() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 2: Complex Data Dependencies");
    println!("─────────────────────────────────────────────────────────");
    
    let mut graph = Graph::new();
    
    // Two independent sources
    graph.add(
        |_: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            result.insert("source1_data".to_string(), "100".to_string());
            result
        },
        Some("Source1"),
        None,
        Some(vec![("source1_data", "data1")])
    );
    
    graph.add(
        |_: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            result.insert("source2_data".to_string(), "200".to_string());
            result
        },
        Some("Source2"),
        None,
        Some(vec![("source2_data", "data2")])
    );
    
    // Process each source independently (can run in parallel)
    graph.add(
        |inputs: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("in").and_then(|s| s.parse::<i32>().ok()) {
                thread::sleep(Duration::from_millis(50));
                result.insert("processed".to_string(), (val * 2).to_string());
            }
            result
        },
        Some("Process1[50ms]"),
        Some(vec![("data1", "in")]),
        Some(vec![("processed", "proc1")])
    );
    
    graph.add(
        |inputs: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("in").and_then(|s| s.parse::<i32>().ok()) {
                thread::sleep(Duration::from_millis(50));
                result.insert("processed".to_string(), (val * 3).to_string());
            }
            result
        },
        Some("Process2[50ms]"),
        Some(vec![("data2", "in")]),
        Some(vec![("processed", "proc2")])
    );
    
    // Combine results (depends on both processors)
    graph.add(
        |inputs: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            let v1 = inputs.get("p1").and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
            let v2 = inputs.get("p2").and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
            thread::sleep(Duration::from_millis(30));
            result.insert("combined".to_string(), format!("{}", v1 + v2));
            result
        },
        Some("Combine[30ms]"),
        Some(vec![("proc1", "p1"), ("proc2", "p2")]),
        Some(vec![("combined", "final")])
    );
    
    let dag = graph.build();
    
    println!("\n📊 Execution with timing:");
    let start = Instant::now();
    let context = dag.execute();
    let total_time = start.elapsed();
    
    println!("  Source1: data1 = {}", context.get("data1").unwrap());
    println!("  Source2: data2 = {}", context.get("data2").unwrap());
    println!("  Process1: proc1 = {} (data1 * 2)", context.get("proc1").unwrap());
    println!("  Process2: proc2 = {} (data2 * 3)", context.get("proc2").unwrap());
    println!("  Combine: final = {} (proc1 + proc2)", context.get("final").unwrap());
    println!("\n  Total execution time: {}ms", total_time.as_millis());
    
    println!("\n📈 Execution Levels (showing parallelism):");
    for (level_idx, level) in dag.execution_levels().iter().enumerate() {
        print!("  Level {}: ", level_idx);
        let node_names: Vec<String> = level.iter()
            .map(|&node_id| dag.nodes().iter().find(|n| n.id == node_id).unwrap().display_name())
            .collect();
        println!("{}", node_names.join(", "));
        if level.len() > 1 {
            println!("           ↑ {} nodes can execute in parallel!", level.len());
        }
    }
    
    println!("\n⚡ Parallel Execution Analysis:");
    println!("  Sequential time would be: 50+50+30 = 130ms");
    println!("  With parallelism: Level0→Level1(parallel)→Level2 = ~80ms");
    println!("  Speedup: 1.6x");
    
    println!("\n🔍 Mermaid Visualization (shows data dependencies):");
    println!("{}", dag.to_mermaid());
    println!();
}

fn demo_variant_parallelism() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 3: Variant Parameter Sweep Parallelism");
    println!("─────────────────────────────────────────────────────────");
    
    let mut graph = Graph::new();
    
    // Source
    graph.add(
        |_: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            result.insert("value".to_string(), "1000".to_string());
            result
        },
        Some("DataSource"),
        None,
        Some(vec![("value", "data")])
    );
    
    // Variant factory with different multipliers
    fn make_multiplier(factor: f64) -> impl Fn(&HashMap<String, String>, &HashMap<String, String>) -> HashMap<String, String> {
        move |inputs: &HashMap<String, String>, _| {
            let start = Instant::now();
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("input").and_then(|s| s.parse::<f64>().ok()) {
                // Simulate 100ms of work
                thread::sleep(Duration::from_millis(100));
                result.insert("result".to_string(), format!("{:.1}", val * factor));
            }
            println!("    [{}ms] Variant (factor={}) completed", start.elapsed().as_millis(), factor);
            result
        }
    }
    
    // Create 5 variants
    graph.variant(
        make_multiplier,
        vec![0.5, 1.0, 1.5, 2.0, 2.5],
        Some("Multiply[100ms]"),
        Some(vec![("data", "input")]),
        Some(vec![("result", "result")])
    );
    
    let dag = graph.build();
    
    println!("\n📊 Executing 5 variants (each takes 100ms):");
    let start = Instant::now();
    let _ = dag.execute();
    let total_time = start.elapsed();
    
    println!("\n  Total execution time: {}ms", total_time.as_millis());
    
    println!("\n⚡ Parallelism Analysis:");
    println!("  Sequential execution: 100 × 5 = 500ms");
    println!("  With parallel execution: ~100ms (all run simultaneously)");
    println!("  Speedup: 5x");
    
    println!("\n📈 DAG Statistics:");
    let stats = dag.stats();
    println!("{}", stats.summary());
    println!("  ↑ All {} variant nodes can execute in parallel!", stats.variant_count);
    
    println!("\n🔍 Mermaid Visualization:");
    println!("{}", dag.to_mermaid());
    println!();
}

fn demo_diamond_pattern() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 4: Diamond Pattern (Fan-Out → Fan-In)");
    println!("─────────────────────────────────────────────────────────");
    println!("This pattern shows:");
    println!("  - One source splits into multiple parallel branches");
    println!("  - Branches are processed independently");
    println!("  - Results merge back into single output");
    
    let mut graph = Graph::new();
    
    // Top of diamond: Single source
    graph.add(
        |_: &HashMap<String, String>, _| {
            let mut result = HashMap::new();
            result.insert("raw".to_string(), "input_data".to_string());
            result
        },
        Some("Source"),
        None,
        Some(vec![("raw", "data")])
    );
    
    // Left branch: Transform A (50ms)
    let mut branch_a = Graph::new();
    branch_a.add(
        |inputs: &HashMap<String, String>, _| {
            let start = Instant::now();
            thread::sleep(Duration::from_millis(50));
            let mut result = HashMap::new();
            if let Some(data) = inputs.get("in") {
                result.insert("out".to_string(), format!("{}_transformA", data));
            }
            println!("    [{}ms] Transform A completed", start.elapsed().as_millis());
            result
        },
        Some("TransformA[50ms]"),
        Some(vec![("data", "in")]),
        Some(vec![("out", "result")])
    );
    
    // Right branch: Transform B (50ms)
    let mut branch_b = Graph::new();
    branch_b.add(
        |inputs: &HashMap<String, String>, _| {
            let start = Instant::now();
            thread::sleep(Duration::from_millis(50));
            let mut result = HashMap::new();
            if let Some(data) = inputs.get("in") {
                result.insert("out".to_string(), format!("{}_transformB", data));
            }
            println!("    [{}ms] Transform B completed", start.elapsed().as_millis());
            result
        },
        Some("TransformB[50ms]"),
        Some(vec![("data", "in")]),
        Some(vec![("out", "result")])
    );
    
    let branch_a_id = graph.branch(branch_a);
    let branch_b_id = graph.branch(branch_b);
    
    // Bottom of diamond: Merge (30ms)
    graph.merge(
        |inputs: &HashMap<String, String>, _| {
            let start = Instant::now();
            thread::sleep(Duration::from_millis(30));
            let mut result = HashMap::new();
            let a = inputs.get("left").cloned().unwrap_or_default();
            let b = inputs.get("right").cloned().unwrap_or_default();
            result.insert("merged".to_string(), format!("[{}+{}]", a, b));
            println!("    [{}ms] Merge completed", start.elapsed().as_millis());
            result
        },
        Some("Merge[30ms]"),
        vec![
            (branch_a_id, "result", "left"),
            (branch_b_id, "result", "right")
        ],
        Some(vec![("merged", "final")])
    );
    
    let dag = graph.build();
    
    println!("\n📊 Executing diamond pattern:");
    let start = Instant::now();
    let context = dag.execute();
    let total_time = start.elapsed();
    
    println!("\n  Result: {}", context.get("final").unwrap());
    println!("  Total execution time: {}ms", total_time.as_millis());
    
    println!("\n📈 Execution Levels:");
    for (level_idx, level) in dag.execution_levels().iter().enumerate() {
        print!("  Level {}: ", level_idx);
        let node_names: Vec<String> = level.iter()
            .map(|&node_id| dag.nodes().iter().find(|n| n.id == node_id).unwrap().display_name())
            .collect();
        println!("{}", node_names.join(", "));
    }
    
    println!("\n⚡ Timing Analysis:");
    println!("  Sequential: Source(0ms) + TransformA(50ms) + TransformB(50ms) + Merge(30ms) = 130ms");
    println!("  Parallel:   Source(0ms) → [TransformA + TransformB](50ms) → Merge(30ms) = 80ms");
    println!("  Speedup: 1.6x");
    
    println!("\n🔍 Mermaid Visualization (Diamond Shape):");
    println!("{}", dag.to_mermaid());
    
    println!("\n  The visualization shows:");
    println!("  - Port mappings on edges (data→in, result→left, result→right)");
    println!("  - Data dependencies between nodes");
    println!("  - Parallel branches can execute simultaneously");
    
    println!("\n═══════════════════════════════════════════════════════════");
    println!("  Parallel Execution Demo Complete!");
    println!("═══════════════════════════════════════════════════════════");
}
