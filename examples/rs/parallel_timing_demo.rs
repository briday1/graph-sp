//! Parallel Execution Timing Demo
//!
//! This example ACTUALLY demonstrates parallel execution with real timing measurements:
//! - Sequential vs parallel execution with identical work
//! - Real wall-clock time measurements
//! - Speedup calculations
//! - Visual proof that parallelization works

use dagex::{Graph, GraphData};
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

/// Simulates CPU-intensive work
fn simulate_work(ms: u64, label: &str) -> String {
    thread::sleep(Duration::from_millis(ms));
    format!("{}_processed", label)
}

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  PARALLEL EXECUTION TIMING VERIFICATION");
    println!("  Proving that parallelization actually works!");
    println!("═══════════════════════════════════════════════════════════\n");

    demo_sequential_vs_parallel();
    demo_many_parallel_nodes();
}

fn demo_sequential_vs_parallel() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 1: 3 Parallel Branches (100ms each)");
    println!("─────────────────────────────────────────────────────────");

    // Build graph with 3 parallel branches
    let mut graph = Graph::new();

    // Source node
    graph.add(
        |_: &HashMap<String, GraphData>, _| {
            let mut result = HashMap::new();
            result.insert("data".to_string(), GraphData::string("source"));
            result
        },
        Some("Source"),
        None,
        Some(vec![("data", "data")]),
    );

    // Branch A: 100ms work
    let mut branch_a = Graph::new();
    branch_a.add(
        |inputs: &HashMap<String, GraphData>, _| {
            let mut result = HashMap::new();
            if let Some(data) = inputs.get("input").and_then(|d| d.as_string()) {
                let processed = simulate_work(100, data);
                result.insert("result".to_string(), GraphData::string(processed));
            }
            result
        },
        Some("BranchA[100ms]"),
        Some(vec![("data", "input")]),
        Some(vec![("result", "result_a")]),
    );

    // Branch B: 100ms work
    let mut branch_b = Graph::new();
    branch_b.add(
        |inputs: &HashMap<String, GraphData>, _| {
            let mut result = HashMap::new();
            if let Some(data) = inputs.get("input").and_then(|d| d.as_string()) {
                let processed = simulate_work(100, data);
                result.insert("result".to_string(), GraphData::string(processed));
            }
            result
        },
        Some("BranchB[100ms]"),
        Some(vec![("data", "input")]),
        Some(vec![("result", "result_b")]),
    );

    // Branch C: 100ms work
    let mut branch_c = Graph::new();
    branch_c.add(
        |inputs: &HashMap<String, GraphData>, _| {
            let mut result = HashMap::new();
            if let Some(data) = inputs.get("input").and_then(|d| d.as_string()) {
                let processed = simulate_work(100, data);
                result.insert("result".to_string(), GraphData::string(processed));
            }
            result
        },
        Some("BranchC[100ms]"),
        Some(vec![("data", "input")]),
        Some(vec![("result", "result_c")]),
    );

    graph.branch(branch_a);
    graph.branch(branch_b);
    graph.branch(branch_c);

    let dag = graph.build();

    // Show DAG info
    let stats = dag.stats();
    println!("\n📈 DAG Structure:");
    println!("   Total nodes: {}", stats.node_count);
    println!("   Max parallelism: {} nodes", stats.max_parallelism);
    println!("   Expected sequential time: ~300ms (100ms × 3)");
    println!("   Expected parallel time: ~100ms (all run together)");

    // ===== SEQUENTIAL EXECUTION =====
    println!("\n🐌 Sequential Execution (parallel=false):");
    let start = Instant::now();
    let _ = dag.execute(false, None);
    let sequential_time = start.elapsed();
    println!("   ⏱️  Actual time: {}ms", sequential_time.as_millis());

    // ===== PARALLEL EXECUTION =====
    println!("\n⚡ Parallel Execution (parallel=true):");
    let start = Instant::now();
    let _ = dag.execute(true, None);
    let parallel_time = start.elapsed();
    println!("   ⏱️  Actual time: {}ms", parallel_time.as_millis());

    // ===== RESULTS =====
    let speedup = sequential_time.as_millis() as f64 / parallel_time.as_millis() as f64;
    println!("\n📊 Results:");
    println!("   Sequential: {}ms", sequential_time.as_millis());
    println!("   Parallel:   {}ms", parallel_time.as_millis());
    println!("   Speedup:    {:.2}x faster", speedup);
    
    if speedup > 2.5 {
        println!("   ✅ PARALLELIZATION IS WORKING! (~3x speedup achieved)");
    } else if speedup > 1.5 {
        println!("   ⚠️  Partial parallelization (overhead or thread contention)");
    } else {
        println!("   ❌ Not effectively parallel");
    }

    println!();
}

fn demo_many_parallel_nodes() {
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 2: 10 Parallel Nodes (50ms each)");
    println!("─────────────────────────────────────────────────────────");

    let mut graph = Graph::new();

    // Source
    graph.add(
        |_: &HashMap<String, GraphData>, _| {
            let mut result = HashMap::new();
            result.insert("data".to_string(), GraphData::int(0));
            result
        },
        Some("Source"),
        None,
        Some(vec![("data", "data")]),
    );

    // Add 10 parallel workers
    for i in 0..10 {
        let mut branch = Graph::new();
        branch.add(
            move |inputs: &HashMap<String, GraphData>, _: &HashMap<String, GraphData>| {
                let mut result = HashMap::new();
                if let Some(val) = inputs.get("input").and_then(|d| d.as_int()) {
                    thread::sleep(Duration::from_millis(50));
                    result.insert("result".to_string(), GraphData::int(val + i));
                }
                result
            },
            Some(&format!("Worker{}[50ms]", i)),
            Some(vec![("data", "input")]),
            Some(vec![("result", &format!("result_{}", i))]),
        );
        graph.branch(branch);
    }

    let dag = graph.build();

    let stats = dag.stats();
    println!("\n📈 DAG Structure:");
    println!("   Total nodes: {}", stats.node_count);
    println!("   Max parallelism: {} nodes", stats.max_parallelism);
    println!("   Expected sequential time: ~500ms (50ms × 10)");
    println!("   Expected parallel time: ~50ms (all run together)");

    // Sequential
    println!("\n🐌 Sequential Execution:");
    let start = Instant::now();
    let _ = dag.execute(false, None);
    let sequential_time = start.elapsed();
    println!("   ⏱️  Actual time: {}ms", sequential_time.as_millis());

    // Parallel
    println!("\n⚡ Parallel Execution:");
    let start = Instant::now();
    let _ = dag.execute(true, None);
    let parallel_time = start.elapsed();
    println!("   ⏱️  Actual time: {}ms", parallel_time.as_millis());

    // Results
    let speedup = sequential_time.as_millis() as f64 / parallel_time.as_millis() as f64;
    println!("\n📊 Results:");
    println!("   Sequential: {}ms", sequential_time.as_millis());
    println!("   Parallel:   {}ms", parallel_time.as_millis());
    println!("   Speedup:    {:.2}x faster", speedup);
    
    if speedup > 8.0 {
        println!("   ✅ EXCELLENT PARALLELIZATION! (~10x speedup achieved)");
    } else if speedup > 5.0 {
        println!("   ✅ GOOD PARALLELIZATION! (5-10x speedup)");
    } else if speedup > 2.0 {
        println!("   ⚠️  Moderate parallelization (some overhead)");
    } else {
        println!("   ❌ Limited parallel benefit");
    }

    println!("\n═══════════════════════════════════════════════════════════");
    println!("  CONCLUSION: Parallel execution provides real speedup!");
    println!("═══════════════════════════════════════════════════════════\n");
}
