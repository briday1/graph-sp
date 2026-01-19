#!/usr/bin/env python3
"""
Generate visual representations of the mermaid diagrams as text-based images.
Creates ASCII art representations that can be viewed in terminals and markdown.
"""

def create_rust_data_types_diagram():
    return """
Rust Data Types Demo - Pipeline Flow
=====================================

    ┌─────────────────┐
    │  DataGenerator  │  Creates: Int, Float, String, Vecs, Maps
    └────────┬────────┘
             │ 7 outputs: int_val, float_val, str_val, 
             │            int_list, float_list, meta, custom
             ▼
    ┌─────────────────┐
    │  TypeInspector  │  Analyzes and reports all received types
    └────────┬────────┘
             │ Same 7 values passed through
             │ (inspected_int, inspected_float, etc.)
             ▼
    ┌─────────────────┐
    │  DataProcessor  │  Processes each type appropriately
    └────────┬────────┘
             │ 7 results: integer_doubled, float_squared,
             │            string_upper, list_sum, list_avg,
             │            meta_summary, sensor_avg
             ▼
    ┌─────────────────┐
    │ResultAggregator │  Summarizes: 7 total, 5 numeric, 2 string
    └─────────────────┘

Key Features:
• Demonstrates ALL GraphData types
• Shows type inspection capabilities  
• Processes diverse data appropriately
• Aggregates results with statistics
"""

def create_python_data_types_diagram():
    return """
Python Data Types Demo - Pipeline Flow
=======================================

    ┌─────────────────┐
    │  DataGenerator  │  Creates: int, float, str, lists, dicts,
    └────────┬────────┘           complex (as tuples), custom objects
             │ 9 outputs: int_val, float_val, str_val, int_list,
             │            float_list, complex_val, complex_list,
             │            meta, custom
             ▼
    ┌─────────────────┐
    │  TypeInspector  │  Analyzes Python types including nested dicts
    └────────┬────────┘
             │ 7 inspected values passed through
             │ (excludes some intermediate values)
             ▼
    ┌─────────────────┐
    │  DataProcessor  │  Processes including complex magnitude
    └────────┬────────┘
             │ 8 results: integer_doubled, float_squared,
             │            string_upper, list_sum, list_avg,
             │            complex_magnitude, meta_summary, sensor_avg
             ▼
    ┌─────────────────┐
    │ResultAggregator │  Summarizes: 8 total, 6 numeric, 2 string
    └─────────────────┘

Key Features:
• Includes complex number handling (tuples)
• Demonstrates nested dict structures
• Shows custom object serialization
• Python-specific type processing
"""

def create_rust_radar_diagram():
    return """
Rust Radar Processing Demo - Pipeline Flow
===========================================

    ┌─────────────────┐
    │  LFMGenerator   │  Generates 256-sample LFM chirp
    └────────┬────────┘  Using: ndarray + complex math
             │ Output: Complex array (pulse)
             ▼
    ┌─────────────────┐
    │   StackPulses   │  Stacks 4 identical pulses
    └────────┬────────┘  Total: 1024 samples
             │ Output: Stacked complex array
             ▼
    ┌─────────────────┐
    │ RangeCompress   │  FFT-based compression
    └────────┬────────┘  Using: rustfft
             │ Output: Frequency-domain data
             ▼
    ┌─────────────────┐
    │MagnitudeExtract │  Peak detection
    └─────────────────┘  Result: Magnitude 101.61 @ index 828

Signal Processing Steps:
1. Generate: LFM chirp (bandwidth 100MHz, duration 10μs)
2. Stack:    Simulate multiple transmissions
3. Compress: FFT reveals target range
4. Detect:   Find peak magnitude location

DAG Statistics:
• Nodes: 4
• Depth: 4 levels (sequential)
• Max Parallelism: 1 node
"""

def create_python_radar_diagram():
    return """
Python Radar Processing Demo - Pipeline Flow
=============================================

    ┌─────────────────┐
    │  LFMGenerator   │  Generates 256-sample LFM pulse
    └────────┬────────┘  Using: numpy
             │ Output: Complex array (as tuples)
             ▼
    ┌─────────────────┐
    │   StackPulses   │  Stacks 16 pulses with Doppler shifts
    └────────┬────────┘  Simulates target velocity (1000 Hz)
             │ Output: 16×256 matrix (4096 samples)
             ▼
    ┌─────────────────┐
    │ RangeCompress   │  FFT along fast-time (range)
    └────────┬────────┘  Using: numpy.fft
             │ Output: Range-compressed matrix
             ▼
    ┌─────────────────┐
    │DopplerCompress  │  FFT along slow-time (Doppler)
    └─────────────────┘  Result: Range-Doppler map (16×256)
                         Peak @ Doppler bin 2, Range bin 207
                         Magnitude: 307.93

Signal Processing Steps:
1. Generate:  LFM chirp pulse
2. Stack:     16 pulses with phase shifts (Doppler simulation)
3. Range:     FFT → target range information
4. Doppler:   FFT → target velocity information
5. Detect:    Peak finding in Range-Doppler map

Range-Doppler Map:
┌────────────────────────────────┐
│  D  ░░░░░░░░░░░░░░░░░░░░░░░░  │  D = Doppler (velocity)
│  o  ░░░░░░░░░░░░░░░░░░░░░░░░  │  R = Range (distance)
│  p  ░░░█░░░░░░░░░░░░░░░░░░░░  │  █ = Target (bright spot)
│  p  ░░░░░░░░░░░░░░░░░░░░░░░░  │
│  l  ░░░░░░░░░░░░░░░░░░░░░░░░  │  Peak: (Doppler=2, Range=207)
│  e  ░░░░░░░░░░░░░░░░░░░░░░░░  │  Magnitude: 307.93
│  r  ────────────────────────  │
│        Range bins (0-255) →   │
└────────────────────────────────┘
"""

def main():
    print("=" * 70)
    print("GraphData API - ASCII Visualizations")
    print("=" * 70)
    print()
    
    diagrams = {
        "Rust Data Types": create_rust_data_types_diagram(),
        "Python Data Types": create_python_data_types_diagram(),
        "Rust Radar Processing": create_rust_radar_diagram(),
        "Python Radar Processing": create_python_radar_diagram(),
    }
    
    for title, diagram in diagrams.items():
        print(diagram)
        print()

if __name__ == "__main__":
    main()
