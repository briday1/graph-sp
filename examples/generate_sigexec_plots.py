#!/usr/bin/env python3
"""
Generate radar signal processing plots matching sigexec style.
Creates individual plots (not 2x2 grids) for 128-pulse radar processing.
"""

import numpy as np
import matplotlib.pyplot as plt
from matplotlib import cm

# Parameters matching sigexec
num_samples = 256
num_pulses = 128
bandwidth = 100e6  # 100 MHz
duration = 10e-6   # 10 microseconds
sample_rate = num_samples / duration
chirp_rate = bandwidth / duration

# 1. Generate LFM Pulse
print("Generating LFM pulse...")
t = np.arange(num_samples) / sample_rate
phase = 2 * np.pi * (chirp_rate / 2.0 * t * t)
lfm_pulse = np.exp(1j * phase)

# Plot 1: Single LFM Pulse (Real & Imaginary)
plt.figure(figsize=(12, 5))
plt.subplot(121)
plt.plot(t * 1e6, lfm_pulse.real, 'b-', linewidth=0.8, label='Real')
plt.plot(t * 1e6, lfm_pulse.imag, 'r-', linewidth=0.8, label='Imag')
plt.xlabel('Time (μs)', fontsize=12)
plt.ylabel('Amplitude', fontsize=12)
plt.title('Single LFM Pulse (Real & Imaginary)', fontsize=14, fontweight='bold')
plt.legend()
plt.grid(True, alpha=0.3)

# Frequency domain
plt.subplot(122)
freq_spectrum = np.fft.fftshift(np.fft.fft(lfm_pulse))
freqs = np.fft.fftshift(np.fft.fftfreq(num_samples, 1/sample_rate)) / 1e6
plt.plot(freqs, np.abs(freq_spectrum), 'b-', linewidth=0.8)
plt.xlabel('Frequency (MHz)', fontsize=12)
plt.ylabel('Magnitude', fontsize=12)
plt.title('Single Pulse - Frequency Domain', fontsize=14, fontweight='bold')
plt.grid(True, alpha=0.3)

plt.tight_layout()
plt.savefig('01_lfm_pulse_sigexec.png', dpi=150, bbox_inches='tight')
print("Saved: 01_lfm_pulse_sigexec.png")
plt.close()

# 2. Stack Pulses with Doppler simulation
print(f"Stacking {num_pulses} pulses with Doppler shifts...")
doppler_shift = 1000.0  # Hz
prf = 1000.0  # Pulse Repetition Frequency (Hz)
pri = 1 / prf  # Pulse Repetition Interval

stacked_pulses = np.zeros((num_pulses, num_samples), dtype=complex)
for pulse_idx in range(num_pulses):
    doppler_phase = 2 * np.pi * doppler_shift * pulse_idx * pri
    doppler_factor = np.exp(1j * doppler_phase)
    stacked_pulses[pulse_idx, :] = lfm_pulse * doppler_factor

# Plot 2: Pulse Stacking
plt.figure(figsize=(12, 6))
plt.imshow(np.abs(stacked_pulses), aspect='auto', cmap='viridis',
           extent=[0, num_samples, num_pulses, 0])
plt.colorbar(label='Magnitude')
plt.xlabel('Fast-time (Range samples)', fontsize=12)
plt.ylabel('Slow-time (Pulse #)', fontsize=12)
plt.title(f'Pulse Stacking ({num_pulses} Pulses with Doppler)', fontsize=14, fontweight='bold')
plt.tight_layout()
plt.savefig('02_pulse_stacking_sigexec.png', dpi=150, bbox_inches='tight')
print("Saved: 02_pulse_stacking_sigexec.png")
plt.close()

# 3. Range Compression (Matched Filter)
print("Performing range compression via matched filter...")
# Matched filter is conjugate of time-reversed reference
matched_filter = np.conj(lfm_pulse[::-1])

# Apply matched filter to each pulse
range_compressed = np.zeros_like(stacked_pulses)
for pulse_idx in range(num_pulses):
    # FFT-based correlation for efficiency
    range_compressed[pulse_idx, :] = np.fft.ifft(
        np.fft.fft(stacked_pulses[pulse_idx, :]) * np.fft.fft(matched_filter, num_samples)
    )

# Plot 3: After Range Compression
plt.figure(figsize=(12, 6))
plt.imshow(np.abs(range_compressed), aspect='auto', cmap='hot',
           extent=[0, num_samples, num_pulses, 0])
plt.colorbar(label='Magnitude')
plt.xlabel('Range Bin', fontsize=12)
plt.ylabel('Pulse #', fontsize=12)
plt.title('After Range Compression (Matched Filter)', fontsize=14, fontweight='bold')
plt.tight_layout()
plt.savefig('03_range_compression_sigexec.png', dpi=150, bbox_inches='tight')
print("Saved: 03_range_compression_sigexec.png")
plt.close()

# 4. Doppler Compression (FFT along slow-time)
print("Performing Doppler compression...")
rd_map = np.fft.fftshift(np.fft.fft(range_compressed, axis=0), axes=0)

# Find peak
peak_idx = np.unravel_index(np.argmax(np.abs(rd_map)), rd_map.shape)
peak_mag = np.abs(rd_map[peak_idx])
print(f"Peak detected at Doppler bin {peak_idx[0]}, Range bin {peak_idx[1]}")
print(f"Peak magnitude: {peak_mag:.2f}")

# Plot 4: Range-Doppler Map
plt.figure(figsize=(12, 6))
rd_map_db = 20 * np.log10(np.abs(rd_map) + 1e-10)
plt.imshow(rd_map_db, aspect='auto', cmap='jet',
           extent=[0, num_samples, num_pulses, 0], origin='lower')
plt.colorbar(label='Magnitude (dB)')
plt.xlabel('Range Bin', fontsize=12)
plt.ylabel('Doppler Bin', fontsize=12)
plt.title('Range-Doppler Map (128 Pulses)', fontsize=14, fontweight='bold')

# Mark peak
plt.plot(peak_idx[1], peak_idx[0], 'w*', markersize=15, markeredgecolor='black', markeredgewidth=1.5)
plt.text(peak_idx[1] + 10, peak_idx[0], f'Peak\n({peak_idx[0]}, {peak_idx[1]})\nMag: {peak_mag:.1f}',
         color='white', fontsize=10, fontweight='bold',
         bbox=dict(boxstyle='round', facecolor='black', alpha=0.7))

plt.tight_layout()
plt.savefig('04_range_doppler_map_sigexec.png', dpi=150, bbox_inches='tight')
print("Saved: 04_range_doppler_map_sigexec.png")
plt.close()

print("\nAll sigexec-style plots generated successfully!")
print(f"Pipeline: {num_samples} samples/pulse × {num_pulses} pulses")
print(f"Target detected at Doppler bin {peak_idx[0]}, Range bin {peak_idx[1]}, Magnitude {peak_mag:.2f}")
