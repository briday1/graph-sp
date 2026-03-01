//! Integration tests for graph-sp

use dagex::{Dag, Distribution, Graph, GraphData, PredictTarget};
use std::collections::HashMap;

// Helper functions for tests

fn data_source(
    _: &HashMap<String, GraphData>,
) -> HashMap<String, GraphData> {
    let mut result = HashMap::new();
    result.insert("raw_data".to_string(), GraphData::int(100));
    result
}

fn processor(
    inputs: &HashMap<String, GraphData>,
) -> HashMap<String, GraphData> {
    let mut result = HashMap::new();
    if let Some(data) = inputs.get("input_data") {
        if let Some(val) = data.as_int() {
            result.insert("processed_value".to_string(), GraphData::int(val * 2));
        }
    }
    result
}

fn adder(
    inputs: &HashMap<String, GraphData>,
) -> HashMap<String, GraphData> {
    let mut result = HashMap::new();
    if let Some(val) = inputs.get("input") {
        if let Some(num) = val.as_int() {
            result.insert("sum".to_string(), GraphData::int(num + 10));
        }
    }
    result
}

#[test]
fn test_simple_pipeline() {
    let mut graph = Graph::new();

    graph.add(
        data_source,
        Some("Source"),
        None,
        Some(vec![("raw_data", "data")]),
    );

    graph.add(
        processor,
        Some("Process"),
        Some(vec![("data", "input_data")]),
        Some(vec![("processed_value", "result")]),
    );

    let dag = graph.build();
    let context = dag.execute(false, None);

    assert_eq!(context.get("data").and_then(|d: &GraphData| d.as_int()), Some(100));
    assert_eq!(context.get("result").and_then(|d: &GraphData| d.as_int()), Some(200));
}

#[test]
fn test_branching() {
    let mut graph = Graph::new();

    // Source node
    graph.add(
        data_source,
        Some("Source"),
        None,
        Some(vec![("raw_data", "data")]),
    );

    // Branch A
    let mut branch_a = Graph::new();
    branch_a.add(
        |inputs: &HashMap<String, GraphData>| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("x").and_then(|d: &GraphData| d.as_int()) {
                result.insert("output".to_string(), GraphData::int(val * 2));
            }
            result
        },
        Some("Branch A"),
        Some(vec![("data", "x")]),
        Some(vec![("output", "result_a")]),
    );

    // Branch B
    let mut branch_b = Graph::new();
    branch_b.add(
        |inputs: &HashMap<String, GraphData>| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("x").and_then(|d: &GraphData| d.as_int()) {
                result.insert("output".to_string(), GraphData::int(val * 3));
            }
            result
        },
        Some("Branch B"),
        Some(vec![("data", "x")]),
        Some(vec![("output", "result_b")]),
    );

    let _branch_a_id = graph.branch(branch_a);
    let _branch_b_id = graph.branch(branch_b);

    let dag = graph.build();
    let context = dag.execute(false, None);

    assert_eq!(context.get("data").and_then(|d: &GraphData| d.as_int()), Some(100));
    assert_eq!(context.get("result_a").and_then(|d: &GraphData| d.as_int()), Some(200));
    assert_eq!(context.get("result_b").and_then(|d: &GraphData| d.as_int()), Some(300));
}

#[test]
fn test_merge() {
    let mut graph = Graph::new();

    // Source node
    graph.add(
        data_source,
        Some("Source"),
        None,
        Some(vec![("raw_data", "data")]),
    );

    // Branch A
    let mut branch_a = Graph::new();
    branch_a.add(
        |inputs: &HashMap<String, GraphData>| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("x").and_then(|d: &GraphData| d.as_int()) {
                result.insert("output".to_string(), GraphData::int(val + 10));
            }
            result
        },
        Some("Branch A"),
        Some(vec![("data", "x")]),
        Some(vec![("output", "result")]),
    );

    // Branch B
    let mut branch_b = Graph::new();
    branch_b.add(
        |inputs: &HashMap<String, GraphData>| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("x").and_then(|d: &GraphData| d.as_int()) {
                result.insert("output".to_string(), GraphData::int(val + 20));
            }
            result
        },
        Some("Branch B"),
        Some(vec![("data", "x")]),
        Some(vec![("output", "result")]),
    );

    let branch_a_id = graph.branch(branch_a);
    let branch_b_id = graph.branch(branch_b);

    // Merge function combines both branches
    graph.merge(
        |inputs: &HashMap<String, GraphData>| {
            let mut result = HashMap::new();
            let a = inputs.get("from_a").and_then(|d: &GraphData| d.as_int()).unwrap_or(0);
            let b = inputs.get("from_b").and_then(|d: &GraphData| d.as_int()).unwrap_or(0);
            result.insert("merged".to_string(), GraphData::int(a + b));
            result
        },
        Some("Merge"),
        vec![
            (branch_a_id, "result", "from_a"),
            (branch_b_id, "result", "from_b"),
        ],
        Some(vec![("merged", "final")]),
    );

    let dag = graph.build();
    let context = dag.execute(false, None);

    // Branch A: 100 + 10 = 110
    // Branch B: 100 + 20 = 120
    // Merge: 110 + 120 = 230
    assert_eq!(context.get("final").and_then(|d: &GraphData| d.as_int()), Some(230));
}

#[test]
fn test_variants() {
    let mut graph = Graph::new();

    // Source
    graph.add(
        |_: &HashMap<String, GraphData>| {
            let mut result = HashMap::new();
            result.insert("value".to_string(), GraphData::int(10));
            result
        },
        Some("Source"),
        None,
        Some(vec![("value", "data")]),
    );

    // Variant sweep: multiply by different factors using closures
    let _factors = vec![2.0, 3.0, 5.0];
    let multipliers = vec![
        (|inputs: &HashMap<String, GraphData>| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("x").and_then(|d: &GraphData| d.as_float()) {
                result.insert("scaled".to_string(), GraphData::float(val * 1.5));
            }
            result
        }),
        (|inputs: &HashMap<String, GraphData>| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("x").and_then(|d: &GraphData| d.as_float()) {
                result.insert("scaled".to_string(), GraphData::float(val * 2.0));
            }
            result
        }),
        (|inputs: &HashMap<String, GraphData>| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("x").and_then(|d: &GraphData| d.as_float()) {
                result.insert("scaled".to_string(), GraphData::float(val * 3.0));
            }
            result
        }),
    ];

    graph.variants(
        multipliers,
        Some("Multiply"),
        Some(vec![("data", "x")]),
        Some(vec![("scaled", "result")]),
    );

    let dag = graph.build();
    let stats = dag.stats();

    // Should have 1 source + 3 variants = 4 nodes
    assert_eq!(stats.node_count, 4);
    assert_eq!(stats.variant_count, 3);

    // All 3 variants should be at the same execution level (parallel)
    assert!(stats.max_parallelism >= 3);
}

#[test]
fn test_dag_stats() {
    let mut graph = Graph::new();

    graph.add(
        data_source,
        Some("Source"),
        None,
        Some(vec![("raw_data", "data")]),
    );
    graph.add(
        processor,
        Some("Process"),
        Some(vec![("data", "input_data")]),
        Some(vec![("processed_value", "result")]),
    );
    graph.add(
        adder,
        Some("Add"),
        Some(vec![("result", "input")]),
        Some(vec![("sum", "final")]),
    );

    let dag = graph.build();
    let stats = dag.stats();

    assert_eq!(stats.node_count, 3);
    assert_eq!(stats.depth, 3); // 3 sequential levels
    assert_eq!(stats.max_parallelism, 1); // All sequential, no parallelism
}

#[test]
fn test_mermaid_visualization() {
    let mut graph = Graph::new();

    graph.add(
        data_source,
        Some("Source"),
        None,
        Some(vec![("raw_data", "data")]),
    );
    graph.add(
        processor,
        Some("Process"),
        Some(vec![("data", "input_data")]),
        Some(vec![("processed_value", "result")]),
    );

    let dag = graph.build();
    let mermaid = dag.to_mermaid();

    // Should contain graph declaration
    assert!(mermaid.contains("graph TD"));
    // Should contain node labels
    assert!(mermaid.contains("Source"));
    assert!(mermaid.contains("Process"));
    // Should contain edges
    assert!(mermaid.contains("-->"));
}

// ─── execute_detailed ─────────────────────────────────────────────────────────

#[test]
fn test_execute_detailed_node_outputs() {
    let mut graph = Graph::new();
    graph.add(data_source, Some("Source"), None, Some(vec![("raw_data", "data")]));
    graph.add(processor, Some("Process"),
        Some(vec![("data", "input_data")]),
        Some(vec![("processed_value", "result")]));

    let dag = graph.build();
    let result = dag.execute_detailed(false, None);

    // Global context should have both outputs
    assert_eq!(result.get("data").and_then(|d| d.as_int()), Some(100));
    assert_eq!(result.get("result").and_then(|d| d.as_int()), Some(200));

    // Per-node outputs: each node should have an entry
    let orders = dag.execution_order();
    assert_eq!(orders.len(), 2);
    let source_id = orders[0];
    let proc_id = orders[1];

    let source_out = result.get_node_outputs(source_id).expect("source node outputs missing");
    assert!(source_out.contains_key("data"));

    let proc_out = result.get_node_outputs(proc_id).expect("process node outputs missing");
    assert!(proc_out.contains_key("result"));
}

#[test]
fn test_execute_detailed_branch_outputs() {
    let mut graph = Graph::new();
    graph.add(data_source, Some("Source"), None, Some(vec![("raw_data", "data")]));

    let mut branch_a = Graph::new();
    branch_a.add(
        |inputs: &HashMap<String, GraphData>| {
            let mut out = HashMap::new();
            if let Some(v) = inputs.get("x").and_then(|d| d.as_int()) {
                out.insert("output".to_string(), GraphData::int(v + 1));
            }
            out
        },
        Some("A"), Some(vec![("data", "x")]), Some(vec![("output", "result_a")]),
    );
    let bid = graph.branch(branch_a);

    let dag = graph.build();
    let result = dag.execute_detailed(false, None);

    // Branch outputs should be keyed by branch id
    let bout = result.get_branch_outputs(bid).expect("branch outputs missing");
    assert!(bout.contains_key("result_a"));
    assert_eq!(bout.get("result_a").and_then(|d| d.as_int()), Some(101));
}

#[test]
fn test_execute_get_from_node_and_branch() {
    let mut graph = Graph::new();
    graph.add(data_source, Some("Source"), None, Some(vec![("raw_data", "data")]));
    let mut branch = Graph::new();
    branch.add(
        |inputs: &HashMap<String, GraphData>| {
            let mut out = HashMap::new();
            if let Some(v) = inputs.get("x").and_then(|d| d.as_int()) {
                out.insert("y".to_string(), GraphData::int(v * 10));
            }
            out
        },
        Some("B"), Some(vec![("data", "x")]), Some(vec![("y", "ten_x")]),
    );
    let bid = graph.branch(branch);
    let dag = graph.build();
    let result = dag.execute_detailed(false, None);

    let branch_node_id = dag.execution_order().last().copied().unwrap();
    assert_eq!(result.get_from_node(branch_node_id, "ten_x").and_then(|d| d.as_int()), Some(1000));
    assert_eq!(result.get_from_branch(bid, "ten_x").and_then(|d| d.as_int()), Some(1000));
    assert!(result.contains_key("data"));
}

// ─── Parallel execution ───────────────────────────────────────────────────────

#[test]
fn test_parallel_matches_sequential() {
    let build = || {
        let mut graph = Graph::new();
        graph.add(data_source, Some("Source"), None, Some(vec![("raw_data", "data")]));
        graph.add(processor, Some("Process"),
            Some(vec![("data", "input_data")]),
            Some(vec![("processed_value", "result")]));
        graph.add(adder, Some("Add"),
            Some(vec![("result", "input")]),
            Some(vec![("sum", "final")]));
        graph.build()
    };

    let seq = build().execute(false, None);
    let par = build().execute(true, None);
    assert_eq!(seq.get("final").and_then(|d| d.as_int()),
               par.get("final").and_then(|d| d.as_int()));
}

#[test]
fn test_parallel_branches() {
    let build = || {
        let mut graph = Graph::new();
        graph.add(data_source, Some("Source"), None, Some(vec![("raw_data", "data")]));
        let mut b1 = Graph::new();
        b1.add(|inputs: &HashMap<String, GraphData>| {
            let mut o = HashMap::new();
            if let Some(v) = inputs.get("x").and_then(|d| d.as_int()) {
                o.insert("out".to_string(), GraphData::int(v + 1));
            }
            o
        }, Some("B1"), Some(vec![("data", "x")]), Some(vec![("out", "r1")]));
        let mut b2 = Graph::new();
        b2.add(|inputs: &HashMap<String, GraphData>| {
            let mut o = HashMap::new();
            if let Some(v) = inputs.get("x").and_then(|d| d.as_int()) {
                o.insert("out".to_string(), GraphData::int(v + 2));
            }
            o
        }, Some("B2"), Some(vec![("data", "x")]), Some(vec![("out", "r2")]));
        graph.branch(b1);
        graph.branch(b2);
        graph.build()
    };
    let seq = build().execute(false, None);
    let par = build().execute(true, None);
    assert_eq!(seq.get("r1").and_then(|d| d.as_int()), par.get("r1").and_then(|d| d.as_int()));
    assert_eq!(seq.get("r2").and_then(|d| d.as_int()), par.get("r2").and_then(|d| d.as_int()));
}

// ─── DagStats::summary ────────────────────────────────────────────────────────

#[test]
fn test_dag_stats_summary_format() {
    let mut graph = Graph::new();
    graph.add(data_source, Some("S"), None, Some(vec![("raw_data", "d")]));
    graph.add(processor, Some("P"), Some(vec![("d", "input_data")]), Some(vec![("processed_value", "r")]));
    let stats = graph.build().stats();
    let s = stats.summary();
    assert!(s.contains("Nodes: 2"));
    assert!(s.contains("Depth: 2"));
    assert!(s.contains("Max Parallelism: 1"));
}

// ─── predict() / predict_at() ────────────────────────────────────────────────

fn build_linear_dag() -> Dag {
    // y = x * 2.0,  x ~ Normal(3, 1)
    let mut graph = Graph::new();
    graph.add(
        |inputs: &HashMap<String, GraphData>| {
            let mut out = HashMap::new();
            if let Some(v) = inputs.get("x").and_then(|d| d.as_float()) {
                out.insert("y".to_string(), GraphData::float(v * 2.0));
            }
            out
        },
        Some("Double"),
        Some(vec![("x", "x")]),
        Some(vec![("y", "y")]),
    );
    graph.build()
}

#[test]
fn test_predict_normal_linear() {
    let dag = build_linear_dag();
    let mut inputs = HashMap::new();
    inputs.insert("x".to_string(), Distribution::normal(3.0, 1.0));

    let stat = dag.predict_at(inputs, Some(5000), None);
    let mean_y = stat.get("y").expect("y missing").mean();
    assert!((mean_y - 6.0).abs() < 0.3, "mean_y = {mean_y}");

    // std(y) = 2 * std(x) = 2.0
    let std_y = stat.get("y").expect("y missing").std();
    assert!((std_y - 2.0).abs() < 0.3, "std_y = {std_y}");
}

#[test]
fn test_predict_contains_and_summary() {
    let dag = build_linear_dag();
    let mut inputs = HashMap::new();
    inputs.insert("x".to_string(), Distribution::deterministic(5.0));
    let stat = dag.predict_at(inputs, Some(100), None);
    assert!(!stat.contains("nonexistent"));
    let summary = stat.summary("y").expect("summary missing");
    assert!((summary.mean - 10.0).abs() < 1e-6);
    assert_eq!(summary.variance, 0.0);
}

#[test]
fn test_predict_at_early_stop() {
    // Three-node chain: A -> B -> C
    let mut graph = Graph::new();
    graph.add(
        |_: &HashMap<String, GraphData>| {
            let mut o = HashMap::new(); o.insert("a".to_string(), GraphData::float(1.0)); o
        }, Some("A"), None, Some(vec![("a", "a")]));
    graph.add(
        |inputs: &HashMap<String, GraphData>| {
            let mut o = HashMap::new();
            if let Some(v) = inputs.get("a").and_then(|d| d.as_float()) {
                o.insert("b".to_string(), GraphData::float(v + 10.0));
            }
            o
        }, Some("B"), Some(vec![("a", "a")]), Some(vec![("b", "b")]));
    graph.add(
        |inputs: &HashMap<String, GraphData>| {
            let mut o = HashMap::new();
            if let Some(v) = inputs.get("b").and_then(|d| d.as_float()) {
                o.insert("c".to_string(), GraphData::float(v + 100.0));
            }
            o
        }, Some("C"), Some(vec![("b", "b")]), Some(vec![("c", "c")]));
    let dag = graph.build();

    let stat = dag.predict_at(HashMap::new(), Some(100), Some(&PredictTarget::NodeLabel("B".into())));

    // B should be computed ...
    assert!(stat.contains("b"), "b should be in result");
    // ... but C should not (early stop)
    assert!(!stat.contains("c"), "c should not be computed when stopping at B");
}

#[test]
fn test_predict_branch_id_target() {
    let mut graph = Graph::new();
    graph.add(data_source, Some("Source"), None, Some(vec![("raw_data", "data")]));
    let mut b = Graph::new();
    b.add(|inputs: &HashMap<String, GraphData>| {
        let mut o = HashMap::new();
        if let Some(v) = inputs.get("x").and_then(|d| d.as_float()) {
            o.insert("br_out".to_string(), GraphData::float(v));
        }
        o
    }, Some("Branch"), Some(vec![("data", "x")]), Some(vec![("br_out", "branch_result")]));
    let bid = graph.branch(b);
    let dag = graph.build();

    let mut inputs = HashMap::new();
    inputs.insert("data".to_string(), Distribution::deterministic(5.0));
    let stat = dag.predict_at(inputs, Some(100), Some(&PredictTarget::BranchId(bid)));
    assert!(stat.for_branch(bid).is_some());
}

// ─── predict() ──────────────────────────────────────────────────────────────

#[test]
fn test_predict_aligned() {
    // y = x + noise,  x ~ N(0,1), noise ~ N(0,0.1)
    let mut graph = Graph::new();
    graph.add(
        |inputs: &HashMap<String, GraphData>| {
            let mut o = HashMap::new();
            if let Some(v) = inputs.get("x").and_then(|d| d.as_float()) {
                o.insert("y".to_string(), GraphData::float(v * 2.0));
            }
            o
        },
        Some("Double"),
        Some(vec![("x", "x")]),
        Some(vec![("y", "y")]),
    );
    let dag = graph.build();

    let mut inputs = HashMap::new();
    inputs.insert("x".to_string(), Distribution::normal(0.0, 1.0));
    let stat = dag.predict(inputs, 2000);

    // particles should be populated
    let particles = stat.particles.as_ref().expect("particles missing");
    assert_eq!(particles.len(), 2000);

    // Each particle should have both x and y
    for p in particles {
        assert!(p.contains_key("x"), "particle missing x");
        assert!(p.contains_key("y"), "particle missing y");
        // y = 2x must hold per-particle (perfect linear relationship)
        let y = p["y"];
        let x = p["x"];
        assert!((y - 2.0 * x).abs() < 1e-8, "y={y} != 2*x={}", 2.0 * x);
    }

    // Marginal distributions should still be correct
    let mean_y = stat.get("y").unwrap().mean();
    assert!((mean_y - 0.0).abs() < 0.15, "mean_y = {mean_y}");
}

#[test]
fn test_stat_result_iter_and_accessors() {
    let mut graph = Graph::new();
    graph.add(|_: &HashMap<String, GraphData>| {
        let mut o = HashMap::new();
        o.insert("v".to_string(), GraphData::float(3.0));
        o
    }, Some("N"), None, Some(vec![("v", "v")]));
    let dag = graph.build();

    let mut inputs = HashMap::new();
    inputs.insert("v".to_string(), Distribution::deterministic(3.0));
    let stat = dag.predict_at(inputs, Some(10), None);

    // iter() should yield at least the "v" key
    let keys: Vec<&String> = stat.iter().map(|(k, _)| k).collect();
    assert!(keys.contains(&&"v".to_string()));

    let node_id = dag.execution_order()[0];
    assert!(stat.get_node_dists(node_id).is_some());
    assert!(stat.get_from_node(node_id, "v").is_some());
}

