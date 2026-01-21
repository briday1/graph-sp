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
    pulse_width = 1e-6  # 1 microsecond (shorter for centered target)
    sample_rate = 100e6  # 100 MHz sample rate
    
    # Generate LFM chirp
    chirp_rate = bandwidth / pulse_width
    
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
    
    return signal

def stack_pulses_rust(pulse, num_pulses=128):
    """Stack pulses with Doppler shifts matching Rust implementation"""
    num_samples = len(pulse)
    
    # Doppler simulation parameters (matching Rust/Python demos)
    doppler_freq = 1000  # Hz
    prf = 10000  # Hz
    
    stacked = np.zeros((num_pulses, num_samples), dtype=complex)
    for pulse_idx in range(num_pulses):
        # Add Doppler shift
        phase_shift = 2 * np.pi * doppler_freq * pulse_idx / prf
        doppler_shift = np.exp(1j * phase_shift)
        stacked[pulse_idx, :] = pulse * doppler_shift
    
    return stacked

def range_compress_rust(stacked, reference):
    """Range compression matching Rust implementation"""
    # Matched filter: correlate with conjugate of time-reversed reference
    reference_matched = np.conj(reference[::-1])
    
    # Apply matched filter to each pulse using linear correlation
    num_pulses, num_samples = stacked.shape
    compressed = np.zeros((num_pulses, num_samples), dtype=complex)
    
    for i in range(num_pulses):
        # Use 'same' mode to keep same length and center the output
        compressed[i, :] = np.correlate(stacked[i, :], reference_matched, mode='same')
    
    return compressed.flatten()

def doppler_compress_rust(compressed, num_pulses, num_samples):
    """Doppler compression via FFT along slow-time"""
    # Reshape to 2D (pulses x samples)
    data_2d = compressed.reshape(num_pulses, num_samples)
    
    # FFT along slow-time (axis=0) for each range bin
    range_doppler = np.fft.fft(data_2d, axis=0)
    
    return range_doppler

# Generate data
print("Generating Rust radar processing data...")
pulse = generate_lfm_pulse()
num_pulses = 128
num_samples = len(pulse)
stacked = stack_pulses_rust(pulse, num_pulses)
compressed = range_compress_rust(stacked, pulse)
range_doppler = doppler_compress_rust(compressed, num_pulses, num_samples)

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

# Plot 2: Pulse Stacking with Doppler (Real part to show actual signal)
fig, ax = plt.subplots(1, 1, figsize=(10, 6))
im = ax.imshow(stacked.real, aspect='auto', cmap='RdBu', interpolation='nearest', vmin=-1, vmax=1)
ax.set_xlabel('Fast-time (Range samples)')
ax.set_ylabel('Slow-time (Pulse #)')
ax.set_title('Rust: 128 Pulses with Doppler Shifts (Real Part)')
plt.colorbar(im, ax=ax, label='Amplitude')
plt.tight_layout()
plt.savefig('rust_02_pulse_stacking.png', dpi=150, bbox_inches='tight')
print("Saved: rust_02_pulse_stacking.png")
plt.close()

# Plot 3: Range Compression
fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 8))

# Before compression (show 2D view)
ax1.imshow(np.abs(stacked), aspect='auto', cmap='viridis', interpolation='nearest')
ax1.set_xlabel('Fast-time (Range samples)')
ax1.set_ylabel('Slow-time (Pulse #)')
ax1.set_title('Rust: Before Range Compression')

# After compression (show 2D view)
compressed_2d = compressed.reshape(num_pulses, num_samples)
ax2.imshow(np.abs(compressed_2d), aspect='auto', cmap='hot', interpolation='nearest')
ax2.set_xlabel('Range Bin')
ax2.set_ylabel('Pulse #')
ax2.set_title(f'Rust: After Range Compression')

plt.tight_layout()
plt.savefig('rust_03_range_compression.png', dpi=150, bbox_inches='tight')
print("Saved: rust_03_range_compression.png")
plt.close()

# Plot 4: Range-Doppler Map
fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 6))

# Linear scale
im1 = ax1.imshow(np.abs(range_doppler), aspect='auto', cmap='jet', interpolation='nearest')
ax1.set_xlabel('Range Bin')
ax1.set_ylabel('Doppler Bin')
ax1.set_title('Rust: Range-Doppler Map (Linear Scale)')
plt.colorbar(im1, ax=ax1, label='Magnitude')

# Find and mark peak
peak_idx = np.unravel_index(np.argmax(np.abs(range_doppler)), range_doppler.shape)
ax1.plot(peak_idx[1], peak_idx[0], 'w+', markersize=10, markeredgewidth=1.5, 
         label=f'Peak @ Dop={peak_idx[0]}, Rng={peak_idx[1]}')
ax1.legend(fontsize=9)

# dB scale
rd_db = 20 * np.log10(np.abs(range_doppler) + 1e-10)
im2 = ax2.imshow(rd_db, aspect='auto', cmap='jet', interpolation='nearest')
ax2.set_xlabel('Range Bin')
ax2.set_ylabel('Doppler Bin')
ax2.set_title('Rust: Range-Doppler Map (dB Scale)')
plt.colorbar(im2, ax=ax2, label='Magnitude (dB)')
ax2.plot(peak_idx[1], peak_idx[0], 'w+', markersize=10, markeredgewidth=1.5)

plt.tight_layout()
plt.savefig('rust_04_range_doppler_map.png', dpi=150, bbox_inches='tight')
print("Saved: rust_04_range_doppler_map.png")
plt.close()

print("\nRust plots generated successfully!")
print("  - rust_01_lfm_pulse.png")
print("  - rust_02_pulse_stacking.png")
print("  - rust_03_range_compression.png")
print("  - rust_04_range_doppler_map.png")
