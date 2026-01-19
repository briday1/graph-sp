//! Integration tests for graph-sp

use graph_sp::{Graph, GraphData, Linspace};
use std::collections::HashMap;

// Helper functions for tests

fn data_source(
    _: &HashMap<String, GraphData>,
    _: &HashMap<String, GraphData>,
) -> HashMap<String, GraphData> {
    let mut result = HashMap::new();
    result.insert("raw_data".to_string(), GraphData::int(100));
    result
}

fn processor(
    inputs: &HashMap<String, GraphData>,
    _: &HashMap<String, GraphData>,
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
    _: &HashMap<String, GraphData>,
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

    assert_eq!(context.get("data").and_then(|d| d.as_int()), Some(100));
    assert_eq!(context.get("result").and_then(|d| d.as_int()), Some(200));
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
        |inputs: &HashMap<String, GraphData>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("x").and_then(|d| d.as_int()) {
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
        |inputs: &HashMap<String, GraphData>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("x").and_then(|d| d.as_int()) {
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

    assert_eq!(context.get("data").and_then(|d| d.as_int()), Some(100));
    assert_eq!(context.get("result_a").and_then(|d| d.as_int()), Some(200));
    assert_eq!(context.get("result_b").and_then(|d| d.as_int()), Some(300));
}

#[test]
#[ignore] // TODO: Merge functionality needs special handling for branch-specific inputs
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
        |inputs: &HashMap<String, GraphData>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("x").and_then(|d| d.as_int()) {
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
        |inputs: &HashMap<String, GraphData>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("x").and_then(|d| d.as_int()) {
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
        |inputs: &HashMap<String, GraphData>, _| {
            let mut result = HashMap::new();
            let a = inputs.get("from_a").and_then(|d| d.as_int()).unwrap_or(0);
            let b = inputs.get("from_b").and_then(|d| d.as_int()).unwrap_or(0);
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
    assert_eq!(context.get("final").and_then(|d| d.as_int()), Some(230));
}

#[test]
fn test_variants() {
    let mut graph = Graph::new();

    // Source
    graph.add(
        |_: &HashMap<String, GraphData>, _| {
            let mut result = HashMap::new();
            result.insert("value".to_string(), GraphData::int(10));
            result
        },
        Some("Source"),
        None,
        Some(vec![("value", "data")]),
    );

    // Variant factory: multiply by different factors
    fn make_multiplier(
        factor: f64,
    ) -> impl Fn(&HashMap<String, GraphData>, &HashMap<String, GraphData>) -> HashMap<String, GraphData>
    {
        move |inputs: &HashMap<String, GraphData>, _| {
            let mut result = HashMap::new();
            if let Some(val) = inputs.get("x").and_then(|d| d.as_float()) {
                result.insert("scaled".to_string(), GraphData::float(val * factor));
            }
            result
        }
    }

    graph.variant(
        make_multiplier,
        vec![2.0, 3.0, 5.0],
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

#[test]
fn test_linspace_helper() {
    use graph_sp::IntoVariantValues;
    let values = Linspace::new(0.0, 1.0, 5).into_variant_values();
    assert_eq!(values.len(), 5);
    assert_eq!(values[0].parse::<f64>().unwrap(), 0.0);
    assert_eq!(values[4].parse::<f64>().unwrap(), 1.0);
}
