//! Demonstration of the tuple-based API for variable mapping
//!
//! This example shows how to use the new tuple-based API where:
//! - Inputs: (broadcast_var, impl_var) - context variable mapped to function parameter name
//! - Outputs: (impl_var, broadcast_var) - function return value mapped to context variable
//! - Merge inputs: (branch_id, broadcast_var, impl_var) - branch-specific variable resolution

use graph_sp::Graph;
use std::collections::HashMap;

fn main() {
    println!("=== Tuple-Based API Demo ===\n");

    let mut graph = Graph::new();

    // Source node - no inputs, produces "dataset" in context
    fn data_source(_inputs: &HashMap<String, String>, _variant: &HashMap<String, String>) -> HashMap<String, String> {
        let mut outputs = HashMap::new();
        outputs.insert("raw".to_string(), "Sample Data".to_string());
        outputs
    }

    graph.add(
        data_source,
        Some("Source"),
        None,  // No inputs
        Some(vec![("raw", "dataset")])  // Function returns "raw", stored as "dataset" in context
    );

    println!("✓ Added source node");
    println!("  Output mapping: function's 'raw' → context's 'dataset'\n");

    // Process node - consumes "dataset" from context as "input_data", produces "processed_data" to context as "result"
    fn processor(inputs: &HashMap<String, String>, _variant: &HashMap<String, String>) -> HashMap<String, String> {
        let default = String::new();
        let data = inputs.get("input_data").unwrap_or(&default);
        let mut outputs = HashMap::new();
        outputs.insert("processed_data".to_string(), format!("Processed: {}", data));
        outputs
    }

    graph.add(
        processor,
        Some("Process"),
        Some(vec![("dataset", "input_data")]),      // Context's "dataset" → function's "input_data"
        Some(vec![("processed_data", "result")])    // Function's "processed_data" → context's "result"
    );

    println!("✓ Added processor node");
    println!("  Input mapping: context's 'dataset' → function's 'input_data'");
    println!("  Output mapping: function's 'processed_data' → context's 'result'\n");

    // Create branches that both use the same variable names internally
    let mut branch_a = Graph::new();
    fn transform_a(inputs: &HashMap<String, String>, _variant: &HashMap<String, String>) -> HashMap<String, String> {
        let default = String::new();
        let data = inputs.get("x").unwrap_or(&default);
        let mut outputs = HashMap::new();
        outputs.insert("y".to_string(), format!("{} [Path A]", data));
        outputs
    }
    branch_a.add(
        transform_a,
        Some("Transform A"),
        Some(vec![("result", "x")]),  // Context's "result" → function's "x"
        Some(vec![("y", "output")])    // Function's "y" → context's "output"
    );

    let mut branch_b = Graph::new();
    fn transform_b(inputs: &HashMap<String, String>, _variant: &HashMap<String, String>) -> HashMap<String, String> {
        let default = String::new();
        let data = inputs.get("x").unwrap_or(&default);
        let mut outputs = HashMap::new();
        outputs.insert("y".to_string(), format!("{} [Path B]", data));
        outputs
    }
    branch_b.add(
        transform_b,
        Some("Transform B"),
        Some(vec![("result", "x")]),  // Same variable names as branch_a
        Some(vec![("y", "output")])    // Same variable names as branch_a
    );

    println!("✓ Created two branches");
    println!("  Both branches use same internal variable names (x, y)");
    println!("  Both map 'result' → 'x' and 'y' → 'output'\n");

    // Branch from the processor
    let branch_a_id = graph.branch(branch_a);
    let branch_b_id = graph.branch(branch_b);

    println!("✓ Added branches to graph");
    println!("  Branch A ID: {}", branch_a_id);
    println!("  Branch B ID: {}\n", branch_b_id);

    // Merge branches with branch-specific variable resolution
    fn combine(inputs: &HashMap<String, String>, _variant: &HashMap<String, String>) -> HashMap<String, String> {
        let default = String::new();
        let a = inputs.get("from_a").unwrap_or(&default);
        let b = inputs.get("from_b").unwrap_or(&default);
        let mut outputs = HashMap::new();
        outputs.insert("merged".to_string(), format!("Combined: {} + {}", a, b));
        outputs
    }

    graph.merge(
        combine,
        Some("Combine"),
        vec![
            (branch_a_id, "output", "from_a"),  // Branch A's "output" → merge function's "from_a"
            (branch_b_id, "output", "from_b")   // Branch B's "output" → merge function's "from_b"
        ],
        Some(vec![("merged", "final_result")])  // Merge function's "merged" → context's "final_result"
    );

    println!("✓ Added merge node");
    println!("  Branch-specific input mapping:");
    println!("    Branch {} 'output' → merge function's 'from_a'", branch_a_id);
    println!("    Branch {} 'output' → merge function's 'from_b'", branch_b_id);
    println!("  Output mapping: merge function's 'merged' → context's 'final_result'\n");

    // Variant example with factory pattern
    fn make_multiplier(factor: f64) -> impl Fn(&HashMap<String, String>, &HashMap<String, String>) -> HashMap<String, String> {
        move |inputs, _variant| {
            let default = "1.0".to_string();
            let data = inputs.get("value").unwrap_or(&default);
            if let Ok(val) = data.parse::<f64>() {
                let mut outputs = HashMap::new();
                outputs.insert("scaled".to_string(), (val * factor).to_string());
                outputs
            } else {
                HashMap::new()
            }
        }
    }

    graph.variant(
        make_multiplier,
        vec![2.0, 3.0, 5.0],
        Some("Multiply"),
        Some(vec![("final_result", "value")]),  // Context's "final_result" → function's "value"
        Some(vec![("scaled", "multiplied")])    // Function's "scaled" → context's "multiplied"
    );

    println!("✓ Added variant nodes with parameter sweep");
    println!("  Three variants: factor = 2.0, 3.0, 5.0");
    println!("  Input mapping: context's 'final_result' → function's 'value'");
    println!("  Output mapping: function's 'scaled' → context's 'multiplied'\n");

    println!("=== Summary ===");
    println!("The tuple-based API provides clear separation between:");
    println!("1. Context variable names (broadcast vars) - shared across the graph");
    println!("2. Function parameter names (impl vars) - internal to each function");
    println!("\nThis allows:");
    println!("- Branches to use consistent internal naming (x, y)");
    println!("- Merge to distinguish branch outputs using branch IDs");
    println!("- Clear data flow visualization and debugging");
}
