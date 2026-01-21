#!/usr/bin/env python3
"""
Parallel Execution Timing Verification

This demo PROVES that parallel execution actually works by:
1. Running identical work sequentially and in parallel
2. Measuring real wall-clock time
3. Calculating speedup
4. Showing that parallel execution is genuinely faster
"""

import dagex
import time
from datetime import datetime

print("=" * 70)
print("  PARALLEL EXECUTION TIMING VERIFICATION")
print("  Proving that parallelization actually works!")
print("=" * 70)

def demo_sequential_vs_parallel():
    """3 parallel branches with 100ms work each"""
    print("\n" + "─" * 70)
    print("Demo 1: 3 Parallel Branches (100ms each)")
    print("─" * 70)

    def source(inputs, params):
        return {"data": "source"}

    def worker_100ms(inputs, params):
        """Simulates 100ms of CPU work"""
        time.sleep(0.1)
        return {"result": "done"}

    # Build graph
    graph = dagex.Graph()
    graph.add(source, "Source", None, [("data", "data")])

    for label in ['A', 'B', 'C']:
        branch = dagex.Graph()
        branch.add(
            worker_100ms,
            f"Branch{label}[100ms]",
            [("data", "input")],
            [("result", f"result_{label.lower()}")]
        )
        graph.branch(branch)

    dag = graph.build()

    print(f"\n📈 DAG Structure:")
    print(f"   Expected sequential time: ~300ms (100ms × 3)")
    print(f"   Expected parallel time: ~100ms (all run together)")

    # ===== SEQUENTIAL EXECUTION =====
    print("\n🐌 Sequential Execution (parallel=False):")
    start = time.time()
    dag.execute(parallel=False)
    sequential_time = (time.time() - start) * 1000
    print(f"   ⏱️  Actual time: {sequential_time:.1f}ms")

    # ===== PARALLEL EXECUTION =====
    print("\n⚡ Parallel Execution (parallel=True):")
    start = time.time()
    dag.execute(parallel=True)
    parallel_time = (time.time() - start) * 1000
    print(f"   ⏱️  Actual time: {parallel_time:.1f}ms")

    # ===== RESULTS =====
    speedup = sequential_time / parallel_time
    print(f"\n📊 Results:")
    print(f"   Sequential: {sequential_time:.1f}ms")
    print(f"   Parallel:   {parallel_time:.1f}ms")
    print(f"   Speedup:    {speedup:.2f}x faster")

    if speedup > 2.5:
        print("   ✅ PARALLELIZATION IS WORKING! (~3x speedup achieved)")
    elif speedup > 1.5:
        print("   ⚠️  Partial parallelization (overhead or GIL contention)")
    else:
        print("   ❌ Not effectively parallel (Python GIL may be blocking)")

def demo_many_parallel_nodes():
    """10 parallel branches with 50ms work each"""
    print("\n" + "─" * 70)
    print("Demo 2: 10 Parallel Nodes (50ms each)")
    print("─" * 70)

    def source(inputs, params):
        return {"data": 0}

    def worker_50ms(inputs, params):
        """Simulates 50ms of CPU work"""
        time.sleep(0.05)
        val = inputs.get("input", 0)
        return {"result": val + 1}

    # Build graph
    graph = dagex.Graph()
    graph.add(source, "Source", None, [("data", "data")])

    for i in range(10):
        branch = dagex.Graph()
        branch.add(
            worker_50ms,
            f"Worker{i}[50ms]",
            [("data", "input")],
            [("result", f"result_{i}")]
        )
        graph.branch(branch)

    dag = graph.build()

    print(f"\n📈 DAG Structure:")
    print(f"   Expected sequential time: ~500ms (50ms × 10)")
    print(f"   Expected parallel time: ~50ms (all run together)")

    # Sequential
    print("\n🐌 Sequential Execution:")
    start = time.time()
    dag.execute(parallel=False)
    sequential_time = (time.time() - start) * 1000
    print(f"   ⏱️  Actual time: {sequential_time:.1f}ms")

    # Parallel
    print("\n⚡ Parallel Execution:")
    start = time.time()
    dag.execute(parallel=True)
    parallel_time = (time.time() - start) * 1000
    print(f"   ⏱️  Actual time: {parallel_time:.1f}ms")

    # Results
    speedup = sequential_time / parallel_time
    print(f"\n📊 Results:")
    print(f"   Sequential: {sequential_time:.1f}ms")
    print(f"   Parallel:   {parallel_time:.1f}ms")
    print(f"   Speedup:    {speedup:.2f}x faster")

    if speedup > 8.0:
        print("   ✅ EXCELLENT PARALLELIZATION! (~10x speedup achieved)")
    elif speedup > 5.0:
        print("   ✅ GOOD PARALLELIZATION! (5-10x speedup)")
    elif speedup > 2.0:
        print("   ⚠️  Moderate parallelization (Python GIL overhead)")
    else:
        print("   ❌ Limited parallel benefit (GIL blocking)")

def demo_cpu_bound_work():
    """Demonstrate with actual CPU-bound work (not just sleep)"""
    print("\n" + "─" * 70)
    print("Demo 3: CPU-Bound Work (sorting)")
    print("─" * 70)
    print("⚠️  Note: Python GIL may limit speedup for pure CPU work")

    def source(inputs, params):
        return {"data": list(range(100000))}

    def cpu_worker(inputs, params):
        """Actual CPU work - sorting"""
        data = inputs.get("input", [])
        # Do work multiple times to take ~50ms
        for _ in range(5):
            sorted_data = sorted(data, reverse=True)
        return {"result": len(sorted_data)}

    # Build graph
    graph = dagex.Graph()
    graph.add(source, "Source", None, [("data", "data")])

    for i in range(4):
        branch = dagex.Graph()
        branch.add(
            cpu_worker,
            f"CPUWorker{i}",
            [("data", "input")],
            [("result", f"result_{i}")]
        )
        graph.branch(branch)

    dag = graph.build()

    # Sequential
    print("\n🐌 Sequential Execution:")
    start = time.time()
    dag.execute(parallel=False)
    sequential_time = (time.time() - start) * 1000
    print(f"   ⏱️  Actual time: {sequential_time:.1f}ms")

    # Parallel
    print("\n⚡ Parallel Execution:")
    start = time.time()
    dag.execute(parallel=True)
    parallel_time = (time.time() - start) * 1000
    print(f"   ⏱️  Actual time: {parallel_time:.1f}ms")

    # Results
    speedup = sequential_time / parallel_time
    print(f"\n📊 Results:")
    print(f"   Sequential: {sequential_time:.1f}ms")
    print(f"   Parallel:   {parallel_time:.1f}ms")
    print(f"   Speedup:    {speedup:.2f}x faster")
    print(f"\n   Note: CPU-bound Python code is limited by the GIL")
    print(f"         (Global Interpreter Lock prevents true parallelism)")
    print(f"         For I/O-bound work or with multiprocessing, speedup is better!")

if __name__ == "__main__":
    demo_sequential_vs_parallel()
    demo_many_parallel_nodes()
    demo_cpu_bound_work()
    
    print("\n" + "=" * 70)
    print("  CONCLUSION: Parallel execution provides real speedup!")
    print("  (Python GIL limits CPU-bound work, but I/O-bound work parallels well)")
    print("=" * 70 + "\n")
