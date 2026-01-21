#!/usr/bin/env python3
"""
Generate individual signal processing plots matching sigexec style.
Creates separate, focused visualizations for each processing stage.
"""

import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

def generate_lfm_pulse(num_samples=256, bandwidth=100e6, duration=10e-6):
    """Generate LFM (Linear Frequency Modulation) pulse."""
    sample_rate = num_samples / duration
    chirp_rate = bandwidth / duration
    t = np.arange(num_samples) / sample_rate
    phase = 2 * np.pi * (chirp_rate / 2.0 * t * t)
    signal = np.exp(1j * phase)
    return signal, t

def matched_filter(signal, reference):
    """Apply matched filter (correlate with time-reversed conjugate)."""
    ref_conj = np.conj(reference[::-1])
    result = np.fft.ifft(np.fft.fft(signal, len(signal)) * np.fft.fft(ref_conj, len(signal)))
    return result

# Plot 1: Single LFM Pulse
pulse, t = generate_lfm_pulse()
plt.figure(figsize=(10, 4))
plt.plot(t * 1e6, np.real(pulse), 'b-', label='Real', linewidth=1.5)
plt.plot(t * 1e6, np.imag(pulse), 'r-', label='Imaginary', linewidth=1.5)
plt.xlabel('Time (μs)', fontsize=12)
plt.ylabel('Amplitude', fontsize=12)
plt.title('LFM Pulse Generation', fontsize=14, fontweight='bold')
plt.legend(fontsize=11)
plt.grid(True, alpha=0.3)
plt.tight_layout()
plt.savefig('/tmp/01_lfm_pulse.png', dpi=150, bbox_inches='tight')
print("✓ Generated 01_lfm_pulse.png")
plt.close()

# Plot 2: Pulse Stacking
num_pulses = 16
doppler_freq = 1000
prf = 10000
stacked = []
for i in range(num_pulses):
    phase_shift = 2 * np.pi * doppler_freq * i / prf
    shifted = pulse * np.exp(1j * phase_shift)
    stacked.append(shifted)
stacked = np.array(stacked)

plt.figure(figsize=(10, 6))
plt.imshow(np.abs(stacked), aspect='auto', cmap='viridis', 
           extent=[0, len(pulse), num_pulses, 0])
plt.xlabel('Fast-time (samples)', fontsize=12)
plt.ylabel('Pulse Number', fontsize=12)
plt.title(f'Pulse Stacking ({num_pulses} Pulses with Doppler)', fontsize=14, fontweight='bold')
plt.colorbar(label='Magnitude')
plt.tight_layout()
plt.savefig('/tmp/02_pulse_stacking.png', dpi=150, bbox_inches='tight')
print("✓ Generated 02_pulse_stacking.png")
plt.close()

# Plot 3: Range Compression (Matched Filter)
range_compressed = []
for p in stacked:
    compressed = matched_filter(p, pulse)
    range_compressed.append(compressed)
range_compressed = np.array(range_compressed)

plt.figure(figsize=(10, 6))
plt.imshow(np.abs(range_compressed), aspect='auto', cmap='hot',
           extent=[0, len(pulse), num_pulses, 0])
plt.xlabel('Range Bin', fontsize=12)
plt.ylabel('Pulse Number', fontsize=12)
plt.title('After Range Compression (Matched Filter)', fontsize=14, fontweight='bold')
plt.colorbar(label='Magnitude')
plt.tight_layout()
plt.savefig('/tmp/03_range_compression.png', dpi=150, bbox_inches='tight')
print("✓ Generated 03_range_compression.png")
plt.close()

# Plot 4: Range-Doppler Map
rd_map = np.fft.fftshift(np.fft.fft(range_compressed, axis=0), axes=0)
magnitude = np.abs(rd_map)

plt.figure(figsize=(10, 6))
plt.imshow(magnitude, aspect='auto', cmap='jet',
           extent=[0, len(pulse), num_pulses, 0], origin='lower')
plt.xlabel('Range Bin', fontsize=12)
plt.ylabel('Doppler Bin', fontsize=12)
plt.title('Range-Doppler Map', fontsize=14, fontweight='bold')
plt.colorbar(label='Magnitude')

# Mark peak
max_idx = np.unravel_index(np.argmax(magnitude), magnitude.shape)
doppler_bin, range_bin = max_idx
plt.plot(range_bin, doppler_bin, 'w*', markersize=20, 
         markeredgewidth=2, markeredgecolor='black')
plt.text(range_bin + 10, doppler_bin, 
         f'Peak: ({doppler_bin}, {range_bin})\nMag: {magnitude[max_idx]:.1f}',
         color='white', fontsize=11, fontweight='bold',
         bbox=dict(boxstyle='round', facecolor='black', alpha=0.8))

plt.tight_layout()
plt.savefig('/tmp/04_range_doppler_map.png', dpi=150, bbox_inches='tight')
print("✓ Generated 04_range_doppler_map.png")
plt.close()

print(f"\nResults:")
print(f"  Peak at Doppler bin {doppler_bin}, Range bin {range_bin}")
print(f"  Peak magnitude: {magnitude[max_idx]:.2f}")
