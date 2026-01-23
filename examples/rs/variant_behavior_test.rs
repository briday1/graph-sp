use dagex::{Graph, GraphData, NodeFunction};
use std::collections::HashMap;
use std::sync::Arc;

fn main() {
    println!("=== Variant Behavior Test ===\n");
    println!("Testing what happens with variants and downstream nodes\n");

    // Test 1: Variants + downstream node
    println!("─────────────────────────────────────────────────────────");
    println!("Test 1: Do downstream nodes connect to ALL variants?");
    println!("─────────────────────────────────────────────────────────\n");

    let mut graph = Graph::new();

    graph.add(
        Arc::new(|_| {
            let mut result = HashMap::new();
            result.insert("x".to_string(), GraphData::int(10));
            result
        }),
        Some("Source"),
        None,
        Some(vec![("x", "data")]),
    );

    let factors = vec![2, 3, 5];
    graph.variants(
        factors
            .iter()
            .map(|&f| {
                Arc::new(move |inputs: &HashMap<String, GraphData>| {
                    let mut result = HashMap::new();
                    if let Some(val) = inputs.get("input").and_then(|d: &GraphData| d.as_int()) {
                        println!("  Variant {} executing: {} * {}", f, val, f);
                        result.insert("scaled".to_string(), GraphData::int(val * f));
                    }
                    result
                }) as NodeFunction
            })
            .collect(),
        Some("Scale"),
        Some(vec![("data", "input")]),
        Some(vec![("scaled", "result")]),
    );

    // Add a downstream node AFTER variants
    graph.add(
        Arc::new(|inputs: &HashMap<String, GraphData>| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("r").and_then(|d: &GraphData| d.as_int()) {
                println!("  Downstream executing with: {}", val);
                result.insert("final".to_string(), GraphData::int(val + 100));
            }
            result
        }),
        Some("Downstream"),
        Some(vec![("result", "r")]),
        Some(vec![("final", "output")]),
    );

    let dag = graph.build();
    println!("\nMermaid diagram:");
    println!("{}", dag.to_mermaid());

    println!("\n📊 DAG Stats:");
    let stats = dag.stats();
    println!("  Total nodes: {}", stats.node_count);
    println!("  Depth: {}", stats.depth);
    println!("  Max parallelism: {}", stats.max_parallelism);

    println!("\n🔍 Execution levels:");
    for (level, nodes) in dag.execution_levels().iter().enumerate() {
        print!("  Level {}: ", level);
        for node_id in nodes {
            if let Some(node) = dag.nodes().iter().find(|n| n.id == *node_id) {
                print!("{} ", node.display_name());
            }
        }
        println!();
    }

    println!("\n⚙️  Executing...");
    let context = dag.execute(false, None);

    println!("\n✅ Final context:");
    println!("  data: {}", context.get("data").unwrap().to_string_repr());
    println!("  result: {}", context.get("result").unwrap().to_string_repr());
    if let Some(output) = context.get("output") {
        println!("  output: {}", output.to_string_repr());
    } else {
        println!("  output: NOT IN CONTEXT");
    }

    // Test 2: Variants + branches
    println!("\n─────────────────────────────────────────────────────────");
    println!("Test 2: Do branches replicate for each variant?");
    println!("─────────────────────────────────────────────────────────\n");

    let mut graph2 = Graph::new();

    graph2.add(
        Arc::new(|_| {
            let mut result = HashMap::new();
            result.insert("x".to_string(), GraphData::int(10));
            result
        }),
        Some("Source"),
        None,
        Some(vec![("x", "data")]),
    );

    let factors2 = vec![2, 3];
    graph2.variants(
        factors2
            .iter()
            .map(|&f| {
                Arc::new(move |inputs: &HashMap<String, GraphData>| {
                    let mut result = HashMap::new();
                    if let Some(val) = inputs.get("input").and_then(|d: &GraphData| d.as_int()) {
                        println!("  Variant {} executing: {} * {}", f, val, f);
                        result.insert("scaled".to_string(), GraphData::int(val * f));
                    }
                    result
                }) as NodeFunction
            })
            .collect(),
        Some("Scale"),
        Some(vec![("data", "input")]),
        Some(vec![("scaled", "result")]),
    );

    // Add branches AFTER variants
    let mut branch_a = Graph::new();
    branch_a.add(
        Arc::new(|inputs: &HashMap<String, GraphData>| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("r").and_then(|d: &GraphData| d.as_int()) {
                println!("  Branch A executing with: {}", val);
                result.insert("a_result".to_string(), GraphData::int(val + 10));
            }
            result
        }),
        Some("Branch A"),
        Some(vec![("result", "r")]),
        Some(vec![("a_result", "branch_a_out")]),
    );

    let mut branch_b = Graph::new();
    branch_b.add(
        Arc::new(|inputs: &HashMap<String, GraphData>| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("r").and_then(|d: &GraphData| d.as_int()) {
                println!("  Branch B executing with: {}", val);
                result.insert("b_result".to_string(), GraphData::int(val + 20));
            }
            result
        }),
        Some("Branch B"),
        Some(vec![("result", "r")]),
        Some(vec![("b_result", "branch_b_out")]),
    );

    graph2.branch(branch_a);
    graph2.branch(branch_b);

    let dag2 = graph2.build();
    println!("\nMermaid diagram:");
    println!("{}", dag2.to_mermaid());

    println!("\n📊 DAG Stats:");
    let stats2 = dag2.stats();
    println!("  Total nodes: {}", stats2.node_count);

    println!("\n⚙️  Executing...");
    let context2 = dag2.execute(false, None);

    println!("\n✅ Results:");
    println!("  result: {}", context2.get("result").unwrap().to_string_repr());
    if let Some(a) = context2.get("branch_a_out") {
        println!("  branch_a_out: {}", a.to_string_repr());
    } else {
        println!("  branch_a_out: NOT IN CONTEXT");
    }
    if let Some(b) = context2.get("branch_b_out") {
        println!("  branch_b_out: {}", b.to_string_repr());
    } else {
        println!("  branch_b_out: NOT IN CONTEXT");
    }

    // Test 3: Nested variants (cartesian product test)
    println!("\n─────────────────────────────────────────────────────────");
    println!("Test 3: Nested variants - do we get cartesian products?");
    println!("─────────────────────────────────────────────────────────\n");

    let mut graph3 = Graph::new();

    graph3.add(
        Arc::new(|_| {
            let mut result = HashMap::new();
            result.insert("x".to_string(), GraphData::int(10));
            result
        }),
        Some("Source"),
        None,
        Some(vec![("x", "data")]),
    );

    let factors_a = vec![2, 3];
    graph3.variants(
        factors_a
            .iter()
            .map(|&f| {
                Arc::new(move |inputs: &HashMap<String, GraphData>| {
                    let mut result = HashMap::new();
                    if let Some(val) = inputs.get("input").and_then(|d: &GraphData| d.as_int()) {
                        println!("  Variant A-{} executing: {} * {}", f, val, f);
                        result.insert("scaled".to_string(), GraphData::int(val * f));
                    }
                    result
                }) as NodeFunction
            })
            .collect(),
        Some("ScaleA"),
        Some(vec![("data", "input")]),
        Some(vec![("scaled", "result_a")]),
    );

    let factors_b = vec![5, 7];
    graph3.variants(
        factors_b
            .iter()
            .map(|&f| {
                Arc::new(move |inputs: &HashMap<String, GraphData>| {
                    let mut result = HashMap::new();
                    if let Some(val) = inputs.get("input").and_then(|d: &GraphData| d.as_int()) {
                        println!("  Variant B-{} executing: {} * {}", f, val, f);
                        result.insert("scaled".to_string(), GraphData::int(val * f));
                    }
                    result
                }) as NodeFunction
            })
            .collect(),
        Some("ScaleB"),
        Some(vec![("result_a", "input")]),
        Some(vec![("scaled", "result_b")]),
    );

    let dag3 = graph3.build();
    println!("\nMermaid diagram:");
    println!("{}", dag3.to_mermaid());

    println!("\n📊 DAG Stats:");
    let stats3 = dag3.stats();
    println!("  Total nodes: {}", stats3.node_count);
    println!("  Expected for cartesian: 1 + 2 + (2*2) = 7 nodes");
    println!("  Actual: {} nodes", stats3.node_count);

    println!("\n⚙️  Executing...");
    let context3 = dag3.execute(false, None);

    println!("\n✅ Results:");
    if let Some(a) = context3.get("result_a") {
        println!("  result_a: {}", a.to_string_repr());
    }
    if let Some(b) = context3.get("result_b") {
        println!("  result_b: {}", b.to_string_repr());
    }

    println!("\n═══════════════════════════════════════════════════════════");
    println!("CONCLUSIONS:");
    println!("═══════════════════════════════════════════════════════════");
    println!("1. Downstream nodes after variants():");
    println!("   → Check if Downstream connects to all variants or just last");
    println!("\n2. Branches after variants():");
    println!("   → Check if branches replicate for each variant");
    println!("\n3. Nested variants():");
    println!("   → Check if we get cartesian product (2*2=4) or sequential (2+2)");
}
