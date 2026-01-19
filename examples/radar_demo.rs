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
    
    let pulse_width = params.get("pulse_width")
        .and_then(|d| d.as_float())
        .unwrap_or(2e-6); // 2 microseconds
    
    let sample_rate = 100e6; // 100 MHz sample rate
    
    // Generate LFM pulse with rectangular envelope
    let chirp_rate = bandwidth / pulse_width;
    
    let pulse_start = (num_samples as f64 * 0.15) as usize; // Start at 15% so peak appears mid-range
    let pulse_samples = (pulse_width * sample_rate) as usize;
    let pulse_end = (pulse_start + pulse_samples).min(num_samples);
    
    let mut signal = Array1::<Complex<f64>>::zeros(num_samples);
    
    // Generate chirp only within pulse envelope
    for i in pulse_start..pulse_end {
        let t_pulse = (i - pulse_start) as f64 / sample_rate;
        let phase = 2.0 * PI * (chirp_rate / 2.0 * t_pulse * t_pulse);
        signal[i] = Complex::new(phase.cos(), phase.sin());
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
        .unwrap_or(128) as usize;  // Default to 128 pulses (matching sigexec and Python)
    
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
    
    // Doppler simulation parameters (matching Python demo)
    let doppler_freq = 1000.0; // Hz, simulated target velocity
    let prf = 10000.0; // Pulse Repetition Frequency (Hz)
    
    // Create a 2D-like representation as a flattened array with Doppler shifts
    let mut stacked = Array1::<Complex<f64>>::zeros(num_samples * num_pulses);
    
    for pulse_idx in 0..num_pulses {
        // Add Doppler shift (matching Python implementation exactly)
        let phase_shift = 2.0 * PI * doppler_freq * pulse_idx as f64 / prf;
        let doppler_shift = Complex::new(phase_shift.cos(), phase_shift.sin());
        
        let offset = pulse_idx * num_samples;
        for (i, &val) in pulse.iter().enumerate() {
            stacked[offset + i] = val * doppler_shift;
        }
    }
    
    println!("StackPulses: Stacked {} pulses with Doppler shifts", num_pulses);
    
    let mut output = HashMap::new();
    output.insert("stacked".to_string(), GraphData::complex_array(stacked));
    output.insert("num_pulses".to_string(), GraphData::int(num_pulses as i64));
    output.insert("num_samples".to_string(), GraphData::int(num_samples as i64));
    output
}

#[cfg(feature = "radar_examples")]
fn range_compress(inputs: &HashMap<String, GraphData>, _params: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    // Get the stacked data and reference pulse
    let stacked = match inputs.get("data").and_then(|d| d.as_complex_array()) {
        Some(s) => s,
        None => {
            eprintln!("RangeCompress: No stacked data found");
            let mut output = HashMap::new();
            output.insert("compressed".to_string(), GraphData::none());
            return output;
        }
    };
    
    let reference = match inputs.get("reference").and_then(|d| d.as_complex_array()) {
        Some(r) => r,
        None => {
            eprintln!("RangeCompress: No reference pulse found");
            let mut output = HashMap::new();
            output.insert("compressed".to_string(), GraphData::none());
            return output;
        }
    };
    
    // Matched filter: correlate with conjugate of reference pulse
    // Time-reverse and conjugate the reference
    let mut ref_conj: Vec<Complex64> = reference.iter().rev()
        .map(|c| Complex64::new(c.re, -c.im))
        .collect();
    
    // Pad reference conjugate to match stacked length
    let stacked_len = stacked.len();
    ref_conj.resize(stacked_len, Complex64::new(0.0, 0.0));
    
    // Convert stacked to Complex64 for FFT
    let signal: Vec<Complex64> = stacked.iter()
        .map(|c| Complex64::new(c.re, c.im))
        .collect();
    
    // Perform matched filtering via FFT
    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(stacked_len);
    let ifft = planner.plan_fft_inverse(stacked_len);
    
    // FFT of signal and reference
    let mut signal_fft = signal.clone();
    let mut ref_fft = ref_conj;
    fft.process(&mut signal_fft);
    fft.process(&mut ref_fft);
    
    // Multiply in frequency domain
    for (s, r) in signal_fft.iter_mut().zip(ref_fft.iter()) {
        *s = *s * r;
    }
    
    // IFFT to get correlation
    ifft.process(&mut signal_fft);
    
    // Normalize by length
    let norm = stacked_len as f64;
    let compressed: Array1<Complex<f64>> = signal_fft.iter()
        .map(|c| Complex::new(c.re / norm, c.im / norm))
        .collect();
    
    println!("RangeCompress: Performed matched filtering on {} samples", compressed.len());
    
    let mut output = HashMap::new();
    output.insert("compressed".to_string(), GraphData::complex_array(compressed));
    output
}

#[cfg(feature = "radar_examples")]
fn doppler_compress(inputs: &HashMap<String, GraphData>, _params: &HashMap<String, GraphData>) -> HashMap<String, GraphData> {
    // Get the range-compressed data and metadata
    let compressed = match inputs.get("data").and_then(|d| d.as_complex_array()) {
        Some(c) => c,
        None => {
            eprintln!("DopplerCompress: No compressed data found");
            let mut output = HashMap::new();
            output.insert("range_doppler".to_string(), GraphData::none());
            return output;
        }
    };
    
    let num_pulses = match inputs.get("num_pulses").and_then(|d| d.as_int()) {
        Some(n) => n as usize,
        None => {
            eprintln!("DopplerCompress: No num_pulses found");
            let mut output = HashMap::new();
            output.insert("range_doppler".to_string(), GraphData::none());
            return output;
        }
    };
    
    let num_samples = match inputs.get("num_samples").and_then(|d| d.as_int()) {
        Some(n) => n as usize,
        None => {
            eprintln!("DopplerCompress: No num_samples found");
            let mut output = HashMap::new();
            output.insert("range_doppler".to_string(), GraphData::none());
            return output;
        }
    };
    
    // Perform FFT along slow-time (Doppler) dimension
    let mut planner = FftPlanner::<f64>::new();
    let fft = planner.plan_fft_forward(num_pulses);
    
    // Process each range bin
    let mut range_doppler = Array1::<Complex<f64>>::zeros(num_pulses * num_samples);
    
    for range_bin in 0..num_samples {
        // Extract slow-time samples for this range bin
        let mut slow_time: Vec<Complex64> = (0..num_pulses)
            .map(|pulse_idx| {
                let idx = pulse_idx * num_samples + range_bin;
                let c = &compressed[idx];
                Complex64::new(c.re, c.im)
            })
            .collect();
        
        // Apply FFT
        fft.process(&mut slow_time);
        
        // Store in range-doppler map
        for (doppler_bin, val) in slow_time.iter().enumerate() {
            let idx = doppler_bin * num_samples + range_bin;
            range_doppler[idx] = Complex::new(val.re, val.im);
        }
    }
    
    // Find peak in Range-Doppler map
    let magnitudes: Vec<f64> = range_doppler.iter().map(|c| c.norm()).collect();
    let max_val = magnitudes.iter().cloned().fold(0.0f64, f64::max);
    let max_idx = magnitudes.iter().enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(idx, _)| idx)
        .unwrap_or(0);
    
    let doppler_bin = max_idx / num_samples;
    let range_bin = max_idx % num_samples;
    
    println!("DopplerCompress: Created Range-Doppler map of shape ({}, {})", num_pulses, num_samples);
    println!("  Peak at Doppler bin {}, Range bin {}", doppler_bin, range_bin);
    println!("  Magnitude: {:.2}", max_val);
    
    let mut output = HashMap::new();
    output.insert("range_doppler".to_string(), GraphData::complex_array(range_doppler));
    output.insert("peak_value".to_string(), GraphData::float(max_val));
    output.insert("peak_doppler_bin".to_string(), GraphData::int(doppler_bin as i64));
    output.insert("peak_range_bin".to_string(), GraphData::int(range_bin as i64));
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
    
    // Add range compression (needs both stacked data and reference pulse)
    graph.add(
        range_compress,
        Some("RangeCompress"),
        Some(vec![("stacked_data", "data"), ("lfm_pulse", "reference")]),
        Some(vec![("compressed", "compressed_data")])
    );
    
    // Add Doppler compression (FFT along slow-time dimension)
    graph.add(
        doppler_compress,
        Some("DopplerCompress"),
        Some(vec![
            ("compressed_data", "data"),
            ("num_pulses", "num_pulses"),
            ("num_samples", "num_samples")
        ]),
        Some(vec![
            ("range_doppler", "range_doppler_map"),
            ("peak_value", "peak"),
            ("peak_doppler_bin", "peak_doppler"),
            ("peak_range_bin", "peak_range")
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
    if let Some(doppler) = context.get("peak_doppler").and_then(|d| d.as_int()) {
        println!("Peak Doppler bin: {}", doppler);
    }
    if let Some(range) = context.get("peak_range").and_then(|d| d.as_int()) {
        println!("Peak Range bin: {}", range);
    }
    
    println!("\nRadar demo completed successfully!");
}

#[cfg(not(feature = "radar_examples"))]
fn main() {
    println!("This example requires the 'radar_examples' feature.");
    println!("Run with: cargo run --example radar_demo --features radar_examples");
}
