// Comprehensive demonstration of the variant pattern (sigexec-style)
// Shows the full actual syntax with working examples

use graph_sp::{Graph, GraphData};
use std::collections::HashMap;

// =============================================================================
// Example 1: Basic Variant with Factory Function
// =============================================================================

/// Factory function that creates a scaler for a specific factor
/// This is the key pattern: factory takes a parameter, returns a closure
fn make_scaler(
    factor: f64,
) -> impl Fn(&HashMap<String, GraphData>, &HashMap<String, GraphData>) -> HashMap<String, GraphData>
{
    move |inputs, _variant_params| {
        let value = inputs
            .get("input_data")
            .and_then(|d| d.as_float())
            .unwrap_or(0.0);
        let scaled = value * factor;

        let mut outputs = HashMap::new();
        outputs.insert("scaled_value".to_string(), GraphData::float(scaled));
        outputs.insert("factor_used".to_string(), GraphData::float(factor));
        outputs
    }
}

// =============================================================================
// Example 2: Filter Factory with Multiple Parameters
// =============================================================================

#[derive(Clone)]
struct FilterConfig {
    cutoff: f64,
    mode: String,
}

impl std::fmt::Display for FilterConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_cutoff{}", self.mode, self.cutoff)
    }
}

fn make_filter(
    config: FilterConfig,
) -> impl Fn(&HashMap<String, GraphData>, &HashMap<String, GraphData>) -> HashMap<String, GraphData>
{
    move |inputs, _variant_params| {
        let value = inputs.get("data").and_then(|d| d.as_float()).unwrap_or(0.0);

        // Apply filtering based on config
        let filtered = match config.mode.as_str() {
            "lowpass" => value * config.cutoff,
            "highpass" => value * (1.0 - config.cutoff),
            _ => value,
        };

        let mut outputs = HashMap::new();
        outputs.insert("filtered".to_string(), GraphData::float(filtered));
        outputs.insert(
            "filter_mode".to_string(),
            GraphData::string(config.mode.clone()),
        );
        outputs
    }
}

// =============================================================================
// Example 3: Offset Factory (Simple Parameter Sweep)
// =============================================================================

fn make_offsetter(
    offset: i64,
) -> impl Fn(&HashMap<String, GraphData>, &HashMap<String, GraphData>) -> HashMap<String, GraphData>
{
    move |inputs, _variant_params| {
        let value = inputs.get("number").and_then(|d| d.as_int()).unwrap_or(0);
        let result = value + offset;

        let mut outputs = HashMap::new();
        outputs.insert("offset_result".to_string(), GraphData::int(result));
        outputs
    }
}

// =============================================================================
// Example 4: String Processor Factory
// =============================================================================

fn make_processor(
    prefix: &'static str,
) -> impl Fn(&HashMap<String, GraphData>, &HashMap<String, GraphData>) -> HashMap<String, GraphData>
{
    move |inputs, _variant_params| {
        let text = inputs.get("text").and_then(|d| d.as_string()).unwrap_or("");
        let processed = format!("[{}] {}", prefix, text);

        let mut outputs = HashMap::new();
        outputs.insert("processed_text".to_string(), GraphData::string(processed));
        outputs
    }
}

// =============================================================================
// Helper Functions for Demonstrations
// =============================================================================

fn data_source(
    _inputs: &HashMap<String, GraphData>,
    _variant_params: &HashMap<String, GraphData>,
) -> HashMap<String, GraphData> {
    let mut outputs = HashMap::new();
    outputs.insert("value".to_string(), GraphData::float(10.0));
    outputs
}

fn number_source(
    _inputs: &HashMap<String, GraphData>,
    _variant_params: &HashMap<String, GraphData>,
) -> HashMap<String, GraphData> {
    let mut outputs = HashMap::new();
    outputs.insert("num".to_string(), GraphData::int(42));
    outputs
}

fn text_source(
    _inputs: &HashMap<String, GraphData>,
    _variant_params: &HashMap<String, GraphData>,
) -> HashMap<String, GraphData> {
    let mut outputs = HashMap::new();
    outputs.insert(
        "message".to_string(),
        GraphData::string("Hello World".to_string()),
    );
    outputs
}

fn stats_node(
    inputs: &HashMap<String, GraphData>,
    _variant_params: &HashMap<String, GraphData>,
) -> HashMap<String, GraphData> {
    let result_str = inputs
        .get("result")
        .map(|v| v.to_string_repr())
        .unwrap_or_else(|| "N/A".to_string());
    let mut outputs = HashMap::new();
    outputs.insert(
        "summary".to_string(),
        GraphData::string(format!("Result: {}", result_str)),
    );
    outputs
}

// =============================================================================
// MAIN DEMONSTRATION
// =============================================================================

fn main() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  Variant Pattern Demo (sigexec-style)");
    println!("  Full Actual Syntax Examples");
    println!("═══════════════════════════════════════════════════════════\n");

    // =========================================================================
    // Demo 1: Single Variant - Basic Factory Pattern
    // =========================================================================
    println!("Demo 1: Single Variant with Factory Function");
    println!("─────────────────────────────────────────────────────────\n");

    println!("📝 Code:");
    println!("```rust");
    println!("fn make_scaler(factor: f64) -> impl Fn(...) -> ... {{");
    println!("    move |inputs, _| {{");
    println!("        let value = inputs.get(\"input_data\").unwrap().parse::<f64>().unwrap();");
    println!("        let scaled = value * factor;");
    println!("        outputs.insert(\"scaled_value\", scaled.to_string());");
    println!("    }}");
    println!("}}");
    println!();
    println!("let mut graph = Graph::new();");
    println!("graph.add(data_source, Some(\"Source\"), None, Some(vec![(\"value\", \"data\")]));");
    println!("graph.variant(");
    println!("    make_scaler,              // Factory function");
    println!("    vec![2.0, 3.0, 5.0],      // Parameter values to sweep");
    println!("    Some(\"Scale\"),            // Label");
    println!("    Some(vec![(\"data\", \"input_data\")]),  // Input mapping");
    println!("    Some(vec![(\"scaled_value\", \"result\")])  // Output mapping");
    println!(");");
    println!("```\n");

    let mut graph1 = Graph::new();
    graph1.add(
        data_source,
        Some("Source"),
        None,
        Some(vec![("value", "data")]),
    );
    graph1.variant(
        make_scaler,
        vec![2.0, 3.0, 5.0],
        Some("Scale"),
        Some(vec![("data", "input_data")]),
        Some(vec![("scaled_value", "result")]),
    );

    let dag1 = graph1.build();
    println!("🎯 What happens:");
    println!("  • Factory creates 3 nodes: Scale_2.0, Scale_3.0, Scale_5.0");
    println!("  • Each node multiplies input by its factor");
    println!("  • All variants can execute in parallel");
    println!();

    let stats1 = dag1.stats();
    println!("📈 DAG Statistics:");
    println!("  - Total nodes: {}", stats1.node_count);
    println!("  - Depth: {} levels", stats1.depth);
    println!(
        "  - Max parallelism: {} nodes can run simultaneously",
        stats1.max_parallelism
    );
    println!();

    println!("🔍 Mermaid Visualization:");
    println!("{}", dag1.to_mermaid());
    println!();

    // =========================================================================
    // Demo 2: Multiple Variants - Cartesian Product
    // =========================================================================
    println!("\nDemo 2: Multiple Variants (Cartesian Product)");
    println!("─────────────────────────────────────────────────────────\n");

    println!("📝 Code:");
    println!("```rust");
    println!(
        "graph.add(data_source, Some(\"Generate\"), None, Some(vec![(\"value\", \"data\")]));"
    );
    println!("graph.variant(make_scaler, vec![2.0, 3.0], Some(\"Scale\"), ...);");
    println!("graph.variant(make_offsetter, vec![10, 20], Some(\"Offset\"), ...);");
    println!("graph.add(stats_node, Some(\"Stats\"), Some(vec![(\"result\", \"result\")]), None);");
    println!("```\n");

    let mut graph2 = Graph::new();
    graph2.add(
        data_source,
        Some("Generate"),
        None,
        Some(vec![("value", "data")]),
    );
    graph2.variant(
        make_scaler,
        vec![2.0, 3.0],
        Some("Scale"),
        Some(vec![("data", "input_data")]),
        Some(vec![("scaled_value", "result")]),
    );
    graph2.variant(
        make_offsetter,
        vec![10, 20],
        Some("Offset"),
        Some(vec![("result", "number")]),
        Some(vec![("offset_result", "result")]),
    );
    graph2.add(
        stats_node,
        Some("Stats"),
        Some(vec![("result", "result")]),
        Some(vec![("summary", "final")]),
    );

    let dag2 = graph2.build();
    println!("🎯 What happens:");
    println!("  • Scale creates 2 variants: x2.0, x3.0");
    println!("  • Offset creates 2 variants: +10, +20");
    println!("  • Total combinations: 2 × 2 = 4 execution paths");
    println!("  • Each path: Generate → Scale[variant] → Offset[variant] → Stats");
    println!();

    let stats2 = dag2.stats();
    println!("📈 DAG Statistics:");
    println!("  - Total nodes: {}", stats2.node_count);
    println!("  - Depth: {} levels", stats2.depth);
    println!("  - Execution paths: 4 (2 scales × 2 offsets)");
    println!();

    println!("🔍 Mermaid Visualization:");
    println!("{}", dag2.to_mermaid());
    println!();

    // =========================================================================
    // Demo 3: Complex Factory - Struct Configuration
    // =========================================================================
    println!("\nDemo 3: Complex Factory with Struct Configuration");
    println!("─────────────────────────────────────────────────────────\n");

    println!("📝 Code:");
    println!("```rust");
    println!("#[derive(Clone)]");
    println!("struct FilterConfig {{");
    println!("    cutoff: f64,");
    println!("    mode: String,");
    println!("}}");
    println!();
    println!("fn make_filter(config: FilterConfig) -> impl Fn(...) -> ... {{");
    println!("    move |inputs, _| {{");
    println!("        let value = inputs.get(\"data\").unwrap().parse::<f64>().unwrap();");
    println!("        let filtered = match config.mode.as_str() {{");
    println!("            \"lowpass\" => value * config.cutoff,");
    println!("            \"highpass\" => value * (1.0 - config.cutoff),");
    println!("            _ => value,");
    println!("        }};");
    println!("    }}");
    println!("}}");
    println!();
    println!("let configs = vec![");
    println!("    FilterConfig {{ cutoff: 0.5, mode: \"lowpass\".to_string() }},");
    println!("    FilterConfig {{ cutoff: 0.3, mode: \"highpass\".to_string() }},");
    println!("    FilterConfig {{ cutoff: 0.7, mode: \"lowpass\".to_string() }},");
    println!("];");
    println!("graph.variant(make_filter, configs, Some(\"Filter\"), ...);");
    println!("```\n");

    let configs = vec![
        FilterConfig {
            cutoff: 0.5,
            mode: "lowpass".to_string(),
        },
        FilterConfig {
            cutoff: 0.3,
            mode: "highpass".to_string(),
        },
        FilterConfig {
            cutoff: 0.7,
            mode: "lowpass".to_string(),
        },
    ];

    let mut graph3 = Graph::new();
    graph3.add(
        data_source,
        Some("Source"),
        None,
        Some(vec![("value", "data")]),
    );
    graph3.variant(
        make_filter,
        configs,
        Some("Filter"),
        Some(vec![("data", "data")]),
        Some(vec![("filtered", "result")]),
    );

    let dag3 = graph3.build();
    println!("🎯 What happens:");
    println!("  • 3 filter variants created with different configurations");
    println!("  • Each variant uses its own FilterConfig struct");
    println!("  • Demonstrates passing complex types to factory");
    println!();

    let stats3 = dag3.stats();
    println!("📈 DAG Statistics:");
    println!("  - Total nodes: {}", stats3.node_count);
    println!("  - Filter variants: 3");
    println!("  - Max parallelism: {} nodes", stats3.max_parallelism);
    println!();

    // =========================================================================
    // Demo 4: String Processing Variants
    // =========================================================================
    println!("\nDemo 4: String Processing Variants");
    println!("─────────────────────────────────────────────────────────\n");

    println!("📝 Code:");
    println!("```rust");
    println!("fn make_processor(prefix: &'static str) -> impl Fn(...) -> ... {{");
    println!("    move |inputs, _| {{");
    println!("        let text = inputs.get(\"text\").unwrap();");
    println!("        let processed = format!(\"[{{}}] {{}}\", prefix, text);");
    println!("        outputs.insert(\"processed_text\", processed);");
    println!("    }}");
    println!("}}");
    println!();
    println!("graph.variant(");
    println!("    make_processor,");
    println!("    vec![\"INFO\", \"WARN\", \"ERROR\"],");
    println!("    Some(\"LogLevel\"),");
    println!("    Some(vec![(\"message\", \"text\")]),");
    println!("    Some(vec![(\"processed_text\", \"log\")])");
    println!(");");
    println!("```\n");

    let mut graph4 = Graph::new();
    graph4.add(
        text_source,
        Some("Source"),
        None,
        Some(vec![("message", "message")]),
    );
    graph4.variant(
        make_processor,
        vec!["INFO", "WARN", "ERROR"],
        Some("LogLevel"),
        Some(vec![("message", "text")]),
        Some(vec![("processed_text", "log")]),
    );

    let dag4 = graph4.build();
    println!("🎯 What happens:");
    println!("  • 3 log level variants: INFO, WARN, ERROR");
    println!("  • Each prefixes the message with its log level");
    println!("  • Demonstrates string/static str parameters");
    println!();

    let stats4 = dag4.stats();
    println!("📈 DAG Statistics:");
    println!("  - Total nodes: {}", stats4.node_count);
    println!("  - Log variants: 3");
    println!();

    println!("🔍 Mermaid Visualization:");
    println!("{}", dag4.to_mermaid());
    println!();

    // =========================================================================
    // Summary
    // =========================================================================
    println!("\n═══════════════════════════════════════════════════════════");
    println!("  Summary: Key Variant Pattern Features");
    println!("═══════════════════════════════════════════════════════════\n");

    println!("✅ Factory Function Pattern:");
    println!("   • Factory takes parameter(s), returns closure");
    println!("   • Closure captures parameters in its environment");
    println!("   • Same signature as regular node functions");
    println!();

    println!("✅ Parameter Flexibility:");
    println!("   • Primitives: f64, i32, &str");
    println!("   • Structs: Custom configuration objects");
    println!("   • Arrays/Vectors: Multiple values at once");
    println!();

    println!("✅ Cartesian Products:");
    println!("   • Multiple .variant() calls create all combinations");
    println!("   • Example: 2 scales × 3 filters = 6 execution paths");
    println!();

    println!("✅ Port Mapping:");
    println!("   • Variants use same tuple-based syntax");
    println!("   • (broadcast_var, impl_var) for inputs");
    println!("   • (impl_var, broadcast_var) for outputs");
    println!();

    println!("✅ Parallel Execution:");
    println!("   • All variants at same level can run in parallel");
    println!("   • DAG analysis identifies parallelization opportunities");
    println!();
}
