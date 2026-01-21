use dagex::{Graph, GraphData};
use std::collections::HashMap;

/// The actual computation function - factor is just another parameter
fn scale_by_factor(
    inputs: &HashMap<String, GraphData>,
    factor: f64,
) -> HashMap<String, GraphData> {
    let mut outputs = HashMap::new();
    if let Some(val) = inputs.get("x").and_then(|d| d.as_int()) {
        outputs.insert("result".to_string(), GraphData::int(val * factor as i64));
    }
    outputs
}

/// Another example - filtering function
fn filter_by_threshold(
    inputs: &HashMap<String, GraphData>,
    threshold: f64,
) -> HashMap<String, GraphData> {
    let mut outputs = HashMap::new();
    if let Some(val) = inputs.get("value").and_then(|d| d.as_float()) {
        if val > threshold {
            outputs.insert("passed".to_string(), GraphData::float(val));
            outputs.insert("status".to_string(), GraphData::string("pass".to_string()));
        } else {
            outputs.insert("passed".to_string(), GraphData::float(0.0));
            outputs.insert("status".to_string(), GraphData::string("fail".to_string()));
        }
    }
    outputs
}

/// Power function example
fn power_function(
    inputs: &HashMap<String, GraphData>,
    exponent: u32,
) -> HashMap<String, GraphData> {
    let mut outputs = HashMap::new();
    if let Some(val) = inputs.get("base").and_then(|d| d.as_int()) {
        outputs.insert("powered".to_string(), GraphData::int(val.pow(exponent)));
    }
    outputs
}

fn main() {
    println!("=== Variant Parameter Sweep Demo ===\n");
    println!("Demonstrating clean variant syntax by defining functions separately\n");

    // Demo 1: Simple scaling sweep
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 1: Scale by Factor Sweep");
    println!("─────────────────────────────────────────────────────────\n");

    let mut graph = Graph::new();

    graph.add(
        |_| {
            let mut result = HashMap::new();
            result.insert("value".to_string(), GraphData::int(10));
            result
        },
        Some("Source"),
        None,
        Some(vec![("value", "data")]),
    );

    // Clean syntax - just wrap the function call with the swept parameter
    let factors = vec![2.0, 3.0, 5.0, 10.0];
    graph.variants(
        factors
            .iter()
            .map(|&f| {
                move |inputs: &HashMap<String, GraphData>| {
                    scale_by_factor(inputs, f)
                }
            })
            .collect(),
        Some("Scale"),
        Some(vec![("data", "x")]),
        Some(vec![("result", "scaled")]),
    );

    let dag = graph.build();
    let context = dag.execute(false, None);

    println!("Input: {}", context.get("data").unwrap().to_string_repr());
    println!("Factors swept: {:?}", factors);
    println!(
        "Output (last variant): {}",
        context.get("scaled").unwrap().to_string_repr()
    );

    // Demo 2: Threshold filtering
    println!("\n─────────────────────────────────────────────────────────");
    println!("Demo 2: Filter by Threshold Sweep");
    println!("─────────────────────────────────────────────────────────\n");

    let mut graph2 = Graph::new();

    graph2.add(
        |_| {
            let mut result = HashMap::new();
            result.insert("x".to_string(), GraphData::float(7.5));
            result
        },
        Some("Source"),
        None,
        Some(vec![("x", "input")]),
    );

    let thresholds = vec![5.0, 7.0, 8.0, 10.0];
    graph2.variants(
        thresholds
            .iter()
            .map(|&t| {
                move |inputs: &HashMap<String, GraphData>| {
                    filter_by_threshold(inputs, t)
                }
            })
            .collect(),
        Some("Filter"),
        Some(vec![("input", "value")]),
        Some(vec![("passed", "result"), ("status", "state")]),
    );

    let dag2 = graph2.build();
    let context2 = dag2.execute(false, None);

    println!("Input value: {}", context2.get("input").unwrap().to_string_repr());
    println!("Thresholds tested: {:?}", thresholds);
    println!(
        "Result (last variant): {}",
        context2.get("result").unwrap().to_string_repr()
    );
    println!(
        "Status (last variant): {}",
        context2.get("state").unwrap().to_string_repr()
    );

    // Demo 3: Power sweep
    println!("\n─────────────────────────────────────────────────────────");
    println!("Demo 3: Power Function Sweep");
    println!("─────────────────────────────────────────────────────────\n");

    let mut graph3 = Graph::new();

    graph3.add(
        |_| {
            let mut result = HashMap::new();
            result.insert("n".to_string(), GraphData::int(2));
            result
        },
        Some("Source"),
        None,
        Some(vec![("n", "number")]),
    );

    let exponents = vec![2, 3, 4, 5];
    graph3.variants(
        exponents
            .iter()
            .map(|&e| {
                move |inputs: &HashMap<String, GraphData>| {
                    power_function(inputs, e)
                }
            })
            .collect(),
        Some("Power"),
        Some(vec![("number", "base")]),
        Some(vec![("powered", "result")]),
    );

    let dag3 = graph3.build();
    let context3 = dag3.execute(false, None);

    println!("Base: {}", context3.get("number").unwrap().to_string_repr());
    println!("Exponents: {:?}", exponents);
    println!(
        "Result (2^5): {}",
        context3.get("result").unwrap().to_string_repr()
    );

    println!("\n🎯 Key Insight:");
    println!("  Define your computation function separately with the swept parameter");
    println!("  as the last argument, then graph.variants() becomes a simple one-liner:");
    println!("  ");
    println!("  graph.variants(");
    println!("      params.iter().map(|&p| move |inputs, params| func(inputs, params, p)).collect(),");
    println!("      ...");
    println!("  );");

    println!("\n=== Demo Complete! ===");
}
