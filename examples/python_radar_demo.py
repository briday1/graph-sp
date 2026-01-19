#!/usr/bin/env python3
"""
Radar processing demo using graph-sp Python bindings with GraphData

This example demonstrates a radar signal processing pipeline:
1. LFM (Linear Frequency Modulation) pulse generation
2. Pulse stacking (accumulating multiple pulses)
3. Range compression using FFT
4. Doppler compression

The example uses Python's numpy for array processing and shows how
graph-sp can handle complex signal processing workflows.
"""

import graph_sp
import numpy as np

def lfm_generator(inputs, variant_params):
    """
    Generate a Linear Frequency Modulation (LFM) pulse with rectangular envelope.
    
    Args:
        inputs: Dictionary of input variables (empty for source nodes)
        variant_params: Dictionary of variant parameters
    
    Returns:
        Dictionary with 'pulse' (complex array) and 'num_samples'
    """
    num_samples = 256
    bandwidth = 100e6  # 100 MHz
    pulse_width = 1e-6  # 1 microsecond pulse width (shorter for centered target)
    sample_rate = 100e6  # 100 MHz sample rate
    
    # Generate LFM chirp
    chirp_rate = bandwidth / pulse_width
    t = np.arange(num_samples) / sample_rate
    
    # Create rectangular pulse envelope
    # Position pulse so peak appears at center (bin ~128) after matched filtering
    # With 'same' mode correlation, peak appears at pulse center
    # pulse_samples = 100, center should be at 128, so start at 78
    pulse_envelope = np.zeros(num_samples)
    pulse_samples = int(pulse_width * sample_rate)
    pulse_start = 78  # Chosen so peak appears at bin 128
    pulse_end = pulse_start + pulse_samples
    pulse_envelope[pulse_start:pulse_end] = 1.0
    
    # Generate chirp phase (only within pulse)
    phase = np.zeros(num_samples)
    for i in range(pulse_start, min(pulse_end, num_samples)):
        t_pulse = (i - pulse_start) / sample_rate
        phase[i] = 2 * np.pi * (chirp_rate / 2.0 * t_pulse * t_pulse)
    
    # Apply envelope to create pulsed LFM signal
    signal = pulse_envelope * np.exp(1j * phase)
    
    print(f"LFMGenerator: Generated {num_samples} sample LFM pulse")
    
    # Return complex array directly (numpy array supported)
    return {
        "pulse": signal,  # Can pass numpy array directly
        "num_samples": num_samples
    }

def stack_pulses(inputs, variant_params):
    """
    Stack multiple identical pulses to create a pulse-Doppler radar data cube.
    
    Args:
        inputs: Dictionary with 'pulse' key containing the pulse to stack
        variant_params: Dictionary of variant parameters
    
    Returns:
        Dictionary with 'stacked' (2D array as list) and metadata
    """
    num_pulses = 128  # Number of pulses to stack (matching sigexec)
    
    pulse_data = inputs.get("pulse", None)
    if not pulse_data or not isinstance(pulse_data, list):
        print(f"StackPulses: No pulse data found or wrong format. Got: {type(pulse_data)}")
        return {"stacked": None}
    
    # Convert directly to numpy complex array (no tuple conversion needed)
    pulse = np.array(pulse_data, dtype=complex)
    num_samples = len(pulse)
    
    # Stack pulses (in real radar, these would be from different transmit times)
    # For demo purposes, we add slight phase variations to simulate Doppler
    stacked = []
    doppler_freq = 1000  # Hz, simulated target velocity
    prf = 10000  # Pulse Repetition Frequency (Hz)
    
    for pulse_idx in range(num_pulses):
        # Add Doppler shift
        phase_shift = 2 * np.pi * doppler_freq * pulse_idx / prf
        shifted_pulse = pulse * np.exp(1j * phase_shift)
        stacked.append(shifted_pulse)
    
    stacked = np.array(stacked)  # Shape: (num_pulses, num_samples)
    
    print(f"StackPulses: Stacked {num_pulses} pulses of {num_samples} samples each")
    
    # Return as numpy array directly (no conversion needed)
    return {
        "stacked": stacked,  # Can pass numpy array directly
        "num_pulses": num_pulses,
        "num_samples": num_samples
    }

def range_compress(inputs, variant_params):
    """
    Perform range compression using matched filter (correlate with reference pulse).
    This implements pulse compression for radar signal processing.
    
    Args:
        inputs: Dictionary with 'data' (stacked pulses) and 'reference' (LFM pulse) keys
        variant_params: Dictionary of variant parameters
    
    Returns:
        Dictionary with 'compressed' range-compressed data
    """
    stacked_data = inputs.get("data", None)
    reference_data = inputs.get("reference", None)
    
    if stacked_data is None or not isinstance(stacked_data, list):
        print(f"RangeCompress: No stacked data found or wrong format. Got: {type(stacked_data)}")
        return {"compressed": None}
    
    if reference_data is None or not isinstance(reference_data, list):
        print(f"RangeCompress: No reference pulse found")
        return {"compressed": None}
    
    # Convert directly to numpy complex arrays (implicit handling)
    reference = np.array(reference_data, dtype=complex)
    stacked = np.array([np.array(pulse, dtype=complex) for pulse in stacked_data])
    
    # Matched filter: correlate with conjugate of reference pulse
    # This is the standard pulse compression technique
    reference_conj = np.conj(reference[::-1])  # Time-reversed conjugate
    
    # Apply matched filter to each pulse
    compressed = []
    for pulse in stacked:
        # Correlation via FFT (more efficient)
        compressed_pulse = np.fft.ifft(np.fft.fft(pulse) * np.fft.fft(reference_conj, len(pulse)))
        compressed.append(compressed_pulse)
    
    compressed = np.array(compressed)
    
    print(f"RangeCompress: Performed matched filtering on {compressed.shape} data")
    
    # Return as numpy array directly (no conversion needed)
    return {
        "compressed": compressed  # Can pass numpy array directly
    }

def doppler_compress(inputs, variant_params):
    """
    Perform Doppler compression using FFT along the slow-time dimension.
    
    Args:
        inputs: Dictionary with 'data' key containing range-compressed data
        variant_params: Dictionary of variant parameters
    
    Returns:
        Dictionary with 'rd_map' (Range-Doppler map) and peak information
    """
    compressed_data = inputs.get("data", None)
    if compressed_data is None or not isinstance(compressed_data, list):
        print(f"DopplerCompress: No compressed data found or wrong format. Got: {type(compressed_data)}")
        return {"rd_map": None}
    
    # Convert directly to numpy complex array (implicit handling)
    compressed = np.array([np.array(pulse, dtype=complex) for pulse in compressed_data])
    
    # Perform FFT along slow-time (axis=0, pulse dimension)
    rd_map = np.fft.fft(compressed, axis=0)
    
    # Extract magnitude for visualization
    magnitude = np.abs(rd_map)
    
    # Find peak (target detection)
    max_val = np.max(magnitude)
    max_idx = np.unravel_index(np.argmax(magnitude), magnitude.shape)
    doppler_bin, range_bin = max_idx
    
    print(f"DopplerCompress: Created Range-Doppler map of shape {magnitude.shape}")
    print(f"DopplerCompress: Peak at Doppler bin {doppler_bin}, Range bin {range_bin}")
    print(f"DopplerCompress: Peak magnitude: {max_val:.2f}")
    
    return {
        "rd_map": magnitude,  # Can pass numpy array directly
        "peak_magnitude": float(max_val),
        "doppler_bin": int(doppler_bin),
        "range_bin": int(range_bin)
    }

def plot_lfm_pulse(inputs, variant_params):
    """
    Plotting node for LFM pulse visualization (terminal node).
    
    Args:
        inputs: Dictionary with 'pulse' key containing the LFM pulse
        variant_params: Dictionary of variant parameters
    
    Returns:
        Empty dictionary (terminal node - no outputs)
    """
    pulse_data = inputs.get("pulse", None)
    if pulse_data is None:
        return {}
    
    pulse = np.array(pulse_data, dtype=complex)
    
    print("\n📊 PlotLFM: Visualizing {} sample LFM pulse".format(len(pulse)))
    print("  ├─ Time domain: Real and imaginary components")
    print("  ├─ Frequency domain: Magnitude spectrum")
    print("  └─ Phase: Linear chirp characteristic")
    
    # Calculate some basic statistics
    max_mag = np.max(np.abs(pulse))
    print(f"  Max magnitude: {max_mag:.4f}")
    
    # No outputs - terminal visualization node
    return {}

def plot_range_compressed(inputs, variant_params):
    """
    Plotting node for range-compressed data (terminal node).
    
    Args:
        inputs: Dictionary with 'compressed' key
        variant_params: Dictionary of variant parameters
    
    Returns:
        Empty dictionary (terminal node - no outputs)
    """
    compressed_data = inputs.get("compressed", None)
    if compressed_data is None:
        return {}
    
    # Convert to complex array
    compressed = np.array([np.array(pulse, dtype=complex) for pulse in compressed_data])
    
    print("\n📊 PlotRangeCompression: Visualizing range-compressed data")
    print(f"  ├─ Total samples: {compressed.size}")
    print("  ├─ Range bins with peak detection")
    print("  └─ Compressed pulse response")
    
    # Find peak
    magnitude = np.abs(compressed)
    max_mag = np.max(magnitude)
    peak_idx = np.unravel_index(np.argmax(magnitude), magnitude.shape)
    
    print(f"  Peak at sample: {peak_idx} (magnitude: {max_mag:.2f})")
    
    # No outputs - terminal visualization node
    return {}

def plot_range_doppler_map(inputs, variant_params):
    """
    Plotting node for Range-Doppler map (terminal node).
    
    Args:
        inputs: Dictionary with range-doppler map and peak information
        variant_params: Dictionary of variant parameters
    
    Returns:
        Empty dictionary (terminal node - no outputs)
    """
    print("\n📊 PlotRangeDopplerMap: Visualizing target detection")
    
    rd_map = inputs.get("rd_map", None)
    if rd_map is not None:
        rd_array = np.array(rd_map)
        print(f"  ├─ Map dimensions: {rd_array.shape[0]} pulses × {rd_array.shape[1]} range bins")
        print("  ├─ Colormap: Magnitude heatmap")
        print("  └─ Axes: Range (fast-time) vs Doppler (slow-time)")
    
    peak = inputs.get("peak", None)
    doppler = inputs.get("doppler", None)
    range_bin = inputs.get("range", None)
    
    if peak is not None:
        print(f"  Peak magnitude: {peak:.2f}")
    if doppler is not None:
        print(f"  Doppler bin: {doppler}")
    if range_bin is not None:
        print(f"  Range bin: {range_bin}")
    
    print("  ✓ Target detected and localized")
    
    # No outputs - terminal visualization node
    return {}

def main():
    separator = "=" * 70
    print(separator)
    print("Python Radar Processing Demo with graph-sp")
    print(separator)
    print()
    
    # Create the graph
    print("Building radar processing pipeline...")
    graph = graph_sp.PyGraph()
    
    # Add LFM generator
    graph.add(
        function=lfm_generator,
        label="LFMGenerator",
        inputs=None,
        outputs=[("pulse", "lfm_pulse"), ("num_samples", "num_samples")]
    )
    
    # Add pulse stacker
    graph.add(
        function=stack_pulses,
        label="StackPulses",
        inputs=[("lfm_pulse", "pulse")],
        outputs=[
            ("stacked", "stacked_data"),
            ("num_pulses", "num_pulses"),
            ("num_samples", "num_samples")
        ]
    )
    
    # Add range compression (needs both stacked data and reference pulse)
    graph.add(
        function=range_compress,
        label="RangeCompress",
        inputs=[("stacked_data", "data"), ("lfm_pulse", "reference")],
        outputs=[("compressed", "compressed_data")]
    )
    
    # Add Doppler compression
    graph.add(
        function=doppler_compress,
        label="DopplerCompress",
        inputs=[("compressed_data", "data")],
        outputs=[
            ("rd_map", "range_doppler_map"),
            ("peak_magnitude", "peak"),
            ("doppler_bin", "doppler_idx"),
            ("range_bin", "range_idx")
        ]
    )
    
    # Add plotting nodes (terminal nodes with no outputs)
    # These visualize the data at different stages
    
    # Plot 1: LFM pulse after generation
    graph.add(
        function=plot_lfm_pulse,
        label="PlotLFMPulse",
        inputs=[("lfm_pulse", "pulse")],
        outputs=None  # No outputs - terminal visualization node
    )
    
    # Plot 2: Range-compressed data
    graph.add(
        function=plot_range_compressed,
        label="PlotRangeCompressed",
        inputs=[("compressed_data", "compressed")],
        outputs=None  # No outputs - terminal visualization node
    )
    
    # Plot 3: Range-Doppler map with target detection
    graph.add(
        function=plot_range_doppler_map,
        label="PlotRangeDopplerMap",
        inputs=[
            ("range_doppler_map", "rd_map"),
            ("peak", "peak"),
            ("doppler_idx", "doppler"),
            ("range_idx", "range")
        ],
        outputs=None  # No outputs - terminal visualization node
    )
    
    # Build and execute
    print("\nBuilding DAG...")
    dag = graph.build()
    
    print("\nMermaid diagram:")
    print(dag.to_mermaid())
    print()
    
    print("Executing radar processing pipeline...\n")
    result = dag.execute(parallel=False)
    
    print()
    print(separator)
    print("Execution Complete!")
    print(separator)
    
    # Display results
    if 'peak' in result:
        print(f"Peak magnitude: {result['peak']:.2f}")
    if 'doppler_idx' in result and 'range_idx' in result:
        print(f"Peak location: Doppler bin {result['doppler_idx']}, Range bin {result['range_idx']}")
    
    print("\nRadar demo completed successfully!")

if __name__ == "__main__":
    main()
