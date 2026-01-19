//! Radar processing demo using GraphData and ndarray
//!
//! This example demonstrates using GraphData to pass signal processing data
//! (ndarray arrays and complex numbers) through graph nodes.
//!
//! The pipeline implements a simple radar processing chain:
//! 1. LFM (Linear Frequency Modulation) pulse generation
//! 2. Pulse stacking (accumulating multiple pulses)
//! 3. Range compression using FFT
//! 4. Simple magnitude extraction

#[cfg(feature = "radar_examples")]
use graph_sp::{Graph, GraphData};

#[cfg(feature = "radar_examples")]
use ndarray::Array1;
#[cfg(feature = "radar_examples")]
use num_complex::Complex;
#[cfg(feature = "radar_examples")]
use rustfft::{FftPlanner, num_complex::Complex64};
#[cfg(feature = "radar_examples")]
use std::collections::HashMap;
#[cfg(feature = "radar_examples")]
use std::f64::consts::PI;

#[cfg(feature = "radar_examples")]
fn lfm_generator(_inputs: &HashMap<String, GraphData>, params: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    // Get parameters or use defaults
    let num_samples = params.get("num_samples")
        .and_then(|d| d.as_int())
        .unwrap_or(256) as usize;
    
    let bandwidth = params.get("bandwidth")
        .and_then(|d| d.as_float())
        .unwrap_or(100e6); // 100 MHz
    
    let duration = params.get("duration")
        .and_then(|d| d.as_float())
        .unwrap_or(10e-6); // 10 microseconds
    
    // Generate LFM pulse
    let sample_rate = num_samples as f64 / duration;
    let chirp_rate = bandwidth / duration;
    
    let mut signal = Array1::<Complex<f64>>::zeros(num_samples);
    for (i, sample) in signal.iter_mut().enumerate() {
        let t = i as f64 / sample_rate;
        let phase = 2.0 * PI * (chirp_rate / 2.0 * t * t);
        *sample = Complex::new(phase.cos(), phase.sin());
    }
    
    println!("LFMGenerator: Generated {} sample LFM pulse", num_samples);
    
    let mut output = HashMap::new();
    output.insert("pulse".to_string(), GraphData::complex_array(signal));
    output.insert("num_samples".to_string(), GraphData::int(num_samples as i64));
    output
}

#[cfg(feature = "radar_examples")]
fn stack_pulses(inputs: &HashMap<String, GraphData>, params: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    let num_pulses = params.get("num_pulses")
        .and_then(|d| d.as_int())
        .unwrap_or(4) as usize;
    
    // Get the input pulse
    let pulse = match inputs.get("pulse").and_then(|d| d.as_complex_array()) {
        Some(p) => p.clone(),
        None => {
            eprintln!("StackPulses: No pulse data found");
            let mut output = HashMap::new();
            output.insert("stacked".to_string(), GraphData::none());
            return output;
        }
    };
    
    let num_samples = pulse.len();
    
    // Create a 2D-like representation as a flattened array
    // In a real system, this would be a 2D ndarray
    let mut stacked = Array1::<Complex<f64>>::zeros(num_samples * num_pulses);
    
    for pulse_idx in 0..num_pulses {
        let offset = pulse_idx * num_samples;
        for (i, &val) in pulse.iter().enumerate() {
            stacked[offset + i] = val;
        }
    }
    
    println!("StackPulses: Stacked {} pulses of {} samples each", num_pulses, num_samples);
    
    let mut output = HashMap::new();
    output.insert("stacked".to_string(), GraphData::complex_array(stacked));
    output.insert("num_pulses".to_string(), GraphData::int(num_pulses as i64));
    output.insert("num_samples".to_string(), GraphData::int(num_samples as i64));
    output
}

#[cfg(feature = "radar_examples")]
fn range_compress(inputs: &HashMap<String, GraphData>, _params: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    // Get the stacked data
    let stacked = match inputs.get("data").and_then(|d| d.as_complex_array()) {
        Some(s) => s,
        None => {
            eprintln!("RangeCompress: No stacked data found");
            let mut output = HashMap::new();
            output.insert("compressed".to_string(), GraphData::none());
            return output;
        }
    };
    
    // Convert to rustfft's Complex64
    let mut buffer: Vec<Complex64> = stacked.iter()
        .map(|c| Complex64::new(c.re, c.im))
        .collect();
    
    // Perform FFT
    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(buffer.len());
    fft.process(&mut buffer);
    
    // Convert back to our Complex type and wrap in Array1
    let compressed: Array1<Complex<f64>> = buffer.iter()
        .map(|c| Complex::new(c.re, c.im))
        .collect();
    
    println!("RangeCompress: Performed FFT on {} samples", compressed.len());
    
    let mut output = HashMap::new();
    output.insert("compressed".to_string(), GraphData::complex_array(compressed));
    output
}

#[cfg(feature = "radar_examples")]
fn magnitude_extractor(inputs: &HashMap<String, GraphData>, _params: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    // Get the compressed data
    let compressed = match inputs.get("data").and_then(|d| d.as_complex_array()) {
        Some(c) => c,
        None => {
            eprintln!("MagnitudeExtractor: No compressed data found");
            let mut output = HashMap::new();
            output.insert("magnitude".to_string(), GraphData::none());
            return output;
        }
    };
    
    // Extract magnitude
    let magnitude: Array1<f64> = compressed.iter()
        .map(|c| c.norm())
        .collect();
    
    // Find peak
    let max_val = magnitude.iter().cloned().fold(0.0f64, f64::max);
    let max_idx = magnitude.iter().enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(idx, _)| idx)
        .unwrap_or(0);
    
    println!("MagnitudeExtractor: Peak magnitude {} at index {}", max_val, max_idx);
    
    let mut output = HashMap::new();
    output.insert("magnitude".to_string(), GraphData::float_array(magnitude));
    output.insert("peak_value".to_string(), GraphData::float(max_val));
    output.insert("peak_index".to_string(), GraphData::int(max_idx as i64));
    output
}

#[cfg(feature = "radar_examples")]
fn main() {
    let separator = "=".repeat(70);
    println!("{}", separator);
    println!("Radar Processing Demo with GraphData and ndarray");
    println!("{}", separator);
    println!();
    
    // Create the graph
    let mut graph = Graph::new();
    
    // Add LFM generator
    println!("Building radar processing pipeline...");
    graph.add(
        lfm_generator,
        Some("LFMGenerator"),
        None,
        Some(vec![("pulse", "lfm_pulse"), ("num_samples", "num_samples")])
    );
    
    // Add pulse stacker
    graph.add(
        stack_pulses,
        Some("StackPulses"),
        Some(vec![("lfm_pulse", "pulse")]),
        Some(vec![
            ("stacked", "stacked_data"),
            ("num_pulses", "num_pulses"),
            ("num_samples", "num_samples")
        ])
    );
    
    // Add range compression
    graph.add(
        range_compress,
        Some("RangeCompress"),
        Some(vec![("stacked_data", "data")]),
        Some(vec![("compressed", "compressed_data")])
    );
    
    // Add magnitude extraction
    graph.add(
        magnitude_extractor,
        Some("MagnitudeExtractor"),
        Some(vec![("compressed_data", "data")]),
        Some(vec![
            ("magnitude", "magnitude"),
            ("peak_value", "peak"),
            ("peak_index", "peak_idx")
        ])
    );
    
    // Build and execute
    println!("\nBuilding DAG...");
    let dag = graph.build();
    
    println!("\n{}", dag.to_mermaid());
    println!();
    
    let stats = dag.stats();
    println!("DAG Statistics:");
    println!("  Nodes: {}", stats.node_count);
    println!("  Depth: {} levels", stats.depth);
    println!("  Max Parallelism: {} nodes", stats.max_parallelism);
    println!();
    
    println!("Executing radar processing pipeline...\n");
    let context = dag.execute(false, None);
    
    println!("\n{}", separator);
    println!("Execution Complete!");
    println!("{}", separator);
    
    // Display results
    if let Some(peak) = context.get("peak").and_then(|d| d.as_float()) {
        println!("Peak magnitude: {:.2}", peak);
    }
    if let Some(idx) = context.get("peak_idx").and_then(|d| d.as_int()) {
        println!("Peak index: {}", idx);
    }
    
    println!("\nRadar demo completed successfully!");
}

#[cfg(not(feature = "radar_examples"))]
fn main() {
    println!("This example requires the 'radar_examples' feature.");
    println!("Run with: cargo run --example radar_demo --features radar_examples");
}
