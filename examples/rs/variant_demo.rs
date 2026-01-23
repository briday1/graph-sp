use dagex::{Graph, GraphData, NodeFunction};
use std::collections::HashMap;
use std::sync::Arc;

fn main() {
    println!("=== Variant Closures Demo ===\n");
    println!("Demonstrating the simpler variant API using closures\n");

    // Demo 1: Simple parameter sweep
    println!("─────────────────────────────────────────────────────────");
    println!("Demo 1: Simple Parameter Sweep with Closures");
    println!("─────────────────────────────────────────────────────────\n");

    let mut graph = Graph::new();

    // Source node
    graph.add(
        Arc::new(|_| {
            let mut result = HashMap::new();
            result.insert("value".to_string(), GraphData::int(10));
            result
        }),
        Some("Source"),
        None,
        Some(vec![("value", "data")]),
    );

    // Variant sweep: multiply by different factors
    let factors = vec![2, 3, 5, 10];
    graph.variants(
        factors
            .iter()
            .map(|&factor| {
                Arc::new(move |inputs: &HashMap<String, GraphData>| {
                    let mut outputs = HashMap::new();
                    if let Some(val) = inputs.get("x").and_then(|d| d.as_int()) {
                        outputs.insert("result".to_string(), GraphData::int(val * factor));
                    }
                    outputs
                }) as NodeFunction
            })
            .collect(),
        Some("Scale"),
        Some(vec![("data", "x")]),
        Some(vec![("result", "scaled")]),
    );

    let dag = graph.build();
    let context = dag.execute(false, None);

    println!("Results:");
    println!("  Input: {}", context.get("data").unwrap().to_string_repr());
    println!("  Factors: {:?}", factors);
    println!(
        "  Output (last variant): {}",
        context.get("scaled").unwrap().to_string_repr()
    );
    println!("\nMermaid:\n{}", dag.to_mermaid());

    // Demo 2: Power functions
    println!("\n─────────────────────────────────────────────────────────");
    println!("Demo 2: Power Function Variants");
    println!("─────────────────────────────────────────────────────────\n");

    let mut graph2 = Graph::new();

    graph2.add(
        Arc::new(|_| {
            let mut result = HashMap::new();
            result.insert("x".to_string(), GraphData::int(2));
            result
        }),
        Some("Source"),
        None,
        Some(vec![("x", "number")]),
    );

    let exponents = vec![2, 3, 4, 5];
    graph2.variants(
        exponents
            .iter()
            .map(|&exp| {
                Arc::new(move |inputs: &HashMap<String, GraphData>| {
                    let mut outputs = HashMap::new();
                    if let Some(val) = inputs.get("n").and_then(|d| d.as_int()) {
                        outputs.insert("powered".to_string(), GraphData::int(val.pow(exp as u32)));
                    }
                    outputs
                }) as NodeFunction
            })
            .collect(),
        Some("Power"),
        Some(vec![("number", "n")]),
        Some(vec![("powered", "result")]),
    );

    let dag2 = graph2.build();
    let context2 = dag2.execute(false, None);

    println!("Base: {}", context2.get("number").unwrap().to_string_repr());
    println!("Exponents: {:?}", exponents);
    println!(
        "Result (last variant, 2^5): {}",
        context2.get("result").unwrap().to_string_repr()
    );

    // Demo 3: Using ranges
    println!("\n─────────────────────────────────────────────────────────");
    println!("Demo 3: Using Range Iterator");
    println!("─────────────────────────────────────────────────────────\n");

    let mut graph3 = Graph::new();

    graph3.add(
        Arc::new(|_| {
            let mut result = HashMap::new();
            result.insert("base".to_string(), GraphData::float(100.0));
            result
        }),
        Some("Source"),
        None,
        Some(vec![("base", "value")]),
    );

    // Create closures for scaling factors from 0.5 to 1.5 in steps of 0.25
    let steps = 5;
    graph3.variants(
        (0..steps)
            .map(|i| {
                let factor = 0.5 + (i as f64) * 0.25;
                Arc::new(move |inputs: &HashMap<String, GraphData>| {
                    let mut outputs = HashMap::new();
                    if let Some(val) = inputs.get("v").and_then(|d| d.as_float()) {
                        outputs.insert("scaled".to_string(), GraphData::float(val * factor));
                    }
                    outputs
                }) as NodeFunction
            })
            .collect(),
        Some("LinearScale"),
        Some(vec![("value", "v")]),
        Some(vec![("scaled", "result")]),
    );

    let dag3 = graph3.build();
    let context3 = dag3.execute(false, None);

    println!("Used {} scaling factors from 0.5 to 1.5", steps);
    println!("Base value: {}", context3.get("value").unwrap().to_string_repr());
    println!(
        "Final result: {}",
        context3.get("result").unwrap().to_string_repr()
    );

    println!("\n=== Demo Complete! ===");
}
