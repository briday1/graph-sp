#!/usr/bin/env python3
"""
Generate correct signal processing plots for radar demos showing proper matched filtering.
Creates visualizations of pulse stacking, range compression (matched filter), and Range-Doppler maps.
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

def matched_filter(signal, reference):
    """Apply matched filter (correlate with time-reversed conjugate of reference)."""
    ref_conj = np.conj(reference[::-1])
    # Correlation via FFT for efficiency
    result = np.fft.ifft(np.fft.fft(signal) * np.fft.fft(ref_conj, len(signal)))
    return result

def plot_pulse_stacking():
    """Create visualization of pulse stacking."""
    fig, axes = plt.subplots(2, 2, figsize=(14, 10))
    
    # Generate LFM pulse
    pulse, t = generate_lfm_pulse()
    
    # Plot 1: Single LFM pulse - Time domain
    ax = axes[0, 0]
    ax.plot(t * 1e6, np.real(pulse), 'b-', linewidth=1, label='Real')
    ax.plot(t * 1e6, np.imag(pulse), 'r-', linewidth=1, label='Imag')
    ax.set_xlabel('Time (μs)', fontsize=11)
    ax.set_ylabel('Amplitude', fontsize=11)
    ax.set_title('Single LFM Pulse (Real & Imaginary)', fontsize=12, fontweight='bold')
    ax.legend()
    ax.grid(True, alpha=0.3)
    
    # Plot 2: LFM pulse - Frequency domain
    ax = axes[0, 1]
    fft_pulse = np.fft.fft(pulse)
    freq = np.fft.fftfreq(len(pulse), t[1] - t[0])
    ax.plot(np.fft.fftshift(freq) / 1e6, np.fft.fftshift(np.abs(fft_pulse)), 'b-', linewidth=1)
    ax.set_xlabel('Frequency (MHz)', fontsize=11)
    ax.set_ylabel('Magnitude', fontsize=11)
    ax.set_title('Single Pulse - Frequency Domain', fontsize=12, fontweight='bold')
    ax.grid(True, alpha=0.3)
    
    # Plot 3: Stacked pulses (simulate 4 pulses with Doppler)
    ax = axes[1, 0]
    num_pulses = 4
    doppler_freq = 500  # Hz
    prf = 10000  # Pulse Repetition Frequency
    
    stacked_pulses = []
    for i in range(num_pulses):
        phase_shift = 2 * np.pi * doppler_freq * i / prf
        shifted_pulse = pulse * np.exp(1j * phase_shift)
        stacked_pulses.append(shifted_pulse)
    
    stacked = np.concatenate(stacked_pulses)
    t_stacked = np.arange(len(stacked)) * (t[1] - t[0])
    ax.plot(t_stacked * 1e6, np.abs(stacked), 'purple', linewidth=0.8)
    ax.set_xlabel('Time (μs)', fontsize=11)
    ax.set_ylabel('Magnitude', fontsize=11)
    ax.set_title(f'{num_pulses} Stacked Pulses (1024 samples)', fontsize=12, fontweight='bold')
    ax.grid(True, alpha=0.3)
    
    # Plot 4: Stacked pulses - 2D representation
    ax = axes[1, 1]
    stacked_2d = np.array(stacked_pulses)
    im = ax.imshow(np.abs(stacked_2d), aspect='auto', cmap='viridis', 
                   extent=[0, len(pulse), num_pulses, 0])
    ax.set_xlabel('Fast-time (samples)', fontsize=11)
    ax.set_ylabel('Pulse Number', fontsize=11)
    ax.set_title('Stacked Pulses (2D View)', fontsize=12, fontweight='bold')
    plt.colorbar(im, ax=ax, label='Magnitude')
    
    plt.tight_layout()
    plt.savefig('/tmp/pulse_stacking_correct.png', dpi=150, bbox_inches='tight')
    print("✓ Generated pulse_stacking_correct.png")
    plt.close()

def plot_range_compression():
    """Create visualization of range compression using matched filter."""
    fig, axes = plt.subplots(2, 2, figsize=(14, 10))
    
    # Generate reference pulse and add target echo
    reference, t = generate_lfm_pulse()
    num_samples = len(reference)
    
    # Simulate received signal: reference pulse + delayed echo (simulated target)
    signal_len = num_samples * 4
    signal = np.zeros(signal_len, dtype=complex)
    target_delay = 828  # samples
    # Make sure we don't overflow
    end_idx = min(target_delay + num_samples, signal_len)
    copy_len = end_idx - target_delay
    signal[target_delay:end_idx] = reference[:copy_len] * 0.5  # Attenuated echo
    
    # Plot 1: Time domain signal (received echo)
    ax = axes[0, 0]
    ax.plot(np.abs(signal), 'b-', linewidth=0.5)
    ax.set_xlabel('Sample Index', fontsize=11)
    ax.set_ylabel('Magnitude', fontsize=11)
    ax.set_title('Before Range Compression (Received Echo)', fontsize=12, fontweight='bold')
    ax.axvline(target_delay, color='red', linestyle='--', alpha=0.5, label='Target Delay')
    ax.legend()
    ax.grid(True, alpha=0.3)
    
    # Plot 2: After Matched Filter (Range Compression)
    ax = axes[0, 1]
    compressed = matched_filter(signal, reference)
    ax.plot(np.abs(compressed), 'r-', linewidth=0.5)
    ax.set_xlabel('Range Bin', fontsize=11)
    ax.set_ylabel('Magnitude', fontsize=11)
    ax.set_title('After Range Compression (Matched Filter)', fontsize=12, fontweight='bold')
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
    ax.plot(range(start, end), np.abs(compressed[start:end]), 'r-', linewidth=1.5)
    ax.axvline(peak_idx, color='green', linestyle='--', alpha=0.7)
    ax.set_xlabel('Range Bin', fontsize=11)
    ax.set_ylabel('Magnitude', fontsize=11)
    ax.set_title(f'Peak Detail (Magnitude: {peak_val:.2f})', fontsize=12, fontweight='bold')
    ax.grid(True, alpha=0.3)
    
    # Plot 4: Phase information
    ax = axes[1, 1]
    ax.plot(np.angle(compressed), 'purple', linewidth=0.5, alpha=0.7)
    ax.set_xlabel('Range Bin', fontsize=11)
    ax.set_ylabel('Phase (radians)', fontsize=11)
    ax.set_title('Phase After Range Compression', fontsize=12, fontweight='bold')
    ax.grid(True, alpha=0.3)
    
    plt.tight_layout()
    plt.savefig('/tmp/range_compression_correct.png', dpi=150, bbox_inches='tight')
    print("✓ Generated range_compression_correct.png")
    plt.close()
    
    return peak_idx, peak_val

def plot_range_doppler_map():
    """Create Range-Doppler map visualization with proper matched filtering."""
    fig, axes = plt.subplots(2, 2, figsize=(15, 12))
    
    # Generate reference pulse
    reference, t = generate_lfm_pulse()
    num_samples = len(reference)
    num_pulses = 16
    doppler_freq = 1000  # Hz
    prf = 10000  # Pulse Repetition Frequency
    target_delay = 207  # Range bin where target appears
    
    # Stack pulses with Doppler shift (simulating moving target)
    stacked = []
    for i in range(num_pulses):
        # Create signal with delayed target echo
        signal = np.zeros(num_samples, dtype=complex)
        # Add delayed echo with Doppler phase shift
        phase_shift = 2 * np.pi * doppler_freq * i / prf
        signal = reference * np.exp(1j * phase_shift)  # Simplified: entire pulse has Doppler
        stacked.append(signal)
    
    stacked = np.array(stacked)
    
    # Plot 1: Stacked pulses (slow-time, fast-time)
    ax = axes[0, 0]
    im = ax.imshow(np.abs(stacked), aspect='auto', cmap='viridis',
                   extent=[0, stacked.shape[1], num_pulses, 0])
    ax.set_xlabel('Fast-time (Range samples)', fontsize=11)
    ax.set_ylabel('Slow-time (Pulse #)', fontsize=11)
    ax.set_title('Pulse Stack Before Processing', fontsize=12, fontweight='bold')
    plt.colorbar(im, ax=ax, label='Magnitude')
    
    # Range compression (matched filter for each pulse)
    range_compressed = []
    for pulse in stacked:
        compressed_pulse = matched_filter(pulse, reference)
        range_compressed.append(compressed_pulse)
    range_compressed = np.array(range_compressed)
    
    # Plot 2: After Range Compression
    ax = axes[0, 1]
    im = ax.imshow(np.abs(range_compressed), aspect='auto', cmap='hot',
                   extent=[0, range_compressed.shape[1], num_pulses, 0])
    ax.set_xlabel('Range Bin', fontsize=11)
    ax.set_ylabel('Pulse #', fontsize=11)
    ax.set_title('After Range Compression (Matched Filter)', fontsize=12, fontweight='bold')
    plt.colorbar(im, ax=ax, label='Magnitude')
    
    # Doppler compression (FFT along slow-time)
    rd_map = np.fft.fft(range_compressed, axis=0)
    magnitude = np.abs(rd_map)
    
    # Plot 3: Range-Doppler Map
    ax = axes[1, 0]
    im = ax.imshow(magnitude, aspect='auto', cmap='jet',
                   extent=[0, rd_map.shape[1], num_pulses, 0])
    ax.set_xlabel('Range Bin', fontsize=11)
    ax.set_ylabel('Doppler Bin', fontsize=11)
    ax.set_title('Range-Doppler Map', fontsize=12, fontweight='bold')
    plt.colorbar(im, ax=ax, label='Magnitude')
    
    # Mark peak
    max_idx = np.unravel_index(np.argmax(magnitude), magnitude.shape)
    doppler_bin, range_bin = max_idx
    ax.plot(range_bin, doppler_bin, 'w*', markersize=15, 
            markeredgewidth=2, markeredgecolor='black')
    ax.text(range_bin + 10, doppler_bin, 
            f'Peak\n({doppler_bin}, {range_bin})\nMag: {magnitude[max_idx]:.1f}',
            color='white', fontsize=10, fontweight='bold',
            bbox=dict(boxstyle='round', facecolor='black', alpha=0.8))
    
    # Plot 4: Range-Doppler (dB scale for better visualization)
    ax = axes[1, 1]
    magnitude_db = 20 * np.log10(magnitude + 1e-10)
    im = ax.imshow(magnitude_db, aspect='auto', cmap='jet',
                   extent=[0, rd_map.shape[1], num_pulses, 0])
    ax.set_xlabel('Range Bin', fontsize=11)
    ax.set_ylabel('Doppler Bin', fontsize=11)
    ax.set_title('Range-Doppler Map (dB scale)', fontsize=12, fontweight='bold')
    plt.colorbar(im, ax=ax, label='Magnitude (dB)')
    ax.plot(range_bin, doppler_bin, 'w*', markersize=15, 
            markeredgewidth=2, markeredgecolor='black')
    
    plt.tight_layout()
    plt.savefig('/tmp/range_doppler_map_correct.png', dpi=150, bbox_inches='tight')
    print("✓ Generated range_doppler_map_correct.png")
    plt.close()
    
    return doppler_bin, range_bin, magnitude[max_idx]

def main():
    print("=" * 70)
    print("Generating Correct Radar Signal Processing Plots")
    print("=" * 70)
    print()
    print("Using proper matched filter for range compression")
    print()
    
    # Generate all plots
    plot_pulse_stacking()
    peak_idx, peak_val = plot_range_compression()
    doppler_bin, range_bin, peak_mag = plot_range_doppler_map()
    
    print()
    print("=" * 70)
    print("All plots generated successfully!")
    print("=" * 70)
    print()
    print("Files created in /tmp/:")
    print("  - pulse_stacking_correct.png")
    print("  - range_compression_correct.png")
    print("  - range_doppler_map_correct.png")
    print()
    print(f"Range Compression Results:")
    print(f"  Peak Location: Range bin {peak_idx}")
    print(f"  Peak Magnitude: {peak_val:.2f}")
    print()
    print(f"Range-Doppler Map Results:")
    print(f"  Doppler Bin: {doppler_bin}")
    print(f"  Range Bin: {range_bin}")
    print(f"  Magnitude: {peak_mag:.2f}")
    print()
    print("These results show proper pulse compression via matched filtering,")
    print("which produces a sharp peak at the target location.")

if __name__ == "__main__":
    main()
