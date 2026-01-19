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
    Generate a Linear Frequency Modulation (LFM) pulse.
    
    Args:
        inputs: Dictionary of input variables (empty for source nodes)
        variant_params: Dictionary of variant parameters
    
    Returns:
        Dictionary with 'pulse' (complex array) and 'num_samples'
    """
    num_samples = 256
    bandwidth = 100e6  # 100 MHz
    duration = 10e-6   # 10 microseconds
    
    # Generate LFM pulse
    sample_rate = num_samples / duration
    chirp_rate = bandwidth / duration
    
    t = np.arange(num_samples) / sample_rate
    phase = 2 * np.pi * (chirp_rate / 2.0 * t * t)
    signal = np.exp(1j * phase)
    
    print(f"LFMGenerator: Generated {num_samples} sample LFM pulse")
    
    # Convert complex array to list of tuples (real, imag) for compatibility
    pulse_data = [(float(c.real), float(c.imag)) for c in signal]
    
    return {
        "pulse": pulse_data,
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
    num_pulses = 16  # Number of pulses to stack
    
    pulse_data = inputs.get("pulse", None)
    if not pulse_data or not isinstance(pulse_data, list):
        print(f"StackPulses: No pulse data found or wrong format. Got: {type(pulse_data)}")
        return {"stacked": None}
    
    # Convert list of tuples (real, imag) back to complex
    pulse = np.array([complex(r, i) for r, i in pulse_data])
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
        # Convert back to list of tuples
        shifted_data = [(float(c.real), float(c.imag)) for c in shifted_pulse]
        stacked.append(shifted_data)
    
    print(f"StackPulses: Stacked {num_pulses} pulses of {num_samples} samples each")
    
    return {
        "stacked": stacked,  # 2D list of tuples
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
    
    # Convert list of tuples back to complex numpy arrays
    reference = np.array([complex(r, i) for r, i in reference_data])
    stacked = np.array([[complex(r, i) for r, i in pulse] for pulse in stacked_data])
    
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
    
    # Convert back to list of lists of tuples
    compressed_data = [[(float(c.real), float(c.imag)) for c in pulse] for pulse in compressed]
    
    print(f"RangeCompress: Performed matched filtering on {compressed.shape} data")
    
    return {
        "compressed": compressed_data
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
    
    # Convert list of lists of tuples to numpy array
    compressed = np.array([[complex(r, i) for r, i in pulse] for pulse in compressed_data])
    
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
        "rd_map": magnitude.tolist(),
        "peak_magnitude": float(max_val),
        "doppler_bin": int(doppler_bin),
        "range_bin": int(range_bin)
    }

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
