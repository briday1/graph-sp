use dagex::{Graph, GraphData};
use std::collections::HashMap;

#[test]
fn test_minimal_pipeline_mermaid() {
    let mut g = Graph::new();

    // Source node
    g.add(
        (|_: &HashMap<String, GraphData>| {
            let mut o = HashMap::new();
            o.insert("n".to_string(), GraphData::int(10));
            o
        }),
        Some("Source"),
        None,
        Some(vec![("n", "x")]),
    );

    // Double node
    g.add(
        (|inputs: &HashMap<String, GraphData>| {
            let mut o = HashMap::new();
            if let Some(v) = inputs.get("x").and_then(|d| d.as_int()) {
                o.insert("y".to_string(), GraphData::int(v * 2));
            }
            o
        }),
        Some("Double"),
        Some(vec![("x", "x")]),
        Some(vec![("y", "out")]),
    );

    let dag = g.build();
    let mermaid = dag.to_mermaid();

    // Basic sanity checks: labels and the broadcast->impl mapping appear
    assert!(mermaid.contains("Source"), "mermaid missing 'Source': {}", mermaid);
    assert!(mermaid.contains("Double"), "mermaid missing 'Double': {}", mermaid);
    // check that the edge label contains a reasonable mapping (accept either 'n → x' or 'x → x')
    let mapping_ok = mermaid.contains("n → x")
        || mermaid.contains("n \u{2192} x")
        || mermaid.contains("x → x")
        || mermaid.contains("x \u{2192} x");
    assert!(mapping_ok, "mermaid missing expected edge label (e.g. 'n → x' or 'x → x'): {}", mermaid);
}

#[test]
fn test_processor_formatter_mermaid() {
    let mut g = Graph::new();

    // Simple pipeline: Source -> Processor -> Formatter
    g.add(
        (|_: &HashMap<String, GraphData>| {
            let mut o = HashMap::new();
            o.insert("data".to_string(), GraphData::string("100".to_string()));
            o
        }),
        Some("Source"),
        None,
        Some(vec![("data", "data")]),
    );

    g.add(
        (|inputs: &HashMap<String, GraphData>| {
            let mut o = HashMap::new();
            if let Some(v) = inputs.get("input").and_then(|d| d.as_string()) {
                o.insert("result".to_string(), GraphData::string((v.parse::<i64>().unwrap_or(0) * 2).to_string()));
            }
            o
        }),
        Some("Processor"),
        Some(vec![("data", "input")]),
        Some(vec![("result", "final")]),
    );

    g.add(
        (|inputs: &HashMap<String, GraphData>| {
            let mut o = HashMap::new();
            if let Some(v) = inputs.get("value").and_then(|d| d.as_string()) {
                o.insert("formatted".to_string(), GraphData::string(format!("Result: {}", v)));
            }
            o
        }),
        Some("Formatter"),
        Some(vec![("final", "value")]),
        Some(vec![("formatted", "display")]),
    );

    let dag = g.build();
    let mermaid = dag.to_mermaid();

    assert!(mermaid.contains("Processor"), "mermaid missing 'Processor': {}", mermaid);
    assert!(mermaid.contains("Formatter"), "mermaid missing 'Formatter': {}", mermaid);
    assert!(mermaid.contains("data → input") || mermaid.contains("data \u{2192} input"), "mermaid missing 'data → input': {}", mermaid);
}
