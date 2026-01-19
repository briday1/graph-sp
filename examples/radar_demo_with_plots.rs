//! Radar processing demo with plotting nodes
//!
//! This example demonstrates using graph nodes for plotting/visualization
//! that take input data but produce no output (terminal nodes).
//!
//! The pipeline implements:
//! 1. LFM pulse generation
//! 2. Pulse stacking
//! 3. Range compression
//! 4. Doppler compression
//! 5. Plotting nodes (no outputs, just visualization)

#[cfg(feature = "radar_examples")]
use graph_sp::{Graph, GraphData};

#[cfg(feature = "radar_examples")]
use ndarray::Array1;
#[cfg(feature = "radar_examples")]
use num_complex::Complex;
#[cfg(feature = "radar_examples")]
use std::collections::HashMap;

// ... (Include all the processing functions from radar_demo.rs here)

#[cfg(feature = "radar_examples")]
fn plot_lfm_pulse(
    inputs: &HashMap<String, GraphData>,
    _params: &HashMap<String, GraphData>,
) -> HashMap<String, GraphData> {
    // Plotting node - takes input but produces no output
    // In a real application, this would generate a plot file or display

    let pulse = match inputs.get("pulse").and_then(|d| d.as_complex_array()) {
        Some(p) => p,
        None => {
            eprintln!("PlotLFM: No pulse data found");
            return HashMap::new();
        }
    };

    println!(
        "PlotLFM: Would generate plot of {} sample LFM pulse",
        pulse.len()
    );
    println!("  - Time domain (real/imag components)");
    println!("  - Frequency domain (magnitude spectrum)");

    // No outputs - this is a terminal node
    HashMap::new()
}

#[cfg(feature = "radar_examples")]
fn plot_range_doppler(
    inputs: &HashMap<String, GraphData>,
    _params: &HashMap<String, GraphData>,
) -> HashMap<String, GraphData> {
    // Plotting node for Range-Doppler map

    if let Some(map) = inputs
        .get("range_doppler")
        .and_then(|d| d.as_complex_array())
    {
        println!("PlotRangeDoppler: Would generate Range-Doppler map plot");
        println!("  - Data shape: {} samples", map.len());
        println!("  - Colormap: Heat map showing target detection");
        println!("  - Axes: Range (bins) vs Doppler (bins)");
    }

    if let Some(peak) = inputs.get("peak_value").and_then(|d| d.as_float()) {
        println!("  - Peak magnitude: {:.2}", peak);
    }

    // No outputs - this is a terminal node
    HashMap::new()
}

#[cfg(feature = "radar_examples")]
fn main() {
    let separator = "=".repeat(70);
    println!("{}", separator);
    println!("Radar Processing Demo with Plotting Nodes");
    println!("{}", separator);
    println!();

    // Note: This is a skeleton - you would include the full processing functions
    // from radar_demo.rs here. For brevity, showing just the graph structure.

    let mut graph = Graph::new();

    println!("Building radar processing pipeline with plotting nodes...");

    // Processing pipeline (simplified for demonstration)
    // ... add all processing nodes ...

    // Add plotting nodes - they take input but produce no output
    graph.add(
        plot_lfm_pulse,
        Some("PlotLFMPulse"),
        Some(vec![("lfm_pulse", "pulse")]),
        None, // No outputs - terminal node for visualization
    );

    graph.add(
        plot_range_doppler,
        Some("PlotRangeDoppler"),
        Some(vec![
            ("range_doppler_map", "range_doppler"),
            ("peak", "peak_value"),
        ]),
        None, // No outputs - terminal node for visualization
    );

    println!("\nBuilding DAG...");
    let dag = graph.build();

    println!("\n{}", dag.to_mermaid());
    println!();

    println!("Executing radar processing with plotting...\n");
    let _context = dag.execute(false, None);

    println!("\n{}", separator);
    println!("Demo completed! Plot files would be saved.");
    println!("{}", separator);
}

#[cfg(not(feature = "radar_examples"))]
fn main() {
    println!("This example requires the 'radar_examples' feature.");
    println!("Run with: cargo run --example radar_demo_with_plots --features radar_examples");
}
