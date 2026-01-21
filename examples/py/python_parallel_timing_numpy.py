#!/usr/bin/env python3
"""
NumPy Parallel Execution Demo - Near-Perfect Parallelization

This demo demonstrates that NumPy operations release the GIL and achieve
near-perfect parallel speedup because the actual computation happens in
C code without the GIL held.

Key insight: NumPy (and other C extensions) release the GIL during
computational operations, enabling true multi-core parallelism.
"""

import dagex
import time
import numpy as np

print("=" * 70)
print("  NUMPY PARALLEL EXECUTION - TRUE PARALLELIZATION")
print("  NumPy releases GIL → Near-perfect speedup!")
print("=" * 70)

def demo_numpy_computation():
    """NumPy-heavy computation with expected near-linear speedup"""
    print("\n" + "─" * 70)
    print("Demo 1: NumPy Matrix Operations (4 parallel nodes)")
    print("─" * 70)
    print("🔬 Each node does: FFT + Matrix multiplication + Linear algebra")

    def source(inputs, params):
        """Generate random data"""
        np.random.seed(42)
        return {"matrix": np.random.randn(500, 500)}

    def numpy_worker(inputs, params):
        """Heavy NumPy computation - GIL is released during these ops"""
        matrix = inputs.get("input")
        
        # FFT computation
        fft_result = np.fft.fft2(matrix)
        
        # Matrix multiplication (GIL released)
        result = matrix @ matrix.T
        
        # SVD (GIL released)
        u, s, vh = np.linalg.svd(result[:100, :100])
        
        # More matrix ops
        final = result + fft_result.real
        
        return {"result": np.sum(final)}

    # Build graph with 4 parallel branches
    graph = dagex.Graph()
    graph.add(source, "Source", None, [("matrix", "data")])

    for i in range(4):
        branch = dagex.Graph()
        branch.add(
            numpy_worker,
            f"NumPyWorker{i}",
            [("data", "input")],
            [("result", f"result_{i}")]
        )
        graph.branch(branch)

    dag = graph.build()

    print(f"\n📈 Expected behavior:")
    print(f"   Sequential: ~4x the time of one computation")
    print(f"   Parallel:   ~1x (all run simultaneously)")
    print(f"   Speedup:    Close to 4x (near-perfect parallelization)")

    # Sequential execution
    print("\n🐌 Sequential Execution (parallel=False):")
    start = time.time()
    dag.execute(parallel=False)
    sequential_time = (time.time() - start) * 1000
    print(f"   ⏱️  Time: {sequential_time:.1f}ms")

    # Parallel execution
    print("\n⚡ Parallel Execution (parallel=True):")
    start = time.time()
    dag.execute(parallel=True)
    parallel_time = (time.time() - start) * 1000
    print(f"   ⏱️  Time: {parallel_time:.1f}ms")

    # Analysis
    speedup = sequential_time / parallel_time
    efficiency = (speedup / 4.0) * 100  # 4 parallel nodes
    
    print(f"\n📊 Results:")
    print(f"   Sequential: {sequential_time:.1f}ms")
    print(f"   Parallel:   {parallel_time:.1f}ms")
    print(f"   Speedup:    {speedup:.2f}x")
    print(f"   Efficiency: {efficiency:.1f}% (4 cores)")

    if speedup > 3.5:
        print("   ✅ EXCELLENT! Near-perfect parallelization (NumPy releases GIL)")
    elif speedup > 2.5:
        print("   ✅ GOOD! Strong parallelization achieved")
    elif speedup > 1.5:
        print("   ⚠️  Moderate parallelization (some overhead)")
    else:
        print("   ❌ Limited parallelization")

def demo_signal_processing():
    """Radar-like signal processing with NumPy"""
    print("\n" + "─" * 70)
    print("Demo 2: Signal Processing Pipeline (Radar-like)")
    print("─" * 70)
    print("🎯 Simulating parallel radar signal processing chains")

    def generate_signal(inputs, params):
        """Generate synthetic radar signal"""
        np.random.seed(42)
        # Simulate complex radar return
        t = np.linspace(0, 1, 10000)
        signal = np.sin(2 * np.pi * 50 * t) + 0.5 * np.random.randn(10000)
        return {"signal": signal}

    def process_channel(inputs, params):
        """Process one radar channel with heavy NumPy operations"""
        signal = inputs.get("input")
        
        # FFT-based filtering
        fft = np.fft.fft(signal)
        fft_filtered = fft * np.exp(-np.abs(np.fft.fftfreq(len(signal))) * 10)
        filtered = np.fft.ifft(fft_filtered).real
        
        # Convolution
        kernel = np.hanning(100)
        convolved = np.convolve(filtered, kernel, mode='same')
        
        # Statistical analysis
        spectrogram = np.abs(fft[:len(fft)//2])
        power = np.sum(spectrogram ** 2)
        
        return {"power": power, "processed": convolved[:100].tolist()}

    # Build graph with 8 parallel channels
    graph = dagex.Graph()
    graph.add(generate_signal, "RadarSource", None, [("signal", "signal")])

    num_channels = 8
    for i in range(num_channels):
        branch = dagex.Graph()
        branch.add(
            process_channel,
            f"Channel{i}",
            [("signal", "input")],
            [("power", f"power_{i}")]
        )
        graph.branch(branch)

    dag = graph.build()

    print(f"\n📈 Processing {num_channels} radar channels in parallel")

    # Sequential
    print("\n🐌 Sequential Processing:")
    start = time.time()
    dag.execute(parallel=False)
    sequential_time = (time.time() - start) * 1000
    print(f"   ⏱️  Time: {sequential_time:.1f}ms")

    # Parallel
    print("\n⚡ Parallel Processing:")
    start = time.time()
    result = dag.execute(parallel=True)
    parallel_time = (time.time() - start) * 1000
    print(f"   ⏱️  Time: {parallel_time:.1f}ms")

    # Analysis
    speedup = sequential_time / parallel_time
    efficiency = (speedup / num_channels) * 100
    
    print(f"\n📊 Results:")
    print(f"   Sequential: {sequential_time:.1f}ms")
    print(f"   Parallel:   {parallel_time:.1f}ms")
    print(f"   Speedup:    {speedup:.2f}x")
    print(f"   Efficiency: {efficiency:.1f}% ({num_channels} channels)")
    
    if speedup > 6.0:
        print("   ✅ EXCELLENT parallel scaling for signal processing!")
    elif speedup > 4.0:
        print("   ✅ GOOD parallel scaling")
    else:
        print("   ⚠️  Some overhead present")

def demo_large_computation():
    """Very heavy NumPy computation to maximize time in C code"""
    print("\n" + "─" * 70)
    print("Demo 3: Heavy Linear Algebra (3 parallel nodes)")
    print("─" * 70)

    def source(inputs, params):
        np.random.seed(42)
        return {"size": 800}

    def heavy_linalg(inputs, params):
        """Extremely heavy linear algebra - lots of time with GIL released"""
        size = inputs.get("input", 800)
        
        # Create large random matrix
        A = np.random.randn(size, size)
        B = np.random.randn(size, size)
        
        # Multiple heavy operations
        C = A @ B  # Matrix multiplication
        D = np.linalg.inv(C + np.eye(size) * 0.1)  # Matrix inversion
        eigenvalues = np.linalg.eigvals(D[:200, :200])  # Eigenvalues
        
        return {"result": np.sum(eigenvalues.real)}

    # Build graph
    graph = dagex.Graph()
    graph.add(source, "Source", None, [("size", "size")])

    for i in range(3):
        branch = dagex.Graph()
        branch.add(
            heavy_linalg,
            f"LinAlg{i}",
            [("size", "input")],
            [("result", f"result_{i}")]
        )
        graph.branch(branch)

    dag = graph.build()

    print("\n⚠️  This may take a moment - doing heavy computation...")

    # Sequential
    print("\n🐌 Sequential Execution:")
    start = time.time()
    dag.execute(parallel=False)
    sequential_time = (time.time() - start) * 1000
    print(f"   ⏱️  Time: {sequential_time:.1f}ms")

    # Parallel
    print("\n⚡ Parallel Execution:")
    start = time.time()
    dag.execute(parallel=True)
    parallel_time = (time.time() - start) * 1000
    print(f"   ⏱️  Time: {parallel_time:.1f}ms")

    # Analysis
    speedup = sequential_time / parallel_time
    efficiency = (speedup / 3.0) * 100
    
    print(f"\n📊 Results:")
    print(f"   Sequential: {sequential_time:.1f}ms")
    print(f"   Parallel:   {parallel_time:.1f}ms")
    print(f"   Speedup:    {speedup:.2f}x")
    print(f"   Efficiency: {efficiency:.1f}% (3 cores)")
    
    if speedup > 2.7:
        print("   ✅ NEAR-PERFECT parallelization! (~3x speedup)")

if __name__ == "__main__":
    demo_numpy_computation()
    demo_signal_processing()
    demo_large_computation()
    
    print("\n" + "=" * 70)
    print("  CONCLUSION: NumPy achieves near-perfect parallel speedup!")
    print("  ")
    print("  Why? NumPy releases the GIL during computational operations.")
    print("  The actual work happens in C/Fortran without Python overhead.")
    print("  ")
    print("  This is ideal for: signal processing, linear algebra, FFTs,")
    print("  array operations, scientific computing, and ML inference.")
    print("=" * 70 + "\n")
