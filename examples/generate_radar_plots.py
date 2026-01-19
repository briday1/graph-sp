#!/usr/bin/env python3
"""
Generate signal processing plots for radar demos.
Creates visualizations of pulse stacking, range compression, and Range-Doppler maps.
"""

import numpy as np
import matplotlib
matplotlib.use('Agg')  # Use non-interactive backend
import matplotlib.pyplot as plt
from pathlib import Path

def generate_lfm_pulse(num_samples=256, bandwidth=100e6, duration=10e-6):
    """Generate LFM (Linear Frequency Modulation) pulse."""
    sample_rate = num_samples / duration
    chirp_rate = bandwidth / duration
    
    t = np.arange(num_samples) / sample_rate
    phase = 2 * np.pi * (chirp_rate / 2.0 * t * t)
    signal = np.exp(1j * phase)
    
    return signal, t

def plot_pulse_stacking():
    """Create visualization of pulse stacking."""
    fig, axes = plt.subplots(2, 2, figsize=(12, 10))
    
    # Generate LFM pulse
    pulse, t = generate_lfm_pulse()
    
    # Plot 1: Single LFM pulse - Time domain
    ax = axes[0, 0]
    ax.plot(t * 1e6, np.real(pulse), 'b-', linewidth=1)
    ax.plot(t * 1e6, np.imag(pulse), 'r-', linewidth=1)
    ax.set_xlabel('Time (μs)')
    ax.set_ylabel('Amplitude')
    ax.set_title('Single LFM Pulse (Real & Imaginary)')
    ax.legend(['Real', 'Imag'])
    ax.grid(True, alpha=0.3)
    
    # Plot 2: LFM pulse - Frequency domain
    ax = axes[0, 1]
    fft_pulse = np.fft.fft(pulse)
    freq = np.fft.fftfreq(len(pulse), t[1] - t[0])
    ax.plot(np.fft.fftshift(freq) / 1e6, np.fft.fftshift(np.abs(fft_pulse)))
    ax.set_xlabel('Frequency (MHz)')
    ax.set_ylabel('Magnitude')
    ax.set_title('Single Pulse - Frequency Domain')
    ax.grid(True, alpha=0.3)
    
    # Plot 3: Stacked pulses (simulate 4 pulses)
    ax = axes[1, 0]
    num_pulses = 4
    stacked = np.tile(pulse, num_pulses)
    t_stacked = np.arange(len(stacked)) * (t[1] - t[0])
    ax.plot(t_stacked * 1e6, np.abs(stacked), 'purple', linewidth=0.5)
    ax.set_xlabel('Time (μs)')
    ax.set_ylabel('Magnitude')
    ax.set_title(f'{num_pulses} Stacked Pulses (1024 samples)')
    ax.grid(True, alpha=0.3)
    
    # Plot 4: Stacked pulses - 2D representation
    ax = axes[1, 1]
    stacked_2d = np.tile(pulse, (num_pulses, 1))
    im = ax.imshow(np.abs(stacked_2d), aspect='auto', cmap='viridis', 
                   extent=[0, len(pulse), num_pulses, 0])
    ax.set_xlabel('Fast-time (samples)')
    ax.set_ylabel('Pulse Number')
    ax.set_title('Stacked Pulses (2D View)')
    plt.colorbar(im, ax=ax, label='Magnitude')
    
    plt.tight_layout()
    plt.savefig('/tmp/pulse_stacking.png', dpi=150, bbox_inches='tight')
    print("✓ Generated pulse_stacking.png")
    plt.close()

def plot_range_compression():
    """Create visualization of range compression using FFT."""
    fig, axes = plt.subplots(2, 2, figsize=(12, 10))
    
    # Generate and stack pulses
    pulse, t = generate_lfm_pulse()
    num_pulses = 4
    stacked = np.tile(pulse, num_pulses)
    
    # Plot 1: Time domain signal
    ax = axes[0, 0]
    ax.plot(np.abs(stacked), 'b-', linewidth=0.5)
    ax.set_xlabel('Sample Index')
    ax.set_ylabel('Magnitude')
    ax.set_title('Before Range Compression (Time Domain)')
    ax.grid(True, alpha=0.3)
    
    # Plot 2: After FFT (Range Compression)
    ax = axes[0, 1]
    compressed = np.fft.fft(stacked)
    ax.plot(np.abs(compressed), 'r-', linewidth=0.5)
    ax.set_xlabel('Range Bin')
    ax.set_ylabel('Magnitude')
    ax.set_title('After Range Compression (FFT)')
    ax.grid(True, alpha=0.3)
    
    # Find peak
    peak_idx = np.argmax(np.abs(compressed))
    peak_val = np.abs(compressed[peak_idx])
    ax.axvline(peak_idx, color='green', linestyle='--', alpha=0.7, label=f'Peak @ {peak_idx}')
    ax.legend()
    
    # Plot 3: Zoom on peak
    ax = axes[1, 0]
    zoom_range = 100
    start = max(0, peak_idx - zoom_range)
    end = min(len(compressed), peak_idx + zoom_range)
    ax.plot(range(start, end), np.abs(compressed[start:end]), 'r-', linewidth=1)
    ax.axvline(peak_idx, color='green', linestyle='--', alpha=0.7)
    ax.set_xlabel('Range Bin')
    ax.set_ylabel('Magnitude')
    ax.set_title(f'Peak Detail (Magnitude: {peak_val:.2f})')
    ax.grid(True, alpha=0.3)
    
    # Plot 4: Phase information
    ax = axes[1, 1]
    ax.plot(np.angle(compressed), 'purple', linewidth=0.5, alpha=0.7)
    ax.set_xlabel('Range Bin')
    ax.set_ylabel('Phase (radians)')
    ax.set_title('Phase After Range Compression')
    ax.grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.savefig('/tmp/range_compression.png', dpi=150, bbox_inches='tight')
    print("✓ Generated range_compression.png")
    plt.close()

def plot_range_doppler_map():
    """Create Range-Doppler map visualization."""
    fig, axes = plt.subplots(2, 2, figsize=(14, 12))
    
    # Generate pulse with Doppler
    pulse, t = generate_lfm_pulse()
    num_pulses = 16
    doppler_freq = 1000  # Hz
    prf = 10000  # Pulse Repetition Frequency
    
    # Stack pulses with Doppler shift
    stacked = []
    for i in range(num_pulses):
        phase_shift = 2 * np.pi * doppler_freq * i / prf
        shifted_pulse = pulse * np.exp(1j * phase_shift)
        stacked.append(shifted_pulse)
    
    stacked = np.array(stacked)
    
    # Plot 1: Stacked pulses (slow-time, fast-time)
    ax = axes[0, 0]
    im = ax.imshow(np.abs(stacked), aspect='auto', cmap='viridis',
                   extent=[0, stacked.shape[1], num_pulses, 0])
    ax.set_xlabel('Fast-time (Range samples)')
    ax.set_ylabel('Slow-time (Pulse #)')
    ax.set_title('Pulse Stack Before Processing')
    plt.colorbar(im, ax=ax, label='Magnitude')
    
    # Range compression (FFT along fast-time)
    range_compressed = np.fft.fft(stacked, axis=1)
    
    # Plot 2: After Range Compression
    ax = axes[0, 1]
    im = ax.imshow(np.abs(range_compressed), aspect='auto', cmap='hot',
                   extent=[0, range_compressed.shape[1], num_pulses, 0])
    ax.set_xlabel('Range Bin')
    ax.set_ylabel('Pulse #')
    ax.set_title('After Range Compression')
    plt.colorbar(im, ax=ax, label='Magnitude')
    
    # Doppler compression (FFT along slow-time)
    rd_map = np.fft.fft(range_compressed, axis=0)
    magnitude = np.abs(rd_map)
    
    # Plot 3: Range-Doppler Map
    ax = axes[1, 0]
    im = ax.imshow(magnitude, aspect='auto', cmap='jet',
                   extent=[0, rd_map.shape[1], num_pulses, 0])
    ax.set_xlabel('Range Bin')
    ax.set_ylabel('Doppler Bin')
    ax.set_title('Range-Doppler Map')
    plt.colorbar(im, ax=ax, label='Magnitude')
    
    # Mark peak
    max_idx = np.unravel_index(np.argmax(magnitude), magnitude.shape)
    doppler_bin, range_bin = max_idx
    ax.plot(range_bin, doppler_bin, 'w*', markersize=15, 
            markeredgewidth=2, markeredgecolor='black')
    ax.text(range_bin + 10, doppler_bin, 
            f'Peak\n({doppler_bin}, {range_bin})\nMag: {magnitude[max_idx]:.1f}',
            color='white', fontsize=9, 
            bbox=dict(boxstyle='round', facecolor='black', alpha=0.7))
    
    # Plot 4: Range-Doppler (log scale for better visualization)
    ax = axes[1, 1]
    magnitude_db = 20 * np.log10(magnitude + 1e-10)
    im = ax.imshow(magnitude_db, aspect='auto', cmap='jet',
                   extent=[0, rd_map.shape[1], num_pulses, 0])
    ax.set_xlabel('Range Bin')
    ax.set_ylabel('Doppler Bin')
    ax.set_title('Range-Doppler Map (dB scale)')
    plt.colorbar(im, ax=ax, label='Magnitude (dB)')
    ax.plot(range_bin, doppler_bin, 'w*', markersize=15, 
            markeredgewidth=2, markeredgecolor='black')
    
    plt.tight_layout()
    plt.savefig('/tmp/range_doppler_map.png', dpi=150, bbox_inches='tight')
    print("✓ Generated range_doppler_map.png")
    plt.close()
    
    return doppler_bin, range_bin, magnitude[max_idx]

def main():
    print("=" * 60)
    print("Generating Radar Signal Processing Plots")
    print("=" * 60)
    print()
    
    # Generate all plots
    plot_pulse_stacking()
    plot_range_compression()
    doppler_bin, range_bin, peak_mag = plot_range_doppler_map()
    
    print()
    print("=" * 60)
    print("All plots generated successfully!")
    print("=" * 60)
    print()
    print("Files created in /tmp/:")
    print("  - pulse_stacking.png")
    print("  - range_compression.png")
    print("  - range_doppler_map.png")
    print()
    print(f"Range-Doppler Peak Detection:")
    print(f"  Doppler Bin: {doppler_bin}")
    print(f"  Range Bin: {range_bin}")
    print(f"  Magnitude: {peak_mag:.2f}")

if __name__ == "__main__":
    main()
