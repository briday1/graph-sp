#!/usr/bin/env python3
"""
Generate signal processing plots from Rust radar demo output.
This script visualizes the radar processing chain for the Rust implementation.
"""

import numpy as np
import matplotlib.pyplot as plt
import subprocess
import json

# Run Rust demo and capture intermediate data
# For now, we'll generate example data matching the Rust implementation

def generate_lfm_pulse():
    """Generate LFM pulse matching Rust implementation"""
    num_samples = 256
    bandwidth = 100e6  # 100 MHz
    pulse_width = 2e-6  # 2 microseconds
    sample_rate = 100e6  # 100 MHz sample rate
    
    # Generate LFM chirp
    chirp_rate = bandwidth / pulse_width
    
    # Create rectangular pulse envelope
    pulse_envelope = np.zeros(num_samples)
    pulse_start = int(num_samples * 0.2)  # Start at 20%
    pulse_end = pulse_start + int(pulse_width * sample_rate)  # Pulse duration
    pulse_envelope[pulse_start:pulse_end] = 1.0
    
    # Generate chirp phase (only within pulse)
    phase = np.zeros(num_samples)
    for i in range(pulse_start, min(pulse_end, num_samples)):
        t_pulse = (i - pulse_start) / sample_rate
        phase[i] = 2 * np.pi * (chirp_rate / 2.0 * t_pulse * t_pulse)
    
    # Apply envelope to create pulsed LFM signal
    signal = pulse_envelope * np.exp(1j * phase)
    
    return signal

def stack_pulses_rust(pulse, num_pulses=128):
    """Stack pulses matching Rust implementation"""
    num_samples = len(pulse)
    stacked = np.tile(pulse, num_pulses)
    return stacked.reshape(num_pulses, num_samples)

def range_compress_rust(stacked, reference):
    """Range compression matching Rust implementation"""
    # Flatten stacked for processing
    stacked_flat = stacked.flatten()
    
    # Matched filter: correlate with conjugate of reference pulse
    reference_conj = np.conj(reference[::-1])  # Time-reversed conjugate
    
    # Pad to match stacked length
    ref_padded = np.pad(reference_conj, (0, len(stacked_flat) - len(reference_conj)), 'constant')
    
    # FFT-based correlation
    signal_fft = np.fft.fft(stacked_flat)
    ref_fft = np.fft.fft(ref_padded)
    compressed = np.fft.ifft(signal_fft * ref_fft)
    
    return compressed

# Generate data
print("Generating Rust radar processing data...")
pulse = generate_lfm_pulse()
stacked = stack_pulses_rust(pulse)
compressed = range_compress_rust(stacked, pulse)

# Create plots
print("Creating Rust radar plots...")

# Plot 1: LFM Pulse
fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 4))
t_us = np.arange(len(pulse)) / 100  # Time in microseconds
ax1.plot(t_us, pulse.real, 'b-', label='Real', linewidth=1)
ax1.plot(t_us, pulse.imag, 'r-', label='Imag', linewidth=1)
ax1.set_xlabel('Time (μs)')
ax1.set_ylabel('Amplitude')
ax1.set_title('Rust: LFM Pulse (Time Domain)')
ax1.legend()
ax1.grid(True, alpha=0.3)

freq = np.fft.fftshift(np.fft.fftfreq(len(pulse), 1/100e6)) / 1e6
spectrum = np.fft.fftshift(np.abs(np.fft.fft(pulse)))
ax2.plot(freq, spectrum, 'b-', linewidth=1)
ax2.set_xlabel('Frequency (MHz)')
ax2.set_ylabel('Magnitude')
ax2.set_title('Rust: LFM Pulse (Frequency Domain)')
ax2.grid(True, alpha=0.3)

plt.tight_layout()
plt.savefig('rust_01_lfm_pulse.png', dpi=150, bbox_inches='tight')
print("Saved: rust_01_lfm_pulse.png")
plt.close()

# Plot 2: Pulse Stacking (Real part to show actual signal)
fig, ax = plt.subplots(1, 1, figsize=(10, 6))
im = ax.imshow(stacked.real, aspect='auto', cmap='RdBu', interpolation='nearest', vmin=-1, vmax=1)
ax.set_xlabel('Fast-time (Range samples)')
ax.set_ylabel('Slow-time (Pulse #)')
ax.set_title('Rust: 128 Stacked Pulses (Real Part)')
plt.colorbar(im, ax=ax, label='Amplitude')
plt.tight_layout()
plt.savefig('rust_02_pulse_stacking.png', dpi=150, bbox_inches='tight')
print("Saved: rust_02_pulse_stacking.png")
plt.close()

# Plot 3: Range Compression
fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 8))

# Before compression (first pulse)
ax1.plot(np.abs(stacked.flatten()), 'b-', linewidth=0.5)
ax1.set_xlabel('Sample Index')
ax1.set_ylabel('Magnitude')
ax1.set_title('Rust: Before Range Compression')
ax1.grid(True, alpha=0.3)

# After compression
compressed_mag = np.abs(compressed)
ax2.plot(compressed_mag, 'r-', linewidth=0.5)
ax2.set_xlabel('Range Bin')
ax2.set_ylabel('Magnitude')
ax2.set_title(f'Rust: After Range Compression (Peak: {compressed_mag.max():.2f})')
ax2.grid(True, alpha=0.3)

# Mark peak
peak_idx = np.argmax(compressed_mag)
ax2.axvline(peak_idx, color='g', linestyle='--', alpha=0.7, label=f'Peak @ {peak_idx}')
ax2.legend()

plt.tight_layout()
plt.savefig('rust_03_range_compression.png', dpi=150, bbox_inches='tight')
print("Saved: rust_03_range_compression.png")
plt.close()

print("\nRust plots generated successfully!")
print("  - rust_01_lfm_pulse.png")
print("  - rust_02_pulse_stacking.png")
print("  - rust_03_range_compression.png")
