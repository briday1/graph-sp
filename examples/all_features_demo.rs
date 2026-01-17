//! Comprehensive Example: Showing Both Implicit and Explicit Edge Mapping
//!
//! This example demonstrates ALL features with both mapping modes:
//! - Implicit edge mapping (default): automatic connections based on port names
//! - Explicit edge mapping (strict mode): manual add_edge() calls required
//! - Branching, variants, merging
//! - Complete mermaid diagram output for everything

use graph_sp::core::{
    Edge, Graph, MergeConfig, Node, NodeConfig, Port, PortData, VariantConfig, VariantFunction,
};
use graph_sp::inspector::Inspector;
use std::collections::HashMap;
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║  Graph-SP: Complete Feature Demonstration                       ║");
    println!("║  Implicit vs Explicit Edge Mapping + All Features               ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // ========================================================================
    // PART 1: IMPLICIT EDGE MAPPING (Default Mode - Auto-connects)
    // ========================================================================
    println!("═══════════════════════════════════════════════════════════════════");
    println!("PART 1: IMPLICIT EDGE MAPPING (Default Mode)");
    println!("═══════════════════════════════════════════════════════════════════\n");

    println!("In implicit mode, nodes are automatically connected based on:");
    println!("  1. Port name matching (e.g., 'output' -> 'output')");
    println!("  2. Single port auto-connect (if only 1 output and 1 input)\n");

    let mut implicit_graph = Graph::new(); // Default: implicit mapping ON

    // Example 1a: Simple pipeline with matching port names
    println!("Example 1a: Pipeline with matching port names");
    println!("---------------------------------------------");

    let source1 = NodeConfig::new(
        "source1",
        "Data Source",
        vec![],
        vec![Port::new("data", "Output Data")],
        Arc::new(|_: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            outputs.insert("data".to_string(), PortData::Int(42));
            Ok(outputs)
        }),
    );

    let processor1 = NodeConfig::new(
        "processor1",
        "Data Processor",
        vec![Port::new("data", "Input Data")], // Matches "data" output
        vec![Port::new("result", "Processed")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            if let Some(PortData::Int(val)) = inputs.get("data") {
                outputs.insert("result".to_string(), PortData::Int(val * 2));
            }
            Ok(outputs)
        }),
    );

    let sink1 = NodeConfig::new(
        "sink1",
        "Result Sink",
        vec![Port::new("result", "Final")], // Matches "result" output
        vec![],
        Arc::new(|_: &HashMap<String, PortData>| Ok(HashMap::new())),
    );

    // Just add nodes - edges are created automatically!
    implicit_graph.add(Node::new(source1))?;
    implicit_graph.add(Node::new(processor1))?;
    implicit_graph.add(Node::new(sink1))?;

    println!("Added 3 nodes with matching port names");
    println!(
        "✅ Edges created automatically: {}",
        implicit_graph.edge_count()
    );
    println!("   source1 (data) -> processor1 (data)");
    println!("   processor1 (result) -> sink1 (result)");
    println!();

    println!("🎨 Mermaid Diagram (Implicit Mapping):");
    println!("{}", Inspector::to_mermaid(&implicit_graph)?);

    // Example 1b: Single-port auto-connect
    println!("\nExample 1b: Single-port auto-connect");
    println!("-------------------------------------");

    let mut implicit_graph2 = Graph::new();

    let source2 = NodeConfig::new(
        "src",
        "Source",
        vec![],
        vec![Port::new("out", "Only Output")], // Only 1 output
        Arc::new(|_: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            outputs.insert("out".to_string(), PortData::Int(100));
            Ok(outputs)
        }),
    );

    let proc2 = NodeConfig::new(
        "proc",
        "Processor",
        vec![Port::new("in", "Only Input")], // Only 1 input (different name!)
        vec![Port::new("result", "Output")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            if let Some(PortData::Int(val)) = inputs.get("in") {
                outputs.insert("result".to_string(), PortData::Int(val + 50));
            }
            Ok(outputs)
        }),
    );

    implicit_graph2.add(Node::new(source2))?;
    implicit_graph2.add(Node::new(proc2))?;

    println!("Port names don't match ('out' vs 'in')");
    println!("BUT both have only 1 port each");
    println!(
        "✅ Auto-connected anyway: {} edge",
        implicit_graph2.edge_count()
    );
    println!();

    println!("🎨 Mermaid Diagram:");
    println!("{}", Inspector::to_mermaid(&implicit_graph2)?);

    // ========================================================================
    // PART 2: EXPLICIT EDGE MAPPING (Strict Mode - Manual add_edge)
    // ========================================================================
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("PART 2: EXPLICIT EDGE MAPPING (Strict Mode)");
    println!("═══════════════════════════════════════════════════════════════════\n");

    println!("In strict mode, you MUST call add_edge() explicitly");
    println!("No automatic connections are made\n");

    let mut explicit_graph = Graph::with_strict_edges(); // Strict mode ON

    println!("Example 2: Manual edge specification");
    println!("------------------------------------");

    let source3 = NodeConfig::new(
        "source",
        "Data Source",
        vec![],
        vec![Port::new("value", "Value")],
        Arc::new(|_: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            outputs.insert("value".to_string(), PortData::Int(25));
            Ok(outputs)
        }),
    );

    let doubler = NodeConfig::new(
        "doubler",
        "Doubler",
        vec![Port::new("input", "Input")],
        vec![Port::new("output", "Output")],
        Arc::new(|inputs: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            if let Some(PortData::Int(val)) = inputs.get("input") {
                outputs.insert("output".to_string(), PortData::Int(val * 2));
            }
            Ok(outputs)
        }),
    );

    let sink3 = NodeConfig::new(
        "sink",
        "Sink",
        vec![Port::new("final", "Final")],
        vec![],
        Arc::new(|_: &HashMap<String, PortData>| Ok(HashMap::new())),
    );

    // Add nodes
    explicit_graph.add(Node::new(source3))?;
    explicit_graph.add(Node::new(doubler))?;
    explicit_graph.add(Node::new(sink3))?;

    println!("Added 3 nodes");
    println!(
        "Edges before manual connection: {}",
        explicit_graph.edge_count()
    );

    // NOW manually add edges
    explicit_graph.add_edge(Edge::new("source", "value", "doubler", "input"))?;
    explicit_graph.add_edge(Edge::new("doubler", "output", "sink", "final"))?;

    println!(
        "✅ After add_edge() calls: {} edges",
        explicit_graph.edge_count()
    );
    println!("   Manually specified:");
    println!("   - source (value) -> doubler (input)");
    println!("   - doubler (output) -> sink (final)");
    println!();

    println!("🎨 Mermaid Diagram (Explicit Mapping):");
    println!("{}", Inspector::to_mermaid(&explicit_graph)?);

    // ========================================================================
    // PART 3: VARIANTS WITH IMPLICIT MAPPING
    // ========================================================================
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("PART 3: VARIANTS (Config Sweeps) - Implicit Mode");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let mut variant_graph = Graph::new();

    let lr_fn: VariantFunction = Arc::new(|i: usize| PortData::Float((i as f64 + 1.0) * 0.01));

    let variant_config = VariantConfig::new("lr", 4, "learning_rate", lr_fn);
    let branches = variant_graph.create_variants(variant_config)?;

    println!("Created {} learning rate variants", branches.len());
    for (i, branch_name) in branches.iter().enumerate() {
        println!("  - {} = {}", branch_name, (i + 1) as f64 * 0.01);
    }
    println!();

    println!("🎨 Mermaid Diagram (Shows '4 variants' hexagon):");
    println!("{}", Inspector::to_mermaid(&variant_graph)?);

    // ========================================================================
    // PART 4: MERGE WITH CUSTOM FUNCTION
    // ========================================================================
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("PART 4: MERGE (Combining Branch Outputs)");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let mut merge_graph = Graph::new();

    // Create 3 model branches
    for i in 0..3 {
        let branch_name = format!("model_{}", i);
        let branch = merge_graph.create_branch(&branch_name)?;

        let score = (i + 1) * 20;
        let model_src = NodeConfig::new(
            format!("src_{}", i),
            format!("Model {}", i),
            vec![],
            vec![Port::new("accuracy", "Accuracy")],
            Arc::new(move |_: &HashMap<String, PortData>| {
                let mut outputs = HashMap::new();
                outputs.insert("accuracy".to_string(), PortData::Int(score as i64));
                Ok(outputs)
            }),
        );
        branch.add(Node::new(model_src))?;
    }

    // Merge with MAX function
    let max_fn = Arc::new(
        |inputs: Vec<&PortData>| -> graph_sp::core::Result<PortData> {
            let max_val = inputs
                .iter()
                .filter_map(|d| {
                    if let PortData::Int(v) = d {
                        Some(*v)
                    } else {
                        None
                    }
                })
                .max()
                .unwrap_or(0);
            Ok(PortData::Int(max_val))
        },
    );

    let merge_cfg = MergeConfig::new(
        vec!["model_0".into(), "model_1".into(), "model_2".into()],
        "accuracy".into(),
    )
    .with_merge_fn(max_fn);

    merge_graph.merge("best_model", merge_cfg)?;

    println!("Merged 3 model branches with MAX function");
    println!("  Models: model_0 (20), model_1 (40), model_2 (60)");
    println!("  Merge function: selects maximum accuracy");
    println!();

    println!("🎨 Mermaid Diagram:");
    println!("{}", Inspector::to_mermaid(&merge_graph)?);

    // ========================================================================
    // PART 5: NESTED VARIANTS (Cartesian Product)
    // ========================================================================
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("PART 5: NESTED VARIANTS (Grid Search: 2 LR × 3 Batch = 6 configs)");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let mut grid_graph = Graph::new();

    // Level 1: Learning rates
    let lr_fn2: VariantFunction = Arc::new(|i: usize| PortData::Float((i as f64 + 1.0) * 0.01));
    let lr_cfg = VariantConfig::new("lr", 2, "learning_rate", lr_fn2);
    let lr_branches = grid_graph.create_variants(lr_cfg)?;

    println!("Level 1: {} learning rate variants", lr_branches.len());

    // Level 2: Batch sizes in each LR
    for lr_branch in &lr_branches {
        let branch = grid_graph.get_branch_mut(lr_branch)?;
        let batch_fn: VariantFunction = Arc::new(|i: usize| PortData::Int((i as i64 + 1) * 32));
        let batch_cfg = VariantConfig::new("batch", 3, "batch_size", batch_fn);
        let batch_branches = branch.create_variants(batch_cfg)?;
        println!("  └─ {}: {} batch size variants", lr_branch, batch_branches.len());
    }

    println!("\nTotal combinations: 2 × 3 = 6 configurations");
    println!();

    println!("🎨 Main Graph (LR variants):");
    println!("{}", Inspector::to_mermaid(&grid_graph)?);

    println!("\n🎨 Inside 'lr_0' (Batch variants):");
    let lr0 = grid_graph.get_branch("lr_0")?;
    println!("{}", Inspector::to_mermaid(lr0)?);

    // ========================================================================
    // PART 6: TOGGLING MODES
    // ========================================================================
    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("PART 6: TOGGLING BETWEEN MODES");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let mut toggle_graph = Graph::new();
    println!("Created graph in implicit mode (default)");

    let n1 = NodeConfig::new(
        "n1",
        "Node 1",
        vec![],
        vec![Port::new("out", "Out")],
        Arc::new(|_: &HashMap<String, PortData>| {
            let mut outputs = HashMap::new();
            outputs.insert("out".to_string(), PortData::Int(1));
            Ok(outputs)
        }),
    );

    let n2 = NodeConfig::new(
        "n2",
        "Node 2",
        vec![Port::new("out", "In")],
        vec![],
        Arc::new(|_: &HashMap<String, PortData>| Ok(HashMap::new())),
    );

    toggle_graph.add(Node::new(n1))?;
    toggle_graph.add(Node::new(n2))?;
    println!(
        "Added 2 nodes → {} edge (auto-connected)",
        toggle_graph.edge_count()
    );

    // Switch to strict mode
    toggle_graph.set_strict_edge_mapping(true);
    println!("\nSwitched to strict mode");

    let n3 = NodeConfig::new(
        "n3",
        "Node 3",
        vec![Port::new("x", "X")],
        vec![],
        Arc::new(|_: &HashMap<String, PortData>| Ok(HashMap::new())),
    );

    toggle_graph.add(Node::new(n3))?;
    println!(
        "Added another node → Still {} edge (no auto-connect in strict mode)",
        toggle_graph.edge_count()
    );
    println!();

    // ========================================================================
    // SUMMARY
    // ========================================================================
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║  SUMMARY: Complete Feature Set                                  ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    println!("📋 EDGE MAPPING MODES:");
    println!("   ✅ Implicit (default): Graph::new()");
    println!("      - Auto-connects based on port name matching");
    println!("      - Auto-connects single port pairs");
    println!("   ✅ Explicit (strict): Graph::with_strict_edges()");
    println!("      - Requires manual add_edge() calls");
    println!("      - Full control over connections");
    println!("   ✅ Toggle: set_strict_edge_mapping(bool)");
    println!();

    println!("📋 OTHER FEATURES:");
    println!("   ✅ add() - Add nodes (simplified API)");
    println!("   ✅ Branches - create_branch() for isolated subgraphs");
    println!("   ✅ Variants - create_variants() for parameter sweeps");
    println!("   ✅ Nested Variants - Cartesian product (N × M configs)");
    println!("   ✅ Merge - Custom merge functions (max, sum, list, etc.)");
    println!("   ✅ Mermaid - Full visualization with variant hexagons");
    println!();

    println!("📖 LEGEND:");
    println!("   🔵 Blue nodes      = Source (no inputs)");
    println!("   🟣 Purple nodes    = Sink (no outputs)");
    println!("   🟠 Orange nodes    = Processing");
    println!("   🟢 Green hexagons  = Variant groups (N variants)");
    println!();

    Ok(())
}
