#!/usr/bin/env python3
"""
Generate signal processing plots from Python radar demo output.
This script visualizes the radar processing chain for the Python implementation.
"""

import numpy as np
import matplotlib.pyplot as plt

def generate_lfm_pulse():
    """Generate LFM pulse matching Python implementation"""
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

def stack_pulses_python(pulse, num_pulses=128):
    """Stack pulses with Doppler matching Python implementation"""
    num_samples = len(pulse)
    stacked = []
    doppler_freq = 1000  # Hz
    prf = 10000  # Pulse Repetition Frequency (Hz)
    
    for pulse_idx in range(num_pulses):
        # Add Doppler shift
        phase_shift = 2 * np.pi * doppler_freq * pulse_idx / prf
        shifted_pulse = pulse * np.exp(1j * phase_shift)
        stacked.append(shifted_pulse)
    
    return np.array(stacked)

def range_compress_python(stacked, reference):
    """Range compression matching Python implementation"""
    # Matched filter: correlate with conjugate of time-reversed reference
    reference_matched = np.conj(reference[::-1])
    
    # Apply matched filter to each pulse using linear correlation
    compressed = []
    for pulse in stacked:
        # Use 'same' mode to keep same length and center the output
        compressed_pulse = np.correlate(pulse, reference_matched, mode='same')
        compressed.append(compressed_pulse)
    
    return np.array(compressed)

def doppler_compress_python(compressed):
    """Doppler compression matching Python implementation"""
    # Perform FFT along slow-time (axis=0)
    rd_map = np.fft.fft(compressed, axis=0)
    return rd_map

# Generate data
print("Generating Python radar processing data...")
pulse = generate_lfm_pulse()
stacked = stack_pulses_python(pulse)
compressed = range_compress_python(stacked, pulse)
rd_map = doppler_compress_python(compressed)

# Create plots
print("Creating Python radar plots...")

# Plot 1: LFM Pulse
fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 4))
t_us = np.arange(len(pulse)) / 100  # Time in microseconds
ax1.plot(t_us, pulse.real, 'b-', label='Real', linewidth=1)
ax1.plot(t_us, pulse.imag, 'r-', label='Imag', linewidth=1)
ax1.set_xlabel('Time (μs)')
ax1.set_ylabel('Amplitude')
ax1.set_title('Python: LFM Pulse (Time Domain)')
ax1.legend()
ax1.grid(True, alpha=0.3)

freq = np.fft.fftshift(np.fft.fftfreq(len(pulse), 1/100e6)) / 1e6
spectrum = np.fft.fftshift(np.abs(np.fft.fft(pulse)))
ax2.plot(freq, spectrum, 'b-', linewidth=1)
ax2.set_xlabel('Frequency (MHz)')
ax2.set_ylabel('Magnitude')
ax2.set_title('Python: LFM Pulse (Frequency Domain)')
ax2.grid(True, alpha=0.3)

plt.tight_layout()
plt.savefig('python_01_lfm_pulse.png', dpi=150, bbox_inches='tight')
print("Saved: python_01_lfm_pulse.png")
plt.close()

# Plot 2: Pulse Stacking (Real part to show actual signal)
fig, ax = plt.subplots(1, 1, figsize=(10, 6))
im = ax.imshow(stacked.real, aspect='auto', cmap='RdBu', interpolation='nearest', vmin=-1, vmax=1)
ax.set_xlabel('Fast-time (Range samples)')
ax.set_ylabel('Slow-time (Pulse #)')
ax.set_title('Python: 128 Stacked Pulses with Doppler (Real Part)')
plt.colorbar(im, ax=ax, label='Amplitude')
plt.tight_layout()
plt.savefig('python_02_pulse_stacking.png', dpi=150, bbox_inches='tight')
print("Saved: python_02_pulse_stacking.png")
plt.close()

# Plot 3: Range Compression
fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(12, 8))

# Before compression (all pulses)
ax1.imshow(np.abs(stacked), aspect='auto', cmap='viridis', interpolation='nearest')
ax1.set_xlabel('Fast-time (Range samples)')
ax1.set_ylabel('Pulse #')
ax1.set_title('Python: Before Range Compression')

# After compression
ax2.imshow(np.abs(compressed), aspect='auto', cmap='hot', interpolation='nearest')
ax2.set_xlabel('Range Bin')
ax2.set_ylabel('Pulse #')
ax2.set_title('Python: After Range Compression')

plt.tight_layout()
plt.savefig('python_03_range_compression.png', dpi=150, bbox_inches='tight')
print("Saved: python_03_range_compression.png")
plt.close()

# Plot 4: Range-Doppler Map
fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 6))

# Linear scale
magnitude = np.abs(rd_map)
im1 = ax1.imshow(magnitude, aspect='auto', cmap='jet', interpolation='nearest')
ax1.set_xlabel('Range Bin')
ax1.set_ylabel('Doppler Bin')
ax1.set_title('Python: Range-Doppler Map (Linear)')
plt.colorbar(im1, ax=ax1, label='Magnitude')

# Mark peak
max_val = magnitude.max()
max_idx = np.unravel_index(np.argmax(magnitude), magnitude.shape)
ax1.plot(max_idx[1], max_idx[0], 'w+', markersize=10, markeredgewidth=1.5)
ax1.text(max_idx[1] - 15, max_idx[0], f'Peak @ Dop={max_idx[0]}, Rng={max_idx[1]}', 
         color='white', ha='right', fontsize=9, fontweight='bold',
         bbox=dict(boxstyle='round', facecolor='black', alpha=0.7))

# dB scale
magnitude_db = 20 * np.log10(magnitude + 1e-10)
im2 = ax2.imshow(magnitude_db, aspect='auto', cmap='jet', interpolation='nearest')
ax2.set_xlabel('Range Bin')
ax2.set_ylabel('Doppler Bin')
ax2.set_title('Python: Range-Doppler Map (dB scale)')
plt.colorbar(im2, ax=ax2, label='Magnitude (dB)')

# Mark peak
ax2.plot(max_idx[1], max_idx[0], 'w+', markersize=10, markeredgewidth=1.5)

plt.tight_layout()
plt.savefig('python_04_range_doppler_map.png', dpi=150, bbox_inches='tight')
print("Saved: python_04_range_doppler_map.png")
plt.close()

print("\nPython plots generated successfully!")
print(f"Peak at Doppler bin {max_idx[0]}, Range bin {max_idx[1]}")
print(f"Peak magnitude: {max_val:.2f}")
print("\nGenerated files:")
print("  - python_01_lfm_pulse.png")
print("  - python_02_pulse_stacking.png")
print("  - python_03_range_compression.png")
print("  - python_04_range_doppler_map.png")
